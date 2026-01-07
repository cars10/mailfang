use diesel::prelude::*;
use diesel::query_dsl::methods::FilterDsl;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper, TextExpressionMethods,
};

use crate::db::{EmailListPartial, EmailListRecord};
use crate::db::{ListQuery, vacuum_database};
use crate::{db::DbConnection, schema, web::error::DieselError};

fn build_search_pattern(search_term: &str) -> String {
    format!("%{}%", search_term)
}

pub fn get_emails(
    conn: &mut DbConnection,
    query_params: &ListQuery,
) -> Result<(Vec<EmailListRecord>, u64), DieselError> {
    let build_query = || {
        let mut query = schema::emails::table.into_boxed();
        if let Some(search_term) = query_params.search.as_deref() {
            let pattern = build_search_pattern(search_term);
            query = FilterDsl::filter(
                query,
                schema::emails::subject
                    .like(pattern.clone())
                    .or(schema::emails::message_id.like(pattern.clone()))
                    .or(schema::emails::from.like(pattern.clone()))
                    .or(schema::emails::body_text.like(pattern.clone()))
                    .or(schema::emails::body_html.like(pattern)),
            );
        }
        query
    };

    let total_count: i64 = build_query().count().get_result(conn)?;
    let num_pages = (total_count as f64 / query_params.per_page as f64).ceil() as u64;

    let emails = build_query()
        .select(EmailListPartial::as_select())
        .order(schema::emails::created_at.desc())
        .offset(((query_params.page - 1) * query_params.per_page) as i64)
        .limit(query_params.per_page as i64)
        .load::<EmailListPartial>(conn)?;

    let records = process_emails_with_recipients(conn, emails)?;

    Ok((records, num_pages))
}

fn process_emails_with_recipients(
    conn: &mut DbConnection,
    emails: Vec<EmailListPartial>,
) -> Result<Vec<EmailListRecord>, DieselError> {
    let email_ids: Vec<String> = emails.iter().map(|e| e.id.clone()).collect();
    let recipients_map = load_recipients_for_emails(conn, &email_ids)?;
    Ok(emails_to_records(emails, recipients_map))
}

fn load_recipients_for_emails(
    conn: &mut DbConnection,
    email_ids: &[String],
) -> Result<std::collections::HashMap<String, Vec<String>>, DieselError> {
    let all_recipients = FilterDsl::filter(
        schema::email_recipients::table.inner_join(schema::recipients::table),
        schema::email_recipients::email_id.eq_any(email_ids),
    )
    .select((
        schema::email_recipients::email_id,
        schema::recipients::email,
    ))
    .load::<(String, String)>(conn)?;

    let mut recipients_map: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for (email_id, recipient_email) in all_recipients {
        recipients_map
            .entry(email_id)
            .or_insert_with(Vec::new)
            .push(recipient_email);
    }

    Ok(recipients_map)
}

fn emails_to_records(
    emails: Vec<EmailListPartial>,
    mut recipients_map: std::collections::HashMap<String, Vec<String>>,
) -> Vec<EmailListRecord> {
    emails
        .into_iter()
        .map(|email| {
            let id_clone = email.id.clone();
            let recipients = recipients_map.remove(&id_clone).unwrap_or_default();
            EmailListRecord {
                id: email.id,
                subject: email.subject,
                date: email.date,
                created_at: email.created_at,
                from: email.from,
                read: email.read,
                has_attachments: email.has_attachments,
                recipients,
            }
        })
        .collect()
}

pub fn get_emails_by_recipient(
    conn: &mut DbConnection,
    recipient_email: &str,
    query_params: &ListQuery,
) -> Result<(Vec<EmailListRecord>, u64), DieselError> {
    let build_query = || {
        let mut query = FilterDsl::filter(
            schema::emails::table
                .inner_join(schema::email_recipients::table)
                .inner_join(
                    schema::recipients::table
                        .on(schema::email_recipients::recipient_id.eq(schema::recipients::id)),
                ),
            schema::recipients::email.eq(recipient_email),
        )
        .into_boxed();

        if let Some(search_term) = query_params.search.as_deref() {
            let pattern = build_search_pattern(search_term);
            query = FilterDsl::filter(
                query,
                schema::emails::subject
                    .like(pattern.clone())
                    .or(schema::emails::message_id.like(pattern.clone()))
                    .or(schema::emails::from.like(pattern.clone()))
                    .or(schema::emails::body_text.like(pattern.clone()))
                    .or(schema::emails::body_html.like(pattern)),
            );
        }
        query
    };

    let total_count: i64 = build_query().count().get_result(conn)?;
    let num_pages = (total_count as f64 / query_params.per_page as f64).ceil() as u64;

    let emails = build_query()
        .select(EmailListPartial::as_select())
        .order(schema::emails::created_at.desc())
        .offset(((query_params.page - 1) * query_params.per_page) as i64)
        .limit(query_params.per_page as i64)
        .load::<EmailListPartial>(conn)?;

    let records = process_emails_with_recipients(conn, emails)?;

    Ok((records, num_pages))
}

pub fn delete_all_emails(conn: &mut DbConnection) -> Result<usize, DieselError> {
    conn.transaction::<_, DieselError, _>(|conn| {
        let affected = diesel::delete(schema::emails::table).execute(conn)?;

        if affected > 0 {
            vacuum_database(conn)?;
        }

        Ok(affected)
    })
}

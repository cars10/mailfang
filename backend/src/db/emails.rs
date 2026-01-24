use diesel::prelude::*;
use diesel::query_dsl::methods::FilterDsl;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper};

use crate::db::search_query::{ParsedSearchQuery, SearchField, parse_search_query};
use crate::db::{EmailListPartial, EmailListRecord};
use crate::db::{ListQuery, vacuum_database};
use crate::{db::DbConnection, schema, web::error::DieselError};

fn build_search_pattern(search_term: &str) -> String {
    format!("%{}%", search_term)
}

fn build_search_sql_condition(
    search_query: &ParsedSearchQuery,
    has_envelope_recipients: bool,
) -> Option<String> {
    if search_query.field_terms.is_empty() && search_query.default_terms.is_empty() {
        return None;
    }

    let mut sql_parts = Vec::new();

    for term in &search_query.field_terms {
        let pattern = build_search_pattern(&term.value);
        let escaped_pattern = pattern.replace("'", "''");

        let condition = match term.field {
            SearchField::Subject => {
                format!("emails.subject LIKE '{}'", escaped_pattern)
            }
            SearchField::From => {
                format!(
                    "(emails.envelope_from LIKE '{}' OR EXISTS (SELECT 1 FROM headers WHERE headers.email_id = emails.id AND LOWER(headers.name) = 'from' AND headers.value LIKE '{}'))",
                    escaped_pattern, escaped_pattern
                )
            }
            SearchField::Recipient | SearchField::To => {
                let envelope_part = if has_envelope_recipients {
                    format!("envelope_recipients.email LIKE '{}'", escaped_pattern)
                } else {
                    format!(
                        "EXISTS (SELECT 1 FROM email_envelope_recipients INNER JOIN envelope_recipients ON email_envelope_recipients.envelope_recipient_id = envelope_recipients.id WHERE email_envelope_recipients.email_id = emails.id AND envelope_recipients.email LIKE '{}')",
                        escaped_pattern
                    )
                };
                format!(
                    "({} OR EXISTS (SELECT 1 FROM headers WHERE headers.email_id = emails.id AND LOWER(headers.name) IN ('to', 'cc', 'bcc') AND headers.value LIKE '{}'))",
                    envelope_part, escaped_pattern
                )
            }
            SearchField::Text => {
                format!("emails.body_text LIKE '{}'", escaped_pattern)
            }
            SearchField::Html => {
                format!("emails.body_html LIKE '{}'", escaped_pattern)
            }
            SearchField::Attachment => {
                format!(
                    "EXISTS (SELECT 1 FROM attachments WHERE attachments.email_id = emails.id AND attachments.filename LIKE '{}')",
                    escaped_pattern
                )
            }
        };
        sql_parts.push(format!("({})", condition));
    }

    if !search_query.default_terms.is_empty() {
        let mut default_parts = Vec::new();
        for term in &search_query.default_terms {
            let pattern = build_search_pattern(term);
            let escaped_pattern = pattern.replace("'", "''");

            let envelope_part = if has_envelope_recipients {
                format!("envelope_recipients.email LIKE '{}'", escaped_pattern)
            } else {
                format!(
                    "EXISTS (SELECT 1 FROM email_envelope_recipients INNER JOIN envelope_recipients ON email_envelope_recipients.envelope_recipient_id = envelope_recipients.id WHERE email_envelope_recipients.email_id = emails.id AND envelope_recipients.email LIKE '{}')",
                    escaped_pattern
                )
            };

            let default_condition = format!(
                "(emails.subject LIKE '{}' OR emails.envelope_from LIKE '{}' OR emails.body_text LIKE '{}' OR {} OR EXISTS (SELECT 1 FROM headers WHERE headers.email_id = emails.id AND LOWER(headers.name) IN ('from', 'to', 'cc', 'bcc') AND headers.value LIKE '{}'))",
                escaped_pattern, escaped_pattern, escaped_pattern, envelope_part, escaped_pattern
            );
            default_parts.push(format!("({})", default_condition));
        }

        if !default_parts.is_empty() {
            sql_parts.push(format!("({})", default_parts.join(" AND ")));
        }
    }

    if !sql_parts.is_empty() {
        Some(sql_parts.join(" AND "))
    } else {
        None
    }
}

pub fn get_emails(
    conn: &mut DbConnection,
    query_params: &ListQuery,
) -> Result<(Vec<EmailListRecord>, u64), DieselError> {
    let parsed_query = if let Some(search) = &query_params.search {
        parse_search_query(search)
    } else {
        ParsedSearchQuery {
            field_terms: Vec::new(),
            default_terms: Vec::new(),
        }
    };

    let search_sql = build_search_sql_condition(&parsed_query, false);

    let build_query = move || {
        use diesel::dsl::sql;
        use diesel::sql_types::Bool;

        let mut query = schema::emails::table.into_boxed();

        if let Some(ref sql_condition) = search_sql {
            query = FilterDsl::filter(query, sql::<Bool>(sql_condition));
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
    let to_headers_map = load_to_headers_for_emails(conn, &email_ids)?;
    Ok(emails_to_records(emails, recipients_map, to_headers_map))
}

fn load_recipients_for_emails(
    conn: &mut DbConnection,
    email_ids: &[String],
) -> Result<std::collections::HashMap<String, Vec<String>>, DieselError> {
    let all_recipients = FilterDsl::filter(
        schema::email_envelope_recipients::table.inner_join(schema::envelope_recipients::table),
        schema::email_envelope_recipients::email_id.eq_any(email_ids),
    )
    .select((
        schema::email_envelope_recipients::email_id,
        schema::envelope_recipients::email,
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

fn load_to_headers_for_emails(
    conn: &mut DbConnection,
    email_ids: &[String],
) -> Result<std::collections::HashMap<String, Vec<String>>, DieselError> {
    let to_headers = FilterDsl::filter(
        schema::headers::table,
        schema::headers::email_id
            .eq_any(email_ids)
            .and(schema::headers::name.eq("To")),
    )
    .select((schema::headers::email_id, schema::headers::value))
    .load::<(String, String)>(conn)?;

    let mut headers_map: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for (email_id, header_value) in to_headers {
        headers_map
            .entry(email_id)
            .or_insert_with(Vec::new)
            .push(header_value);
    }

    Ok(headers_map)
}

fn emails_to_records(
    emails: Vec<EmailListPartial>,
    mut recipients_map: std::collections::HashMap<String, Vec<String>>,
    mut to_headers_map: std::collections::HashMap<String, Vec<String>>,
) -> Vec<EmailListRecord> {
    emails
        .into_iter()
        .map(|email| {
            let id_clone = email.id.clone();
            let recipients = recipients_map.remove(&id_clone).unwrap_or_default();
            let to_header = to_headers_map.remove(&id_clone);
            EmailListRecord {
                id: email.id,
                subject: email.subject,
                date: email.date,
                created_at: email.created_at,
                envelope_from: email.envelope_from,
                read: email.read,
                has_attachments: email.has_attachments,
                recipients,
                to_header,
            }
        })
        .collect()
}

pub fn get_emails_by_recipient(
    conn: &mut DbConnection,
    recipient_email: &str,
    query_params: &ListQuery,
) -> Result<(Vec<EmailListRecord>, u64), DieselError> {
    let parsed_query = if let Some(search) = &query_params.search {
        parse_search_query(search)
    } else {
        ParsedSearchQuery {
            field_terms: Vec::new(),
            default_terms: Vec::new(),
        }
    };

    let search_sql = build_search_sql_condition(&parsed_query, true);

    let build_query = move || {
        use diesel::dsl::sql;
        use diesel::sql_types::Bool;

        let mut query = FilterDsl::filter(
            schema::emails::table
                .inner_join(schema::email_envelope_recipients::table)
                .inner_join(
                    schema::envelope_recipients::table
                        .on(schema::email_envelope_recipients::envelope_recipient_id
                            .eq(schema::envelope_recipients::id)),
                ),
            schema::envelope_recipients::email.eq(recipient_email),
        )
        .into_boxed();

        if let Some(ref sql_condition) = search_sql {
            query = FilterDsl::filter(query, sql::<Bool>(sql_condition));
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
    let affected = conn.transaction::<_, DieselError, _>(|conn| {
        diesel::delete(schema::emails::table).execute(conn)
    })?;

    if affected > 0 {
        vacuum_database(conn)?;
    }

    Ok(affected)
}

use diesel::{ExpressionMethods, QueryDsl, Queryable, RunQueryDsl, dsl::count};

use crate::{db::DbConnection, schema, web::error::DieselError};

#[derive(serde::Serialize, Clone, Queryable)]
pub struct RecipientStats {
    pub recipient: String,
    pub count: i64,
}

#[derive(serde::Serialize, Clone)]
pub struct EmailStats {
    pub inbox: i64,
    pub recipients: Vec<RecipientStats>,
}

pub fn get_email_counts(conn: &mut DbConnection) -> Result<EmailStats, DieselError> {
    let total_count: i64 = schema::emails::table.count().get_result(conn)?;

    let recipients_stats: Vec<RecipientStats> = schema::recipients::table
        .inner_join(schema::email_recipients::table)
        .group_by(schema::recipients::id)
        .select((
            schema::recipients::email,
            count(schema::email_recipients::email_id),
        ))
        .order_by(schema::recipients::email.asc())
        .load::<RecipientStats>(conn)?;

    Ok(EmailStats {
        inbox: total_count,
        recipients: recipients_stats,
    })
}

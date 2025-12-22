use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub is_active: bool,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    #[inline]
    pub fn select_query() -> &'static str {
        include_str!("../../../../SQL/auth/users/select_base.sql")
    }
}

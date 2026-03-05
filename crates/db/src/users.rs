use crate::models::UserEntity;
use chrono::Utc;
use sqlx::{PgPool, Result};
use uuid::Uuid;

pub struct UserRepo {
    pool: PgPool,
}

impl UserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<UserEntity>> {
        sqlx::query_as::<_, UserEntity>(
            r#"
            SELECT id, email, password_hash, first_name, last_name, global_role, created_at, updated_at
            FROM users
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_by_email(&self, email: &str) -> Result<Option<UserEntity>> {
        sqlx::query_as::<_, UserEntity>(
            r#"
            SELECT id, email, password_hash, first_name, last_name, global_role, created_at, updated_at
            FROM users
            WHERE email = $1
            "#
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create(
        &self,
        email: &str,
        password_hash: &str,
        first_name: Option<String>,
        last_name: Option<String>,
        global_role: &str,
    ) -> Result<UserEntity> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query_as::<_, UserEntity>(
            r#"
            INSERT INTO users (id, email, password_hash, first_name, last_name, global_role, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, email, password_hash, first_name, last_name, global_role, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(email)
        .bind(password_hash)
        .bind(first_name)
        .bind(last_name)
        .bind(global_role)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
    }
}

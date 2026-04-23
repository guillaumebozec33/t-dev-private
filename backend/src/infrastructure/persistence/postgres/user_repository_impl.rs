use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::User;
use crate::domain::errors::DomainError;
use crate::domain::repositories::UserRepository;

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

    fn map_sqlx_error(e: sqlx::Error) -> DomainError {
        if let sqlx::Error::Database(db_err) = &e {
            if db_err.code().as_deref() == Some("23505") {
                if db_err.message().contains("username") {
                    return DomainError::UsernameAlreadyExists;
                }
                if db_err.message().contains("email") {
                    return DomainError::EmailAlreadyExists;
                }
            }
        }
        DomainError::InternalError(e.to_string())
    }
#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create(&self, user: &User) -> Result<User, DomainError> {
        let created = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, username, email, password_hash, avatar_url, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#
        )
        .bind(user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.avatar_url)
        .bind(&user.status)
        .bind(user.created_at)
        .bind(user.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(created)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(user)
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, DomainError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(user)
    }

    async fn update(&self, user: &User) -> Result<User, DomainError> {
        let updated = sqlx::query_as::<_, User>(
            r#"
            UPDATE users 
            SET username = $2, email = $3, avatar_url = $4, status = $5, updated_at = $6
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.avatar_url)
        .bind(&user.status)
        .bind(user.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(updated)
    }
}

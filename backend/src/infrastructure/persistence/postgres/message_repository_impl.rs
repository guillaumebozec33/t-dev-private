use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::Message;
use crate::domain::errors::DomainError;
use crate::domain::repositories::MessageRepository;

pub struct PgMessageRepository {
    pool: PgPool,
}

impl PgMessageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MessageRepository for PgMessageRepository {
    async fn create(&self, message: &Message) -> Result<Message, DomainError> {
        let created = sqlx::query_as::<_, Message>(
            r#"
            INSERT INTO messages (id, channel_id, author_id, content, edited, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#
        )
        .bind(message.id)
        .bind(message.channel_id)
        .bind(message.author_id)
        .bind(&message.content)
        .bind(message.edited)
        .bind(message.created_at)
        .bind(message.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(created)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Message>, DomainError> {
        let message = sqlx::query_as::<_, Message>("SELECT * FROM messages WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(message)
    }

    async fn find_by_channel_id(&self, channel_id: Uuid, limit: i64, before: Option<Uuid>) -> Result<Vec<Message>, DomainError> {
        let messages = if let Some(before_id) = before {
            sqlx::query_as::<_, Message>(
                r#"
                SELECT m.* FROM messages m
                WHERE m.channel_id = $1 AND m.created_at < (SELECT created_at FROM messages WHERE id = $3)
                ORDER BY m.created_at DESC
                LIMIT $2
                "#
            )
            .bind(channel_id)
            .bind(limit)
            .bind(before_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, Message>(
                r#"
                SELECT m.* FROM messages m
                WHERE m.channel_id = $1
                ORDER BY m.created_at DESC
                LIMIT $2
                "#
            )
            .bind(channel_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(messages)
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM messages WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(())
    }
    async fn update(&self, message: &Message) -> Result<Message, DomainError> {
        let updated = sqlx::query_as::<_, Message>(
            r#"
            UPDATE messages 
            SET content = $1, 
                edited = $2, 
                updated_at = $3
            WHERE id = $4
            RETURNING id, channel_id, author_id, content, edited, created_at, updated_at
            "#
        )
        .bind(&message.content)
        .bind(message.edited)
        .bind(message.updated_at)
        .bind(message.id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(format!("Failed to update message: {}", e)))?;

        Ok(updated)
    }
}

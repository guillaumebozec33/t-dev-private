use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::{Conversation, DirectMessage};
use crate::domain::errors::DomainError;
use crate::domain::repositories::DmRepository;

pub struct PgDmRepository {
    pool: PgPool,
}

impl PgDmRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DmRepository for PgDmRepository {
    async fn find_or_create_conversation(&self, user1_id: Uuid, user2_id: Uuid) -> Result<Conversation, DomainError> {
        let existing = sqlx::query_as::<_, Conversation>(
            "SELECT * FROM conversations WHERE (user1_id = $1 AND user2_id = $2) OR (user1_id = $2 AND user2_id = $1)"
        )
        .bind(user1_id)
        .bind(user2_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        if let Some(conv) = existing {
            return Ok(conv);
        }

        let (lo, hi) = if user1_id < user2_id { (user1_id, user2_id) } else { (user2_id, user1_id) };

        let conv = sqlx::query_as::<_, Conversation>(
            "INSERT INTO conversations (id, user1_id, user2_id, created_at) VALUES ($1, $2, $3, NOW()) RETURNING *"
        )
        .bind(Uuid::new_v4())
        .bind(lo)
        .bind(hi)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(conv)
    }

    async fn find_conversations_by_user(&self, user_id: Uuid) -> Result<Vec<Conversation>, DomainError> {
        let convs = sqlx::query_as::<_, Conversation>(
            "SELECT * FROM conversations WHERE user1_id = $1 OR user2_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(convs)
    }

    async fn find_conversation_by_id(&self, id: Uuid) -> Result<Option<Conversation>, DomainError> {
        let conv = sqlx::query_as::<_, Conversation>(
            "SELECT * FROM conversations WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(conv)
    }

    async fn create_dm_message(&self, msg: &DirectMessage) -> Result<DirectMessage, DomainError> {
        let created = sqlx::query_as::<_, DirectMessage>(
            "INSERT INTO direct_messages (id, conversation_id, sender_id, content, created_at) VALUES ($1, $2, $3, $4, $5) RETURNING *"
        )
        .bind(msg.id)
        .bind(msg.conversation_id)
        .bind(msg.sender_id)
        .bind(&msg.content)
        .bind(msg.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(created)
    }

    async fn find_messages_by_conversation(&self, conversation_id: Uuid, limit: i64, before: Option<Uuid>) -> Result<Vec<DirectMessage>, DomainError> {
        let messages = if let Some(before_id) = before {
            sqlx::query_as::<_, DirectMessage>(
                r#"
                SELECT * FROM direct_messages
                WHERE conversation_id = $1
                  AND created_at < (SELECT created_at FROM direct_messages WHERE id = $3)
                ORDER BY created_at DESC
                LIMIT $2
                "#
            )
            .bind(conversation_id)
            .bind(limit)
            .bind(before_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, DirectMessage>(
                "SELECT * FROM direct_messages WHERE conversation_id = $1 ORDER BY created_at DESC LIMIT $2"
            )
            .bind(conversation_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(messages)
    }

    async fn find_message_by_id(&self, id: Uuid) -> Result<Option<DirectMessage>, DomainError> {
        let msg = sqlx::query_as::<_, DirectMessage>(
            "SELECT * FROM direct_messages WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(msg)
    }
}

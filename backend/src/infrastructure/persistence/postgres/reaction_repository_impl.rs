use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::Reaction;
use crate::domain::errors::DomainError;
use crate::domain::repositories::ReactionRepository;

pub struct PgReactionRepository {
    pool: PgPool,
}

impl PgReactionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReactionRepository for PgReactionRepository {
    async fn find_by_message_id(&self, message_id: Uuid) -> Result<Vec<Reaction>, DomainError> {
        let reactions = sqlx::query_as::<_, Reaction>("SELECT * FROM reactions WHERE message_id = $1")
            .bind(message_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(reactions)
    }

    async fn find_by_message_ids(&self, message_ids: &[Uuid]) -> Result<Vec<Reaction>, DomainError> {
        let reactions = sqlx::query_as::<_, Reaction>("SELECT * FROM reactions WHERE message_id = ANY($1)")
            .bind(message_ids)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(reactions)
    }

    async fn find_by_user_and_message(&self, user_id: Uuid, message_id: Uuid) -> Result<Option<Reaction>, DomainError> {
        let reaction = sqlx::query_as::<_, Reaction>(
            "SELECT * FROM reactions WHERE user_id = $1 AND message_id = $2"
        )
            .bind(user_id)
            .bind(message_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(reaction)
    }

    async fn create(&self, reaction: &Reaction) -> Result<Reaction, DomainError> {
        let created = sqlx::query_as::<_, Reaction>(
            r#"
            INSERT INTO reactions (id, message_id, user_id, emoji, created_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#
        )
            .bind(reaction.id)
            .bind(reaction.message_id)
            .bind(reaction.user_id)
            .bind(&reaction.emoji)
            .bind(reaction.created_at)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(created)
    }

    async fn update_emoji(&self, id: Uuid, emoji: &str) -> Result<Reaction, DomainError> {
        let updated = sqlx::query_as::<_, Reaction>(
            "UPDATE reactions SET emoji = $1 WHERE id = $2 RETURNING *"
        )
            .bind(emoji)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM reactions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(())
    }
}

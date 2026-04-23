use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::Channel;
use crate::domain::errors::DomainError;
use crate::domain::repositories::ChannelRepository;

pub struct PgChannelRepository {
    pool: PgPool,
}

impl PgChannelRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ChannelRepository for PgChannelRepository {
    async fn create(&self, channel: &Channel) -> Result<Channel, DomainError> {
        let created = sqlx::query_as::<_, Channel>(
            r#"
            INSERT INTO channels (id, server_id, name, description, channel_type, position, is_private, icon, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#
        )
        .bind(channel.id)
        .bind(channel.server_id)
        .bind(&channel.name)
        .bind(&channel.description)
        .bind(&channel.channel_type)
        .bind(channel.position)
        .bind(channel.is_private)
        .bind(&channel.icon)
        .bind(channel.created_at)
        .bind(channel.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(created)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Channel>, DomainError> {
        let channel = sqlx::query_as::<_, Channel>("SELECT * FROM channels WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(channel)
    }

    async fn find_by_server_id(&self, server_id: Uuid) -> Result<Vec<Channel>, DomainError> {
        let channels = sqlx::query_as::<_, Channel>(
            "SELECT * FROM channels WHERE server_id = $1 ORDER BY position ASC"
        )
        .bind(server_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(channels)
    }

    async fn update(&self, channel: &Channel) -> Result<Channel, DomainError> {
        let updated = sqlx::query_as::<_, Channel>(
            r#"
            UPDATE channels 
            SET name = $2, description = $3, channel_type = $4, position = $5, is_private = $6, icon = $7, updated_at = $8
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(channel.id)
        .bind(&channel.name)
        .bind(&channel.description)
        .bind(&channel.channel_type)
        .bind(channel.position)
        .bind(channel.is_private)
        .bind(&channel.icon)
        .bind(channel.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM channels WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(())
    }
}

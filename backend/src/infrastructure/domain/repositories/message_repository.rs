use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::Message;
use crate::domain::errors::DomainError;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn create(&self, message: &Message) -> Result<Message, DomainError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Message>, DomainError>;
    async fn find_by_channel_id(&self, channel_id: Uuid, limit: i64, before: Option<Uuid>) -> Result<Vec<Message>, DomainError>;
    async fn update(&self, message: &Message) -> Result<Message, DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
}


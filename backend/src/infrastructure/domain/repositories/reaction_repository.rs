use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::Reaction;
use crate::domain::errors::DomainError;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ReactionRepository: Send + Sync {
    async fn find_by_message_id(&self, message_id: Uuid) -> Result<Vec<Reaction>, DomainError>;
    async fn find_by_message_ids(&self, message_ids: &[Uuid]) -> Result<Vec<Reaction>, DomainError>;
    async fn find_by_user_and_message(&self, user_id: Uuid, message_id: Uuid) -> Result<Option<Reaction>, DomainError>;
    async fn create(&self, reaction: &Reaction) -> Result<Reaction, DomainError>;
    async fn update_emoji(&self, id: Uuid, emoji: &str) -> Result<Reaction, DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
}

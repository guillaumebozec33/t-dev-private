use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::{Conversation, DirectMessage};
use crate::domain::errors::DomainError;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DmRepository: Send + Sync {
    async fn find_or_create_conversation(&self, user1_id: Uuid, user2_id: Uuid) -> Result<Conversation, DomainError>;
    async fn find_conversations_by_user(&self, user_id: Uuid) -> Result<Vec<Conversation>, DomainError>;
    async fn find_conversation_by_id(&self, id: Uuid) -> Result<Option<Conversation>, DomainError>;
    async fn create_dm_message(&self, msg: &DirectMessage) -> Result<DirectMessage, DomainError>;
    async fn find_messages_by_conversation(&self, conversation_id: Uuid, limit: i64, before: Option<Uuid>) -> Result<Vec<DirectMessage>, DomainError>;
    async fn find_message_by_id(&self, id: Uuid) -> Result<Option<DirectMessage>, DomainError>;
}

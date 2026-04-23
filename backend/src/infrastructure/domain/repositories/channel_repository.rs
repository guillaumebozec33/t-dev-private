use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::Channel;
use crate::domain::errors::DomainError;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ChannelRepository: Send + Sync {
    async fn create(&self, channel: &Channel) -> Result<Channel, DomainError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Channel>, DomainError>;
    async fn find_by_server_id(&self, server_id: Uuid) -> Result<Vec<Channel>, DomainError>;
    async fn update(&self, channel: &Channel) -> Result<Channel, DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
}

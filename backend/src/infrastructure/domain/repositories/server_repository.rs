use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::{Server, Member, Invitation, Ban};
use crate::domain::enums::Role;
use crate::domain::errors::DomainError;

#[derive(Debug, Clone)]
pub struct MemberWithUser {
    pub id: Uuid,
    pub user_id: Uuid,
    pub server_id: Uuid,
    pub role: Role,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub username: String,
    pub avatar_url: Option<String>,
    pub status: String,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ServerRepository: Send + Sync {
    async fn create(&self, server: &Server) -> Result<Server, DomainError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Server>, DomainError>;
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Server>, DomainError>;
    async fn update(&self, server: &Server) -> Result<Server, DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
    
    async fn add_member(&self, member: &Member) -> Result<Member, DomainError>;
    async fn find_member(&self, user_id: Uuid, server_id: Uuid) -> Result<Option<Member>, DomainError>;
    async fn find_members(&self, server_id: Uuid) -> Result<Vec<Member>, DomainError>;
    async fn find_members_with_users(&self, server_id: Uuid) -> Result<Vec<MemberWithUser>, DomainError>;
    async fn update_member_role(&self, member_id: Uuid, role: Role) -> Result<Member, DomainError>;
    async fn remove_member(&self, user_id: Uuid, server_id: Uuid) -> Result<(), DomainError>;
    
    async fn create_invitation(&self, invitation: &Invitation) -> Result<Invitation, DomainError>;
    async fn find_invitation_by_code(&self, code: &str) -> Result<Option<Invitation>, DomainError>;
    async fn increment_invitation_uses(&self, id: Uuid) -> Result<(), DomainError>;
    
    async fn create_ban(&self, ban: &Ban) -> Result<Ban, DomainError>;
    async fn find_active_ban(&self, user_id: Uuid, server_id: Uuid) -> Result<Option<Ban>, DomainError>;
    async fn find_bans_by_server(&self, server_id: Uuid) -> Result<Vec<Ban>, DomainError>;
    async fn remove_ban(&self, user_id: Uuid, server_id: Uuid) -> Result<(), DomainError>;
    async fn cleanup_expired_bans(&self) -> Result<(), DomainError>;
}

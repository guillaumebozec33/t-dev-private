use std::sync::Arc;
use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::application::dto::*;
use crate::domain::entities::{Server, Member, Invitation, Channel, Ban};
use crate::domain::enums::Role;
use crate::domain::errors::DomainError;
use crate::domain::repositories::{ServerRepository, ChannelRepository};

pub struct ServerService {
    server_repo: Arc<dyn ServerRepository>,
    channel_repo: Arc<dyn ChannelRepository>,
}

impl ServerService {
    pub fn new(server_repo: Arc<dyn ServerRepository>, channel_repo: Arc<dyn ChannelRepository>) -> Self {
        Self { server_repo, channel_repo }
    }

    pub async fn create_server(&self, user_id: Uuid, req: CreateServerRequest) -> Result<ServerResponse, DomainError> {
        let mut server = Server::new(req.name, user_id);
        server.description = req.description;
        if let Some(icon_url) = req.icon_url {
            server.icon_url = if icon_url.is_empty() { None } else { Some(icon_url) };
        }

        let created_server = self.server_repo.create(&server).await?;
        
        let member = Member::new(user_id, created_server.id, Role::Owner);
        self.server_repo.add_member(&member).await?;
        
        let channel = Channel::new(created_server.id, "general".to_string());
        self.channel_repo.create(&channel).await?;
        
        Ok(ServerResponse::from(created_server))
    }
    

    pub async fn get_user_servers(&self, user_id: Uuid) -> Result<Vec<ServerResponse>, DomainError> {
        let servers = self.server_repo.find_by_user_id(user_id).await?;
        Ok(servers.into_iter().map(ServerResponse::from).collect())
    }

    pub async fn get_server(&self, user_id: Uuid, server_id: Uuid) -> Result<ServerResponse, DomainError> {
        self.ensure_member(user_id, server_id).await?;
        
        let server = self.server_repo
            .find_by_id(server_id)
            .await?
            .ok_or(DomainError::ServerNotFound)?;
        
        Ok(ServerResponse::from(server))
    }

    pub async fn update_server(&self, user_id: Uuid, server_id: Uuid, req: UpdateServerRequest) -> Result<ServerResponse, DomainError> {
        let member = self.ensure_member(user_id, server_id).await?;
        
        if !member.is_owner() {
            return Err(DomainError::Forbidden("Only owner can update server".to_string()));
        }
        
        let mut server = self.server_repo
            .find_by_id(server_id)
            .await?
            .ok_or(DomainError::ServerNotFound)?;
        
        if let Some(name) = req.name {
            server.name = name;
        }
        if let Some(description) = req.description {
            server.description = Some(description);
        }
        if let Some(icon_url) = req.icon_url {
            server.icon_url = if icon_url.is_empty() { None } else { Some(icon_url) };
        }
        server.updated_at = Utc::now();
        
        let updated = self.server_repo.update(&server).await?;
        Ok(ServerResponse::from(updated))
    }

    pub async fn delete_server(&self, user_id: Uuid, server_id: Uuid) -> Result<(), DomainError> {
        let member = self.ensure_member(user_id, server_id).await?;
        
        if !member.is_owner() {
            return Err(DomainError::Forbidden("Only owner can delete server".to_string()));
        }
        
        self.server_repo.delete(server_id).await
    }

    pub async fn join_server(&self, user_id: Uuid, invite_code: &str) -> Result<ServerResponse, DomainError> {
        let invitation = self.server_repo
            .find_invitation_by_code(invite_code)
            .await?
            .ok_or(DomainError::InvitationNotFound)?;
        
        if !invitation.is_valid() {
            return Err(DomainError::InvitationNotFound);
        }

        if invitation.created_by == user_id{
            return Err(DomainError::UseOwnInvitation);
}
        
        // Check if user is banned
        if let Some(ban) = self.server_repo.find_active_ban(user_id, invitation.server_id).await? {
            if ban.is_permanent() {
                return Err(DomainError::UserBannedPermanently);
            } else {
                return Err(DomainError::UserBanned(ban.expires_at.to_rfc3339()));
            }
        }
        
        if self.server_repo.find_member(user_id, invitation.server_id).await?.is_some() {
            return Err(DomainError::AlreadyMember);
        }
        
        let member = Member::new(user_id, invitation.server_id, Role::Member);
        self.server_repo.add_member(&member).await?;
        
        self.server_repo.increment_invitation_uses(invitation.id).await?;
        
        let server = self.server_repo
            .find_by_id(invitation.server_id)
            .await?
            .ok_or(DomainError::ServerNotFound)?;
        
        Ok(ServerResponse::from(server))
    }

    pub async fn leave_server(&self, user_id: Uuid, server_id: Uuid) -> Result<(), DomainError> {
        let member = self.ensure_member(user_id, server_id).await?;
        
        if member.is_owner() {
            return Err(DomainError::OwnerCannotLeave);
        }
        
        self.server_repo.remove_member(user_id, server_id).await
    }
pub async fn kick_member(
    &self,
    requester_id: Uuid,
    server_id: Uuid,
    member_id: Uuid,
) -> Result<(), DomainError> {

    let requester = self.ensure_member(requester_id, server_id).await?;

    if member_id == requester_id {
        return Err(DomainError::ValidationError(
            "Cannot kick yourself".to_string()
        ));
    }

    let target = self.server_repo
        .find_member(member_id, server_id)
        .await?
        .ok_or(DomainError::MemberNotFound)?;

    if requester.is_owner() {
        // owner can kick anyone
    } else if requester.is_admin() {
        if target.role != Role::Member {
            return Err(DomainError::Forbidden(
                "Admins can only kick members".to_string()
            ));
        }
    } else {
        return Err(DomainError::Forbidden(
            "Only owner and admins can kick members".to_string()
        ));
    }

    self.server_repo.remove_member(member_id, server_id).await?;

    Ok(())
}

    pub async fn get_members(&self, user_id: Uuid, server_id: Uuid) -> Result<Vec<MemberResponse>, DomainError> {
        self.ensure_member(user_id, server_id).await?;
        
        let members = self.server_repo.find_members_with_users(server_id).await?;
        Ok(members
            .into_iter()
            .map(|m| MemberResponse::new(
                m.id.to_string(),
                m.user_id.to_string(),
                m.server_id.to_string(),
                m.role.to_string(),
                m.joined_at.to_rfc3339(),
                m.username,
                m.avatar_url,
                m.status,
            ))
            .collect())
    }

    pub async fn update_member_role(&self, user_id: Uuid, server_id: Uuid, target_user_id: Uuid, role: Role) -> Result<MemberResponse, DomainError> {
        let member = self.ensure_member(user_id, server_id).await?;
        
        if !member.is_owner() {
            return Err(DomainError::Forbidden("Only owner can manage roles".to_string()));
        }
        
        if role == Role::Owner {
            return Err(DomainError::Forbidden("Use transfer ownership instead".to_string()));
        }
        
        let target_member = self.server_repo
            .find_member(target_user_id, server_id)
            .await?
            .ok_or(DomainError::MemberNotFound)?;
        
        if target_member.is_owner() {
            return Err(DomainError::Forbidden("Cannot change owner role".to_string()));
        }
        
        let updated = self.server_repo.update_member_role(target_member.id, role).await?;
        
        let member_with_user = self.server_repo.find_members_with_users(server_id).await?
            .into_iter()
            .find(|m| m.id == updated.id)
            .ok_or(DomainError::MemberNotFound)?;
            
        Ok(MemberResponse::new(
            member_with_user.id.to_string(),
            member_with_user.user_id.to_string(),
            member_with_user.server_id.to_string(),
            member_with_user.role.to_string(),
            member_with_user.joined_at.to_rfc3339(),
            member_with_user.username,
            member_with_user.avatar_url,
            member_with_user.status,
        ))
    }

    pub async fn transfer_ownership(&self, user_id: Uuid, server_id: Uuid, new_owner_id: Uuid) -> Result<(), DomainError> {
        let member = self.ensure_member(user_id, server_id).await?;
        
        if !member.is_owner() {
            return Err(DomainError::Forbidden("Only owner can transfer ownership".to_string()));
        }
        
        let new_owner_member = self.server_repo
            .find_member(new_owner_id, server_id)
            .await?
            .ok_or(DomainError::MemberNotFound)?;
        
        self.server_repo.update_member_role(new_owner_member.id, Role::Owner).await?;
        self.server_repo.update_member_role(member.id, Role::Member).await?;
        
        let mut server = self.server_repo
            .find_by_id(server_id)
            .await?
            .ok_or(DomainError::ServerNotFound)?;
        server.owner_id = new_owner_id;
        self.server_repo.update(&server).await?;
        
        Ok(())
    }

    pub async fn create_invitation(&self, user_id: Uuid, server_id: Uuid, req: CreateInvitationRequest) -> Result<InvitationResponse, DomainError> {
        let member = self.ensure_member(user_id, server_id).await?;
        
        if !member.can_create_invitation() {
            return Err(DomainError::Forbidden("Only admins can create invitations".to_string()));
        }
        
        let expires_at = req.expires_in_hours.map(|h| Utc::now() + Duration::hours(h));
        let invitation = Invitation::new(server_id, user_id, req.max_uses, expires_at);
        
        let created = self.server_repo.create_invitation(&invitation).await?;
        
        Ok(InvitationResponse {
            code: created.code,
            server_id: created.server_id.to_string(),
            max_uses: created.max_uses,
            uses: created.uses,
            expires_at: created.expires_at.map(|e| e.to_rfc3339()),
        })
    }

    async fn ensure_member(&self, user_id: Uuid, server_id: Uuid) -> Result<Member, DomainError> {
        self.server_repo
            .find_member(user_id, server_id)
            .await?
            .ok_or(DomainError::Forbidden("Not a member of this server".to_string()))
    }

    pub async fn ban_member(
        &self,
        requester_id: Uuid,
        server_id: Uuid,
        member_id: Uuid,
        req: BanMemberRequest,
    ) -> Result<BanResponse, DomainError> {
        let requester = self.ensure_member(requester_id, server_id).await?;
        let target_member = self.server_repo
            .find_member(member_id, server_id)
            .await?
            .ok_or(DomainError::MemberNotFound)?;

        // Owner can ban everyone, Admin can only ban Members
        if requester.is_owner() {
            // Owner can ban anyone except themselves
            if member_id == requester_id {
                return Err(DomainError::ValidationError("Cannot ban yourself".to_string()));
            }
        } else if requester.is_admin() {
            // Admin can only ban Members (not Owner or other Admins)
            if target_member.role != Role::Member {
                return Err(DomainError::Forbidden(
                    "Admins can only ban members".to_string()
                ));
            }
        } else {
            return Err(DomainError::Forbidden(
                "Only owner and admins can ban members".to_string()
            ));
        }

        // Calculate expiration date
        let expires_at = if let Some(hours) = req.duration_hours {
            Utc::now() + Duration::hours(hours)
        } else {
            // Permanent ban = 1000 years
            Utc::now() + Duration::days(365 * 1000)
        };

        // Create ban
        let ban = Ban::new(member_id, server_id, requester_id, expires_at);
        let created_ban = self.server_repo.create_ban(&ban).await?;

        // Remove member from server (kick)
        self.server_repo.remove_member(member_id, server_id).await?;

        Ok(BanResponse::from(created_ban))
    }

    pub async fn unban_member(
        &self,
        requester_id: Uuid,
        server_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), DomainError> {
        let requester = self.ensure_member(requester_id, server_id).await?;

        if !requester.is_owner() && !requester.is_admin() {
            return Err(DomainError::Forbidden(
                "Only owner and admins can unban members".to_string()
            ));
        }

        self.server_repo.remove_ban(user_id, server_id).await?;
        Ok(())
    }

    pub async fn get_bans(
        &self,
        requester_id: Uuid,
        server_id: Uuid,
    ) -> Result<Vec<BanResponse>, DomainError> {
        let requester = self.ensure_member(requester_id, server_id).await?;

        // Only owner and admins can see bans
        if !requester.is_admin() {
            return Err(DomainError::Forbidden(
                "Only owner and admins can view bans".to_string()
            ));
        }

        let bans = self.server_repo.find_bans_by_server(server_id).await?;
        Ok(bans.into_iter().map(BanResponse::from).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use uuid::Uuid;
    use chrono::{Duration, Utc};
    use crate::domain::entities::{Server, Channel, Member, Invitation, Ban};
    use crate::domain::enums::Role;
    use crate::domain::repositories::server_repository::MockServerRepository;
    use crate::domain::repositories::channel_repository::MockChannelRepository;

    // ── Helpers ────────────────────────────────────────────────

    fn make_server(owner_id: Uuid) -> Server {
        Server {
            id: Uuid::new_v4(),
            name: "Mon serveur".to_string(),
            description: None,
            icon_url: None,
            owner_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_member(user_id: Uuid, server_id: Uuid, role: Role) -> Member {
        Member::new(user_id, server_id, role)
    }

    fn make_invitation(server_id: Uuid, user_id: Uuid, valid: bool) -> Invitation {
        let expires_at = if valid {
            Some(Utc::now() + chrono::Duration::days(1)) // pas encore expirée
        } else {
            Some(Utc::now() - chrono::Duration::days(1)) // déjà expirée
        };
        Invitation::new(server_id, user_id, Some(10), expires_at)
    }

    // ── create_server ──────────────────────────────────────────

    #[tokio::test]
    async fn test_create_server_success() {
        let user_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_create()
            .returning(move |_| Ok(make_server(user_id)));

        // add_member pour l'owner
        mock_server_repo
            .expect_add_member()
            .returning(move |_| Ok(make_member(user_id, Uuid::new_v4(), Role::Owner)));

        // channel "general" créé automatiquement
        mock_channel_repo
            .expect_create()
            .returning(move |_| Ok(Channel {
                id: Uuid::new_v4(),
                server_id: Uuid::new_v4(),
                name: "general".to_string(),
                description: None,
                channel_type: "text".to_string(),
                position: 0,
                is_private: false,
                icon: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let req = CreateServerRequest { name: "Mon serveur".to_string(), description: None };

        let result = service.create_server(user_id, req).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Mon serveur");
    }

    // ── get_server ─────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_server_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        // ensure_member → membre trouvé
        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        mock_server_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_server(user_id))));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.get_server(user_id, server_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_server_not_member() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(|_, _| Ok(None));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.get_server(user_id, server_id).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    // ── update_server ──────────────────────────────────────────

    #[tokio::test]
    async fn test_update_server_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Owner))));

        mock_server_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_server(user_id))));

        mock_server_repo
            .expect_update()
            .returning(move |_| Ok(make_server(user_id)));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let req = UpdateServerRequest {
            name: Some("Nouveau nom".to_string()),
            description: None,
            icon_url: None,
        };

        let result = service.update_server(user_id, server_id, req).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_server_not_owner() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        // Membre simple → pas owner → Forbidden
        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let req = UpdateServerRequest { name: None, description: None, icon_url: None };

        let result = service.update_server(user_id, server_id, req).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    // ── delete_server ──────────────────────────────────────────

    #[tokio::test]
    async fn test_delete_server_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Owner))));

        mock_server_repo
            .expect_delete()
            .returning(|_| Ok(()));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.delete_server(user_id, server_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_server_not_owner() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.delete_server(user_id, server_id).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    // ── join_server ────────────────────────────────────────────

    #[tokio::test]
    async fn test_join_server_success() {
        let user_id = Uuid::new_v4();
        let inviter_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();
        mock_server_repo
            .expect_find_active_ban()
            .returning(|_, _| Ok(None));
        mock_server_repo
            .expect_find_invitation_by_code()
            .returning(move |_| Ok(Some(make_invitation(server_id, inviter_id, true))));

        // Pas encore membre
        mock_server_repo
            .expect_find_member()
            .returning(|_, _| Ok(None));

        mock_server_repo
            .expect_add_member()
            .returning(move |_| Ok(make_member(user_id, Uuid::new_v4(), Role::Owner)));

        mock_server_repo
            .expect_increment_invitation_uses()
            .returning(|_| Ok(()));

        mock_server_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_server(user_id))));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.join_server(user_id, "CODE1234").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_join_server_invitation_invalide() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        // Invitation expirée → is_valid() = false
        mock_server_repo
            .expect_find_invitation_by_code()
            .returning(move |_| Ok(Some(make_invitation(server_id, user_id, false))));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.join_server(user_id, "EXPIRE").await;
        assert!(matches!(result, Err(DomainError::InvitationNotFound)));
    }

    #[tokio::test]
    async fn test_join_server_deja_membre() {
        let user_id = Uuid::new_v4();
        let inviter_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();
        mock_server_repo
            .expect_find_active_ban()
            .returning(|_, _| Ok(None));
        mock_server_repo
            .expect_find_invitation_by_code()
            .returning(move |_| Ok(Some(make_invitation(server_id, inviter_id, true))));

        // Déjà membre → AlreadyMember
        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.join_server(user_id, "CODE1234").await;
        assert!(matches!(result, Err(DomainError::AlreadyMember)));
    }

    #[tokio::test]
    async fn test_join_server_banned_temp() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_invitation_by_code()
            .returning(move |_| Ok(Some(make_invitation(server_id, creator_id, true))));

        // Temporary ban (expires in 1 hour)
        mock_server_repo
            .expect_find_active_ban()
            .returning(move |_, _| {
                let ban = Ban::new(
                    user_id,
                    server_id,
                    creator_id,
                    Utc::now() + Duration::hours(1),
                );
                Ok(Some(ban))
            });

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.join_server(user_id, "CODE1234").await;
        assert!(matches!(result, Err(DomainError::UserBanned(_))));
    }

    #[tokio::test]
    async fn test_join_server_banned_permanently() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_invitation_by_code()
            .returning(move |_| Ok(Some(make_invitation(server_id, creator_id, true))));

        // Permanent ban (1000 years)
        mock_server_repo
            .expect_find_active_ban()
            .returning(move |_, _| {
                let ban = Ban::new(
                    user_id,
                    server_id,
                    creator_id,
                    Utc::now() + Duration::days(365 * 1000),
                );
                Ok(Some(ban))
            });

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.join_server(user_id, "CODE1234").await;
        assert!(matches!(result, Err(DomainError::UserBannedPermanently)));
    }

    // ── leave_server ───────────────────────────────────────────

    #[tokio::test]
    async fn test_leave_server_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        mock_server_repo
            .expect_remove_member()
            .returning(|_, _| Ok(()));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.leave_server(user_id, server_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_leave_server_owner_cannot_leave() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        // Owner → OwnerCannotLeave
        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Owner))));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.leave_server(user_id, server_id).await;
        assert!(matches!(result, Err(DomainError::OwnerCannotLeave)));
    }

    // ── kick_member ────────────────────────────────────────────

    #[tokio::test]
    async fn test_kick_member_success() {
        let owner_id = Uuid::new_v4();
        let member_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        let mut server = make_server(owner_id);
        server.id = server_id;

        mock_server_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(server.clone())));

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(member_id, server_id, Role::Member))));

        mock_server_repo
            .expect_remove_member()
            .returning(|_, _| Ok(()));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.kick_member(owner_id, server_id, member_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_kick_member_not_owner() {
        let requester_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4(); // quelqu'un d'autre est owner
        let member_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        let mut server = make_server(owner_id); // owner != requester
        server.id = server_id;

        mock_server_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(server.clone())));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.kick_member(requester_id, server_id, member_id).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_kick_member_cannot_kick_yourself() {
        let owner_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        let mut server = make_server(owner_id);
        server.id = server_id;

        mock_server_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(server.clone())));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        // member_id == requester_id → ValidationError
        let result = service.kick_member(owner_id, server_id, owner_id).await;
        assert!(matches!(result, Err(DomainError::ValidationError(_))));
    }

    // ── create_invitation ──────────────────────────────────────

    #[tokio::test]
    async fn test_create_invitation_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        mock_server_repo
            .expect_create_invitation()
            .returning(move |_| Ok(make_invitation(server_id, user_id, true)));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let req = CreateInvitationRequest { max_uses: Some(5), expires_in_hours: Some(24) };

        let result = service.create_invitation(user_id, server_id, req).await;
        assert!(result.is_ok());
    }

    // ── get_user_servers ───────────────────────────────────────

    #[tokio::test]
    async fn test_get_user_servers_success() {
        let user_id = Uuid::new_v4();
        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_by_user_id()
            .returning(move |_| Ok(vec![make_server(user_id), make_server(user_id)]));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.get_user_servers(user_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    // ── get_members ────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_members_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Owner))));

        mock_server_repo
            .expect_find_members_with_users()
            .returning(move |_| Ok(vec![
                crate::domain::repositories::server_repository::MemberWithUser {
                    id: Uuid::new_v4(),
                    user_id,
                    server_id,
                    role: Role::Owner,
                    joined_at: Utc::now(),
                    username: "alice".to_string(),
                    avatar_url: None,
                    status: "online".to_string(),
                }
            ]));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.get_members(user_id, server_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    // ── ban_member ─────────────────────────────────────────────

    #[tokio::test]
    async fn test_ban_member_success() {
        let owner_id = Uuid::new_v4();
        let member_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .times(2)
            .returning(move |uid, _| {
                if uid == owner_id {
                    Ok(Some(make_member(owner_id, server_id, Role::Owner)))
                } else {
                    Ok(Some(make_member(member_id, server_id, Role::Member)))
                }
            });

        mock_server_repo
            .expect_create_ban()
            .returning(move |ban| Ok(ban.clone()));

        mock_server_repo
            .expect_remove_member()
            .returning(|_, _| Ok(()));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let req = BanMemberRequest { duration_hours: Some(24) };
        let result = service.ban_member(owner_id, server_id, member_id, req).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ban_member_permanent() {
        let owner_id = Uuid::new_v4();
        let member_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .times(2)
            .returning(move |uid, _| {
                if uid == owner_id {
                    Ok(Some(make_member(owner_id, server_id, Role::Owner)))
                } else {
                    Ok(Some(make_member(member_id, server_id, Role::Member)))
                }
            });

        mock_server_repo
            .expect_create_ban()
            .returning(move |ban| Ok(ban.clone()));

        mock_server_repo
            .expect_remove_member()
            .returning(|_, _| Ok(()));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let req = BanMemberRequest { duration_hours: None }; // permanent
        let result = service.ban_member(owner_id, server_id, member_id, req).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ban_member_cannot_ban_self() {
        let owner_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(owner_id, server_id, Role::Owner))));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let req = BanMemberRequest { duration_hours: Some(24) };
        let result = service.ban_member(owner_id, server_id, owner_id, req).await;
        assert!(matches!(result, Err(DomainError::ValidationError(_))));
    }

    #[tokio::test]
    async fn test_ban_member_not_privileged() {
        let user_id = Uuid::new_v4();
        let member_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let req = BanMemberRequest { duration_hours: Some(24) };
        let result = service.ban_member(user_id, server_id, member_id, req).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    // ── unban_member ───────────────────────────────────────────

    #[tokio::test]
    async fn test_unban_member_success() {
        let owner_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(owner_id, server_id, Role::Owner))));

        mock_server_repo
            .expect_remove_ban()
            .returning(|_, _| Ok(()));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.unban_member(owner_id, server_id, user_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unban_member_not_owner() {
        let user_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Admin))));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.unban_member(user_id, server_id, target_id).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    // ── get_bans ───────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_bans_success() {
        let owner_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(owner_id, server_id, Role::Owner))));

        mock_server_repo
            .expect_find_bans_by_server()
            .returning(|_| Ok(vec![]));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.get_bans(owner_id, server_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_bans_not_admin() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.get_bans(user_id, server_id).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    // ── update_member_role ─────────────────────────────────────

    #[tokio::test]
    async fn test_update_member_role_success() {
        let owner_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .times(2)
            .returning(move |uid, _| {
                if uid == owner_id {
                    Ok(Some(make_member(owner_id, server_id, Role::Owner)))
                } else {
                    Ok(Some(make_member(target_id, server_id, Role::Member)))
                }
            });

        let fixed_member_id = Uuid::new_v4();
        let fixed_member_id_clone = fixed_member_id;
        mock_server_repo
            .expect_update_member_role()
            .returning(move |_, _| {
                let mut m = make_member(target_id, server_id, Role::Admin);
                m.id = fixed_member_id;
                Ok(m)
            });

        mock_server_repo
            .expect_find_members_with_users()
            .returning(move |_| Ok(vec![
                crate::domain::repositories::server_repository::MemberWithUser {
                    id: fixed_member_id_clone,
                    user_id: target_id,
                    server_id,
                    role: Role::Admin,
                    joined_at: Utc::now(),
                    username: "bob".to_string(),
                    avatar_url: None,
                    status: "online".to_string(),
                }
            ]));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.update_member_role(owner_id, server_id, target_id, Role::Admin).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_member_role_not_owner() {
        let user_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Admin))));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.update_member_role(user_id, server_id, target_id, Role::Member).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_update_member_role_cannot_set_owner() {
        let owner_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(owner_id, server_id, Role::Owner))));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.update_member_role(owner_id, server_id, target_id, Role::Owner).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_update_member_role_cannot_change_owner_role() {
        let owner_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        // Requester is owner, target is also owner
        mock_server_repo
            .expect_find_member()
            .returning(move |uid, _| {
                if uid == owner_id {
                    Ok(Some(make_member(owner_id, server_id, Role::Owner)))
                } else {
                    Ok(Some(make_member(target_id, server_id, Role::Owner)))
                }
            });

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.update_member_role(owner_id, server_id, target_id, Role::Admin).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_join_server_own_invitation() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_invitation_by_code()
            .returning(move |_| Ok(Some(make_invitation(server_id, user_id, true))));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.join_server(user_id, "MYCODE").await;
        assert!(matches!(result, Err(DomainError::UseOwnInvitation)));
    }

    #[tokio::test]
    async fn test_update_server_with_description_and_icon() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Owner))));
        mock_server_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_server(user_id))));
        mock_server_repo
            .expect_update()
            .returning(move |_| Ok(make_server(user_id)));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let req = UpdateServerRequest {
            name: None,
            description: Some("A description".to_string()),
            icon_url: Some("https://example.com/icon.png".to_string()),
        };
        let result = service.update_server(user_id, server_id, req).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ban_member_admin_cannot_ban_admin() {
        let admin_id = Uuid::new_v4();
        let target_admin_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |uid, _| {
                if uid == admin_id {
                    Ok(Some(make_member(admin_id, server_id, Role::Admin)))
                } else {
                    Ok(Some(make_member(target_admin_id, server_id, Role::Admin)))
                }
            });

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let req = BanMemberRequest { duration_hours: Some(24) };
        let result = service.ban_member(admin_id, server_id, target_admin_id, req).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_transfer_ownership_not_owner() {
        let user_id = Uuid::new_v4();
        let new_owner_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Admin))));

        let service = ServerService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.transfer_ownership(user_id, server_id, new_owner_id).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }
}
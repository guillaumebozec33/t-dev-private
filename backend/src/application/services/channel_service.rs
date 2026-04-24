use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;

use crate::application::dto::*;
use crate::domain::entities::Channel;
use crate::domain::errors::DomainError;
use crate::domain::repositories::{ServerRepository, ChannelRepository};

pub struct ChannelService {
    server_repo: Arc<dyn ServerRepository>,
    channel_repo: Arc<dyn ChannelRepository>,
}

impl ChannelService {
    pub fn new(server_repo: Arc<dyn ServerRepository>, channel_repo: Arc<dyn ChannelRepository>) -> Self {
        Self { server_repo, channel_repo }
    }

    pub async fn create_channel(&self, user_id: Uuid, server_id: Uuid, req: CreateChannelRequest) -> Result<ChannelResponse, DomainError> {
        let member = self.server_repo
            .find_member(user_id, server_id)
            .await?
            .ok_or(DomainError::Forbidden("Not a member".to_string()))?;

        if !member.can_manage_channels() {
            return Err(DomainError::Forbidden("Only admins can create channels".to_string()));
        }

        let mut channel = Channel::new(server_id, req.name);
        channel.description = req.description;
        if let Some(channel_type) = req.channel_type {
            channel.channel_type = channel_type;
        }
        if let Some(is_private) = req.is_private {
            channel.is_private = is_private;
        }
        if let Some(icon) = req.icon {
            channel.icon = Some(icon);
        }

        let created = self.channel_repo.create(&channel).await?;
        Ok(ChannelResponse::from(created))
    }

    pub async fn get_channels(&self, user_id: Uuid, server_id: Uuid) -> Result<Vec<ChannelResponse>, DomainError> {
        let member = self.server_repo
            .find_member(user_id, server_id)
            .await?
            .ok_or(DomainError::Forbidden("Not a member".to_string()))?;

        let channels = self.channel_repo.find_by_server_id(server_id).await?;
        
        let filtered_channels: Vec<Channel> = channels
            .into_iter()
            .filter(|channel| {
                if !channel.is_private {
                    return true;
                }
                member.can_manage_channels()
            })
            .collect();
        
        Ok(filtered_channels.into_iter().map(ChannelResponse::from).collect())
    }

    pub async fn get_channel(&self, user_id: Uuid, channel_id: Uuid) -> Result<ChannelResponse, DomainError> {
        let channel = self.channel_repo
            .find_by_id(channel_id)
            .await?
            .ok_or(DomainError::ChannelNotFound)?;

        let member = self.server_repo
            .find_member(user_id, channel.server_id)
            .await?
            .ok_or(DomainError::Forbidden("Not a member".to_string()))?;

        if channel.is_private && !member.can_manage_channels() {
            return Err(DomainError::Forbidden("You don't have access to this private channel".to_string()));
        }

        Ok(ChannelResponse::from(channel))
    }

    pub async fn update_channel(&self, user_id: Uuid, channel_id: Uuid, req: UpdateChannelRequest) -> Result<ChannelResponse, DomainError> {
        let mut channel = self.channel_repo
            .find_by_id(channel_id)
            .await?
            .ok_or(DomainError::ChannelNotFound)?;

        let member = self.server_repo
            .find_member(user_id, channel.server_id)
            .await?
            .ok_or(DomainError::Forbidden("Not a member".to_string()))?;

        if !member.can_manage_channels() {
            return Err(DomainError::Forbidden("Only admins can update channels".to_string()));
        }

        if let Some(name) = req.name {
            channel.name = name;
        }
        if let Some(description) = req.description {
            channel.description = Some(description);
        }
        if let Some(position) = req.position {
            channel.position = position;
        }
        if let Some(is_private) = req.is_private {
            channel.is_private = is_private;
        }
        if let Some(icon) = req.icon {
            channel.icon = Some(icon);
        }
        channel.updated_at = Utc::now();

        let updated = self.channel_repo.update(&channel).await?;
        Ok(ChannelResponse::from(updated))
    }

    pub async fn delete_channel(&self, user_id: Uuid, channel_id: Uuid) -> Result<(), DomainError> {
        let channel = self.channel_repo
            .find_by_id(channel_id)
            .await?
            .ok_or(DomainError::ChannelNotFound)?;

        let member = self.server_repo
            .find_member(user_id, channel.server_id)
            .await?
            .ok_or(DomainError::Forbidden("Not a member".to_string()))?;

        if !member.can_manage_channels() {
            return Err(DomainError::Forbidden("Only admins can delete channels".to_string()));
        }

        self.channel_repo.delete(channel_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use uuid::Uuid;
    use chrono::Utc;
    use crate::domain::entities::{Channel, Member};
    use crate::domain::enums::Role;
    use crate::domain::repositories::server_repository::MockServerRepository;
    use crate::domain::repositories::channel_repository::MockChannelRepository;

    // Données réutilisables dans tous les tests

    fn make_channel(server_id: Uuid) -> Channel {
        Channel {
            id: Uuid::new_v4(),
            server_id,
            name: "general".to_string(),
            description: None,
            channel_type: "text".to_string(),
            position: 0,
            is_private: false,
            icon: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_member(user_id: Uuid, server_id: Uuid, role: Role) -> Member {
        Member::new(user_id, server_id, role)
    }

    // ── create_channel ─────────────────────────────────────────

    // Cas 1 : un admin crée un channel → ça doit marcher
    #[tokio::test]
    async fn test_create_channel_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();

        // find_member retourne un membre Admin
        // (Admin peut gérer les channels donc can_manage_channels() = true)
        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Admin))));

        // create retourne le channel créé
        mock_channel_repo
            .expect_create()
            .returning(move |_| Ok(make_channel(server_id)));

        let service = ChannelService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let req = CreateChannelRequest {
            name: "general".to_string(),
            description: None,
            channel_type: Some("voice".to_string()),
            is_private: Some(false),
        };

        let result = service.create_channel(user_id, server_id, req).await;
        assert!(result.is_ok());
    }

    // Cas 2 : l'utilisateur n'est pas membre du serveur → Forbidden
    #[tokio::test]
    async fn test_create_channel_not_member() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        // find_member retourne None = pas membre
        mock_server_repo
            .expect_find_member()
            .returning(|_, _| Ok(None));

        let service = ChannelService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let req = CreateChannelRequest {
            name: "general".to_string(),
            description: None,
            channel_type: Some("voice".to_string()),
            is_private: Some(false),
        };

        let result = service.create_channel(user_id, server_id, req).await;
        // On vérifie que l'erreur est bien Forbidden
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    // Cas 3 : l'utilisateur est membre mais pas admin → Forbidden
    #[tokio::test]
    async fn test_create_channel_not_admin() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        // find_member retourne un membre simple (Role::Member)
        // can_manage_channels() = false pour un Member
        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        let service = ChannelService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let req = CreateChannelRequest {
            name: "general".to_string(),
            description: None,
            channel_type: Some("voice".to_string()),
            is_private: Some(false),
        };

        let result = service.create_channel(user_id, server_id, req).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    // ── get_channel ────────────────────────────────────────────

    // Cas 1 : channel trouvé + membre → retourne le channel
    #[tokio::test]
    async fn test_get_channel_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_channel(server_id))));

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        let service = ChannelService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.get_channel(user_id, channel_id).await;
        assert!(result.is_ok());
    }

    // Cas 2 : channel introuvable → ChannelNotFound
    #[tokio::test]
    async fn test_get_channel_not_found() {
        let user_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();

        let mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();

        // find_by_id retourne None = channel inexistant
        mock_channel_repo
            .expect_find_by_id()
            .returning(|_| Ok(None));

        let service = ChannelService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.get_channel(user_id, channel_id).await;
        assert!(matches!(result, Err(DomainError::ChannelNotFound)));
    }

    // ── delete_channel ─────────────────────────────────────────

    #[tokio::test]
    async fn test_delete_channel_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_channel(server_id))));

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Admin))));

        mock_channel_repo
            .expect_delete()
            .returning(|_| Ok(()));

        let service = ChannelService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.delete_channel(user_id, channel_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_channel_not_admin() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_channel(server_id))));

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        let service = ChannelService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.delete_channel(user_id, channel_id).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    // ── get_channel private ────────────────────────────────────

    #[tokio::test]
    async fn test_get_channel_private_denied() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();

        let mut private_channel = make_channel(server_id);
        private_channel.is_private = true;

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(private_channel.clone())));

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        let service = ChannelService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.get_channel(user_id, channel_id).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    // ── update_channel ─────────────────────────────────────────

    #[tokio::test]
    async fn test_update_channel_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_channel(server_id))));

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Admin))));

        mock_channel_repo
            .expect_update()
            .returning(move |c| Ok(c.clone()));

        let service = ChannelService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let req = UpdateChannelRequest {
            name: Some("nouveau-nom".to_string()),
            description: None,
            position: None,
            is_private: None,
        };
        let result = service.update_channel(user_id, channel_id, req).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_channel_not_admin() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_channel(server_id))));

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        let service = ChannelService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let req = UpdateChannelRequest {
            name: Some("nom".to_string()),
            description: None,
            position: None,
            is_private: None,
        };
        let result = service.update_channel(user_id, channel_id, req).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    // ── get_channels ───────────────────────────────────────────

    #[tokio::test]
    async fn test_get_channels_member_sees_only_public() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        let mut private_ch = make_channel(server_id);
        private_ch.is_private = true;
        let public_ch = make_channel(server_id);

        mock_channel_repo
            .expect_find_by_server_id()
            .returning(move |_| Ok(vec![public_ch.clone(), private_ch.clone()]));

        let service = ChannelService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.get_channels(user_id, server_id).await.unwrap();
        assert_eq!(result.len(), 1); // only public
    }

    #[tokio::test]
    async fn test_get_channels_not_member() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(|_, _| Ok(None));

        let service = ChannelService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let result = service.get_channels(user_id, server_id).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_update_channel_all_optional_fields() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_channel(server_id))));
        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Admin))));
        mock_channel_repo
            .expect_update()
            .returning(move |c| Ok(c.clone()));

        let service = ChannelService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let req = UpdateChannelRequest {
            name: Some("new".to_string()),
            description: Some("desc".to_string()),
            position: Some(2),
            is_private: Some(true),
        };
        let result = service.update_channel(user_id, channel_id, req).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_channel_with_channel_type() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Admin))));
        mock_channel_repo
            .expect_create()
            .returning(move |_| Ok(make_channel(server_id)));

        let service = ChannelService::new(Arc::new(mock_server_repo), Arc::new(mock_channel_repo));
        let req = CreateChannelRequest {
            name: "voice".to_string(),
            description: None,
            channel_type: Some("voice".to_string()),
            is_private: None,
        };

        let result = service.create_channel(user_id, server_id, req).await;
        assert!(result.is_ok());
    }
}
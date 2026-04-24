use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::*;
use crate::domain::entities::Message;
use crate::domain::errors::DomainError;
use crate::domain::repositories::{ServerRepository, ChannelRepository, MessageRepository, UserRepository};

pub struct MessageService {
    server_repo: Arc<dyn ServerRepository>,
    channel_repo: Arc<dyn ChannelRepository>,
    message_repo: Arc<dyn MessageRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl MessageService {
    pub fn new(
        server_repo: Arc<dyn ServerRepository>,
        channel_repo: Arc<dyn ChannelRepository>,
        message_repo: Arc<dyn MessageRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self { server_repo, channel_repo, message_repo, user_repo }
    }

    pub async fn send_message(&self, user_id: Uuid, channel_id: Uuid, req: CreateMessageRequest) -> Result<MessageResponse, DomainError> {
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

        let message = Message::new(channel_id, user_id, req.content);
        let created = self.message_repo.create(&message).await?;
        
        let mut response = MessageResponse::from(created.clone());
        
        if let Ok(Some(author)) = self.user_repo.find_by_id(created.author_id).await {
            response.author_username = Some(author.username);
            response.author_avatar_url = author.avatar_url;
        }
        
        Ok(response)
    }

    pub async fn get_messages(&self, user_id: Uuid, channel_id: Uuid, limit: i64, before: Option<Uuid>) -> Result<Vec<MessageResponse>, DomainError> {
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

        let messages = self.message_repo.find_by_channel_id(channel_id, limit, before).await?;
        
        let mut responses = Vec::new();
        for message in messages {
            let mut response = MessageResponse::from(message.clone());
            
            if let Ok(Some(author)) = self.user_repo.find_by_id(message.author_id).await {
                response.author_username = Some(author.username);
                response.author_avatar_url = author.avatar_url;
            }
            
            responses.push(response);
        }
        
        Ok(responses)
    }

    pub async fn delete_message(&self, user_id: Uuid, message_id: Uuid) -> Result<Uuid, DomainError> {
        let message = self.message_repo
            .find_by_id(message_id)
            .await?
            .ok_or(DomainError::MessageNotFound)?;

        let channel = self.channel_repo
            .find_by_id(message.channel_id)
            .await?
            .ok_or(DomainError::ChannelNotFound)?;

        let member = self.server_repo
            .find_member(user_id, channel.server_id)
            .await?
            .ok_or(DomainError::Forbidden("Not a member".to_string()))?;

        if !member.can_delete_message(message.author_id) {
            return Err(DomainError::Forbidden("Cannot delete this message".to_string()));
        }

        let channel_id = message.channel_id;
        self.message_repo.delete(message_id).await?;
        Ok(channel_id)
    }

    pub async fn edit_message(
        &self, 
        user_id: Uuid, 
        message_id: Uuid, 
        req: UpdateMessageRequest
    ) -> Result<MessageResponse, DomainError> {
        let mut message = self.message_repo
            .find_by_id(message_id)
            .await?
            .ok_or(DomainError::MessageNotFound)?;

        if message.author_id != user_id {
            return Err(DomainError::Forbidden(
                "You can only edit your own messages".to_string()
            ));
        }

        let channel = self.channel_repo
            .find_by_id(message.channel_id)
            .await?
            .ok_or(DomainError::ChannelNotFound)?;

        self.server_repo
            .find_member(user_id, channel.server_id)
            .await?
            .ok_or(DomainError::Forbidden("Not a member".to_string()))?;

        message.mark_as_edited(req.content);
        let updated = self.message_repo.update(&message).await?;
        
        let mut response = MessageResponse::from(updated.clone());
        
        if let Ok(Some(author)) = self.user_repo.find_by_id(updated.author_id).await {
            response.author_username = Some(author.username);
            response.author_avatar_url = author.avatar_url;
        }
        
        Ok(response)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use uuid::Uuid;
    use chrono::Utc;
    use crate::domain::entities::{Channel, Message, Member, User};
    use crate::domain::enums::{Role, UserStatus};
    use crate::domain::repositories::server_repository::MockServerRepository;
    use crate::domain::repositories::channel_repository::MockChannelRepository;
    use crate::domain::repositories::message_repository::MockMessageRepository;
    use crate::domain::repositories::user_repository::MockUserRepository;

    // ── Helpers ────────────────────────────────────────────────

    fn make_channel(server_id: Uuid) -> Channel {
        Channel {
            id: Uuid::new_v4(),
            server_id,
            name: "general".to_string(),
            description: None,
            channel_type: "text".to_string(),
            position: 0,
            is_private: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_message(channel_id: Uuid, author_id: Uuid) -> Message {
        Message {
            id: Uuid::new_v4(),
            channel_id,
            author_id,
            content: "Hello !".to_string(),
            edited: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_member(user_id: Uuid, server_id: Uuid, role: Role) -> Member {
        Member::new(user_id, server_id, role)
    }

    fn make_user(user_id: Uuid) -> User {
        User {
            id: user_id,
            username: "guillaume".to_string(),
            email: "guillaume@test.com".to_string(),
            password_hash: "hash".to_string(),
            avatar_url: None,
            status: UserStatus::Online,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // ── send_message ───────────────────────────────────────────

    // Cas 1 : envoi normal → ok
    #[tokio::test]
    async fn test_send_message_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();

        let mut mock_channel_repo = MockChannelRepository::new();
        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_message_repo = MockMessageRepository::new();
        let mut mock_user_repo = MockUserRepository::new();

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_channel(server_id))));

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        mock_message_repo
            .expect_create()
            .returning(move |_| Ok(make_message(channel_id, user_id)));

        // find_by_id pour récupérer l'auteur du message
        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id))));

        let service = MessageService::new(
            Arc::new(mock_server_repo),
            Arc::new(mock_channel_repo),
            Arc::new(mock_message_repo),
            Arc::new(mock_user_repo),
        );

        let req = CreateMessageRequest { content: "Hello !".to_string() };
        let result = service.send_message(user_id, channel_id, req).await;
        assert!(result.is_ok());
        // L'auteur doit être renseigné dans la réponse
        assert_eq!(result.unwrap().author_username, Some("guillaume".to_string()));
    }

    // Cas 2 : channel inexistant → ChannelNotFound
    #[tokio::test]
    async fn test_send_message_channel_not_found() {
        let user_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();

        let mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();
        let mock_message_repo = MockMessageRepository::new();
        let mock_user_repo = MockUserRepository::new();

        mock_channel_repo
            .expect_find_by_id()
            .returning(|_| Ok(None)); // channel introuvable

        let service = MessageService::new(
            Arc::new(mock_server_repo),
            Arc::new(mock_channel_repo),
            Arc::new(mock_message_repo),
            Arc::new(mock_user_repo),
        );

        let req = CreateMessageRequest { content: "Hello !".to_string() };
        let result = service.send_message(user_id, channel_id, req).await;
        assert!(matches!(result, Err(DomainError::ChannelNotFound)));
    }

    // Cas 3 : pas membre du serveur → Forbidden
    #[tokio::test]
    async fn test_send_message_not_member() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();
        let mock_message_repo = MockMessageRepository::new();
        let mock_user_repo = MockUserRepository::new();

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_channel(server_id))));

        mock_server_repo
            .expect_find_member()
            .returning(|_, _| Ok(None)); // pas membre

        let service = MessageService::new(
            Arc::new(mock_server_repo),
            Arc::new(mock_channel_repo),
            Arc::new(mock_message_repo),
            Arc::new(mock_user_repo),
        );

        let req = CreateMessageRequest { content: "Hello !".to_string() };
        let result = service.send_message(user_id, channel_id, req).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    // ── delete_message ─────────────────────────────────────────

    // Cas 1 : auteur supprime son propre message → ok
    #[tokio::test]
    async fn test_delete_message_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();
        let mut mock_message_repo = MockMessageRepository::new();
        let mock_user_repo = MockUserRepository::new();

        // Le message appartient à user_id
        mock_message_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_message(channel_id, user_id))));

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_channel(server_id))));

        // user_id est membre → can_delete_message(user_id) = true car même auteur
        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        mock_message_repo
            .expect_delete()
            .returning(|_| Ok(()));

        let service = MessageService::new(
            Arc::new(mock_server_repo),
            Arc::new(mock_channel_repo),
            Arc::new(mock_message_repo),
            Arc::new(mock_user_repo),
        );

        let result = service.delete_message(user_id, message_id).await;
        assert!(result.is_ok());
    }

    // Cas 2 : tente de supprimer le message d'un autre sans être admin → Forbidden
    #[tokio::test]
    async fn test_delete_message_forbidden() {
        let user_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4(); // auteur du message
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();
        let mut mock_message_repo = MockMessageRepository::new();
        let mock_user_repo = MockUserRepository::new();

        // Le message appartient à other_user_id, pas à user_id
        mock_message_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_message(channel_id, other_user_id))));

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_channel(server_id))));

        // user_id est simple membre → can_delete_message(other_user_id) = false
        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        let service = MessageService::new(
            Arc::new(mock_server_repo),
            Arc::new(mock_channel_repo),
            Arc::new(mock_message_repo),
            Arc::new(mock_user_repo),
        );

        let result = service.delete_message(user_id, message_id).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    // Cas 3 : message inexistant → MessageNotFound
    #[tokio::test]
    async fn test_delete_message_not_found() {
        let user_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();

        let mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();
        let mut mock_message_repo = MockMessageRepository::new();
        let mock_user_repo = MockUserRepository::new();

        mock_message_repo
            .expect_find_by_id()
            .returning(|_| Ok(None));

        let service = MessageService::new(
            Arc::new(mock_server_repo),
            Arc::new(mock_channel_repo),
            Arc::new(mock_message_repo),
            Arc::new(mock_user_repo),
        );

        let result = service.delete_message(user_id, message_id).await;
        assert!(matches!(result, Err(DomainError::MessageNotFound)));
    }

    // ── edit_message ───────────────────────────────────────────

    // Cas 1 : auteur édite son message → ok
    #[tokio::test]
    async fn test_edit_message_success() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();

        let mut mock_server_repo = MockServerRepository::new();
        let mut mock_channel_repo = MockChannelRepository::new();
        let mut mock_message_repo = MockMessageRepository::new();
        let mut mock_user_repo = MockUserRepository::new();

        mock_message_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_message(channel_id, user_id))));

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_channel(server_id))));

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        mock_message_repo
            .expect_update()
            .returning(move |_| Ok(make_message(channel_id, user_id)));

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id))));

        let service = MessageService::new(
            Arc::new(mock_server_repo),
            Arc::new(mock_channel_repo),
            Arc::new(mock_message_repo),
            Arc::new(mock_user_repo),
        );

        let req = UpdateMessageRequest { content: "Message édité".to_string() };
        let result = service.edit_message(user_id, message_id, req).await;
        assert!(result.is_ok());
    }

    // ── send_message private channel ──────────────────────────

    #[tokio::test]
    async fn test_send_message_private_channel_forbidden() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();

        let mut mock_channel_repo = MockChannelRepository::new();
        let mut mock_server_repo = MockServerRepository::new();
        let mock_message_repo = MockMessageRepository::new();
        let mock_user_repo = MockUserRepository::new();

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| {
                let mut ch = make_channel(server_id);
                ch.is_private = true;
                Ok(Some(ch))
            });

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        let service = MessageService::new(
            Arc::new(mock_server_repo),
            Arc::new(mock_channel_repo),
            Arc::new(mock_message_repo),
            Arc::new(mock_user_repo),
        );

        let req = CreateMessageRequest { content: "Hello !".to_string() };
        let result = service.send_message(user_id, channel_id, req).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_get_messages_private_channel_forbidden() {
        let user_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();

        let mut mock_channel_repo = MockChannelRepository::new();
        let mut mock_server_repo = MockServerRepository::new();
        let mock_message_repo = MockMessageRepository::new();
        let mock_user_repo = MockUserRepository::new();

        mock_channel_repo
            .expect_find_by_id()
            .returning(move |_| {
                let mut ch = make_channel(server_id);
                ch.is_private = true;
                Ok(Some(ch))
            });

        mock_server_repo
            .expect_find_member()
            .returning(move |_, _| Ok(Some(make_member(user_id, server_id, Role::Member))));

        let service = MessageService::new(
            Arc::new(mock_server_repo),
            Arc::new(mock_channel_repo),
            Arc::new(mock_message_repo),
            Arc::new(mock_user_repo),
        );

        let result = service.get_messages(user_id, channel_id, 50, None).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    // Cas 2 : tente d'éditer le message d'un autre → Forbidden
    #[tokio::test]
    async fn test_edit_message_not_author() {
        let user_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();
        let channel_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();

        let mock_server_repo = MockServerRepository::new();
        let mock_channel_repo = MockChannelRepository::new();
        let mut mock_message_repo = MockMessageRepository::new();
        let mock_user_repo = MockUserRepository::new();

        // Le message appartient à other_user_id
        mock_message_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_message(channel_id, other_user_id))));

        let service = MessageService::new(
            Arc::new(mock_server_repo),
            Arc::new(mock_channel_repo),
            Arc::new(mock_message_repo),
            Arc::new(mock_user_repo),
        );

        let req = UpdateMessageRequest { content: "Tentative d'édition".to_string() };
        let result = service.edit_message(user_id, message_id, req).await;
        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }
}
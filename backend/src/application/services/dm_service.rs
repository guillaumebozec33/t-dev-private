use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::*;
use crate::domain::entities::DirectMessage;
use crate::domain::errors::DomainError;
use crate::domain::repositories::{DmRepository, UserRepository};

pub struct DmService {
    dm_repo: Arc<dyn DmRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl DmService {
    pub fn new(dm_repo: Arc<dyn DmRepository>, user_repo: Arc<dyn UserRepository>) -> Self {
        Self { dm_repo, user_repo }
    }

    pub async fn open_conversation(&self, user_id: Uuid, req: OpenConversationRequest) -> Result<ConversationResponse, DomainError> {
        if user_id == req.user_id {
            return Err(DomainError::Forbidden("Cannot open a conversation with yourself".to_string()));
        }

        let other_user = self.user_repo.find_by_id(req.user_id).await?.ok_or(DomainError::UserNotFound)?;
        let conversation = self.dm_repo.find_or_create_conversation(user_id, req.user_id).await?;

        Ok(ConversationResponse {
            id: conversation.id.to_string(),
            other_user_id: other_user.id.to_string(),
            other_username: other_user.username,
            other_avatar_url: other_user.avatar_url,
            created_at: conversation.created_at.to_rfc3339(),
        })
    }

    pub async fn list_conversations(&self, user_id: Uuid) -> Result<Vec<ConversationResponse>, DomainError> {
        let conversations = self.dm_repo.find_conversations_by_user(user_id).await?;
        let mut responses = Vec::new();

        for conv in conversations {
            let other_id = conv.other_user(user_id);
            if let Ok(Some(other_user)) = self.user_repo.find_by_id(other_id).await {
                responses.push(ConversationResponse {
                    id: conv.id.to_string(),
                    other_user_id: other_user.id.to_string(),
                    other_username: other_user.username,
                    other_avatar_url: other_user.avatar_url,
                    created_at: conv.created_at.to_rfc3339(),
                });
            }
        }

        Ok(responses)
    }

    pub async fn send_message(&self, user_id: Uuid, conversation_id: Uuid, req: SendDmRequest) -> Result<DmMessageResponse, DomainError> {
        let conversation = self.dm_repo
            .find_conversation_by_id(conversation_id)
            .await?
            .ok_or(DomainError::ConversationNotFound)?;

        if conversation.user1_id != user_id && conversation.user2_id != user_id {
            return Err(DomainError::Forbidden("Not a participant in this conversation".to_string()));
        }

        let msg = DirectMessage::new(conversation_id, user_id, req.content);
        let created = self.dm_repo.create_dm_message(&msg).await?;

        let mut response = DmMessageResponse::from(created);
        if let Ok(Some(sender)) = self.user_repo.find_by_id(user_id).await {
            response.sender_username = Some(sender.username);
            response.sender_avatar_url = sender.avatar_url;
        }

        Ok(response)
    }

    pub async fn get_messages(&self, user_id: Uuid, conversation_id: Uuid, limit: i64, before: Option<Uuid>) -> Result<Vec<DmMessageResponse>, DomainError> {
        let conversation = self.dm_repo
            .find_conversation_by_id(conversation_id)
            .await?
            .ok_or(DomainError::ConversationNotFound)?;

        if conversation.user1_id != user_id && conversation.user2_id != user_id {
            return Err(DomainError::Forbidden("Not a participant in this conversation".to_string()));
        }

        let messages = self.dm_repo.find_messages_by_conversation(conversation_id, limit, before).await?;
        let mut responses = Vec::new();

        for msg in messages {
            let mut response = DmMessageResponse::from(msg.clone());
            if let Ok(Some(sender)) = self.user_repo.find_by_id(msg.sender_id).await {
                response.sender_username = Some(sender.username);
                response.sender_avatar_url = sender.avatar_url;
            }
            responses.push(response);
        }

        Ok(responses)
    }

    pub async fn get_other_user_id(&self, user_id: Uuid, conversation_id: Uuid) -> Result<Uuid, DomainError> {
        let conversation = self.dm_repo
            .find_conversation_by_id(conversation_id)
            .await?
            .ok_or(DomainError::ConversationNotFound)?;

        if conversation.user1_id != user_id && conversation.user2_id != user_id {
            return Err(DomainError::Forbidden("Not a participant in this conversation".to_string()));
        }

        Ok(conversation.other_user(user_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use uuid::Uuid;
    use chrono::Utc;
    use crate::domain::entities::{Conversation, DirectMessage, User};
    use crate::domain::enums::UserStatus;
    use crate::domain::repositories::dm_repository::MockDmRepository;
    use crate::domain::repositories::user_repository::MockUserRepository;

    fn make_user(id: Uuid, username: &str) -> User {
        User {
            id,
            username: username.to_string(),
            email: format!("{}@test.com", username),
            password_hash: "hash".to_string(),
            avatar_url: None,
            status: UserStatus::Online,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_conversation(user1_id: Uuid, user2_id: Uuid) -> Conversation {
        Conversation {
            id: Uuid::new_v4(),
            user1_id,
            user2_id,
            created_at: Utc::now(),
        }
    }

    fn make_dm(conversation_id: Uuid, sender_id: Uuid) -> DirectMessage {
        DirectMessage {
            id: Uuid::new_v4(),
            conversation_id,
            sender_id,
            content: "Hello !".to_string(),
            created_at: Utc::now(),
        }
    }

    // ── open_conversation ──────────────────────────────────────

    #[tokio::test]
    async fn test_open_conversation_success() {
        let user_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();

        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_dm_repo = MockDmRepository::new();

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(other_id, "alice"))));

        mock_dm_repo
            .expect_find_or_create_conversation()
            .returning(move |u1, u2| Ok(make_conversation(u1, u2)));

        let service = DmService::new(Arc::new(mock_dm_repo), Arc::new(mock_user_repo));
        let req = OpenConversationRequest { user_id: other_id };
        let result = service.open_conversation(user_id, req).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().other_username, "alice");
    }

    #[tokio::test]
    async fn test_open_conversation_avec_soi_meme() {
        let user_id = Uuid::new_v4();

        let service = DmService::new(
            Arc::new(MockDmRepository::new()),
            Arc::new(MockUserRepository::new()),
        );
        let req = OpenConversationRequest { user_id };
        let result = service.open_conversation(user_id, req).await;

        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_open_conversation_utilisateur_introuvable() {
        let user_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();

        let mut mock_user_repo = MockUserRepository::new();
        mock_user_repo
            .expect_find_by_id()
            .returning(|_| Ok(None));

        let service = DmService::new(
            Arc::new(MockDmRepository::new()),
            Arc::new(mock_user_repo),
        );
        let req = OpenConversationRequest { user_id: other_id };
        let result = service.open_conversation(user_id, req).await;

        assert!(matches!(result, Err(DomainError::UserNotFound)));
    }

    // ── send_message ───────────────────────────────────────────

    #[tokio::test]
    async fn test_send_message_success() {
        let user_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();
        let conv_id = Uuid::new_v4();

        let mut mock_dm_repo = MockDmRepository::new();
        let mut mock_user_repo = MockUserRepository::new();

        mock_dm_repo
            .expect_find_conversation_by_id()
            .returning(move |_| Ok(Some(make_conversation(user_id, other_id))));

        mock_dm_repo
            .expect_create_dm_message()
            .returning(move |_| Ok(make_dm(conv_id, user_id)));

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id, "bob"))));

        let service = DmService::new(Arc::new(mock_dm_repo), Arc::new(mock_user_repo));
        let req = SendDmRequest { content: "Hello !".to_string() };
        let result = service.send_message(user_id, conv_id, req).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().sender_username, Some("bob".to_string()));
    }

    #[tokio::test]
    async fn test_send_message_conversation_introuvable() {
        let user_id = Uuid::new_v4();
        let conv_id = Uuid::new_v4();

        let mut mock_dm_repo = MockDmRepository::new();
        mock_dm_repo
            .expect_find_conversation_by_id()
            .returning(|_| Ok(None));

        let service = DmService::new(Arc::new(mock_dm_repo), Arc::new(MockUserRepository::new()));
        let req = SendDmRequest { content: "Hello !".to_string() };
        let result = service.send_message(user_id, conv_id, req).await;

        assert!(matches!(result, Err(DomainError::ConversationNotFound)));
    }

    #[tokio::test]
    async fn test_send_message_pas_participant() {
        let user_id = Uuid::new_v4();
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();
        let conv_id = Uuid::new_v4();

        let mut mock_dm_repo = MockDmRepository::new();
        mock_dm_repo
            .expect_find_conversation_by_id()
            .returning(move |_| Ok(Some(make_conversation(user1, user2))));

        let service = DmService::new(Arc::new(mock_dm_repo), Arc::new(MockUserRepository::new()));
        let req = SendDmRequest { content: "Hello !".to_string() };
        let result = service.send_message(user_id, conv_id, req).await;

        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    // ── get_messages ───────────────────────────────────────────

    #[tokio::test]
    async fn test_get_messages_success() {
        let user_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();
        let conv_id = Uuid::new_v4();

        let mut mock_dm_repo = MockDmRepository::new();
        let mut mock_user_repo = MockUserRepository::new();

        mock_dm_repo
            .expect_find_conversation_by_id()
            .returning(move |_| Ok(Some(make_conversation(user_id, other_id))));

        mock_dm_repo
            .expect_find_messages_by_conversation()
            .returning(move |_, _, _| Ok(vec![make_dm(conv_id, user_id)]));

        mock_user_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_user(user_id, "bob"))));

        let service = DmService::new(Arc::new(mock_dm_repo), Arc::new(mock_user_repo));
        let result = service.get_messages(user_id, conv_id, 50, None).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_get_messages_pas_participant() {
        let user_id = Uuid::new_v4();
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();
        let conv_id = Uuid::new_v4();

        let mut mock_dm_repo = MockDmRepository::new();
        mock_dm_repo
            .expect_find_conversation_by_id()
            .returning(move |_| Ok(Some(make_conversation(user1, user2))));

        let service = DmService::new(Arc::new(mock_dm_repo), Arc::new(MockUserRepository::new()));
        let result = service.get_messages(user_id, conv_id, 50, None).await;

        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }

    // ── get_other_user_id ──────────────────────────────────────

    #[tokio::test]
    async fn test_get_other_user_id_success() {
        let user_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();
        let conv_id = Uuid::new_v4();

        let mut mock_dm_repo = MockDmRepository::new();
        mock_dm_repo
            .expect_find_conversation_by_id()
            .returning(move |_| Ok(Some(make_conversation(user_id, other_id))));

        let service = DmService::new(Arc::new(mock_dm_repo), Arc::new(MockUserRepository::new()));
        let result = service.get_other_user_id(user_id, conv_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), other_id);
    }

    #[tokio::test]
    async fn test_get_other_user_id_not_participant() {
        let user_id = Uuid::new_v4();
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();
        let conv_id = Uuid::new_v4();

        let mut mock_dm_repo = MockDmRepository::new();
        mock_dm_repo
            .expect_find_conversation_by_id()
            .returning(move |_| Ok(Some(make_conversation(user1, user2))));

        let service = DmService::new(Arc::new(mock_dm_repo), Arc::new(MockUserRepository::new()));
        let result = service.get_other_user_id(user_id, conv_id).await;

        assert!(matches!(result, Err(DomainError::Forbidden(_))));
    }
}

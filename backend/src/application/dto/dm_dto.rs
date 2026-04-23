use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domain::entities::DirectMessage;

#[derive(Debug, Deserialize)]
pub struct OpenConversationRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SendDmRequest {
    #[validate(length(min = 1, max = 2000))]
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct GetDmMessagesQuery {
    pub limit: Option<i64>,
    pub before: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ConversationResponse {
    pub id: String,
    pub other_user_id: String,
    pub other_username: String,
    pub other_avatar_url: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct DmMessageResponse {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub sender_username: Option<String>,
    pub sender_avatar_url: Option<String>,
    pub content: String,
    pub created_at: String,
}

impl From<DirectMessage> for DmMessageResponse {
    fn from(msg: DirectMessage) -> Self {
        Self {
            id: msg.id.to_string(),
            conversation_id: msg.conversation_id.to_string(),
            sender_id: msg.sender_id.to_string(),
            sender_username: None,
            sender_avatar_url: None,
            content: msg.content,
            created_at: msg.created_at.to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;
    use validator::Validate;
    use crate::domain::entities::DirectMessage;

    #[test]
    fn test_send_dm_valid() {
        let req = SendDmRequest { content: "Bonjour !".to_string() };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_send_dm_vide() {
        let req = SendDmRequest { content: "".to_string() };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_send_dm_trop_long() {
        let req = SendDmRequest { content: "a".repeat(2001) };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_dm_message_response_from() {
        let conv_id = Uuid::new_v4();
        let sender_id = Uuid::new_v4();
        let msg = DirectMessage {
            id: Uuid::new_v4(),
            conversation_id: conv_id,
            sender_id,
            content: "Hello".to_string(),
            created_at: Utc::now(),
        };

        let response = DmMessageResponse::from(msg);

        assert_eq!(response.conversation_id, conv_id.to_string());
        assert_eq!(response.sender_id, sender_id.to_string());
        assert_eq!(response.content, "Hello");
        assert!(response.sender_username.is_none());
        assert!(response.sender_avatar_url.is_none());
    }
}

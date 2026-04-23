use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;

use crate::domain::entities::Message;

#[derive(Debug, Deserialize, Validate,ToSchema)]
pub struct CreateMessageRequest {
    #[validate(length(min = 1, max = 2000))]
    pub content: String,
}

#[derive(Debug, Deserialize,ToSchema)]
pub struct GetMessagesQuery {
    pub limit: Option<i64>,
    pub before: Option<String>,
}

#[derive(Debug, Serialize, Clone,ToSchema)]
pub struct MessageResponse {
    pub id: String,
    pub channel_id: String,
    pub author_id: String,
    pub content: String,
    pub edited: bool,
    pub created_at: String,
    pub author_username: Option<String>,
    pub author_avatar_url: Option<String>,
}

#[derive(Debug, Deserialize, Validate,ToSchema)]
pub struct UpdateMessageRequest {
    #[validate(length(min = 1, max = 2000, message = "Content must be between 1 and 2000 characters"))]
    pub content: String,
}

impl From<Message> for MessageResponse {
    fn from(message: Message) -> Self {
        Self {
            id: message.id.to_string(),
            channel_id: message.channel_id.to_string(),
            author_id: message.author_id.to_string(),
            content: message.content,
            edited: message.edited,
            created_at: message.created_at.to_rfc3339(),
            author_username: None,
            author_avatar_url: None,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;

    #[test]
    fn test_message_from(){
        let channel_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let message_test = Message{
            id: message_id,
            author_id: author_id,
            channel_id: channel_id,
            content: "Ceci est un contenu".to_string(),
            edited: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let message_dto = MessageResponse::from(message_test);

        assert_eq!(message_dto.id, message_id.to_string());
        assert_eq!(message_dto.channel_id, channel_id.to_string());
        assert_eq!(message_dto.author_id, author_id.to_string());
        assert_eq!(message_dto.content, "Ceci est un contenu".to_string());
        assert!(!message_dto.edited);
        }
    }
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Conversation {
    pub id: Uuid,
    pub user1_id: Uuid,
    pub user2_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl Conversation {
    pub fn other_user(&self, user_id: Uuid) -> Uuid {
        if self.user1_id == user_id {
            self.user2_id
        } else {
            self.user1_id
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DirectMessage {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl DirectMessage {
    pub fn new(conversation_id: Uuid, sender_id: Uuid, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            conversation_id,
            sender_id,
            content,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_new_direct_message() {
        let conversation_id = Uuid::new_v4();
        let sender_id = Uuid::new_v4();
        let content = "Salut !".to_string();

        let msg = DirectMessage::new(conversation_id, sender_id, content.clone());

        assert_eq!(msg.conversation_id, conversation_id);
        assert_eq!(msg.sender_id, sender_id);
        assert_eq!(msg.content, content);
    }

    #[test]
    fn test_conversation_other_user_depuis_user1() {
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();
        let conv = Conversation {
            id: Uuid::new_v4(),
            user1_id: user1,
            user2_id: user2,
            created_at: Utc::now(),
        };
        assert_eq!(conv.other_user(user1), user2);
    }

    #[test]
    fn test_conversation_other_user_depuis_user2() {
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();
        let conv = Conversation {
            id: Uuid::new_v4(),
            user1_id: user1,
            user2_id: user2,
            created_at: Utc::now(),
        };
        assert_eq!(conv.other_user(user2), user1);
    }
}

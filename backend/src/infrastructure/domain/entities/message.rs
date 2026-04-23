use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub author_id: Uuid,
    pub content: String,
    pub edited: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Message {
    pub fn new(channel_id: Uuid, author_id: Uuid, content: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            channel_id,
            author_id,
            content,
            edited: false,
            created_at: now,
            updated_at: now,
        }
    }
    pub fn mark_as_edited(&mut self, new_content: String) {
        self.content = new_content;
        self.edited = true;
        self.updated_at = Utc::now();
    }
    
    pub fn is_edited(&self) -> bool {
        self.edited || self.updated_at != self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_message() {
        let channel_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let message = Message::new(channel_id, author_id, "Hello!".to_string());
        assert_eq!(message.content, "Hello!");
        assert_eq!(message.channel_id, channel_id);
        assert_eq!(message.author_id, author_id);
        assert!(!message.edited);
    }

    #[test]
    fn test_mark_as_edited() {
        let channel_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let mut message = Message::new(channel_id, author_id, "Hello!".to_string());
        assert!(!message.is_edited());
        message.mark_as_edited("Hello, world!".to_string());
        assert_eq!(message.content, "Hello, world!");
        assert!(message.edited);
        assert!(message.is_edited());
    }

}

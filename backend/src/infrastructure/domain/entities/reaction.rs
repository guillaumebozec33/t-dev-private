use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Reaction {
    pub id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub emoji: String,
    pub created_at: DateTime<Utc>,
}

impl Reaction {
    pub fn new(message_id: Uuid, user_id: Uuid, emoji: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_id,
            user_id,
            emoji,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_new_reaction() {
        let message_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let emoji = "👍".to_string();

        let reaction = Reaction::new(message_id, user_id, emoji.clone());

        assert_eq!(reaction.message_id, message_id);
        assert_eq!(reaction.user_id, user_id);
        assert_eq!(reaction.emoji, emoji);
    }

    #[test]
    fn test_reaction_ids_uniques() {
        let message_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let r1 = Reaction::new(message_id, user_id, "👍".to_string());
        let r2 = Reaction::new(message_id, user_id, "👍".to_string());
        assert_ne!(r1.id, r2.id);
    }
}

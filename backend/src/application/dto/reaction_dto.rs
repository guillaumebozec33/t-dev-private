use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::domain::entities::Reaction;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ToggleReactionRequest {
    #[validate(length(min = 1, max = 50))]
    pub emoji: String,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct ReactionResponse {
    pub id: String,
    pub message_id: String,
    pub user_id: String,
    pub emoji: String,
    pub username: Option<String>,
}

impl From<Reaction> for ReactionResponse {
    fn from(reaction: Reaction) -> Self {
        Self {
            id: reaction.id.to_string(),
            message_id: reaction.message_id.to_string(),
            user_id: reaction.user_id.to_string(),
            emoji: reaction.emoji,
            username: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;
    use validator::Validate;
    use crate::domain::entities::Reaction;

    #[test]
    fn test_toggle_reaction_valid() {
        let req = ToggleReactionRequest { emoji: "👍".to_string() };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_toggle_reaction_vide() {
        let req = ToggleReactionRequest { emoji: "".to_string() };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_toggle_reaction_trop_long() {
        let req = ToggleReactionRequest { emoji: "a".repeat(51) };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_reaction_response_from() {
        let msg_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let reaction = Reaction {
            id: Uuid::new_v4(),
            message_id: msg_id,
            user_id,
            emoji: "❤️".to_string(),
            created_at: Utc::now(),
        };

        let response = ReactionResponse::from(reaction);

        assert_eq!(response.emoji, "❤️");
        assert_eq!(response.message_id, msg_id.to_string());
        assert_eq!(response.user_id, user_id.to_string());
        assert!(response.username.is_none());
    }
}

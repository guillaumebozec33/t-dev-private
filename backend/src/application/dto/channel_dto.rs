use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;

use crate::domain::entities::Channel;

#[derive(Debug, Deserialize, Validate,ToSchema)]
pub struct CreateChannelRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub description: Option<String>,
    pub channel_type: Option<String>,
    pub is_private: Option<bool>,
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize, Validate,ToSchema)]
pub struct UpdateChannelRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    pub description: Option<String>,
    pub position: Option<i32>,
    pub is_private: Option<bool>,
    pub icon: Option<String>,
}

#[derive(Debug, Serialize,ToSchema)]
pub struct ChannelResponse {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub description: Option<String>,
    pub channel_type: String,
    pub position: i32,
    pub is_private: bool,
    pub icon: Option<String>,
    pub created_at: String,
}

impl From<Channel> for ChannelResponse {
    fn from(channel: Channel) -> Self {
        Self {
            id: channel.id.to_string(),
            server_id: channel.server_id.to_string(),
            name: channel.name,
            description: channel.description,
            channel_type: channel.channel_type,
            position: channel.position,
            is_private: channel.is_private,
            icon: channel.icon,
            created_at: channel.created_at.to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;
    use uuid::Uuid;
    use chrono::Utc;


    #[test]
    fn test_createChannel_valid() {
        let req = CreateChannelRequest {
            name: "channel1".to_string(),
            description: Some("Ceci est une description".to_string()),
            channel_type: Some("textuel".to_string()),
            is_private: Some(false),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_user_from(){
        let channel_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let channel = Channel{
            id: channel_id,
            server_id: server_id,
            name: "channel1".to_string(),
            description: Some("Ceci est une description".to_string()),
            channel_type: "textuel".to_string(),
            position: 1,
            is_private: false,
            icon: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let channel_dto = ChannelResponse::from(channel);

        assert_eq!(channel_dto.id, channel_id.to_string());
        assert_eq!(channel_dto.server_id, server_id.to_string());
        assert_eq!(channel_dto.name, "channel1".to_string());
        assert_eq!(channel_dto.description, Some("Ceci est une description".to_string()));
        assert_eq!(channel_dto.channel_type, "textuel");
        assert_eq!(channel_dto.position, 1);
        }
    }


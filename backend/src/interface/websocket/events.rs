use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SocketEvent {
    #[serde(rename = "join_server")]
    JoinServer { server_id: String },
    
    #[serde(rename = "leave_server")]
    LeaveServer { server_id: String },
    
    #[serde(rename = "join_channel")]
    JoinChannel { channel_id: String },
    
    #[serde(rename = "leave_channel")]
    LeaveChannel { channel_id: String },

    #[serde(rename = "member_kicked")]
    MemberKicked { 
        server_id: String,
        member_id: String,
    },
    
    #[serde(rename = "typing_start")]
    TypingStart { channel_id: String },
    
    #[serde(rename = "typing_stop")]
    TypingStop { channel_id: String },
    
    #[serde(rename = "update_status")]
    UpdateStatus { status: String },
    
    #[serde(rename = "message_deleted")]
    MessageDeleted { message_id: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TypingEvent {
    pub channel_id: String,
    pub user_id: String,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PresenceEvent {
    pub server_id: String,
    pub user_id: String,
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typing_event_serialize() {
        let event = TypingEvent {
            channel_id: "ch1".to_string(),
            user_id: "u1".to_string(),
            username: "test".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("ch1"));
        assert!(json.contains("test"));
    }

    #[test]
    fn test_presence_event_serialize() {
        let event = PresenceEvent {
            server_id: "s1".to_string(),
            user_id: "u1".to_string(),
            status: "online".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("online"));
    }
}

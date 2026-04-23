use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;

use crate::domain::entities::{Server, Ban};
use crate::domain::enums::Role;

#[derive(Debug, Deserialize, Validate,ToSchema)]
pub struct CreateServerRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
}

#[derive(Debug, Deserialize, Validate,ToSchema)]
pub struct UpdateServerRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>, // empty string = remove icon
}

#[derive(Debug, Deserialize,ToSchema)]
pub struct JoinServerRequest {
    pub invite_code: String,
}

#[derive(Debug, Deserialize,ToSchema)]
pub struct UpdateMemberRoleRequest {
    pub role: Role,
}

#[derive(Debug, Deserialize,ToSchema)]
pub struct TransferOwnershipRequest {
    pub new_owner_id: String,
}

#[derive(Debug, Serialize,ToSchema)]
pub struct ServerResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub owner_id: String,
    pub created_at: String,
}

impl From<Server> for ServerResponse {
    fn from(server: Server) -> Self {
        Self {
            id: server.id.to_string(),
            name: server.name,
            description: server.description,
            icon_url: server.icon_url,
            owner_id: server.owner_id.to_string(),
            created_at: server.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize,ToSchema)]
pub struct MemberResponse {
    pub id: String,
    pub user_id: String,
    pub server_id: String,
    pub role: String,
    pub joined_at: String,
    pub username: String,
    pub avatar_url: Option<String>,
    pub status: String,
}

impl MemberResponse {
    pub fn new(
        id: String,
        user_id: String,
        server_id: String,
        role: String,
        joined_at: String,
        username: String,
        avatar_url: Option<String>,
        status: String,
    ) -> Self {
        Self {
            id,
            user_id,
            server_id,
            role,
            joined_at,
            username,
            avatar_url,
            status,
        }
    }
}

#[derive(Debug, Deserialize,ToSchema)]
pub struct CreateInvitationRequest {
    pub max_uses: Option<i32>,
    pub expires_in_hours: Option<i64>,
}

#[derive(Debug, Serialize,ToSchema)]
pub struct InvitationResponse {
    pub code: String,
    pub server_id: String,
    pub max_uses: Option<i32>,
    pub uses: i32,
    pub expires_at: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BanMemberRequest {
    pub duration_hours: Option<i64>, // None = permanent (1000 years)
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BanResponse {
    pub id: String,
    pub user_id: String,
    pub server_id: String,
    pub banned_by: String,
    pub banned_at: String,
    pub expires_at: String,
    pub is_permanent: bool,
    pub username: String,
}

impl From<Ban> for BanResponse {
    fn from(ban: Ban) -> Self {
        Self {
            id: ban.id.to_string(),
            user_id: ban.user_id.to_string(),
            server_id: ban.server_id.to_string(),
            banned_by: ban.banned_by.to_string(),
            banned_at: ban.banned_at.to_rfc3339(),
            expires_at: ban.expires_at.to_rfc3339(),
            is_permanent: ban.is_permanent(),
            username: ban.username.to_string(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;

    #[test]
    fn test_server_from(){
        let server_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let server_test = Server{
            id: server_id,
            owner_id: owner_id,
            name: "Mon server".to_string(),
            description: Some("Ma description de server".to_string()),
            icon_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let server_dto = ServerResponse::from(server_test);

        assert_eq!(server_dto.id, server_id.to_string());
        assert_eq!(server_dto.owner_id, owner_id.to_string());
        assert_eq!(server_dto.name, "Mon server".to_string());
        assert_eq!(server_dto.description, Some("Ma description de server".to_string()));
        assert_eq!(server_dto.icon_url, None);
        }

    #[test]
    fn test_member_response_new() {
        let id = Uuid::new_v4().to_string();
        let user_id = Uuid::new_v4().to_string();
        let server_id = Uuid::new_v4().to_string();
        let joined_at = Utc::now().to_rfc3339();

        let member = MemberResponse::new(
            id.clone(),
            user_id.clone(),
            server_id.clone(),
            "Member".to_string(),
            joined_at.clone(),
            "guillaume".to_string(),
            None,
            "online".to_string(),
        );

        assert_eq!(member.id, id);
        assert_eq!(member.user_id, user_id);
        assert_eq!(member.server_id, server_id);
        assert_eq!(member.role, "Member");
        assert_eq!(member.joined_at, joined_at);
        assert_eq!(member.username, "guillaume");
        assert_eq!(member.avatar_url, None);
        assert_eq!(member.status, "online");
    }

    }
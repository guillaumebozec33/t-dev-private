use serde::{Deserialize, Serialize, Deserializer};
use validator::Validate;
use utoipa::ToSchema;

fn deserialize_optional_nullable<'de, D>(d: D) -> Result<Option<Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Some(Option::<String>::deserialize(d)?))
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateUserRequest {
    #[validate(length(min = 3, max = 32))]
    pub username: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    pub avatar_url: Option<Option<String>>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PublicUserResponse {
    pub id: String,
    pub username: String,
    pub avatar_url: Option<String>,
    pub status: String,
}

impl From<crate::domain::entities::User> for PublicUserResponse {
    fn from(user: crate::domain::entities::User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username,
            avatar_url: user.avatar_url,
            status: format!("{:?}", user.status).to_lowercase(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;
    use crate::domain::entities::User;
    use crate::domain::enums::UserStatus;

    #[test]
    fn test_deserialize_optional_nullable_null() {
        let req: UpdateUserRequest = serde_json::from_str(r#"{"avatar_url": null}"#).unwrap();
        assert_eq!(req.avatar_url, Some(None));
    }

    #[test]
    fn test_deserialize_optional_nullable_value() {
        let req: UpdateUserRequest = serde_json::from_str(r#"{"avatar_url": "https://example.com/img.png"}"#).unwrap();
        assert_eq!(req.avatar_url, Some(Some("https://example.com/img.png".to_string())));
    }

    #[test]
    fn test_user_from(){
        let user_id = Uuid::new_v4();
        let user_test = User{
            id: user_id,
            username: "Guillaume".to_string(),
            email: "guillaume@exemple.fr".to_string(),
            password_hash: "motdepasse".to_string(),
            avatar_url: None,
            status: UserStatus::Online,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let user_dto = PublicUserResponse::from(user_test);

        assert_eq!(user_dto.id, user_id.to_string());
        assert_eq!(user_dto.avatar_url, None);
        assert_eq!(user_dto.username, "Guillaume".to_string());
        assert_eq!(user_dto.status, "online".to_string());
        }
    }
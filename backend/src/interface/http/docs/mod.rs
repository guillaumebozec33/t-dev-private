use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::interface::http::controllers::auth_controller::signup,
        crate::interface::http::controllers::auth_controller::login,
        crate::interface::http::controllers::auth_controller::me,
        crate::interface::http::controllers::auth_controller::update_me,
        crate::interface::http::controllers::server_controller::create_server,
        crate::interface::http::controllers::server_controller::get_servers,
        crate::interface::http::controllers::server_controller::get_server,
        crate::interface::http::controllers::server_controller::update_server,
        crate::interface::http::controllers::server_controller::delete_server,
        crate::interface::http::controllers::server_controller::join_server,
        crate::interface::http::controllers::server_controller::join_server_by_code,
        crate::interface::http::controllers::server_controller::leave_server,
        crate::interface::http::controllers::server_controller::kick_member,
        crate::interface::http::controllers::server_controller::get_members,
        crate::interface::http::controllers::server_controller::update_member_role,
        crate::interface::http::controllers::server_controller::create_invitation,
        crate::interface::http::controllers::server_controller::transfer_ownership,
        crate::interface::http::controllers::channel_controller::create_channel,
        crate::interface::http::controllers::channel_controller::get_channels,
        crate::interface::http::controllers::channel_controller::get_channel,
        crate::interface::http::controllers::channel_controller::update_channel,
        crate::interface::http::controllers::channel_controller::delete_channel,
        crate::interface::http::controllers::message_controller::send_message,
        crate::interface::http::controllers::message_controller::get_messages,
        crate::interface::http::controllers::message_controller::delete_message,
        crate::interface::http::controllers::message_controller::edit_message,
    ),
    components(schemas(
        crate::application::dto::SignupRequest,
        crate::application::dto::LoginRequest,
        crate::application::dto::AuthResponse,
        crate::application::dto::UserResponse,
        crate::application::dto::CreateServerRequest,
        crate::application::dto::UpdateServerRequest,
        crate::application::dto::ServerResponse,
        crate::application::dto::JoinServerRequest,
        crate::application::dto::MemberResponse,
        crate::application::dto::UpdateMemberRoleRequest,
        crate::application::dto::CreateInvitationRequest,
        crate::application::dto::InvitationResponse,
        crate::application::dto::TransferOwnershipRequest,
        crate::application::dto::CreateChannelRequest,
        crate::application::dto::UpdateChannelRequest,
        crate::application::dto::ChannelResponse,
        crate::application::dto::CreateMessageRequest,
        crate::application::dto::UpdateMessageRequest,
        crate::application::dto::MessageResponse,
    )),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            utoipa::openapi::security::SecurityScheme::Http(
                utoipa::openapi::security::Http::new(
                    utoipa::openapi::security::HttpAuthScheme::Bearer,
                )
            ),
        );
        
        openapi.info.title = "RTC Backend API".to_string();
        
        openapi.servers = Some(vec![
            utoipa::openapi::ServerBuilder::new()
                .url("/api")
                .description(Some("API Base Path"))
                .build(),
        ]);
    }
}
use axum::{
    http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, ORIGIN},
    middleware,
    routing::{delete, get, post, put, patch},
    Router,
};
use socketioxide::SocketIo;
use tower_http::cors::{AllowHeaders, Any, CorsLayer};

use crate::interface::http::controllers::*;
use crate::interface::http::middleware::auth_middleware;
use crate::shared::app_state::AppState;
use crate::interface::http::controllers::server_controller::kick_member;
use utoipa_swagger_ui::SwaggerUi;
use crate::interface::http::docs::ApiDoc;
use utoipa::OpenApi;

pub fn create_router(state: AppState, _socket_io: SocketIo) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(AllowHeaders::list(vec![AUTHORIZATION, CONTENT_TYPE, ACCEPT, ORIGIN]));

    let public_routes = Router::new()
        .route("/auth/signup", post(auth_controller::signup))
        .route("/auth/login", post(auth_controller::login));

    let protected_routes = Router::new()
        //.route("/auth/logout", post(auth_controller::logout))
        .route("/me", get(auth_controller::me))
        .route("/me", patch(auth_controller::update_me))
        .route("/servers", post(server_controller::create_server))
        .route("/servers", get(server_controller::get_servers))
        .route("/servers/join", post(server_controller::join_server_by_code))
        .route("/servers/:server_id", get(server_controller::get_server))
        .route("/servers/:server_id", put(server_controller::update_server))
        .route("/servers/:server_id", delete(server_controller::delete_server))
        .route("/servers/:server_id/join", post(server_controller::join_server))
        .route("/servers/:server_id/leave", delete(server_controller::leave_server))
        .route("/servers/:server_id/members/:member_id/kick", delete(kick_member))
        .route("/servers/:server_id/members/:member_id/ban", post(server_controller::ban_member))
        .route("/servers/:server_id/bans/:user_id", delete(server_controller::unban_member))
        .route("/servers/:server_id/bans", get(server_controller::get_bans))
        .route("/servers/:server_id/members/:user_id", put(server_controller::update_member_role))
        .route("/servers/:server_id/owner", put(server_controller::transfer_ownership))
        .route("/servers/:server_id/members", get(server_controller::get_members))
        .route("/servers/:server_id/invitations", post(server_controller::create_invitation))
        .route("/servers/:server_id/channels", post(channel_controller::create_channel))
        .route("/servers/:server_id/channels", get(channel_controller::get_channels))
        .route("/channels/:channel_id", get(channel_controller::get_channel))
        .route("/channels/:channel_id", put(channel_controller::update_channel))
        .route("/channels/:channel_id", delete(channel_controller::delete_channel))
        .route("/channels/:channel_id/messages", post(message_controller::send_message))
        .route("/messages/:message_id", patch(message_controller::edit_message))
        .route("/channels/:channel_id/messages", get(message_controller::get_messages))
        .route("/messages/:message_id", delete(message_controller::delete_message))
        .route("/dm/conversations", post(dm_controller::open_conversation))
        .route("/dm/conversations", get(dm_controller::list_conversations))
        .route("/dm/conversations/:id/messages", get(dm_controller::get_dm_messages))
        .route("/dm/conversations/:id/messages", post(dm_controller::send_dm))
        .route("/messages/:message_id/reactions", put(reaction_controller::toggle_reaction))
        .route("/messages/:message_id/reactions", get(reaction_controller::get_reactions))
        .route("/dm/messages/:message_id/reactions", put(reaction_controller::toggle_dm_reaction))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));


        Router::new()
                .nest("/api", public_routes.merge(protected_routes))
                .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
                .layer(cors)
                .with_state(state)
}

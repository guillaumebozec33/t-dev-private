use socketioxide::{
    extract::{Data, SocketRef},
    layer::SocketIoLayer,
    SocketIo,
};
use tracing::info;
use std::sync::Arc;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::interface::websocket::events::TypingEvent;
use crate::domain::repositories::UserRepository;
use crate::domain::enums::UserStatus;
use serde_json::json;

// Map pour stocker socket_id -> user_id
type SocketUserMap = Arc<RwLock<HashMap<String, Uuid>>>;

pub async fn setup_socket_io(pool: PgPool, user_repo: Arc<dyn UserRepository>, _jwt_secret: String) -> (SocketIoLayer, SocketIo) {
    let (layer, io) = SocketIo::new_layer();

    let pool = Arc::new(pool);
    let user_repo = user_repo.clone();
    let io_clone = io.clone();
    let socket_user_map: SocketUserMap = Arc::new(RwLock::new(HashMap::new()));

    io.ns("/", move |socket: SocketRef| {
        info!("Client connected: {}", socket.id);

        let pool = pool.clone();
        let user_repo = user_repo.clone();
        let io_for_status = io_clone.clone();
        let io_for_disconnect = io_clone.clone();
        let pool_for_status = pool.clone();
        let user_repo_for_status = user_repo.clone();
        let pool_for_disconnect = pool.clone();
        let user_repo_for_disconnect = user_repo.clone();
        let user_repo_for_typing = user_repo.clone();
        let pool_for_connect = pool.clone();
        let io_for_connect = io_clone.clone();
        let socket_user_map_for_identify = socket_user_map.clone();
        let socket_user_map_for_disconnect = socket_user_map.clone();

        socket.on("identify", move |socket: SocketRef, Data::<serde_json::Value>(data)| {
            let pool = pool_for_connect.clone();
            let io = io_for_connect.clone();
            let socket_user_map = socket_user_map_for_identify.clone();
            
            async move {
                if let Some(user_id_str) = data.get("user_id").and_then(|v| v.as_str()) {
                    if let Ok(user_id) = Uuid::parse_str(user_id_str) {
                        // Stocker l'user_id dans la map
                        socket_user_map.write().await.insert(socket.id.to_string(), user_id);
                        let _ = socket.join(format!("dm:{}", user_id));
                        
                        // Émettre user_connected pour tous les serveurs de l'utilisateur
                        if let Ok(servers) = sqlx::query_as::<_, (Uuid,)>("SELECT server_id FROM members WHERE user_id = $1")
                            .bind(user_id)
                            .fetch_all(pool.as_ref())
                            .await
                        {
                            for (server_id,) in servers {
                                let room = format!("server:{}", server_id);
                                let _ = io.to(room).emit("user_connected", &serde_json::json!({
                                    "user_id": user_id.to_string(),
                                }));
                            }
                        }
                        info!("User {} identified on socket {}", user_id, socket.id);
                    }
                }
            }
        });

        socket.on("join_channel", |socket: SocketRef, Data::<serde_json::Value>(data)| async move {
            if let Some(channel_id) = data.get("channel_id").and_then(|v| v.as_str()) {
                let room = format!("channel:{}", channel_id);
                let _ = socket.join(room.clone());
                info!("Socket {} joined channel room {}", socket.id, room);
            }
        });

        socket.on("leave_channel", |socket: SocketRef, Data::<serde_json::Value>(data)| async move {
            if let Some(channel_id) = data.get("channel_id").and_then(|v| v.as_str()) {
                let room = format!("channel:{}", channel_id);
                let _ = socket.leave(room.clone());
                info!("Socket {} left channel room {}", socket.id, room);
            }
        });

        socket.on("join_server", |socket: SocketRef, Data::<serde_json::Value>(data)| async move {
            if let Some(server_id) = data.get("server_id").and_then(|v| v.as_str()) {
                let room = format!("server:{}", server_id);
                let _ = socket.join(room.clone());
                info!("Socket {} joined server room {}", socket.id, room);
            }
        });

        socket.on("leave_server", |socket: SocketRef, Data::<serde_json::Value>(data)| async move {
            if let Some(server_id) = data.get("server_id").and_then(|v| v.as_str()) {
                let room = format!("server:{}", server_id);
                let _ = socket.leave(room.clone());
                info!("Socket {} left server room {}", socket.id, room);
            }
        });

        socket.on("join_dm", |socket: SocketRef, Data::<serde_json::Value>(data)| async move {
            if let Some(user_id) = data.get("user_id").and_then(|v| v.as_str()) {
                let room = format!("dm:{}", user_id);
                let _ = socket.join(room.clone());
                info!("Socket {} joined dm room {}", socket.id, room);
            }
        });

        socket.on("leave_dm", |socket: SocketRef, Data::<serde_json::Value>(data)| async move {
            if let Some(user_id) = data.get("user_id").and_then(|v| v.as_str()) {
                let room = format!("dm:{}", user_id);
                let _ = socket.leave(room.clone());
                info!("Socket {} left dm room {}", socket.id, room);
            }
        });

        socket.on("typing_start", move |socket: SocketRef, Data::<serde_json::Value>(data)| {
            let user_repo = user_repo_for_typing.clone();
            
            async move {
                if let Some(channel_id) = data.get("channel_id").and_then(|v| v.as_str()) {
                    if let Some(user_id_str) = data.get("user_id").and_then(|v| v.as_str()) {
                        if let Ok(user_id) = Uuid::parse_str(user_id_str) {
                            if let Ok(Some(user)) = user_repo.find_by_id(user_id).await {
                                let room = format!("channel:{}", channel_id);
                                let typing_event = TypingEvent {
                                    channel_id: channel_id.to_string(),
                                    user_id: user_id.to_string(),
                                    username: user.username.clone(),
                                };
                                let _ = socket.to(room).emit("user_typing", &typing_event);
                                info!("User {} is typing in channel {}", user.username, channel_id);
                            }
                        }
                    }
                }
            }
        });

        socket.on("typing_stop", |socket: SocketRef, Data::<serde_json::Value>(data)| async move {
            if let Some(channel_id) = data.get("channel_id").and_then(|v| v.as_str()) {
                if let Some(user_id) = data.get("user_id").and_then(|v| v.as_str()) {
                    let room = format!("channel:{}", channel_id);
                    let _ = socket.to(room).emit("user_stop_typing", &serde_json::json!({
                        "channel_id": channel_id,
                        "user_id": user_id,
                    }));
                    info!("User {} stopped typing in channel {}", user_id, channel_id);
                }
            }
        });

        socket.on("update_status", move |socket: SocketRef, Data::<serde_json::Value>(data)| {
            let pool = pool_for_status.clone();
            let user_repo = user_repo_for_status.clone();
            let io = io_for_status.clone();
            
            async move {
                if let (Some(status_str), Some(user_id_str)) = (
                    data.get("status").and_then(|v| v.as_str()),
                    data.get("user_id").and_then(|v| v.as_str())
                ) {
                    if let Ok(user_id) = Uuid::parse_str(user_id_str) {
                        info!("Socket {} updating status for user {} to {}", socket.id, user_id, status_str);
                        
                        if let Ok(Some(mut user)) = user_repo.find_by_id(user_id).await {
                            let status = match status_str {
                                "online" => UserStatus::Online,
                                "dnd" => UserStatus::DoNotDisturb,
                                "away" => UserStatus::Away,
                                "invisible" => UserStatus::Invisible,
                                _ => {
                                    info!("Invalid status '{}' received, ignoring", status_str);
                                    return;
                                }
                            };
                            user.status = status;
                            user.updated_at = Utc::now();
                            
                            if let Ok(_) = user_repo.update(&user).await {
                                if let Ok(servers) = sqlx::query_as::<_, (Uuid,)>("SELECT server_id FROM members WHERE user_id = $1")
                                    .bind(user_id)
                                    .fetch_all(pool.as_ref())
                                    .await
                                {
                                    for (server_id,) in servers {
                                        emit_user_status_changed(
                                            &io,
                                            &server_id.to_string(),
                                            &user_id.to_string(),
                                            status_str
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        socket.on_disconnect(move |socket: SocketRef| {
            let pool = pool_for_disconnect.clone();
            let user_repo = user_repo_for_disconnect.clone();
            let io = io_for_disconnect.clone();
            let socket_user_map = socket_user_map_for_disconnect.clone();
            
            async move {
                info!("Client disconnected: {}", socket.id);
                
                // Récupérer l'user_id depuis la map
                let user_id = socket_user_map.write().await.remove(&socket.id.to_string());
                
                if let Some(user_id) = user_id {
                    // Émettre user_disconnected pour tous les serveurs de l'utilisateur
                    if let Ok(servers) = sqlx::query_as::<_, (Uuid,)>("SELECT server_id FROM members WHERE user_id = $1")
                        .bind(user_id)
                        .fetch_all(pool.as_ref())
                        .await
                    {
                        for (server_id,) in servers {
                            let room = format!("server:{}", server_id);
                            let _ = io.to(room).emit("user_disconnected", &serde_json::json!({
                                "user_id": user_id.to_string(),
                            }));
                        }
                    }
                    info!("User {} disconnected from socket {}", user_id, socket.id);
                }
            }
        });
    });

    (layer, io)
}

pub fn emit_new_message(io: &SocketIo, channel_id: &str, message: &serde_json::Value) {
    let room = format!("channel:{}", channel_id);
    let _ = io.to(room).emit("new_message", message);
}

pub fn emit_message_deleted(io: &SocketIo, channel_id: &str, message_id: &str) {
    let room = format!("channel:{}", channel_id);
    let _ = io.to(room).emit("message_deleted", &serde_json::json!({
        "message_id": message_id,
        "channel_id": channel_id,
    }));
}

pub fn emit_member_joined(io: &SocketIo, server_id: &str, user: &serde_json::Value) {
    let room = format!("server:{}", server_id);
    let _ = io.to(room).emit("member_joined", &serde_json::json!({
        "server_id": server_id,
        "user": user,
    }));
}

pub fn emit_member_left(io: &SocketIo, server_id: &str, user_id: &str) {
    let room = format!("server:{}", server_id);
    let _ = io.to(room).emit("member_left", &serde_json::json!({
        "server_id": server_id,
        "user_id": user_id,
    }));
}
pub fn emit_member_kicked(io: &SocketIo, server_id: &str, member_id: &str) {
    let room = format!("server:{}", server_id);
    
    if let Err(e) = io.to(room.clone()).emit("member_kicked", json!({
        "server_id": server_id,
        "member_id": member_id,
    })) {
        tracing::error!("Failed to emit member_kicked: {}", e);
    }
    
    info!("Emitted member_kicked for user {} in server {}", member_id, server_id);
}

pub fn emit_member_banned(io: &SocketIo, server_id: &str, member_id: &str) {
    let room = format!("server:{}", server_id);
    
    if let Err(e) = io.to(room.clone()).emit("member_banned", json!({
        "server_id": server_id,
        "member_id": member_id,
    })) {
        tracing::error!("Failed to emit member_banned: {}", e);
    }
    
    info!("Emitted member_banned for user {} in server {}", member_id, server_id);
}

pub fn emit_channel_created(io: &SocketIo, server_id: &str, channel: &serde_json::Value) {
    let room = format!("server:{}", server_id);
    let _ = io.to(room).emit("channel_created", channel);
}

pub fn emit_channel_updated(io: &SocketIo, server_id: &str, channel: &serde_json::Value) {
    let room = format!("server:{}", server_id);
    let _ = io.to(room).emit("channel_updated", channel);
}

pub fn emit_channel_deleted(io: &SocketIo, server_id: &str, channel_id: &str) {
    let room = format!("server:{}", server_id);
    let _ = io.to(room).emit("channel_deleted", &serde_json::json!({
        "id": channel_id,
        "server_id": server_id,
    }));
}

pub fn emit_server_updated(io: &SocketIo, server: &serde_json::Value) {
    if let Some(server_id) = server.get("id").and_then(|v| v.as_str()) {
        let room = format!("server:{}", server_id);
        let _ = io.to(room).emit("server_updated", server);
    }
}

pub fn emit_user_status_changed(io: &SocketIo, server_id: &str, user_id: &str, status: &str) {
    let room = format!("server:{}", server_id);
    let _ = io.to(room).emit("user_status_changed", &serde_json::json!({
        "server_id": server_id,
        "user_id": user_id,
        "status": status,
    }));
}

pub fn emit_user_profile_updated(
    io: &SocketIo,
    server_id: &str,
    user_id: &str,
    username: &str,
    avatar_url: Option<&str>,
    status: &str,
) {
    let room = format!("server:{}", server_id);
    let _ = io.to(room).emit("user_profile_updated", &serde_json::json!({
        "server_id": server_id,
        "user_id": user_id,
        "username": username,
        "avatar_url": avatar_url,
        "status": status,
    }));
}

pub fn emit_user_profile_updated_to_user_room(
    io: &SocketIo,
    target_user_id: &str,
    user_id: &str,
    username: &str,
    avatar_url: Option<&str>,
    status: &str,
) {
    let room = format!("dm:{}", target_user_id);
    let _ = io.to(room).emit("user_profile_updated", &serde_json::json!({
        "user_id": user_id,
        "username": username,
        "avatar_url": avatar_url,
        "status": status,
    }));
}

pub fn emit_member_role_changed(io: &SocketIo, server_id: &str, user_id: &str, role: &str) {
    let room = format!("server:{}", server_id);
    let _ = io.to(room).emit("member_role_changed", &serde_json::json!({
        "server_id": server_id,
        "user_id": user_id,
        "role": role,
    }));
}

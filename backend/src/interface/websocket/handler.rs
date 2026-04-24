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
type VoiceChannelMap = Arc<RwLock<HashMap<String, HashMap<String, VoiceParticipant>>>>;

#[derive(Debug, Clone, serde::Serialize)]
struct VoiceParticipant {
    socket_id: String,
    user_id: String,
    username: String,
}

pub async fn setup_socket_io(pool: PgPool, user_repo: Arc<dyn UserRepository>, _jwt_secret: String) -> (SocketIoLayer, SocketIo) {
    let (layer, io) = SocketIo::new_layer();

    let pool = Arc::new(pool);
    let user_repo = user_repo.clone();
    let io_clone = io.clone();
    let socket_user_map: SocketUserMap = Arc::new(RwLock::new(HashMap::new()));
    let voice_channel_map: VoiceChannelMap = Arc::new(RwLock::new(HashMap::new()));

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
        let voice_channel_map_for_join = voice_channel_map.clone();
        let voice_channel_map_for_leave = voice_channel_map.clone();
        let voice_channel_map_for_disconnect = voice_channel_map.clone();
        let io_for_voice_join = io_clone.clone();
        let io_for_voice_leave = io_clone.clone();

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

        socket.on("join_voice_channel", move |socket: SocketRef, Data::<serde_json::Value>(data)| {
            let voice_channel_map = voice_channel_map_for_join.clone();
            let io = io_for_voice_join.clone();

            async move {
                let Some(channel_id) = data.get("channel_id").and_then(|v| v.as_str()) else {
                    return;
                };
                let Some(user_id) = data.get("user_id").and_then(|v| v.as_str()) else {
                    return;
                };
                let Some(username) = data.get("username").and_then(|v| v.as_str()) else {
                    return;
                };

                let room = format!("voice:{}", channel_id);
                let participant = VoiceParticipant {
                    socket_id: socket.id.to_string(),
                    user_id: user_id.to_string(),
                    username: username.to_string(),
                };

                let existing_participants = {
                    let mut channels = voice_channel_map.write().await;
                    let participants = channels.entry(channel_id.to_string()).or_default();
                    let existing = participants.values().cloned().collect::<Vec<_>>();
                    participants.insert(socket.id.to_string(), participant.clone());
                    existing
                };

                let _ = socket.join(room.clone());
                let _ = socket.emit("voice_channel_state", &json!({
                    "channel_id": channel_id,
                    "participants": existing_participants,
                }));
                let _ = socket.to(room.clone()).emit("voice_participant_joined", &participant);
                let _ = io.to(room.clone()).emit("voice_channel_presence", &json!({
                    "channel_id": channel_id,
                    "participant_count": existing_participants.len() + 1,
                }));
                info!("Socket {} joined voice room {}", socket.id, room);
            }
        });

        socket.on("leave_voice_channel", move |socket: SocketRef, Data::<serde_json::Value>(data)| {
            let voice_channel_map = voice_channel_map_for_leave.clone();
            let io = io_for_voice_leave.clone();

            async move {
                let Some(channel_id) = data.get("channel_id").and_then(|v| v.as_str()) else {
                    return;
                };

                let room = format!("voice:{}", channel_id);
                let removed = {
                    let mut channels = voice_channel_map.write().await;
                    let Some(participants) = channels.get_mut(channel_id) else {
                        return;
                    };
                    let removed = participants.remove(&socket.id.to_string());
                    if participants.is_empty() {
                        channels.remove(channel_id);
                    }
                    removed
                };

                let _ = socket.leave(room.clone());

                if let Some(participant) = removed {
                    let remaining_count = {
                        let channels = voice_channel_map.read().await;
                        channels.get(channel_id).map_or(0, |participants| participants.len())
                    };
                    let _ = socket.to(room.clone()).emit("voice_participant_left", &json!({
                        "channel_id": channel_id,
                        "socket_id": participant.socket_id,
                        "user_id": participant.user_id,
                    }));
                    let _ = io.to(room.clone()).emit("voice_channel_presence", &json!({
                        "channel_id": channel_id,
                        "participant_count": remaining_count,
                    }));
                    info!("Socket {} left voice room {}", socket.id, room);
                }
            }
        });

        socket.on("voice_offer", |socket: SocketRef, Data::<serde_json::Value>(data)| async move {
            let Some(target_socket_id) = data
                .get("target_socket_id")
                .and_then(|v| v.as_str())
                .map(str::to_owned)
            else {
                return;
            };
            let _ = socket.to(target_socket_id).emit("voice_offer", &json!({
                "source_socket_id": socket.id.to_string(),
                "sdp": data.get("sdp").cloned().unwrap_or(serde_json::Value::Null),
            }));
        });

        socket.on("voice_answer", |socket: SocketRef, Data::<serde_json::Value>(data)| async move {
            let Some(target_socket_id) = data
                .get("target_socket_id")
                .and_then(|v| v.as_str())
                .map(str::to_owned)
            else {
                return;
            };
            let _ = socket.to(target_socket_id).emit("voice_answer", &json!({
                "source_socket_id": socket.id.to_string(),
                "sdp": data.get("sdp").cloned().unwrap_or(serde_json::Value::Null),
            }));
        });

        socket.on("voice_ice_candidate", |socket: SocketRef, Data::<serde_json::Value>(data)| async move {
            let Some(target_socket_id) = data
                .get("target_socket_id")
                .and_then(|v| v.as_str())
                .map(str::to_owned)
            else {
                return;
            };
            let _ = socket.to(target_socket_id).emit("voice_ice_candidate", &json!({
                "source_socket_id": socket.id.to_string(),
                "candidate": data.get("candidate").cloned().unwrap_or(serde_json::Value::Null),
            }));
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
                if let Some(user_id_str) = data.get("user_id").and_then(|v| v.as_str()) {
                    if let Ok(user_id) = Uuid::parse_str(user_id_str) {
                        if let Ok(Some(user)) = user_repo.find_by_id(user_id).await {
                            if let Some(channel_id) = data.get("channel_id").and_then(|v| v.as_str()) {
                                let room = format!("channel:{}", channel_id);
                                let typing_event = TypingEvent {
                                    channel_id: channel_id.to_string(),
                                    user_id: user_id.to_string(),
                                    username: user.username.clone(),
                                };
                                let _ = socket.to(room).emit("user_typing", &typing_event);
                                info!("User {} is typing in channel {}", user.username, channel_id);
                            } else if let Some(conversation_id) = data.get("conversation_id").and_then(|v| v.as_str()) {
                                if let Some(recipient_id) = data.get("recipient_id").and_then(|v| v.as_str()) {
                                    let room = format!("dm:{}", recipient_id);
                                    let _ = socket.to(room).emit("user_typing", &serde_json::json!({
                                        "conversation_id": conversation_id,
                                        "user_id": user_id.to_string(),
                                        "username": user.username,
                                    }));
                                    info!("User {} is typing in dm conversation {}", user_id, conversation_id);
                                }
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
            } else if let Some(conversation_id) = data.get("conversation_id").and_then(|v| v.as_str()) {
                if let Some(recipient_id) = data.get("recipient_id").and_then(|v| v.as_str()) {
                    if let Some(user_id) = data.get("user_id").and_then(|v| v.as_str()) {
                        let room = format!("dm:{}", recipient_id);
                        let _ = socket.to(room).emit("user_stop_typing", &serde_json::json!({
                            "conversation_id": conversation_id,
                            "user_id": user_id,
                        }));
                        info!("User {} stopped typing in dm conversation {}", user_id, conversation_id);
                    }
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
            let io = io_for_disconnect.clone();
            let socket_user_map = socket_user_map_for_disconnect.clone();
            let voice_channel_map = voice_channel_map_for_disconnect.clone();
            
            async move {
                info!("Client disconnected: {}", socket.id);

                let removed_voice_channel = {
                    let mut channels = voice_channel_map.write().await;
                    let mut found: Option<(String, VoiceParticipant)> = None;

                    for (channel_id, participants) in channels.iter_mut() {
                        if let Some(participant) = participants.remove(&socket.id.to_string()) {
                            found = Some((channel_id.clone(), participant));
                            break;
                        }
                    }

                    if let Some((channel_id, participant)) = found {
                        if channels.get(&channel_id).is_some_and(|participants| participants.is_empty()) {
                            channels.remove(&channel_id);
                        }
                        Some((channel_id, participant))
                    } else {
                        None
                    }
                };

                if let Some((channel_id, participant)) = removed_voice_channel {
                    let room = format!("voice:{}", channel_id);
                    let remaining_count = {
                        let channels = voice_channel_map.read().await;
                        channels.get(&channel_id).map_or(0, |participants| participants.len())
                    };
                    let _ = io.to(room.clone()).emit("voice_participant_left", &json!({
                        "channel_id": channel_id,
                        "socket_id": participant.socket_id,
                        "user_id": participant.user_id,
                    }));
                    let _ = io.to(room).emit("voice_channel_presence", &json!({
                        "channel_id": channel_id,
                        "participant_count": remaining_count,
                    }));
                }
                
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_io() -> SocketIo {
        let (_, io) = SocketIo::new_layer();
        // Register the default namespace so emit calls don't panic
        io.ns("/", |_: SocketRef| async {});
        io
    }

    #[tokio::test]
    async fn test_emit_new_message() {
        let io = make_io();
        emit_new_message(&io, "chan-1", &serde_json::json!({"content": "hello"}));
    }

    #[tokio::test]
    async fn test_emit_message_deleted() {
        let io = make_io();
        emit_message_deleted(&io, "chan-1", "msg-abc");
    }

    #[tokio::test]
    async fn test_emit_member_joined() {
        let io = make_io();
        emit_member_joined(&io, "srv-1", &serde_json::json!({"user_id": "u1"}));
    }

    #[tokio::test]
    async fn test_emit_member_left() {
        let io = make_io();
        emit_member_left(&io, "srv-1", "u1");
    }

    #[tokio::test]
    async fn test_emit_member_kicked() {
        let io = make_io();
        emit_member_kicked(&io, "srv-1", "u1");
    }

    #[tokio::test]
    async fn test_emit_member_banned() {
        let io = make_io();
        emit_member_banned(&io, "srv-1", "u1");
    }

    #[tokio::test]
    async fn test_emit_channel_created() {
        let io = make_io();
        emit_channel_created(&io, "srv-1", &serde_json::json!({"id": "c1"}));
    }

    #[tokio::test]
    async fn test_emit_channel_updated() {
        let io = make_io();
        emit_channel_updated(&io, "srv-1", &serde_json::json!({"id": "c1"}));
    }

    #[tokio::test]
    async fn test_emit_channel_deleted() {
        let io = make_io();
        emit_channel_deleted(&io, "srv-1", "c1");
    }

    #[tokio::test]
    async fn test_emit_user_status_changed() {
        let io = make_io();
        emit_user_status_changed(&io, "srv-1", "u1", "online");
    }

    #[tokio::test]
    async fn test_emit_user_profile_updated() {
        let io = make_io();
        emit_user_profile_updated(&io, "srv-1", "u1", "alice", Some("https://img"), "online");
        emit_user_profile_updated(&io, "srv-1", "u1", "alice", None, "away");
    }

    #[tokio::test]
    async fn test_emit_user_profile_updated_to_user_room() {
        let io = make_io();
        emit_user_profile_updated_to_user_room(&io, "u2", "u1", "alice", Some("https://img"), "dnd");
    }

    #[tokio::test]
    async fn test_emit_member_role_changed() {
        let io = make_io();
        emit_member_role_changed(&io, "srv-1", "u1", "admin");
    }

    #[tokio::test]
    async fn test_setup_socket_io() {
        use crate::domain::repositories::user_repository::MockUserRepository;
        let pool = sqlx::PgPool::connect_lazy(
            "postgres://rtc_user:rtc_password@localhost:5433/rtc_db"
        ).unwrap();
        let mock_user_repo = Arc::new(MockUserRepository::new());
        let (_layer, _io) = setup_socket_io(pool, mock_user_repo, "secret".to_string()).await;
    }
}

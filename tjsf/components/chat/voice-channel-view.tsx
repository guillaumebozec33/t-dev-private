"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { PhoneCall, PhoneOff, Volume2 } from "lucide-react";
import { useTranslation } from "@/lib/i18n/language-context";
import { useSocket } from "@/lib/socket/use-socket";
import { useAuthStore } from "@/lib/store/auth-store";
import { useSelectedChannel } from "@/hooks";
import { SOCKET_EVENTS_EMIT, SOCKET_EVENTS_LISTEN } from "@/lib/constants/socket-events";

type VoiceParticipant = {
  socket_id: string;
  user_id: string;
  username: string;
};

type VoiceChannelState = {
  channel_id: string;
  participants: VoiceParticipant[];
};

const rtcConfig: RTCConfiguration = {
  iceServers: [{ urls: ["stun:stun.l.google.com:19302"] }],
};

export default function VoiceChannelView() {
  const { t } = useTranslation();
  const { socket, isConnected } = useSocket();
  const { selectedChannel } = useSelectedChannel();
  const { user } = useAuthStore();

  const [isJoining, setIsJoining] = useState(false);
  const [isJoined, setIsJoined] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [participants, setParticipants] = useState<VoiceParticipant[]>([]);
  const [remoteStreams, setRemoteStreams] = useState<Record<string, MediaStream>>({});

  const localStreamRef = useRef<MediaStream | null>(null);
  const peerConnectionsRef = useRef<Record<string, RTCPeerConnection>>({});

  useEffect(() => {
    return () => {
      Object.values(peerConnectionsRef.current).forEach((peerConnection) => peerConnection.close());
      peerConnectionsRef.current = {};
      localStreamRef.current?.getTracks().forEach((track) => track.stop());
      localStreamRef.current = null;
    };
  }, []);

  const removePeer = (socketId: string) => {
    peerConnectionsRef.current[socketId]?.close();
    delete peerConnectionsRef.current[socketId];
    setRemoteStreams((current) => {
      const next = { ...current };
      delete next[socketId];
      return next;
    });
  };

  const createPeerConnection = useCallback((targetSocketId: string) => {
    const existingConnection = peerConnectionsRef.current[targetSocketId];
    if (existingConnection) {
      return existingConnection;
    }

    const peerConnection = new RTCPeerConnection(rtcConfig);

    localStreamRef.current?.getTracks().forEach((track) => {
      peerConnection.addTrack(track, localStreamRef.current as MediaStream);
    });

    peerConnection.onicecandidate = (event) => {
      if (!event.candidate || !socket) return;
      socket.emit(SOCKET_EVENTS_EMIT.VOICE_ICE_CANDIDATE, {
        target_socket_id: targetSocketId,
        candidate: event.candidate,
      });
    };

    peerConnection.ontrack = (event) => {
      const [stream] = event.streams;
      if (!stream) return;
      setRemoteStreams((current) => ({ ...current, [targetSocketId]: stream }));
    };

    peerConnection.onconnectionstatechange = () => {
      if (["failed", "closed", "disconnected"].includes(peerConnection.connectionState)) {
        removePeer(targetSocketId);
      }
    };

    peerConnectionsRef.current[targetSocketId] = peerConnection;
    return peerConnection;
  }, [socket]);

  const joinVoiceChannel = async () => {
    if (!socket || !selectedChannel?.id || !user) return;
    setError(null);
    setIsJoining(true);

    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true, video: false });
      localStreamRef.current = stream;
      socket.emit(SOCKET_EVENTS_EMIT.JOIN_VOICE_CHANNEL, {
        channel_id: selectedChannel.id,
        user_id: user.id,
        username: user.username,
      });
      setIsJoined(true);
      setParticipants((current) => {
        if (!socket.id) return current;
        const localParticipant = { socket_id: socket.id, user_id: user.id, username: user.username };
        return current.some((participant) => participant.socket_id === localParticipant.socket_id)
          ? current
          : [...current, localParticipant];
      });
    } catch {
      setError(t("voice.permissionDenied"));
    } finally {
      setIsJoining(false);
    }
  };

  const leaveVoiceChannel = () => {
    if (!socket || !selectedChannel?.id) return;
    socket.emit(SOCKET_EVENTS_EMIT.LEAVE_VOICE_CHANNEL, { channel_id: selectedChannel.id });
    Object.values(peerConnectionsRef.current).forEach((peerConnection) => peerConnection.close());
    peerConnectionsRef.current = {};
    localStreamRef.current?.getTracks().forEach((track) => track.stop());
    localStreamRef.current = null;
    setRemoteStreams({});
    setParticipants((current) => current.filter((participant) => participant.socket_id === socket.id));
    setIsJoined(false);
  };

  useEffect(() => {
    if (!socket || !isConnected || !selectedChannel?.id || selectedChannel.channel_type !== "voice") {
      return;
    }

    const handleVoiceState = async (state: VoiceChannelState) => {
      if (state.channel_id !== selectedChannel.id || !socket.id) return;
      const localParticipant = user
        ? [{ socket_id: socket.id, user_id: user.id, username: user.username }]
        : [];
      setParticipants([...state.participants, ...localParticipant]);

      for (const participant of state.participants) {
        if (participant.socket_id === socket.id) continue;
        const peerConnection = createPeerConnection(participant.socket_id);
        const offer = await peerConnection.createOffer();
        await peerConnection.setLocalDescription(offer);
        socket.emit(SOCKET_EVENTS_EMIT.VOICE_OFFER, {
          target_socket_id: participant.socket_id,
          sdp: offer,
        });
      }
    };

    const handleParticipantJoined = (participant: VoiceParticipant) => {
      setParticipants((current) => {
        if (current.some((item) => item.socket_id === participant.socket_id)) {
          return current;
        }
        return [...current, participant];
      });
    };

    const handleParticipantLeft = (data: { channel_id: string; socket_id: string }) => {
      if (data.channel_id !== selectedChannel.id) return;
      setParticipants((current) => current.filter((participant) => participant.socket_id !== data.socket_id));
      removePeer(data.socket_id);
    };

    const handleOffer = async (data: { source_socket_id: string; sdp: RTCSessionDescriptionInit }) => {
      const peerConnection = createPeerConnection(data.source_socket_id);
      await peerConnection.setRemoteDescription(new RTCSessionDescription(data.sdp));
      const answer = await peerConnection.createAnswer();
      await peerConnection.setLocalDescription(answer);
      socket.emit(SOCKET_EVENTS_EMIT.VOICE_ANSWER, {
        target_socket_id: data.source_socket_id,
        sdp: answer,
      });
    };

    const handleAnswer = async (data: { source_socket_id: string; sdp: RTCSessionDescriptionInit }) => {
      const peerConnection = peerConnectionsRef.current[data.source_socket_id];
      if (!peerConnection) return;
      await peerConnection.setRemoteDescription(new RTCSessionDescription(data.sdp));
    };

    const handleIceCandidate = async (data: { source_socket_id: string; candidate: RTCIceCandidateInit }) => {
      const peerConnection = peerConnectionsRef.current[data.source_socket_id];
      if (!peerConnection || !data.candidate) return;
      await peerConnection.addIceCandidate(new RTCIceCandidate(data.candidate));
    };

    socket.on(SOCKET_EVENTS_LISTEN.VOICE_CHANNEL_STATE, handleVoiceState);
    socket.on(SOCKET_EVENTS_LISTEN.VOICE_PARTICIPANT_JOINED, handleParticipantJoined);
    socket.on(SOCKET_EVENTS_LISTEN.VOICE_PARTICIPANT_LEFT, handleParticipantLeft);
    socket.on(SOCKET_EVENTS_LISTEN.VOICE_OFFER, handleOffer);
    socket.on(SOCKET_EVENTS_LISTEN.VOICE_ANSWER, handleAnswer);
    socket.on(SOCKET_EVENTS_LISTEN.VOICE_ICE_CANDIDATE, handleIceCandidate);

    return () => {
      socket.off(SOCKET_EVENTS_LISTEN.VOICE_CHANNEL_STATE, handleVoiceState);
      socket.off(SOCKET_EVENTS_LISTEN.VOICE_PARTICIPANT_JOINED, handleParticipantJoined);
      socket.off(SOCKET_EVENTS_LISTEN.VOICE_PARTICIPANT_LEFT, handleParticipantLeft);
      socket.off(SOCKET_EVENTS_LISTEN.VOICE_OFFER, handleOffer);
      socket.off(SOCKET_EVENTS_LISTEN.VOICE_ANSWER, handleAnswer);
      socket.off(SOCKET_EVENTS_LISTEN.VOICE_ICE_CANDIDATE, handleIceCandidate);
    };
  }, [socket, isConnected, selectedChannel?.id, selectedChannel?.channel_type, user, t, createPeerConnection]);

  useEffect(() => {
    return () => {
      if (isJoined && socket && selectedChannel?.id) {
        socket.emit(SOCKET_EVENTS_EMIT.LEAVE_VOICE_CHANNEL, { channel_id: selectedChannel.id });
      }
    };
  }, [isJoined, socket, selectedChannel?.id]);

  return (
    <div className="flex-1 bg-white flex flex-col">
      <div className="border-b border-gray-200 px-6 py-4 flex items-center justify-between">
        <div>
          <div className="flex items-center gap-2 text-gray-900 font-semibold">
            <Volume2 size={18} className="text-gray-500" />
            <span>{selectedChannel?.name}</span>
          </div>
          <p className="text-sm text-gray-500 mt-1">
            {isJoined ? t("voice.connected") : t("voice.idle")}
          </p>
        </div>

        {isJoined ? (
          <button
            onClick={leaveVoiceChannel}
            className="inline-flex items-center gap-2 rounded-lg bg-bordeaux px-4 py-2 text-sm font-medium text-white hover:bg-bordeaux-hover"
          >
            <PhoneOff size={16} />
            {t("voice.leave")}
          </button>
        ) : (
          <button
            onClick={joinVoiceChannel}
            disabled={isJoining || !isConnected}
            className="inline-flex items-center gap-2 rounded-lg bg-bordeaux px-4 py-2 text-sm font-medium text-white hover:bg-bordeaux-hover disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <PhoneCall size={16} />
            {isJoining ? t("voice.connecting") : t("voice.join")}
          </button>
        )}
      </div>

      <div className="p-6 flex-1 overflow-auto">
        {error && (
          <div className="mb-4 rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-600">
            {error}
          </div>
        )}

        <div className="rounded-xl border border-gray-200 bg-gray-50 p-4">
          <h2 className="text-sm font-semibold text-gray-700 mb-3">{t("voice.participants")}</h2>
          {participants.length === 0 ? (
            <p className="text-sm text-gray-500">{t("voice.empty")}</p>
          ) : (
            <div className="space-y-2">
              {participants.map((participant) => (
                <div
                  key={participant.socket_id}
                  className="flex items-center justify-between rounded-lg bg-white px-3 py-2 border border-gray-200"
                >
                  <span className="text-sm text-gray-800">{participant.username}</span>
                  {participant.user_id === user?.id && (
                    <span className="text-xs text-gray-500">{t("messages.you")}</span>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        {Object.entries(remoteStreams).map(([socketId, stream]) => (
          <audio
            key={socketId}
            autoPlay
            ref={(element) => {
              if (element && element.srcObject !== stream) {
                element.srcObject = stream;
              }
            }}
          />
        ))}
      </div>
    </div>
  );
}
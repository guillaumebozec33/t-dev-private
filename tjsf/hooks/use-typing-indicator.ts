'use client';

import { useEffect, useState, useCallback } from 'react';
import { useSocket } from '@/lib/socket/use-socket';
import { SOCKET_EVENTS_EMIT, SOCKET_EVENTS_LISTEN } from '@/lib/constants/socket-events';
import { useAuthStore } from '@/lib/store/auth-store';
import {useSelectedChannel} from "@/hooks";

interface TypingUser {
  user_id: string;
  username: string;
}

export function useTypingIndicator() {
  const { socket, isConnected } = useSocket();
  const {selectedChannel}= useSelectedChannel()
  const [typingUsers, setTypingUsers] = useState<TypingUser[]>([]);
  const user = useAuthStore((state) => state.user);

  const startTyping = useCallback(() => {
    if (socket && isConnected && selectedChannel?.id && user) {
      socket.emit(SOCKET_EVENTS_EMIT.TYPING_START, { channel_id: selectedChannel?.id, user_id: user.id });
    }
  }, [socket, isConnected, selectedChannel?.id, user]);

  const stopTyping = useCallback(() => {
    if (socket && isConnected && selectedChannel?.id && user) {
      socket.emit(SOCKET_EVENTS_EMIT.TYPING_STOP, { channel_id: selectedChannel?.id, user_id: user.id });
    }
  }, [socket, isConnected, selectedChannel?.id, user]);

  useEffect(() => {
    if (!socket || !isConnected || !selectedChannel?.id || !user) return;

    const handleUserTyping = (data: { channel_id: string; user_id: string; username: string }) => {
      if (data.channel_id === selectedChannel?.id && data.user_id !== user.id) {
        setTypingUsers((prev) => {
          if (prev.some((u) => u.user_id === data.user_id)) return prev;
          return [...prev, { user_id: data.user_id, username: data.username }];
        });
      }
    };

    const handleUserStopTyping = (data: { channel_id: string; user_id: string }) => {
      if (data.channel_id === selectedChannel?.id) {
        setTypingUsers((prev) => prev.filter((u) => u.user_id !== data.user_id));
      }
    };

    socket.on(SOCKET_EVENTS_LISTEN.USER_TYPING, handleUserTyping);
    socket.on(SOCKET_EVENTS_LISTEN.USER_STOP_TYPING, handleUserStopTyping);

    return () => {
      socket.off(SOCKET_EVENTS_LISTEN.USER_TYPING, handleUserTyping);
      socket.off(SOCKET_EVENTS_LISTEN.USER_STOP_TYPING, handleUserStopTyping);
    };
  }, [socket, isConnected, selectedChannel?.id, user]);

  return { typingUsers, startTyping, stopTyping };
}

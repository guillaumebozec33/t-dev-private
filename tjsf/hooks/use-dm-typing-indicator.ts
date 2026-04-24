'use client';

import { useEffect, useState, useCallback } from 'react';
import { useSocket } from '@/lib/socket/use-socket';
import { SOCKET_EVENTS_EMIT, SOCKET_EVENTS_LISTEN } from '@/lib/constants/socket-events';
import { useAuthStore } from '@/lib/store/auth-store';

interface TypingUser {
  user_id: string;
  username: string;
}

export function useDmTypingIndicator(conversationId: string | undefined, recipientId: string | undefined) {
  const { socket, isConnected } = useSocket();
  const [typingUsers, setTypingUsers] = useState<TypingUser[]>([]);
  const user = useAuthStore((state) => state.user);

  const startTyping = useCallback(() => {
    if (socket && isConnected && conversationId && recipientId && user) {
      socket.emit(SOCKET_EVENTS_EMIT.TYPING_START, {
        conversation_id: conversationId,
        recipient_id: recipientId,
        user_id: user.id,
      });
    }
  }, [socket, isConnected, conversationId, recipientId, user]);

  const stopTyping = useCallback(() => {
    if (socket && isConnected && conversationId && recipientId && user) {
      socket.emit(SOCKET_EVENTS_EMIT.TYPING_STOP, {
        conversation_id: conversationId,
        recipient_id: recipientId,
        user_id: user.id,
      });
    }
  }, [socket, isConnected, conversationId, recipientId, user]);

  useEffect(() => {
    if (!socket || !isConnected || !conversationId || !user) return;

    const handleUserTyping = (data: { conversation_id?: string; channel_id?: string; user_id: string; username: string }) => {
      if (data.conversation_id === conversationId && data.user_id !== user.id) {
        setTypingUsers((prev) => {
          if (prev.some((u) => u.user_id === data.user_id)) return prev;
          return [...prev, { user_id: data.user_id, username: data.username }];
        });
      }
    };

    const handleUserStopTyping = (data: { conversation_id?: string; channel_id?: string; user_id: string }) => {
      if (data.conversation_id === conversationId) {
        setTypingUsers((prev) => prev.filter((u) => u.user_id !== data.user_id));
      }
    };

    socket.on(SOCKET_EVENTS_LISTEN.USER_TYPING, handleUserTyping);
    socket.on(SOCKET_EVENTS_LISTEN.USER_STOP_TYPING, handleUserStopTyping);

    return () => {
      socket.off(SOCKET_EVENTS_LISTEN.USER_TYPING, handleUserTyping);
      socket.off(SOCKET_EVENTS_LISTEN.USER_STOP_TYPING, handleUserStopTyping);
    };
  }, [socket, isConnected, conversationId, user]);

  return { typingUsers, startTyping, stopTyping };
}

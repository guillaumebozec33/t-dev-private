'use client';

import { useEffect, useRef } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { useSocket } from '@/lib/socket/use-socket';
import { SOCKET_EVENTS_EMIT, SOCKET_EVENTS_LISTEN } from '@/lib/constants/socket-events';
import { DmConversation, DmMessage } from '@/types';
import { useTauriNotification } from './use-tauri-notification';

export function useDmSync(
  userId?: string,
  onNewMessage?: (conversationId: string, senderId: string) => void
) {
  const { notify } = useTauriNotification();
  const { socket, isConnected } = useSocket();
  const queryClient = useQueryClient();
  // Ref to always have the latest callback without re-subscribing the socket listener
  const onNewMessageRef = useRef(onNewMessage);
  useEffect(() => { onNewMessageRef.current = onNewMessage; });

  useEffect(() => {
    if (!socket || !isConnected || !userId) return;

    socket.emit(SOCKET_EVENTS_EMIT.JOIN_DM, { user_id: userId });

    return () => {
      socket.emit(SOCKET_EVENTS_EMIT.LEAVE_DM, { user_id: userId });
    };
  }, [socket, isConnected, userId]);

  useEffect(() => {
    if (!socket || !isConnected) return;

      const handle = async (message: DmMessage) => {
          queryClient.setQueryData<DmMessage[]>(
              ['dm_messages', message.conversation_id],
              (old = []) => [...old, message]
          );
          queryClient.invalidateQueries({ queryKey: ['dm_conversations'] });
          onNewMessageRef.current?.(message.conversation_id, message.sender_id);

          if (message.sender_id !== userId) {
            notify(message.sender_username || 'Nouveau message', message.content || 'Vous avez reçu un message direct.');
          }
      };

    const handleUserProfileUpdated = (data: {
      user_id: string;
      username: string;
      avatar_url: string | null;
    }) => {
      queryClient.setQueryData<DmConversation[]>(
        ['dm_conversations'],
        (old = []) =>
          old.map((conversation) =>
            conversation.other_user_id === data.user_id
              ? {
                  ...conversation,
                  other_username: data.username,
                  other_avatar_url: data.avatar_url,
                }
              : conversation
          )
      );

      const conversationIds = queryClient
        .getQueryData<DmConversation[]>(['dm_conversations'])
        ?.filter((conversation) => conversation.other_user_id === data.user_id)
        .map((conversation) => conversation.id) ?? [];

      for (const conversationId of conversationIds) {
        queryClient.setQueryData<DmMessage[]>(
          ['dm_messages', conversationId],
          (old = []) =>
            old.map((message) =>
              message.sender_id === data.user_id
                ? {
                    ...message,
                    sender_username: data.username,
                    sender_avatar_url: data.avatar_url,
                  }
                : message
            )
        );
      }
    };

    socket.on(SOCKET_EVENTS_LISTEN.DM_MESSAGE_RECEIVED, handle);
    socket.on(SOCKET_EVENTS_LISTEN.USER_PROFILE_UPDATED, handleUserProfileUpdated);
    return () => {
      socket.off(SOCKET_EVENTS_LISTEN.DM_MESSAGE_RECEIVED, handle);
      socket.off(SOCKET_EVENTS_LISTEN.USER_PROFILE_UPDATED, handleUserProfileUpdated);
    };
  }, [socket, isConnected, queryClient]);
}

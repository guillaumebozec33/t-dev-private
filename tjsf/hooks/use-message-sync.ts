'use client';

import { useEffect } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { useSocket } from '@/lib/socket/use-socket';
import { SOCKET_EVENTS_LISTEN } from '@/lib/constants/socket-events';
import { Message } from '@/types';
import { useSelectedChannel } from "@/hooks";
import { useAuthStore } from '@/lib/store/auth-store';
import { useTauriNotification } from './use-tauri-notification';

export function useMessageSync() {
    const { selectedChannel } = useSelectedChannel();
    const { socket, isConnected } = useSocket();
    const queryClient = useQueryClient();
    const { user } = useAuthStore();
    const { notify } = useTauriNotification();

  useEffect(() => {
    if (!socket || !isConnected || !selectedChannel?.id) return;

    const handleNewMessage = (message: Message) => {
      if (message.channel_id === selectedChannel?.id) {
          queryClient.setQueryData(
              ['messages', selectedChannel?.id],
              (old: { pages: Message[][], pageParams: any[] } | undefined) => {
                  if (!old) return old;
                  const newPages = [...old.pages];
                  newPages[0] = [message, ...newPages[0].filter((m) => !m.id.startsWith("temp-"))]; // ajoute en tête (ordre DESC)
                  return { ...old, pages: newPages };
              }
          );
          if (message.author_id !== user?.id) {
              notify(`#${selectedChannel.name}`, `${message.author_username}: ${message.content}`);
          }
      }
    };

      const handleMessageDeleted = (data: { message_id: string; channel_id: string }) => {
          if (data.channel_id === selectedChannel?.id) {
              queryClient.setQueryData<{ pages: Message[][], pageParams: any[] }>(
              ['messages', selectedChannel?.id],
                  (old) => {
                      if (!old) return old;
                      return {
                          ...old,
                          pages: old.pages.map(page =>
                              page.filter(msg => msg.id !== data.message_id)
                          ),
                      };
                  }
              );
          }
      };

      const handleMessageEdited = (message: Message) => {
          if (message.channel_id === selectedChannel?.id) {
              queryClient.setQueryData<{ pages: Message[][], pageParams: any[] }>(
                  ['messages', selectedChannel?.id],
                  (old) => {
                      if (!old) return old;
                      return {
                          ...old,
                          pages: old.pages.map(page =>
                              page.map(m => m.id === message.id ? message : m)
                          ),
                      };
                  }
              );
          }
      };

      const handleUserProfileUpdated = (data: {
          user_id: string;
          username: string;
          avatar_url: string | null;
      }) => {
          queryClient.setQueryData<{pages : Message[][], pageParams: any[] }>(
              ['messages', selectedChannel?.id],
              (old) => {
                  if (!old) return old;
                  return {
                      ...old,
                      pages: old.pages.map((page) =>
                          page.map((m) =>
                              m.author_id === data.user_id
                                  ? {
                                      ...m,
                                      author_username: data.username,
                                      author_avatar_url: data.avatar_url ?? "",
                                  }
                                  : m
                          )
                      ),
                  };
              }
          );
      };

    socket.on(SOCKET_EVENTS_LISTEN.NEW_MESSAGE, handleNewMessage);
    socket.on(SOCKET_EVENTS_LISTEN.MESSAGE_DELETED, handleMessageDeleted);
    socket.on(SOCKET_EVENTS_LISTEN.MESSAGE_EDITED, handleMessageEdited);
    socket.on(SOCKET_EVENTS_LISTEN.USER_PROFILE_UPDATED, handleUserProfileUpdated);

    return () => {
      socket.off(SOCKET_EVENTS_LISTEN.NEW_MESSAGE, handleNewMessage);
      socket.off(SOCKET_EVENTS_LISTEN.MESSAGE_DELETED, handleMessageDeleted);
      socket.off(SOCKET_EVENTS_LISTEN.MESSAGE_EDITED, handleMessageEdited);
      socket.off(SOCKET_EVENTS_LISTEN.USER_PROFILE_UPDATED, handleUserProfileUpdated);
    };
  }, [socket, isConnected, selectedChannel?.id, queryClient]);
}

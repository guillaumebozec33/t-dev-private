'use client';

import { useEffect } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { useSocket } from '@/lib/socket/use-socket';
import { SOCKET_EVENTS_LISTEN } from '@/lib/constants/socket-events';
import { Channel, Member, Message } from '@/types';
import {useAuthStore} from "@/lib/store/auth-store";
import {useTranslation} from "@/lib/i18n/language-context";
import {useSelectedServer} from "@/hooks";
import {useServerStore} from "@/lib/store/server-store";
import {useChannelStore} from "@/lib/store/channel-store";

export function useMemberSync() {
    const {selectedServer} = useSelectedServer()
  const { socket, isConnected } = useSocket();
  const queryClient = useQueryClient();
  const user = useAuthStore.getState().user
    const {t} = useTranslation()

  // Global listener: handles kick/ban of the current user regardless of selected server
  useEffect(() => {
    if (!socket || !isConnected || !user) return;

    const handleSelfKicked = (data: { server_id: string; member_id: string }) => {
      if (data.member_id !== user.id) return;
      alert(t("sanction.kick"));
      queryClient.invalidateQueries({ queryKey: ['servers'] });
      queryClient.invalidateQueries({ queryKey: ['members', data.server_id] });
      queryClient.invalidateQueries({ queryKey: ['channels', data.server_id] });
      if (useServerStore.getState().selectedServerId === data.server_id) {
        useServerStore.getState().resetSelectedServerId();
        useChannelStore.getState().resetSelectedChannelId();
      }
    };

    const handleSelfBanned = (data: { server_id: string; member_id: string }) => {
      if (data.member_id !== user.id) return;
      alert(t("sanction.ban"));
      queryClient.invalidateQueries({ queryKey: ['servers'] });
      queryClient.invalidateQueries({ queryKey: ['members', data.server_id] });
      queryClient.invalidateQueries({ queryKey: ['channels', data.server_id] });
      if (useServerStore.getState().selectedServerId === data.server_id) {
        useServerStore.getState().resetSelectedServerId();
        useChannelStore.getState().resetSelectedChannelId();
      }
    };

    socket.on(SOCKET_EVENTS_LISTEN.MEMBER_KICKED, handleSelfKicked);
    socket.on(SOCKET_EVENTS_LISTEN.MEMBER_BANNED, handleSelfBanned);

    return () => {
      socket.off(SOCKET_EVENTS_LISTEN.MEMBER_KICKED, handleSelfKicked);
      socket.off(SOCKET_EVENTS_LISTEN.MEMBER_BANNED, handleSelfBanned);
    };
  }, [socket, isConnected, user, queryClient, t]);

  useEffect(() => {
    if (!socket || !isConnected || !selectedServer?.id) return;

    const injectSystemMessage = (channelId: string, type: 'joined' | 'kicked' | 'banned', username: string) => {
      const systemMsg: Message = {
        id: `system-${Date.now()}-${type}`,
        author_id: 'system',
        channel_id: channelId,
        content: `__system__:${type}:${username}`,
        author_username: 'system',
        author_avatar_url: '',
        edited: false,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      queryClient.setQueryData<{ pages: Message[][], pageParams: any[] }>(
        ['messages', channelId],
        (old) => {
          if (!old) return old;
          const newPages = [...old.pages];
          newPages[0] = [systemMsg, ...newPages[0]];
          return { ...old, pages: newPages };
        }
      );
    };

    const getFirstChannelId = (sid: string): string | undefined => {
      const channels = queryClient.getQueryData<Channel[]>(['channels', sid]);
      return channels?.[0]?.id;
    };

    const handleMemberJoined = (data: { server_id: string; user: Member }) => {
      if (data.server_id === selectedServer?.id) {
        queryClient.setQueryData<Member[]>(
          ['members', selectedServer?.id],
          (old = []) => [...old, data.user]
        );
        const firstChannelId = getFirstChannelId(selectedServer?.id);
        if (firstChannelId) {
          injectSystemMessage(firstChannelId, 'joined', data.user.username);
        }
      }
    };

    const handleMemberLeft = (data: { server_id: string; user_id: string }) => {
      if (data.server_id === selectedServer?.id) {
        queryClient.setQueryData<Member[]>(
          ['members', selectedServer?.id],
          (old = []) => old.filter((m) => m.user_id !== data.user_id)
        );
      }
    };
      const handleMemberKicked = (data: { server_id: string; member_id: string }) => {
          if (data.server_id === selectedServer?.id && data.member_id !== user?.id) {
              const members = queryClient.getQueryData<Member[]>(['members', selectedServer?.id]);
              const kickedMember = members?.find((m: Member) => m.user_id === data.member_id);
              queryClient.setQueryData<Member[]>(
                  ['members', selectedServer?.id],
                  (old = []) => old.filter((m) => m.user_id !== data.member_id)
              );
              const firstChannelId = getFirstChannelId(selectedServer?.id);
              if (firstChannelId && kickedMember) {
                  injectSystemMessage(firstChannelId, 'kicked', kickedMember.username);
              }
          }
      };

      const handleMemberBanned = (data: { server_id: string; member_id: string }) => {
          if (data.server_id === selectedServer?.id && data.member_id !== user?.id) {
              const members = queryClient.getQueryData<Member[]>(['members', selectedServer?.id]);
              const bannedMember = members?.find((m: Member) => m.user_id === data.member_id);
              queryClient.setQueryData<Member[]>(
                  ['members', selectedServer?.id],
                  (old = []) => old.filter((m) => m.user_id !== data.member_id)
              );
              const firstChannelId = getFirstChannelId(selectedServer?.id);
              if (firstChannelId && bannedMember) {
                  injectSystemMessage(firstChannelId, 'banned', bannedMember.username);
              }
          }
      };

    const handleUserStatusChanged = (data: { server_id: string; user_id: string; status: string }) => {
      if (data.server_id === selectedServer?.id) {
        queryClient.setQueryData<Member[]>(
          ['members', selectedServer?.id],
          (old = []) => old.map((m) => 
            m.user_id === data.user_id ? { ...m, status: data.status as Member['status'] } : m
          )
        );
      }
    };

    const handleUserProfileUpdated = (data: {
      server_id: string;
      user_id: string;
      username: string;
      avatar_url: string | null;
      status: string;
    }) => {
      if (data.server_id === selectedServer?.id) {
        queryClient.setQueryData<Member[]>(
          ['members', selectedServer?.id],
          (old = []) =>
            old.map((m) =>
              m.user_id === data.user_id
                ? {
                    ...m,
                    username: data.username,
                    avatar_url: data.avatar_url,
                    status: data.status as Member['status'],
                  }
                : m
            )
        );
      }
    };

    const handleMemberRoleChanged = (data: { server_id: string; user_id: string; role: string }) => {
      if (data.server_id === selectedServer?.id) {
        queryClient.setQueryData<Member[]>(
          ['members', selectedServer?.id],
          (old = []) => old.map((m) => 
            m.user_id === data.user_id ? { ...m, role: data.role } : m
          )
        );
        queryClient.invalidateQueries({ queryKey: ['channels', selectedServer?.id] });
      }
    };

    socket.on(SOCKET_EVENTS_LISTEN.MEMBER_JOINED, handleMemberJoined);
    socket.on(SOCKET_EVENTS_LISTEN.MEMBER_LEFT, handleMemberLeft);
      socket.on(SOCKET_EVENTS_LISTEN.MEMBER_KICKED, handleMemberKicked);
      socket.on(SOCKET_EVENTS_LISTEN.MEMBER_BANNED, handleMemberBanned);
    socket.on(SOCKET_EVENTS_LISTEN.MEMBER_ROLE_CHANGED, handleMemberRoleChanged);
    socket.on(SOCKET_EVENTS_LISTEN.USER_STATUS_CHANGED, handleUserStatusChanged);
    socket.on(SOCKET_EVENTS_LISTEN.USER_PROFILE_UPDATED, handleUserProfileUpdated);

    return () => {
      socket.off(SOCKET_EVENTS_LISTEN.MEMBER_JOINED, handleMemberJoined);
      socket.off(SOCKET_EVENTS_LISTEN.MEMBER_LEFT, handleMemberLeft);
        socket.off(SOCKET_EVENTS_LISTEN.MEMBER_KICKED, handleMemberKicked);
        socket.off(SOCKET_EVENTS_LISTEN.MEMBER_BANNED, handleMemberBanned);
      socket.off(SOCKET_EVENTS_LISTEN.MEMBER_ROLE_CHANGED, handleMemberRoleChanged);
      socket.off(SOCKET_EVENTS_LISTEN.USER_STATUS_CHANGED, handleUserStatusChanged);
      socket.off(SOCKET_EVENTS_LISTEN.USER_PROFILE_UPDATED, handleUserProfileUpdated);
    };
  }, [socket, isConnected, selectedServer?.id, queryClient]);
}

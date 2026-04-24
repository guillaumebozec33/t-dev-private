'use client';

import { useEffect } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { useSocket } from '@/lib/socket/use-socket';
import { SOCKET_EVENTS_LISTEN } from '@/lib/constants/socket-events';
import { Message } from '@/types';
import { useSelectedChannel } from "@/hooks";
import { useAuthStore } from '@/lib/store/auth-store';
import { useTauriNotification } from './use-tauri-notification';
import {useChannels, useServers} from "@/hooks";

export function useMessageSync() {
    const { selectedChannel } = useSelectedChannel();
    const { socket, isConnected } = useSocket();
    const queryClient = useQueryClient();
    const { user } = useAuthStore();
    const { notify } = useTauriNotification();
    const {servers} = useServers();
    const {channels} = useChannels();


    useEffect(() => {
        if (!socket || !isConnected) return;

        const handleNewMessage = (message: Message) => {
            // Notif pour tous les canaux
            if (message.author_id !== user?.id) {
                const channel = channels.find(c => c.id === message.channel_id);
                const server = servers.find(s => s.id === channel?.server_id);
                notify(
                    `${server?.name ?? 'Serveur inconnu'} • #${channel?.name ?? 'canal inconnu'}`,
                    `${message.author_username}: ${message.content}`
                );
            }

            // Mise à jour cache uniquement pour le canal sélectionné
            if (message.channel_id === selectedChannel?.id) {
                queryClient.setQueryData(
                    ['messages', selectedChannel?.id],
                    (old: { pages: Message[][], pageParams: any[] } | undefined) => {
                        if (!old) return old;
                        const newPages = [...old.pages];
                        newPages[0] = [message, ...newPages[0].filter((m) => !m.id.startsWith("temp-"))];
                        return { ...old, pages: newPages };
                    }
                );
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
            queryClient.setQueryData<{ pages: Message[][], pageParams: any[] }>(
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
    }, [socket, isConnected, selectedChannel?.id, queryClient, user?.id, channels, servers, notify]);
}

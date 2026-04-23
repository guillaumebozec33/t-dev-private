'use client';

import { useState, useEffect, useCallback, useRef } from 'react';
import { useRouter } from 'next/navigation';
import { useQueryClient } from '@tanstack/react-query';
import { useQuery } from '@tanstack/react-query';
import { useAuthStore } from '@/lib/store/auth-store';
import { useServerStore } from '@/lib/store/server-store';
import { useChannelStore } from '@/lib/store/channel-store';
import { useDmStore } from '@/lib/store/dm-store';
import { getMe } from '@/lib/api/endpoints/users';
import { openConversation } from '@/lib/api/endpoints/dm';
import { useSocket } from '@/lib/socket/use-socket';
import { useMessageSync } from './use-message-sync';
import { useChannelSync } from './use-channel-sync';
import { useMemberSync } from './use-member-sync';
import { useServerSync } from './use-server-sync';
import { useUserStatus } from './use-user-status';
import { useDmSync } from './use-dm-sync';
import { SOCKET_EVENTS_EMIT } from '@/lib/constants/socket-events';
import { User } from '@/types';
import {useSelectedServer, useSelectedChannel,useServers,useChannels} from '@/hooks';
import {useSelectedDmConversation} from "@/hooks/use-selected-dm-conversation";

export function useChat() {
    const router = useRouter();
    const { isAuthenticated, _hasHydrated } = useAuthStore();
    const { selectedServerId } = useSelectedServer();
    const { selectedChannelId } = useSelectedChannel();
    const { servers, isLoading: serversLoading } = useServers();
    const { channels } = useChannels();
    const {selectedDmConversationId, setSelectedDmConversationId,resetSelectedDmConversationId} = useSelectedDmConversation();
    // const [selectedDmConversationId, setSelectedDmConversationId] = useState<string | undefined>();
    const [isDmInboxOpen, setIsDmInboxOpen] = useState(false);
    const [unreadDmIds, setUnreadDmIds] = useState<Set<string>>(new Set());

    const queryClient = useQueryClient();
    const { socket, isConnected } = useSocket();

    useEffect(() => {
        try {
            const hasServer = !!JSON.parse(localStorage.getItem('server-storage') ?? '{}')?.state?.selectedServerId;
            if (!hasServer) {
                const hasDm = !!JSON.parse(localStorage.getItem('dm-storage') ?? '{}')?.state?.selectedDmConversationId;
                if (hasDm) setIsDmInboxOpen(true);
            }
        } catch { /* ignore */ }
    }, []);

    const selectedDmConversationIdRef = useRef(selectedDmConversationId);
    useEffect(() => {
        selectedDmConversationIdRef.current = selectedDmConversationId;
    }, [selectedDmConversationId]);

    // — Auth
    const handleLogout = () => {
        useAuthStore.getState().logout();
        router.push('/');
    };

    const userQuery = useQuery<User>({
        queryKey: ['me'],
        queryFn: getMe,
        enabled: _hasHydrated && isAuthenticated,
        retry: false,
    });

    useEffect(() => {
        if (!_hasHydrated) return;
        if (!isAuthenticated) { router.push('/login'); return; }
        if (userQuery.isError) handleLogout();
    }, [_hasHydrated, isAuthenticated, userQuery.isError]);

    // — Guards
    useEffect(() => {
        if (serversLoading || !selectedServerId) return;
        const serverExists = servers.some((s) => s.id === selectedServerId);
        if (!serverExists) {
            useServerStore.getState().resetSelectedServerId();
            useChannelStore.getState().resetSelectedChannelId();
            useDmStore.getState().resetSelectedDmConversationId();
            setIsDmInboxOpen(false);
        }
    }, [servers, serversLoading, selectedServerId]);

    useEffect(() => {
        if (!selectedServerId || !channels.length) return;
        if (!selectedChannelId || !channels.find((c) => c.id === selectedChannelId)) {
            useChannelStore.getState().setSelectedChannelId(channels[0].id);
        }
    }, [channels, selectedServerId, selectedChannelId]);

    // — Socket
    useEffect(() => {
        if (!socket || !isConnected || !selectedServerId) return;
        socket.emit(SOCKET_EVENTS_EMIT.JOIN_SERVER, { server_id: selectedServerId });
        return () => { socket.emit(SOCKET_EVENTS_EMIT.LEAVE_SERVER, { server_id: selectedServerId }); };
    }, [socket, isConnected, selectedServerId]);

    useEffect(() => {
        if (!socket || !isConnected || !selectedChannelId) return;
        socket.emit(SOCKET_EVENTS_EMIT.JOIN_CHANNEL, { channel_id: selectedChannelId });
        return () => { socket.emit(SOCKET_EVENTS_EMIT.LEAVE_CHANNEL, { channel_id: selectedChannelId }); };
    }, [socket, isConnected, selectedChannelId]);

    // — Sync hooks
    const handleIncomingDm = useCallback((conversationId: string, senderId: string) => {
        if (senderId === userQuery.data?.id) return;
        if (conversationId === selectedDmConversationIdRef.current) return;
        setUnreadDmIds((prev) => new Set([...prev, conversationId]));
    }, [userQuery.data?.id]);

    useMessageSync();
    useChannelSync();
    useMemberSync();
    useServerSync();
    useUserStatus();
    useDmSync(userQuery.data?.id, handleIncomingDm);

    // — Handlers
    const handleServerSelect = (serverId: string) => {
        if (serverId === selectedServerId && !isDmInboxOpen) return;
        setIsDmInboxOpen(false);
        resetSelectedDmConversationId();
        useChannelStore.getState().resetSelectedChannelId();
        useServerStore.getState().setSelectedServerId(serverId);
    };

    const handleChannelSelect = (channelId: string) => {
        setIsDmInboxOpen(false);
        resetSelectedDmConversationId();
        useChannelStore.getState().setSelectedChannelId(channelId);
    };

    const handleServerLeave = () => {
        useServerStore.getState().resetSelectedServerId();
        useChannelStore.getState().resetSelectedChannelId();
        resetSelectedDmConversationId();
        setIsDmInboxOpen(false);
    };

    const handleOpenDmInbox = () => {
        useServerStore.getState().resetSelectedServerId();
        useChannelStore.getState().resetSelectedChannelId();
        setIsDmInboxOpen(true);
    };

    const markDmRead = (conversationId: string) => {
        setUnreadDmIds((prev) => {
            const next = new Set(prev);
            next.delete(conversationId);
            return next;
        });
    };

    const handleDmOpen = async (userId: string) => {
        try {
            console.log('DM open', userId)
            const conversation = await openConversation(userId);
            console.log('DM open response:', conversation)
            queryClient.invalidateQueries({ queryKey: ['dm_conversations'] });
            useChannelStore.getState().resetSelectedChannelId();
            setSelectedDmConversationId(conversation.id);
            useServerStore.getState().resetSelectedServerId();
            useChannelStore.getState().resetSelectedChannelId();
            setSelectedDmConversationId(conversation.id);
            setIsDmInboxOpen(true);
            markDmRead(conversation.id);
        } catch (error) {
            console.error('DM open failed:', error);
        }
    };

    const handleDmConversationSelect = (conversationId: string) => {
        useServerStore.getState().resetSelectedServerId();
        useChannelStore.getState().resetSelectedChannelId();
        setSelectedDmConversationId(conversationId);
        setIsDmInboxOpen(true);
        markDmRead(conversationId);
    };

    return {
        selectedDmConversationId,
        isDmInboxOpen,
        unreadDmIds,
        hasUnreadDms: unreadDmIds.size > 0,
        handleServerSelect,
        handleChannelSelect,
        handleServerLeave,
        handleOpenDmInbox,
        handleDmOpen,
        handleDmConversationSelect,
        handleLogout,
    };
}
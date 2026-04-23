import { useServerStore } from '@/lib/store/server-store';
import { useServers } from './use-servers';

export function useSelectedServer() {
    const { selectedServerId, setSelectedServerId, resetSelectedServerId } = useServerStore();
    const { servers } = useServers();

    const selectedServer = servers.find((s) => s.id === selectedServerId) ?? null;
    return { selectedServer, selectedServerId, setSelectedServerId, resetSelectedServerId };
}
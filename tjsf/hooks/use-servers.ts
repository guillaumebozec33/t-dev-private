import { useQuery } from '@tanstack/react-query';
import { getServers } from '@/lib/api/endpoints/servers';
import { Server } from '@/types';

export function useServers() {
    const { data: servers = [], isLoading } = useQuery<Server[]>({
        queryKey: ['servers'],
        queryFn: getServers,
    });

    return { servers, isLoading };
}
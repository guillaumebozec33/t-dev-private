import { useQuery } from '@tanstack/react-query';
import { getChannels } from '@/lib/api/endpoints/channels';
import { Channel } from '@/types';
import { useSelectedServer } from './use-selected-server';

export function useChannels() {
    const { selectedServerId } = useSelectedServer();

    const { data: channels = [], isLoading } = useQuery<Channel[]>({
        queryKey: ['channels', selectedServerId],
        queryFn: () => getChannels(selectedServerId!),
        enabled: !!selectedServerId,
    });

    return { channels, isLoading };
}
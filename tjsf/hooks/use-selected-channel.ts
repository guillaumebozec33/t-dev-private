import { useChannelStore } from '@/lib/store/channel-store';
import { useChannels } from './use-channels';

export function useSelectedChannel() {
    const { selectedChannelId, setSelectedChannelId, resetSelectedChannelId } = useChannelStore();
    const { channels } = useChannels();

    const selectedChannel = channels.find((c) => c.id === selectedChannelId) ?? null;

    return { selectedChannel, selectedChannelId, setSelectedChannelId, resetSelectedChannelId };
}
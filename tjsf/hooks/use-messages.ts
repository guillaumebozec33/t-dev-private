import { useInfiniteQuery } from '@tanstack/react-query';
import { getMessages } from '@/lib/api/endpoints';
import { Message } from '@/types';
import { useSelectedChannel } from './use-selected-channel';

export function useMessages() {
    const { selectedChannelId } = useSelectedChannel();

    const { data, isLoading, fetchNextPage, hasNextPage, isFetchingNextPage } =
        useInfiniteQuery<Message[]>({
            queryKey: ['messages', selectedChannelId],
            queryFn: ({ pageParam }) => getMessages(selectedChannelId!, pageParam as string | undefined),
            initialPageParam: undefined,
            getNextPageParam: (lastPage) =>
                lastPage.length < 50 ? undefined : lastPage[lastPage.length - 1].id,
            enabled: !!selectedChannelId,
        });

    const messages = data?.pages
        .slice()
        .reverse()
        .flatMap((page) => [...page].reverse()) ?? [];

    return { messages, isLoading, fetchNextPage, hasNextPage, isFetchingNextPage };
}
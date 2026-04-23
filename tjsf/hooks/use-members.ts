import { useQuery } from '@tanstack/react-query';
import { getMembers } from '@/lib/api/endpoints/members';
import { Member } from '@/types';
import { useSelectedServer } from './use-selected-server';

export function useMembers() {
    const { selectedServerId } = useSelectedServer();

    const { data: members = [], isLoading } = useQuery<Member[]>({
        queryKey: ['members', selectedServerId],
        queryFn: () => getMembers(selectedServerId!),
        enabled: !!selectedServerId,
    });

    return { members, isLoading };
}
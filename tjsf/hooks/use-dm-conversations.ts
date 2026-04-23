import {useQuery} from "@tanstack/react-query";
import {DmConversation} from "@/types";
import {getConversations} from "@/lib/api/endpoints";

export function useDmConversations() {
    const { data: conversations = [], isLoading } = useQuery<DmConversation[]>({
        queryKey: ['dm_conversations'],
        queryFn: getConversations,
    });

    return { conversations, isLoading };
}
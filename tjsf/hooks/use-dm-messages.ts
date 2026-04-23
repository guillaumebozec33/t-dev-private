import {useQuery} from "@tanstack/react-query";
import {DmMessage} from "@/types";
import {getDmMessages} from "@/lib/api/endpoints";
import {useSelectedDmConversation} from "@/hooks/use-selected-dm-conversation";

export function useDmMessages() {
    const {selectedDmConversationId} = useSelectedDmConversation();
    const { data: dmMessages = [], isLoading } = useQuery<DmMessage[]>({
        queryKey: ['dm_messages',selectedDmConversationId],
        queryFn: () => getDmMessages(selectedDmConversationId!),
        enabled: !!selectedDmConversationId,
    });

    return { dmMessages, isLoading };
}

import {useDmConversations} from "@/hooks/use-dm-conversations";
import {useDmStore} from "@/lib/store/dm-store";

export function useSelectedDmConversation() {
    const { selectedDmConversationId, setSelectedDmConversationId, resetSelectedDmConversationId } = useDmStore();
    const {conversations} = useDmConversations();

    const selectedConversation = conversations.find((s) => s.id === selectedDmConversationId) ?? null;
    return { selectedConversation, selectedDmConversationId, setSelectedDmConversationId, resetSelectedDmConversationId };
}
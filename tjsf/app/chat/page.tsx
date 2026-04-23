"use client";
import { useChat } from "@/hooks";
import ChatLayout from "@/components/chat/chat-layout";
import { ToastProvider } from "@/components";

export default function Chat() {
  const chatState = useChat();

  return (
    <ToastProvider>
    <ChatLayout
      isDmInboxOpen={chatState.isDmInboxOpen}
      hasUnreadDms={chatState.hasUnreadDms}
      unreadDmIds={chatState.unreadDmIds}
      onServerSelect={chatState.handleServerSelect}
      onServerLeave={chatState.handleServerLeave}
      onChannelSelect={chatState.handleChannelSelect}
      onOpenDmInbox={chatState.handleOpenDmInbox}
      onDmOpen={chatState.handleDmOpen}
      onDmConversationSelect={chatState.handleDmConversationSelect}
      onLogout={chatState.handleLogout}
    />
    </ToastProvider>
  );
}

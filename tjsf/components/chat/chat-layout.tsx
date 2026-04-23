"use client";

import { useState } from "react";
import ServerList from "@/components/chat/server-list";
import ChannelList from "@/components/chat/channel-list";
import MessageList from "@/components/chat/message-list";
import MembersList from "@/components/chat/members-list";
import DmView from "@/components/chat/dm-view";
import DmConversationList from "@/components/chat/dm-conversation-list";
import { Menu, Users } from "lucide-react";
import { useAuthStore} from "@/lib/store/auth-store";
import Profile from "@/components/chat/profile";
import {useResponsive,useSelectedChannel,useSelectedServer} from '@/hooks'
import { useTranslation } from "@/lib/i18n/language-context";
import {useSelectedDmConversation} from '@/hooks/use-selected-dm-conversation'
import {useDmConversations} from "@/hooks/use-dm-conversations";
import {useDmMessages} from "@/hooks/use-dm-messages";

type MobilePanel = "servers" | "channels" | "messages" | "members" | null;

interface ChatLayoutProps {
  isDmInboxOpen: boolean;
  hasUnreadDms?: boolean;
  unreadDmIds?: Set<string>;
  onServerSelect: (serverId: string) => void;
  onServerLeave: () => void;
  onChannelSelect: (channelId: string) => void;
  onOpenDmInbox: () => void;
  onDmOpen: (userId: string) => void;
  onDmConversationSelect: (conversationId: string) => void;
  onLogout: () => void;
}

export default function ChatLayout({
  isDmInboxOpen,
  hasUnreadDms,
  unreadDmIds,
  onServerSelect,
  onServerLeave,
  onChannelSelect,
  onOpenDmInbox,
  onDmOpen,
  onDmConversationSelect,
  onLogout,
}: ChatLayoutProps) {
  const { isMobile, isTablet } = useResponsive();
  const [mobilePanel, setMobilePanel] = useState<MobilePanel>("messages");
  const [showMobileMenu, setShowMobileMenu] = useState(false);
  const { t } = useTranslation();


  const {selectedChannel} = useSelectedChannel()
    const {selectedServer} = useSelectedServer()
    const {user} = useAuthStore()
    const {selectedConversation} = useSelectedDmConversation();
  const {conversations} = useDmConversations()
    const {dmMessages} = useDmMessages();

  const handleServerSelect = (serverId: string) => {
    onServerSelect(serverId);
    if (isMobile) {
      setMobilePanel("channels");
      setShowMobileMenu(false);
    }
  };

  const handleChannelSelect = (channelId: string) => {
    onChannelSelect(channelId);
    if (isMobile) {
      setMobilePanel("messages");
    }
  };

  const handleOpenDmInbox = () => {
    onOpenDmInbox();
    if (isMobile) {
      setMobilePanel("messages");
      setShowMobileMenu(false);
    }
  };

  const handleDmConversationSelect = (conversationId: string) => {
    onDmConversationSelect(conversationId);
    if (isMobile) {
      setMobilePanel("messages");
    }
  };

  if (isMobile) {
    return (
      <div className="flex flex-col h-screen overflow-hidden">
        <div className="flex items-center justify-between bg-sidebar-bg border-b border-gray-200 p-3">
          <button
            onClick={() => setShowMobileMenu(!showMobileMenu)}
            className="p-2 hover:bg-sidebar-hover rounded-lg transition-colors"
          >
            <Menu size={24} />
          </button>

          <div className="flex-1 text-center font-semibold text-gray-900 truncate px-2">
            {selectedConversation?.id
              ? t("dm.inboxTitle")
              : selectedChannel?.name
              ? `# ${selectedChannel.name}`
              : isDmInboxOpen
              ? t("dm.inboxTitle")
              : selectedServer?.name || "Chat"}
          </div>

          {selectedServer?.id && (
            <button
              onClick={() =>
                setMobilePanel(mobilePanel === "members" ? "messages" : "members")
              }
              className="p-2 hover:bg-sidebar-hover rounded-lg transition-colors"
            >
              <Users size={24} />
            </button>
          )}
        </div>

        {showMobileMenu && (
          <div
            className="fixed inset-0 z-50 bg-black bg-opacity-50"
            onClick={() => setShowMobileMenu(false)}
          >
            <div className="w-full max-w-[22rem] h-full bg-white flex" onClick={(e) => e.stopPropagation()}>
              <ServerList
                onServerSelect={handleServerSelect}
                onServerLeave={onServerLeave}
                onLogout={onLogout}
                onOpenDmInbox={handleOpenDmInbox}
                isDmInboxOpen={isDmInboxOpen}
                hasUnreadDms={hasUnreadDms}
              />

              {selectedServer?.id ? (
                <div className="flex-1 min-w-0">
                  <ChannelList
                    onChannelSelect={handleChannelSelect}
                  />
                </div>
              ) : isDmInboxOpen ? (
                <div className="flex-1 min-w-0">
                  <DmConversationList
                    conversations={conversations}
                    selectedConversationId={selectedConversation?.id}
                    unreadIds={unreadDmIds}
                    onSelectConversation={handleDmConversationSelect}
                  />
                </div>
              ) : (
                <div className="flex-1 min-w-0 bg-sidebar-bg flex flex-col h-full">
                  <div className="flex-1" />
                  <Profile />
                </div>
              )}
            </div>
          </div>
        )}

        <div className="flex-1 overflow-hidden">
          {mobilePanel === "members" && selectedServer?.id ? (
            <MembersList
              onDmOpen={onDmOpen}
            />
          ) : selectedConversation?.id && user ? (
            <DmView
              messages={dmMessages}
              conversationId={selectedConversation?.id}
              otherUsername={
                conversations.find((c) => c.id === selectedConversation?.id)
                  ?.other_username
              }
            />
          ) : isDmInboxOpen ? (
            <DmConversationList
              conversations={conversations}
              selectedConversationId={selectedConversation?.id}
              unreadIds={unreadDmIds}
              onSelectConversation={handleDmConversationSelect}
            />
          ) : mobilePanel === "channels" && selectedServer?.id ? (
            <ChannelList
              onChannelSelect={handleChannelSelect}
            />
          ) : selectedChannel?.id && user ? (
            <MessageList/>
          ) : !selectedServer?.id ? (
            <div className="flex-1 flex items-center justify-center bg-white h-full">
              <div className="text-center text-gray-400 px-4">
                <h2 className="text-xl mb-2">{t("chat.selectServer")}</h2>
                <p>{t("chat.selectServerHintMobile")}</p>
              </div>
            </div>
          ) : (
            <div className="flex-1 flex items-center justify-center bg-white h-full">
              <div className="text-center text-gray-400 px-4">
                <h2 className="text-xl mb-2">{t("chat.selectChannel")}</h2>
                <p>{t("chat.selectChannelHintMobile")}</p>
              </div>
            </div>
          )}
        </div>
      </div>
    );
  }

  if (isTablet) {
    return (
      <div className="flex h-screen overflow-hidden">
        <ServerList
          onServerSelect={handleServerSelect}
          onServerLeave={onServerLeave}
          onLogout={onLogout}
          onOpenDmInbox={onOpenDmInbox}
          isDmInboxOpen={isDmInboxOpen}
          hasUnreadDms={hasUnreadDms}
        />

        {isDmInboxOpen ? (
          <DmConversationList
            conversations={conversations}
            selectedConversationId={selectedConversation?.id}
            unreadIds={unreadDmIds}
            onSelectConversation={onDmConversationSelect}
          />
        ) : selectedServer?.id ? (
          <ChannelList
            onChannelSelect={onChannelSelect}
          />
        ) : (
          <div className="w-60 bg-sidebar-bg flex flex-col h-full shrink-0">
            <div className="flex-1" />
            <Profile />
          </div>
        )}

        {!selectedServer?.id && !isDmInboxOpen && (
          <div className="flex-1 flex items-center justify-center bg-white">
            <div className="text-center text-gray-400">
              <h2 className="text-xl mb-2">{t("chat.selectServer")}</h2>
              <p>{t("chat.selectServerHint")}</p>
            </div>
          </div>
        )}

        {isDmInboxOpen && !selectedConversation?.id && (
          <div className="flex-1 flex items-center justify-center bg-white">
            <div className="text-center text-gray-400">
              <h2 className="text-xl mb-2">{t("dm.selectConversation")}</h2>
            </div>
          </div>
        )}

        {selectedServer?.id && !selectedChannel?.id && !selectedConversation?.id && (
          <div className="flex-1 flex items-center justify-center bg-white">
            <div className="text-center text-gray-400">
              <h2 className="text-xl mb-2">{t("chat.selectChannel")}</h2>
              <p>{t("chat.selectChannelHint")}</p>
            </div>
          </div>
        )}

        {selectedConversation?.id && user && (
          <div className="flex-1 flex">
            <DmView
              messages={dmMessages}
              conversationId={selectedConversation?.id}
              otherUsername={
                conversations.find((c) => c.id === selectedConversation?.id)
                  ?.other_username
              }
            />
          </div>
        )}

        {selectedChannel?.id && !selectedConversation?.id && user && (
          <div className="flex-1 flex">
            <MessageList/>
          </div>
        )}
      </div>
    );
  }

  return (
    <div className="flex h-screen overflow-hidden">
      <ServerList
        onServerSelect={handleServerSelect}
        onServerLeave={onServerLeave}
        onLogout={onLogout}
        onOpenDmInbox={onOpenDmInbox}
        isDmInboxOpen={isDmInboxOpen}
        hasUnreadDms={hasUnreadDms}
      />

      {isDmInboxOpen ? (
        <DmConversationList
          conversations={conversations}
          selectedConversationId={selectedConversation?.id}
          unreadIds={unreadDmIds}
          onSelectConversation={onDmConversationSelect}
        />
      ) : selectedServer?.id ? (
        <ChannelList
          onChannelSelect={onChannelSelect}
        />
      ) : (
        <div className="w-60 bg-sidebar-bg flex flex-col h-full shrink-0">
          <div className="flex-1" />
          <Profile />
        </div>
      )}

      {!selectedServer?.id && !isDmInboxOpen && (
        <div className="flex-1 flex items-center justify-center bg-white">
          <div className="text-center text-bordeaux opacity-70">
            <h2 className="text-xl mb-2">{t("chat.selectServer")}</h2>
            <p>{t("chat.selectServerHint")}</p>
          </div>
        </div>
      )}

      {isDmInboxOpen && !selectedConversation?.id && (
        <div className="flex-1 flex items-center justify-center bg-white">
          <div className="text-center text-gray-400">
            <h2 className="text-xl mb-2">{t("dm.selectConversation")}</h2>
          </div>
        </div>
      )}

      {selectedServer?.id && !selectedChannel?.id && !selectedConversation?.id && (
        <div className="flex-1 flex items-center justify-center bg-white">
          <div className="text-center text-gray-400">
            <h2 className="text-xl mb-2">{t("chat.selectChannel")}</h2>
            <p>{t("chat.selectChannelHint")}</p>
          </div>
        </div>
      )}

      {selectedConversation?.id && user && (
        <DmView
          messages={dmMessages}
          conversationId={selectedConversation?.id}
          otherUsername={
            conversations.find((c) => c.id === selectedConversation?.id)
              ?.other_username
          }
        />
      )}

      {selectedChannel?.id && !selectedConversation?.id && user && (
        <MessageList/>
      )}

      {selectedServer?.id && (
        <MembersList
          onDmOpen={onDmOpen}
        />
      )}
    </div>
  );
}

"use client";

import { DmConversation } from "@/types";
import { MessageSquare } from "lucide-react";
import { useTranslation } from "@/lib/i18n/language-context";

interface DmConversationListProps {
  conversations: DmConversation[];
  selectedConversationId?: string;
  unreadIds?: Set<string>;
  onSelectConversation: (conversationId: string) => void;
}

export default function DmConversationList({
  conversations,
  selectedConversationId,
  unreadIds,
  onSelectConversation,
}: DmConversationListProps) {
  const { t } = useTranslation();

  return (
    <div className="w-full md:w-60 bg-sidebar-bg text-gray-900 flex flex-col h-full border-r border-gray-200">
      {/* Header */}
      <div className="flex-shrink-0 px-4 py-3 border-b border-gray-200 bg-white flex items-center gap-2 shadow-sm">
        <div className="w-7 h-7 rounded-lg bg-bordeaux-light flex items-center justify-center flex-shrink-0">
          <MessageSquare size={14} className="text-bordeaux" />
        </div>
        <h3 className="text-sm font-semibold text-gray-800 truncate">{t("dm.inboxTitle")}</h3>
      </div>

      <div className="flex-1 overflow-y-auto p-2 space-y-0.5">
        {conversations.length === 0 ? (
          <div className="h-full flex flex-col items-center justify-center text-gray-400 text-center px-4 py-8">
            <div className="w-14 h-14 rounded-2xl bg-bordeaux-light flex items-center justify-center mb-3">
              <MessageSquare size={28} className="text-bordeaux opacity-60" />
            </div>
            <p className="text-sm font-medium text-gray-600">{t("dm.emptyTitle")}</p>
            <p className="text-xs mt-1 text-gray-400 leading-relaxed">
              {t("dm.emptyHint")}
            </p>
          </div>
        ) : (
          conversations.map((conversation) => (
            <button
              key={conversation.id}
              onClick={() => onSelectConversation(conversation.id)}
              className={`w-full text-left px-3 py-2.5 rounded-lg transition-all duration-150 flex items-center gap-2.5 group ${
                selectedConversationId === conversation.id
                  ? "bg-bordeaux text-white shadow-sm"
                  : unreadIds?.has(conversation.id)
                  ? "bg-bordeaux-light text-gray-900"
                  : "hover:bg-sidebar-hover text-gray-800"
              }`}
            >
              <div className="relative w-8 h-8 flex-shrink-0">
                {conversation.other_avatar_url ? (
                  <img
                    src={conversation.other_avatar_url}
                    alt={conversation.other_username}
                    className="w-8 h-8 rounded-full object-cover"
                  />
                ) : (
                  <div className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-semibold ${
                    selectedConversationId === conversation.id
                      ? "bg-white/20 text-white"
                      : "bg-bordeaux-light text-bordeaux"
                  }`}>
                    {conversation.other_username.charAt(0).toUpperCase()}
                  </div>
                )}
                {unreadIds?.has(conversation.id) && selectedConversationId !== conversation.id && (
                  <span className="absolute -top-0.5 -right-0.5 w-2.5 h-2.5 bg-bordeaux rounded-full border-2 border-sidebar-bg" />
                )}
              </div>
              <p className={`text-sm truncate ${
                unreadIds?.has(conversation.id) && selectedConversationId !== conversation.id
                  ? "font-bold"
                  : "font-medium"
              }`}>{conversation.other_username}</p>
            </button>
          ))
        )}
      </div>
    </div>
  );
}

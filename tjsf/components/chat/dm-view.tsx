"use client";
import { DmMessage } from "@/types";
import { useState, useEffect, useRef, useCallback } from "react";
import { sendDm } from "@/lib/api/endpoints/dm";
import { toggleDmReaction } from "@/lib/api/endpoints/reactions";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Send, MessageSquare, SmilePlus } from "lucide-react";
import GifPicker from "./gif-picker";
import { useTranslation } from "@/lib/i18n/language-context";
import { formatDate } from "@/lib/utils/format-date";
import { useSocket } from "@/lib/socket/use-socket";
import { SOCKET_EVENTS_LISTEN } from "@/lib/constants/socket-events";
import ReactionPicker from "./reaction-picker";
import ReactionDisplay from "./reaction-display";
import type { Reaction } from "@/types/reaction";
import {useAuthStore} from "@/lib/store/auth-store";

const GIF_PREFIX = "[GIF]";

function isGifMessage(content: string): boolean {
  return content.startsWith(GIF_PREFIX);
}

function getGifUrl(content: string): string {
  return content.slice(GIF_PREFIX.length);
}

function GifContent({ url }: { url: string }) {
  return (
    <img
      src={url}
      alt="GIF"
      className="max-w-[250px] lg:max-w-[320px] rounded-lg"
      loading="lazy"
    />
  );
}

interface DmViewProps {
  messages: DmMessage[];
  conversationId: string;
  otherUsername?: string;
}

export default function DmView({
  messages,
  conversationId,
  otherUsername,
}: DmViewProps) {
  const [messageText, setMessageText] = useState("");
  const [showGifPicker, setShowGifPicker] = useState(false);
  const [reactionsMap, setReactionsMap] = useState<Record<string, Reaction[]>>({});
  const [hoveredMessageId, setHoveredMessageId] = useState<string | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const queryClient = useQueryClient();
  const { t, language } = useTranslation();
  const { user } = useAuthStore();
  const { socket } = useSocket();

  const characterCount = [...messageText].length;
  const MAX_LENGTH = 2000;
  const isOverLimit = characterCount > MAX_LENGTH;
  const isNearLimit = characterCount > MAX_LENGTH * 0.9;

  // Listen for reaction updates via socket
  useEffect(() => {
    if (!socket) return;
    const handleReactionUpdated = (data: { message_id: string; reactions: Reaction[] }) => {
      setReactionsMap((prev) => ({
        ...prev,
        [data.message_id]: data.reactions,
      }));
    };
    socket.on(SOCKET_EVENTS_LISTEN.REACTION_UPDATED, handleReactionUpdated);
    return () => {
      socket.off(SOCKET_EVENTS_LISTEN.REACTION_UPDATED, handleReactionUpdated);
    };
  }, [socket]);

  const handleReaction = useCallback(async (messageId: string, emoji: string) => {
    try {
      const reactions = await toggleDmReaction(messageId, emoji);
      setReactionsMap((prev) => ({
        ...prev,
        [messageId]: reactions,
      }));
    } catch {
      // silently fail
    }
  }, []);

  const mutation = useMutation({
    mutationFn: ({ content }: { content: string }) =>
      sendDm(conversationId, content),
    onMutate: async ({ content }) => {
      await queryClient.cancelQueries({
        queryKey: ["dm_messages", conversationId],
      });
      const previous = queryClient.getQueryData<DmMessage[]>([
        "dm_messages",
        conversationId,
      ]);
      const optimistic: DmMessage = {
        id: `optimistic-${Date.now()}`,
        conversation_id: conversationId,
        sender_id: user?.id ?? "",
        sender_username: null,
        sender_avatar_url: null,
        content,
        created_at: new Date().toISOString(),
      };
      queryClient.setQueryData(
        ["dm_messages", conversationId],
        (old: DmMessage[] | undefined) => [...(old ?? []), optimistic]
      );
      return { previous };
    },
    onError: (_err, _vars, context: { previous?: DmMessage[] } | undefined) => {
      if (context?.previous !== undefined) {
        queryClient.setQueryData(
          ["dm_messages", conversationId],
          context.previous
        );
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({
        queryKey: ["dm_messages", conversationId],
      });
    },
  });

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages]);

  useEffect(() => {
    document.documentElement.style.setProperty("--lang-switcher-top", "0.75rem");
    document.documentElement.style.setProperty("--lang-switcher-bottom", "auto");

    return () => {
      document.documentElement.style.removeProperty("--lang-switcher-top");
      document.documentElement.style.removeProperty("--lang-switcher-bottom");
    };
  }, []);

  const handleSend = () => {
    const trimmed = messageText.trim();
    if (!trimmed || isOverLimit || mutation.isPending) return;
    mutation.mutate({ content: trimmed });
    setMessageText("");
  };

  const handleGifSelect = (gifUrl: string) => {
    mutation.mutate({ content: `[GIF]${gifUrl}` });
    setShowGifPicker(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="flex-1 flex flex-col h-full bg-white min-w-0">
      <div className="p-4 border-b border-gray-200">
        <h2 className="text-gray-900 font-semibold flex items-center">
          <span className="mr-2">
            <MessageSquare size={18} />
          </span>
          {otherUsername ?? t("dm.defaultConversation")}
        </h2>
      </div>

      <div ref={scrollRef} className="flex-1 overflow-y-auto">
        <div className="p-4 space-y-4">
        {messages.length === 0 && (
          <div className="flex flex-col items-center justify-center h-full text-gray-400 py-12">
            <div className="w-16 h-16 rounded-2xl bg-bordeaux-light flex items-center justify-center mb-4">
              <MessageSquare size={32} className="text-bordeaux opacity-70" />
            </div>
            <p className="text-sm font-medium text-gray-600">{t("dm.startConversation")}</p>
            <p className="text-xs text-gray-400 mt-1">{t("dm.privateNotice")}</p>
          </div>
        )}
        {messages.map((msg) => {
          const isHovered = hoveredMessageId === msg.id;
          const msgReactions = reactionsMap[msg.id] ?? [];
          const isMine = msg.sender_id === user?.id;

          if (isMine) {
            return (
              <div
                key={msg.id}
                className="flex justify-end group relative"
                onMouseEnter={() => setHoveredMessageId(msg.id)}
                onMouseLeave={() => setHoveredMessageId(null)}
              >
                <div className="max-w-xs lg:max-w-md">
                  {isGifMessage(msg.content) ? (
                    <GifContent url={getGifUrl(msg.content)} />
                  ) : (
                    <div className="bg-bordeaux rounded-lg px-4 py-2">
                      <p className="text-white text-sm break-words whitespace-pre-wrap">
                        {msg.content}
                      </p>
                    </div>
                  )}
                  <div className="flex justify-end mt-1">
                    <span className="text-xs text-gray-500">
                      {formatDate(msg.created_at, language)}
                    </span>
                  </div>
                  <ReactionDisplay
                    reactions={msgReactions}
                    onToggle={(emoji) => handleReaction(msg.id, emoji)}
                  />
                </div>
                {isHovered && (
                  <div className="mt-1">
                    <ReactionPicker onSelect={(emoji) => handleReaction(msg.id, emoji)} />
                  </div>
                )}
              </div>
            );
          }

          return (
            <div
              key={msg.id}
              className="flex justify-start space-x-3 group relative"
              onMouseEnter={() => setHoveredMessageId(msg.id)}
              onMouseLeave={() => setHoveredMessageId(null)}
            >
              <div className="flex-shrink-0">
                {msg.sender_avatar_url ? (
                  <img
                    src={msg.sender_avatar_url}
                    alt={msg.sender_username ?? "user"}
                    className="w-10 h-10 rounded-full object-cover"
                  />
                ) : (
                  <div className="w-10 h-10 bg-gray-600 rounded-full flex items-center justify-center">
                    <span className="text-sm text-white font-semibold">
                      {(msg.sender_username ?? "?").charAt(0).toUpperCase()}
                    </span>
                  </div>
                )}
              </div>

              <div className="flex-1 min-w-0 max-w-xs lg:max-w-md">
                <div className="flex items-baseline space-x-2 mb-1">
                  <span className="text-gray-900 font-medium">
                    {msg.sender_username}
                  </span>
                  <span className="text-xs text-gray-500">
                    {formatDate(msg.created_at, language)}
                  </span>
                </div>
                {isGifMessage(msg.content) ? (
                  <GifContent url={getGifUrl(msg.content)} />
                ) : (
                  <p className="text-gray-700 text-sm break-words whitespace-pre-wrap">
                    {msg.content}
                  </p>
                )}
                <ReactionDisplay
                  reactions={msgReactions}
                  onToggle={(emoji) => handleReaction(msg.id, emoji)}
                />
                {isHovered && (
                  <div className="mt-1">
                    <ReactionPicker onSelect={(emoji) => handleReaction(msg.id, emoji)} />
                  </div>
                )}
              </div>
            </div>
          );
        })}
        </div>
      </div>

      <div className="p-4 border-t border-gray-200">
        {isNearLimit && (
          <div
            className={`text-xs mb-1 text-right ${
              isOverLimit ? "text-red-500" : "text-yellow-500"
            }`}
          >
            {characterCount}/{MAX_LENGTH}
          </div>
        )}
        <div className="flex justify-center items-center space-x-2 p-0">
          <div className="flex-1 relative mb-0 p-0">
          <textarea
            className={`w-full h-full bg-white border rounded px-4 py-3 text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:border-transparent resize-none min-h-[48px] max-h-[120px] overflow-y-auto scrollbar-hide ${
              isOverLimit ? "border-red-400 bg-red-50 focus:ring-danger" : "border-gray-300 focus:ring-steel-blue"
            }`}
            placeholder={t("dm.placeholder", {
              username: otherUsername ?? "...",
            })}
            value={messageText}
            onChange={(e) => setMessageText(e.target.value)}
            onKeyDown={handleKeyDown}
            rows={1}
            style={{ maxHeight: "120px", overflowY: "auto" }}
          />
          </div>
          <div className="relative">
            <button
              onClick={() => setShowGifPicker((prev) => !prev)}
              className="px-3 py-3 rounded transition-colors text-gray-500 hover:text-bordeaux hover:cursor-pointer"
              title="GIF"
            >
              <SmilePlus size={18} />
            </button>
            {showGifPicker && (
              <GifPicker
                onSelect={handleGifSelect}
                onClose={() => setShowGifPicker(false)}
              />
            )}
          </div>

          <button
            onClick={handleSend}
            disabled={!messageText.trim() || isOverLimit || mutation.isPending}
            className={`px-4 py-3 rounded transition-colors ${
              !messageText.trim() || isOverLimit || mutation.isPending
                ? "cursor-not-allowed text-gray-500"
                : "bg-bordeaux hover:bg-bordeaux-hover text-white"
            }`}
          >
            <Send size={18} />
          </button>
        </div>
      </div>
    </div>
  );
}

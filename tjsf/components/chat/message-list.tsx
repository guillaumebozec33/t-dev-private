"use client";

import React from "react";
import {Message} from "@/types";
import MessageComponent from "./message";
import { useState, useEffect, useRef,useCallback } from "react";
import {sendMessage, deleteMessage, editMessage} from "@/lib/api/endpoints";
import { useMutation } from "@tanstack/react-query";
import { useQueryClient } from "@tanstack/react-query";
import { TypingIndicator } from "./typing-indicator";
import { Send, AlertTriangle, X, SmilePlus, Hash, Volume2, Bell, Star, Heart, Zap, Globe, Music, Video, Image, FileText, Code, Coffee, Gamepad2, BookOpen, Megaphone, Shield, Flame, Smile, Lock } from "lucide-react";
import { Channel } from "@/types";

const CHANNEL_ICONS: Record<string, React.ComponentType<{ size?: number; className?: string }>> = {
  Hash, Volume2, Bell, Star, Heart, Zap, Globe, Music, Video, Image,
  FileText, Code, Coffee, Gamepad2, BookOpen, Megaphone, Shield, Flame, Smile,
};

function ChannelHeaderIcon({ channel }: { channel: Channel | null | undefined }) {
  if (!channel) return <span className="text-gray-500">#</span>;
  if (channel.icon && CHANNEL_ICONS[channel.icon]) {
    const Icon = CHANNEL_ICONS[channel.icon];
    return (
      <span className="relative inline-flex items-center">
        <Icon size={16} className="text-gray-500" />
        {channel.is_private && <Lock size={9} className="absolute -bottom-1 -right-1 text-gray-500" />}
      </span>
    );
  }
  if (channel.is_private) return <Lock size={16} className="text-gray-500" />;
  return <span className="text-gray-500">#</span>;
}
import GifPicker from "./gif-picker";
import { useTranslation } from "@/lib/i18n/language-context";
import { useToast } from "@/components/ui/toast";
import {useAuthStore} from "@/lib/store/auth-store";
import {useTypingIndicator, useMessages,useSelectedChannel,useMembers,useUserRole} from '@/hooks'
import { useSocket } from "@/lib/socket/use-socket";
import { SOCKET_EVENTS_LISTEN } from "@/lib/constants/socket-events";
import type { Reaction } from "@/types/reaction";
import {toggleReaction, getReactions} from "@/lib/api/endpoints/reactions";


export default function MessageList() {
  const [messageText, setMessageText] = useState("");
  const [editingMessageId, setEditingMessageId] = useState<string | null>(null);
  const [showGifPicker, setShowGifPicker] = useState(false);
  const [reactionsMap, setReactionsMap] = useState<Record<string, Reaction[]>>({});
  const queryClient = useQueryClient();
  const typingTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const { t } = useTranslation();
  const { showError } = useToast();
  const { socket } = useSocket();
  const {user} = useAuthStore();
  const {messages, isFetchingNextPage, fetchNextPage, hasNextPage} = useMessages();
  const {selectedChannel} = useSelectedChannel();
  const { startTyping, stopTyping, typingUsers } = useTypingIndicator();
  const {isAdmin, isOwner} = useUserRole();
  const {members} = useMembers();




  const handleError = (error: Error) => showError(t(`errors.code.${error.message}`) !== `errors.code.${error.message}` ? t(`errors.code.${error.message}`) : "Action impossible");

  const characterCount = [...messageText].length;
  const MAX_LENGTH = 2000;
  const isOverLimit = characterCount > MAX_LENGTH;
  const isNearLimit = characterCount > MAX_LENGTH * 0.9;
    const prevScrollHeightRef = useRef(0);
    const prevScrollTopRef = useRef(0);

// Scroll auto vers le bas uniquement si on était déjà en bas
    useEffect(() => {
        const el = scrollRef.current;
        if (!el) return;
        if (isAtBottomRef.current) {
            el.scrollTop = el.scrollHeight;
        }
    }, [messages, typingUsers]);

// Capture la position AVANT le fetch
    useEffect(() => {
        if (isFetchingNextPage) {
            const el = scrollRef.current;
            if (!el) return;
            prevScrollHeightRef.current = el.scrollHeight;
            prevScrollTopRef.current = el.scrollTop;
        }
    }, [isFetchingNextPage]);

// Restaure la position APRÈS que les messages arrivent
    useEffect(() => {
        if (isFetchingNextPage) return;
        const el = scrollRef.current;
        if (!el || prevScrollHeightRef.current === 0) return;

        const diff = el.scrollHeight - prevScrollHeightRef.current;
        if (diff > 0) {
            el.scrollTop = prevScrollTopRef.current + diff;
            prevScrollHeightRef.current = 0;
        }
    }, [messages.length, isFetchingNextPage]);

// Détecte si on est en bas
    useEffect(() => {
        const el = scrollRef.current;
        if (!el) return;
        const handleScroll = () => {
            isAtBottomRef.current = el.scrollTop + el.clientHeight >= el.scrollHeight - 50;
        };
        el.addEventListener("scroll", handleScroll);
        return () => el.removeEventListener("scroll", handleScroll);
    }, []);

// IntersectionObserver sur le sentinel en haut
    useEffect(() => {
        const sentinel = topSentinelRef.current;
        if (!sentinel) return;

        const observer = new IntersectionObserver(
            ([entry]) => {
                if (entry.isIntersecting && hasNextPage && !isFetchingNextPage) {
                    fetchNextPage();
                }
            },
            { threshold: 0.1 }
        );

        observer.observe(sentinel);
        return () => observer.disconnect();
    }, [hasNextPage, isFetchingNextPage, fetchNextPage]);

  // Load reactions only for messages not yet in reactionsMap
  useEffect(() => {
    if (messages.length === 0) return;
    const missing = messages.filter((m) => !m.id.startsWith("temp-") && !(m.id in reactionsMap));
    if (missing.length === 0) return;
    Promise.all(missing.map((m) => getReactions(m.id).then((r) => ({ id: m.id, r })))).then(
      (results) => {
        setReactionsMap((prev) => {
          const next = { ...prev };
          results.forEach(({ id, r }) => { next[id] = r; });
          return next;
        });
      }
    ).catch(() => {});
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [messages.map((m) => m.id).join(",")]);

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
      const reactions = await toggleReaction(messageId, emoji);
      setReactionsMap((prev) => ({
        ...prev,
        [messageId]: reactions,
      }));
    } catch (error) {
      if (error instanceof Error) handleError(error);
    }
  }, []);


    const mutation = useMutation({
        mutationFn: sendMessage,
        onMutate: async (newMessage) => {
            await queryClient.cancelQueries({ queryKey: ["messages", selectedChannel?.id] });

            const previousData = queryClient.getQueryData(["messages", selectedChannel?.id]);

            // Ajouter dans la première page (la plus récente)
            queryClient.setQueryData(
                ["messages", selectedChannel?.id],
                (old: { pages: Message[][], pageParams: any[] } | undefined) => {
                    if (!old) return old;
                    const optimisticMessage: Message = {
                        id: `temp-${Date.now()}`,
                        content: newMessage.content,
                        channel_id: newMessage.channel_id,
                        author_id: user?.id ?? "",
                        author_username: t("messages.you"),
                        author_avatar_url: "",
                        edited: false,
                        created_at: new Date().toISOString(),
                        updated_at: null,
                    };
                    // Les pages sont en ordre inversé dans le flatMap, donc on ajoute à la dernière page affichée
                    const newPages = [...old.pages];
                    newPages[0] = [optimisticMessage, ...newPages[0]]; // page 0 = la plus récente (DESC)
                    return { ...old, pages: newPages };
                }
            );

            return { previousData };
        },
        onError: (error, _, context) => {
            queryClient.setQueryData(["messages", selectedChannel?.id], context?.previousData);
            handleError(error);
        },
        onSettled: () => {
            queryClient.invalidateQueries({ queryKey: ["messages", selectedChannel?.id] });
        },
    });

    const deleteMutation = useMutation({
        mutationFn: deleteMessage,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["messages", selectedChannel?.id] });
        },
        onError: (error) => handleError(error),
    });

    const editMutation = useMutation({
        mutationFn: ({ messageId, content }: { messageId: string; content: string }) =>
            editMessage(messageId, content),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["messages", selectedChannel?.id] });
            setEditingMessageId(null);
            setMessageText("");
        },
        onError: (error) => handleError(error),
    });

    const scrollRef = useRef<HTMLDivElement>(null);
    const topSentinelRef = useRef<HTMLDivElement>(null);
    const isAtBottomRef = useRef(true);



  const handleEditMessage = (messageId: string) => {
    const message = messages.find((m) => m.id === messageId);
    if (message) {
      setEditingMessageId(messageId);
      setMessageText(message.content);
      setTimeout(() => {
        const textarea = document.querySelector("textarea");
        if (textarea) {
          textarea.focus();
          textarea.setSelectionRange(
            textarea.value.length,
            textarea.value.length,
          );
        }
      }, 0);
    }
  };

  const handleCancelEdit = () => {
    setEditingMessageId(null);
    setMessageText("");
    const textarea = document.querySelector("textarea");
    if (textarea) {
      textarea.style.height = "48px";
    }
  };

  const handleDeleteMessage = (messageId: string) => {
    console.log("Tentative de suppression du message:", messageId);
    deleteMutation.mutate(messageId);
  };

  const handleSendMessage = () => {
    if (messageText.trim() === "" || isOverLimit) return;

    if (editingMessageId) {
      editMutation.mutate({
        messageId: editingMessageId,
        content: messageText,
      });
    } else {
        if (!selectedChannel?.id) return;
      mutation.mutate({ channel_id: selectedChannel.id, content: messageText });
    }
    setMessageText("");
    stopTyping();
    setEditingMessageId(null);
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
      typingTimeoutRef.current = null;
    }
    const textarea = document.querySelector("textarea");
    if (textarea) {
      textarea.style.height = "48px";
    }
  };

  const handleMessageTextChange = (
    e: React.ChangeEvent<HTMLTextAreaElement>,
  ) => {
    setMessageText(e.target.value);
    e.target.style.height = "auto";
    e.target.style.height = Math.min(e.target.scrollHeight, 120) + "px";

    if (e.target.value.length > 0) {
      startTyping();

      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
      }

      typingTimeoutRef.current = setTimeout(() => {
        stopTyping();
      }, 3000);
    } else {
      stopTyping();
      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
        typingTimeoutRef.current = null;
      }
    }
  };

  useEffect(() => {
    return () => {
      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
      }
      stopTyping();
    };
  }, [selectedChannel?.id, stopTyping]);

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
    if (e.key === "Escape" && editingMessageId) {
      handleCancelEdit();
    }
  };

  const handleGifSelect = (gifUrl: string) => {
    mutation.mutate({ channel_id: selectedChannel?.id ? selectedChannel?.id:"", content: `[GIF]${gifUrl}` });
    setShowGifPicker(false);
  };

  return (
      <div className="flex-1 bg-white flex flex-col">
          <div className="p-4 border-b border-gray-200">
              <h2 className="text-gray-900 font-semibold flex items-center gap-2">
                  <ChannelHeaderIcon channel={selectedChannel} />
                  {selectedChannel?.name || t("messages.defaultChannel")}
              </h2>
          </div>

            <div className="flex-1 overflow-y-auto" ref={scrollRef}>
                <div className="p-4 space-y-4">
                    <div ref={topSentinelRef} className="h-1" />

                    {isFetchingNextPage && (
                        <div className="text-center text-gray-400 text-sm py-2">
                            Chargement...
                        </div>
                    )}

                    {messages.length === 0 && (
                        <div className="text-gray-600 text-center">
                            {t("messages.emptyChannel")} #{selectedChannel?.name}
                        </div>
                    )}

                    {messages.map((message) => {
                        const authorMember = members.find((m) => m.user_id === message.author_id);
                        return (
                            <MessageComponent
                                key={message.id}
                                message={message}
                                isMyMessage={message.author_id === user?.id}
                                authorRole={authorMember?.role || "member"}
                                onEdit={handleEditMessage}
                                onDelete={handleDeleteMessage}
                                reactions={reactionsMap[message.id] ?? []}
                                onReaction={handleReaction}
                            />
                        );
                    })}
                </div>
            </div>

            <div className="p-4 border-t border-gray-200">
                <TypingIndicator/>
                {editingMessageId && (
                    <div className="mb-2 flex items-center justify-between bg-blue-50 border border-blue-200 rounded px-3 py-2">
                        <span className="text-sm text-blue-700">{t("messages.editing")}</span>
                        <button onClick={handleCancelEdit} className="text-blue-700 hover:text-blue-900" title={t("messages.cancelEdit")}>
                            <X size={18} />
                        </button>
                    </div>
                )}
                {isNearLimit && (
                    <div className={`mb-2 flex items-center gap-2 text-sm font-medium ${
                        isOverLimit ? "text-danger bg-red-50 border-l-4 border-danger px-3 py-2 rounded" : "text-warning bg-orange-50 border-l-4 border-warning px-3 py-2 rounded"
                    }`}>
                        <AlertTriangle size={16} />
                        <span>
            {isOverLimit
                ? t("messages.tooLong", { count: characterCount, max: MAX_LENGTH, excess: characterCount - MAX_LENGTH })
                : t("messages.nearLimit", { count: characterCount, max: MAX_LENGTH })}
          </span>
                    </div>
                )}

        <div className="flex justify-center items-center space-x-2 p-0">
          <div className="flex-1 relative mb-0 p-0">
            <textarea
              value={messageText}
              onChange={handleMessageTextChange}
              onKeyDown={handleKeyPress}
              placeholder={t("messages.placeholder", { channel: selectedChannel?.name || t("messages.defaultChannel") })}
              className={`w-full h-full bg-white border rounded px-4 py-3 text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:border-transparent resize-none min-h-[48px] max-h-[120px] overflow-y-auto scrollbar-hide ${
                  isOverLimit ? "border-danger focus:ring-danger" : isNearLimit ? "border-warning focus:ring-warning" : "border-gray-300 focus:ring-steel-blue"
              }`}
              rows={1}
          />
                        {messageText.length > 0 && (
                            <div className={`absolute bottom-2 right-2 text-xs font-semibold ${
                                isOverLimit ? "text-danger" : isNearLimit ? "text-warning" : "text-gray-400"
                            }`}>
                                {characterCount}/{MAX_LENGTH}
                            </div>
                        )}
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
                        onClick={handleSendMessage}
                        disabled={messageText.trim() === "" || isOverLimit}
                        className={`px-4 py-3 rounded transition-colors ${
                            messageText.trim() === "" || isOverLimit ? "cursor-not-allowed text-gray-500" : "bg-bordeaux hover:bg-bordeaux-hover text-white"
                        }`}
                        title={isOverLimit ? t("messages.tooLongTitle") : t("messages.send")}
                    >
                        <Send size={18} />
                    </button>
                </div>
            </div>
        </div>
    );
}

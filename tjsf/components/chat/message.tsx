"use client";

import { Message } from "@/types";
import { useState } from "react";
import { formatDate } from "@/lib/utils/format-date";
import { Edit, Trash2, Crown, Shield } from "lucide-react";
import { useTranslation } from "@/lib/i18n/language-context";
import ReactionPicker from "./reaction-picker";
import ReactionDisplay from "./reaction-display";
import type { Reaction } from "@/types/reaction";
import {useUserRole} from "@/hooks";

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

interface MessageProps {
  message: Message;
  isMyMessage: boolean;
  authorRole?: string;
  onEdit?: (messageId: string) => void;
  onDelete?: (messageId: string) => void;
  reactions?: Reaction[];
  onReaction?: (messageId: string, emoji: string) => void;
}

export default function MessageComponent({
  message,
  isMyMessage,
  authorRole,
  onEdit,
  onDelete,
  reactions = [],
  onReaction,
}: MessageProps) {
  const [showMenu, setShowMenu] = useState(false);
  const { t, language } = useTranslation();
  const canEdit = isMyMessage;
  const {isAdmin,isOwner} = useUserRole();

  if (message.author_id === "system") {
    const parts = message.content.split(":");
    const type = parts[1] as "joined" | "kicked" | "banned";
    const username = parts.slice(2).join(":");
    const text = t(`system.${type}`, { username });
    return (
      <div className="flex items-center justify-center my-3 px-4">
        <div className="h-px flex-1 bg-gray-200" />
        <span className="mx-3 text-xs text-gray-400 italic">{text}</span>
        <div className="h-px flex-1 bg-gray-200" />
      </div>
    );
  }
  const canDelete =
    isMyMessage || isOwner || (isAdmin && authorRole === "member");

  const handleReaction = (emoji: string) => {
    onReaction?.(message.id, emoji);
  };

    if (isMyMessage) {
        return (
            <div
                className="flex justify-end group relative"
                onMouseEnter={() => setShowMenu(true)}
                onMouseLeave={() => setShowMenu(false)}
            >
                {showMenu && (
                    <div
                        className="absolute right-0 -top-8 flex items-center gap-1 bg-white rounded-lg shadow-lg border border-gray-200 px-1 py-0.5 z-10"
                        onMouseEnter={() => setShowMenu(true)}
                    >
                        <ReactionPicker onSelect={handleReaction} />
                        {canEdit && (
                            <button
                                onClick={() => onEdit?.(message.id)}
                                className="text-gray-500 hover:text-blue-600 p-1 rounded"
                                title={t("messages.editButton")}
                            >
                                <Edit size={16} />
                            </button>
                        )}
                        {canDelete && (
                            <button
                                onClick={() => onDelete?.(message.id)}
                                className="p-1 text-gray-400 hover:text-danger hover:bg-red-50 rounded transition-colors"
                                title={t("messages.deleteButton")}
                            >
                                <Trash2 size={16} />
                            </button>
                        )}
                    </div>
                )}

                <div className="max-w-xs lg:max-w-md">
                    {isGifMessage(message.content) ? (
                        <GifContent url={getGifUrl(message.content)} />
                    ) : (
                        <div className="bg-bordeaux rounded-lg px-4 py-2">
                            <p className="text-white text-sm break-words whitespace-pre-wrap">
                                {message.content}
                            </p>
                        </div>
                    )}
                    <div className="flex justify-end mt-1">
            <span className="text-xs text-gray-500">
              {formatDate(message.created_at, language)}
            </span>
                    </div>
                    {message.edited && (
                        <span className="text-xs text-gray-400 italic">{t('messages.edited')}</span>
                    )}
                    <div className="flex justify-end">
                        <ReactionDisplay
                            reactions={reactions}
                            onToggle={handleReaction}
                        />
                    </div>
                </div>
            </div>
        );
    }
    return (
        <div
            className="flex justify-start space-x-3 group relative"
            onMouseEnter={() => setShowMenu(true)}
            onMouseLeave={() => setShowMenu(false)}
        >
            {showMenu && (
                <div
                    className="absolute left-0 -top-8 flex items-center gap-1 bg-white rounded-lg shadow-lg border border-gray-200 px-1 py-0.5 z-10"
                    onMouseEnter={() => setShowMenu(true)}
                >
                    <ReactionPicker onSelect={handleReaction} />
                    {canDelete && (
                        <button
                            onClick={() => onDelete?.(message.id)}
                            className="p-1 text-gray-400 hover:text-danger hover:bg-red-50 rounded transition-colors"
                            title={t("messages.deleteButton")}
                        >
                            <Trash2 size={16} />
                        </button>
                    )}
                </div>
            )}

            <div className="flex-shrink-0">
                {message.author_avatar_url ? (
                    <img
                        src={message.author_avatar_url}
                        alt={message.author_username}
                        className="w-10 h-10 rounded-full"
                    />
                ) : (
                    <div className="w-10 h-10 bg-gray-600 rounded-full flex items-center justify-center">
            <span className="text-sm text-white font-semibold">
              {message.author_username?.charAt(0).toUpperCase()}
            </span>
                    </div>
                )}
            </div>

            <div className="flex-1 min-w-0 max-w-xs lg:max-w-md">
                <div className="flex items-center flex-wrap gap-1.5 mb-1">
          <span className="text-gray-900 font-medium">
            {message.author_username}
          </span>
                    {authorRole === "owner" && <span className="flex items-center gap-0.5 text-[10px] font-semibold text-amber-600 bg-amber-100 px-1.5 py-0.5 rounded-full"><Crown size={10} />owner</span>}
                    {authorRole === "admin" && <span className="flex items-center gap-0.5 text-[10px] font-semibold text-blue-600 bg-blue-100 px-1.5 py-0.5 rounded-full"><Shield size={10} />admin</span>}
                    <span className="text-xs text-gray-500">
            {formatDate(message.created_at, language)}
          </span>
                </div>
                {isGifMessage(message.content) ? (
                    <GifContent url={getGifUrl(message.content)} />
                ) : (
                    <p className="text-gray-700 text-sm break-words whitespace-pre-wrap">
                        {message.content}
                    </p>
                )}
                {message.edited && (
                    <span className="text-xs text-gray-400 italic">{t("messages.edited")}</span>
                )}
                <ReactionDisplay
                    reactions={reactions}
                    onToggle={handleReaction}
                />
            </div>
        </div>
    );
}

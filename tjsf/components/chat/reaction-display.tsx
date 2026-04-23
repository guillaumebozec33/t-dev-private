"use client";

import { Twemoji } from "./twemoji";
import type { Reaction, ReactionGroup } from "@/types/reaction";
import {useAuthStore} from "@/lib/store/auth-store";

interface ReactionDisplayProps {
  reactions: Reaction[];
  onToggle: (emoji: string) => void;
}

function groupReactions(reactions: Reaction[], currentUserId?: string): ReactionGroup[] {
  const groups = new Map<string, ReactionGroup>();

  for (const reaction of reactions) {
    const existing = groups.get(reaction.emoji);
    if (existing) {
      existing.count++;
      existing.users.push(reaction.username ?? reaction.user_id);
      if (reaction.user_id === currentUserId) existing.hasReacted = true;
    } else {
      groups.set(reaction.emoji, {
        emoji: reaction.emoji,
        count: 1,
        users: [reaction.username ?? reaction.user_id],
        hasReacted: reaction.user_id === currentUserId,
      });
    }
  }

  return Array.from(groups.values());
}

export default function ReactionDisplay({ reactions, onToggle }: ReactionDisplayProps) {
  const {user} = useAuthStore();
    const groups = groupReactions(reactions, user?.id);

  if (groups.length === 0) return null;

  return (
    <div className="flex flex-wrap gap-1 mt-1">
      {groups.map((group) => (
        <button
          key={group.emoji}
          onClick={(e) => {
            e.stopPropagation();
            onToggle(group.emoji);
          }}
          className={`inline-flex items-center gap-1 px-1.5 py-0.5 rounded-full text-xs border transition-colors ${
            group.hasReacted
              ? "bg-bordeaux/10 border-bordeaux text-bordeaux"
              : "bg-gray-50 border-gray-200 text-gray-600 hover:bg-gray-100"
          }`}
          title={group.users.join(", ")}
        >
          <Twemoji emoji={group.emoji} size={14} />
          <span className="font-medium">{group.count}</span>
        </button>
      ))}
    </div>
  );
}

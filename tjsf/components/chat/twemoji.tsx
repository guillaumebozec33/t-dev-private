"use client";

// Twemoji CDN base URL (Twitter/Discord style emojis)
const TWEMOJI_BASE = "https://cdn.jsdelivr.net/gh/twitter/twemoji@14.0.2/assets/svg";

// Fixed set of reaction emojis with their Unicode codepoints
export const REACTION_EMOJIS = [
  { emoji: "👍", code: "1f44d", label: "thumbs_up" },
  { emoji: "❤️", code: "2764", label: "heart" },
  { emoji: "😂", code: "1f602", label: "joy" },
  { emoji: "😮", code: "1f62e", label: "surprised" },
  { emoji: "😢", code: "1f622", label: "cry" },
  { emoji: "🔥", code: "1f525", label: "fire" },
] as const;

interface TwemojiProps {
  emoji: string;
  size?: number;
  className?: string;
}

function getCodeFromEmoji(emoji: string): string | undefined {
  const found = REACTION_EMOJIS.find((e) => e.emoji === emoji);
  return found?.code;
}

export function Twemoji({ emoji, size = 20, className = "" }: TwemojiProps) {
  const code = getCodeFromEmoji(emoji);
  if (!code) return <span className={className}>{emoji}</span>;

  return (
    <img
      src={`${TWEMOJI_BASE}/${code}.svg`}
      alt={emoji}
      width={size}
      height={size}
      className={`inline-block ${className}`}
      draggable={false}
    />
  );
}

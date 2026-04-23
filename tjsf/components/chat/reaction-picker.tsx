"use client";

import { REACTION_EMOJIS, Twemoji } from "./twemoji";

interface ReactionPickerProps {
  onSelect: (emoji: string) => void;
}

export default function ReactionPicker({ onSelect }: ReactionPickerProps) {
  return (
    <div className="flex items-center gap-0.5 bg-white border border-gray-200 rounded-lg shadow-lg px-1 py-0.5">
      {REACTION_EMOJIS.map((item) => (
        <button
          key={item.label}
          onClick={(e) => {
            e.stopPropagation();
            onSelect(item.emoji);
          }}
          className="p-1.5 rounded hover:bg-gray-100 transition-colors"
          title={item.label}
        >
          <Twemoji emoji={item.emoji} size={18} />
        </button>
      ))}
    </div>
  );
}

"use client";

import { useTypingIndicator } from "@/hooks";
import { useTranslation } from "@/lib/i18n/language-context";

export function TypingIndicator() {
  const { typingUsers } = useTypingIndicator();
  const { t } = useTranslation();

  if (typingUsers.length === 0) return null;

  const text =
    typingUsers.length === 1
      ? t("typing.one", { username: typingUsers[0].username })
      : typingUsers.length === 2
        ? t("typing.two", { u1: typingUsers[0].username, u2: typingUsers[1].username })
        : t("typing.many", { count: typingUsers.length });

  return (
    <div className="px-4 pb-2 text-sm text-bordeaux italic flex items-center gap-2">
      <span className="text-bordeaux">•</span>
      {text}
    </div>
  );
}

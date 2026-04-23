"use client";

import { useTranslation, type Language } from "@/lib/i18n/language-context";

const LANGUAGES: { code: Language; label: string }[] = [
  { code: "fr", label: "FR" },
  { code: "en", label: "EN" },
];

export default function LanguageSwitcher() {
  const { language, setLanguage } = useTranslation();

  return (
    <div
      className="fixed right-3 sm:right-4 z-50 flex items-center gap-1 bg-white border border-gray-200 rounded-lg px-3 py-1.5 shadow-sm select-none"
      style={{
        top: "var(--lang-switcher-top, auto)",
        bottom: "var(--lang-switcher-bottom, 1rem)",
      }}
    >
      {LANGUAGES.map((lang, index) => (
        <span key={lang.code} className="flex items-center gap-1">
          {index > 0 && <span className="text-gray-300 text-sm">|</span>}
          <button
            onClick={() => setLanguage(lang.code)}
            className={`text-sm font-medium transition-colors cursor-pointer ${
              language === lang.code
                ? "text-bordeaux underline underline-offset-2"
                : "text-gray-400 hover:text-gray-600"
            }`}
          >
            {lang.label}
          </button>
        </span>
      ))}
    </div>
  );
}

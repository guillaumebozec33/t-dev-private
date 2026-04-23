type FormatLanguage = "fr" | "en";

const labels = {
  fr: {
    now: "maintenant",
    minutes: (n: number) => `il y a ${n}min`,
    hours: (n: number) => `il y a ${n}h`,
    days: (n: number) => `il y a ${n}j`,
    locale: "fr-FR",
  },
  en: {
    now: "just now",
    minutes: (n: number) => `${n}min ago`,
    hours: (n: number) => `${n}h ago`,
    days: (n: number) => `${n}d ago`,
    locale: "en-GB",
  },
};

export function formatDate(
  dateString: string,
  language: FormatLanguage = "fr"
): string {
  const date = new Date(dateString);
  const now = new Date();
  const diffInMs = now.getTime() - date.getTime();
  const diffInMinutes = Math.floor(diffInMs / (1000 * 60));
  const diffInHours = Math.floor(diffInMs / (1000 * 60 * 60));
  const diffInDays = Math.floor(diffInMs / (1000 * 60 * 60 * 24));
  const l = labels[language];

  if (diffInMinutes < 1) {
    return l.now;
  } else if (diffInMinutes < 60) {
    return l.minutes(diffInMinutes);
  } else if (diffInHours < 24) {
    return l.hours(diffInHours);
  } else if (diffInDays < 7) {
    return l.days(diffInDays);
  } else {
    return date.toLocaleDateString(l.locale, {
      day: "2-digit",
      month: "2-digit",
      year: "numeric",
    });
  }
}

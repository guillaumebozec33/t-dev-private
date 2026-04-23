import type { Metadata } from "next";
import "./globals.css";
import { TanstackProvider } from "@/lib/query/query-provider";
import { SocketProvider } from "@/lib/socket/socket-provider";
import { LanguageProvider } from "@/lib/i18n/language-context";
import LanguageSwitcher from "@/components/ui/language-switcher";

export const metadata: Metadata = {
  title: "RTC Chat",
  description: "Discute avec tes amis sur un RTC made in Epitech",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="fr">
      <body>
        <TanstackProvider>
          <SocketProvider>
            <LanguageProvider>
              {children}
              <LanguageSwitcher />
            </LanguageProvider>
          </SocketProvider>
        </TanstackProvider>
      </body>
    </html>
  );
}

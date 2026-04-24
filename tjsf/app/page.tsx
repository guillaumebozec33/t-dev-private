"use client";
import Button from "@/components/ui/button";
import { useTranslation } from "@/lib/i18n/language-context";
import {useAuthStore} from "@/lib/store/auth-store";
    import { useRouter } from 'next/navigation';
import {useEffect} from "react";

export default function Home() {
    const { t } = useTranslation();
    const router = useRouter();
    const token = useAuthStore((state) => state.token);

    useEffect(() => {
        if (token) {
            router.push('/chat');
        }
    }, [token, router]);

    return (
        <div className="min-h-screen bg-white flex items-center justify-center p-4">
            <div className="bg-gray-50 rounded-lg p-12 w-full max-w-lg text-center border border-gray-200">
                <div className="mb-8">
                    <h1 className="text-4xl font-bold text-gray-900 mb-4">{t("home.title")}</h1>
                    <p className="text-gray-700 text-lg mb-2">{t("home.subtitle")}</p>
                    <p className="text-gray-600">{t("home.tagline")}</p>
                </div>

                <div className="space-y-4">
                    <Button
                        className="text-lg py-3 cursor-pointer"
                        onClick={() => router.push('/register')}
                    >
                        {t("home.createAccount")}
                    </Button>

                    <p className="text-gray-600">
                        {t("home.alreadyHaveAccount")}{" "}
                        <span
                            onClick={() => router.push('/login')}
                            className="text-steel-blue hover:underline font-medium cursor-pointer"
                        >
              {t("home.signIn")}
            </span>
                    </p>
                </div>

                <div className="mt-8 pt-6 border-t border-gray-200">
                    <p className="text-gray-600 text-sm">{t("home.catchphrase")}</p>
                </div>
            </div>
        </div>
    );
}
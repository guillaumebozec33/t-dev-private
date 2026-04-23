"use client";

import { useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import Input from "@/components/ui/input";
import Button from "@/components/ui/button";
import { useMutation } from "@tanstack/react-query";
import { login } from "@/lib/api/endpoints/auth";
import { useAuthStore } from "@/lib/store/auth-store";
import { AuthOutput } from "@/types";
import { useTranslation } from "@/lib/i18n/language-context";

export default function LoginForm() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [errorMessage, setErrorMessage] = useState("");
  const router = useRouter();
  const { t } = useTranslation();

  const loginMutation = useMutation({
    mutationFn: login,
    onSuccess: (data: AuthOutput) => {
      useAuthStore.getState().setAuth(data.token,data.user);
      setErrorMessage("");
      router.push("/chat");
    },
    onError: (error) => {
      setErrorMessage(t(`errors.code.${error.message}`));
    },
  });

  const handleSubmit = () => {
    if (!email || !password) {
      setErrorMessage(t("errors.fillAllFields"));
      return;
    }
    loginMutation.mutate({ email: email, password: password });
  };

  return (
    <div className="min-h-screen bg-white flex items-center justify-center p-4">
      <div className="bg-gray-50 rounded-lg p-6 sm:p-8 w-full max-w-md border border-gray-200">
        <div className="text-center mb-6">
          <h1 className="text-xl sm:text-2xl font-semibold text-gray-900 mb-2">
            {t("auth.login.title")}
          </h1>
          <p className="text-sm sm:text-base text-gray-600">
            {t("auth.login.subtitle")}
          </p>
        </div>

        <form
          className="space-y-4"
          onSubmit={(e) => {
            e.preventDefault();
            handleSubmit();
          }}
        >
          {errorMessage && (
            <div className="bg-red-600 text-white p-3 rounded text-sm">
              {errorMessage}
            </div>
          )}
          <Input
            label={t("auth.login.email")}
            type="email"
            value={email}
            onChange={setEmail}
            required
          />
          <Input
            label={t("auth.login.password")}
            type="password"
            value={password}
            onChange={setPassword}
            required
          />
          <Button type="submit">{t("auth.login.submit")}</Button>
          <p className="text-sm text-gray-600">
            {t("auth.login.needAccount")}{" "}
            <Link href="/register" className="text-steel-blue hover:underline">
              {t("auth.login.register")}
            </Link>
          </p>
        </form>
      </div>
    </div>
  );
}

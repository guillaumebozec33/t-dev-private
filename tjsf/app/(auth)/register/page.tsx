"use client";

import {useState} from "react";
import Input from "@/components/ui/input";
import Button from "@/components/ui/button";
import {useMutation} from "@tanstack/react-query";
import {register} from "@/lib/api/endpoints/auth";
import {useAuthStore} from "@/lib/store/auth-store";
import {useRouter} from "next/navigation";
import {AuthOutput} from "@/types";
import {useTranslation} from "@/lib/i18n/language-context";

export default function RegisterForm() {
    const [email, setEmail] = useState("");
    const [username, setUsername] = useState("");
    const [password, setPassword] = useState("");
    const [errorMessage, setErrorMessage] = useState("");
    const [passwordErrors, setPasswordErrors] = useState<string[]>([]);
    const {t} = useTranslation();

    const validatePassword = (pwd: string) => {
        const errors = [];
        if (pwd.length < 12) errors.push(t("auth.register.passwordRequirements.minLength"));
        if (!/[A-Z]/.test(pwd)) errors.push(t("auth.register.passwordRequirements.uppercase"));
        if (!/[a-z]/.test(pwd)) errors.push(t("auth.register.passwordRequirements.lowercase"));
        if (!/\d/.test(pwd)) errors.push(t("auth.register.passwordRequirements.number"));
        if (!/[!@#$%^&*(),.?":{}|<>]/.test(pwd))
            errors.push(t("auth.register.passwordRequirements.special"));
        return errors;
    };

    const router = useRouter();
    const registerMutation = useMutation({
        mutationFn: register,
        onSuccess: (data: AuthOutput) => {
            useAuthStore.getState().setAuth(data.token, data.user);
            setErrorMessage("");
            router.push("/chat");
        },
        onError: (error) => {
            console.error(error);
            setErrorMessage(t(`errors.code.${error.message}`));
        },
    });

    const handleSubmit = () => {
        if (!email || !password || !username) {
            setErrorMessage(t("errors.fillAllFields"));
            return;
        }
        const errors = validatePassword(password);
        if (errors.length > 0) {
            setPasswordErrors(errors);
            setErrorMessage(t("errors.invalidPassword"));
            return;
        }
        registerMutation.mutate({
            email: email,
            password: password,
            username: username,
        });
    };

    return (
        <div className="min-h-screen bg-white flex items-center justify-center p-4">
            <div className="bg-gray-50 rounded-lg p-6 sm:p-8 w-full max-w-md border border-gray-200">
                <div className="text-center mb-6">
                    <h1 className="text-xl sm:text-2xl font-semibold text-gray-900 mb-2">
                        {t("auth.register.title")}
                    </h1>
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
                        label={t("auth.register.email")}
                        type="email"
                        value={email}
                        onChange={setEmail}
                        required
                    />

                    <Input
                        label={t("auth.register.username")}
                        value={username}
                        onChange={setUsername}
                        required
                    />

                    <Input
                        label={t("auth.register.password")}
                        type="password"
                        value={password}
                        onChange={(value) => {
                            setPassword(value);
                            const errors = validatePassword(value);
                            setPasswordErrors(errors);
                        }}
                        required
                    />
                    {passwordErrors.length > 0 && (
                        <div className=" text-bordeaux p-3 rounded text-sm">
                            <p className="font-semibold mb-1">
                                {t("auth.register.passwordRequirements.title")}
                            </p>
                            <ul className="text-xs space-y-1">
                                {passwordErrors.map((error, index) => (
                                    <li key={index}>• {error}</li>
                                ))}
                            </ul>
                        </div>
                    )}

                    <Button type="submit" className="cursor-pointer">{t("auth.register.submit")}</Button>

                    <p className="text-xs text-gray-600">
                        {t("auth.register.termsPrefix")}{" "}
                        <span className="text-steel-blue hover:underline cursor-pointer">
    {t("auth.register.terms")}
  </span>{" "}
                        {t("auth.register.termsAnd")}{" "}
                        <span className="text-steel-blue hover:underline cursor-pointer">
    {t("auth.register.privacy")}
  </span>
                        .
                    </p>

                    <span
                        onClick={() => router.push('/login')}
                        className="block text-center text-steel-blue hover:underline text-sm cursor-pointer"
                    >
  {t("auth.register.alreadyHaveAccount")}
</span>
                </form>
            </div>
        </div>
    );
}

"use client";

import { useState } from "react";
import { X } from "lucide-react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import Button from "@/components/ui/button";
import Input from "@/components/ui/input";
import { useToast } from "@/components/ui/toast";
import { useTranslation } from "@/lib/i18n/language-context";
import { updateMyProfile } from "@/lib/api/endpoints/users";
import { useAuthStore } from "@/lib/store/auth-store";
import { StoredUserStatus } from "@/types";

interface ProfileModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function ProfileModal({ isOpen, onClose }: ProfileModalProps) {
  const user = useAuthStore.getState().user;
  const setUser = useAuthStore((state) => state.setUser);
  const queryClient = useQueryClient();
  const { showError } = useToast();
  const { t } = useTranslation();

  const [username, setUsername] = useState(user?.username ?? "");
  const [avatarUrl, setAvatarUrl] = useState<string |null>(user?.avatar_url ?? "");
  const [status, setStatus] = useState<StoredUserStatus>(
    user?.status === "offline" ? "online" : (user?.status as StoredUserStatus) || "online"
  );

  const mutation = useMutation({
    mutationFn: updateMyProfile,
    onSuccess: async (updatedUser) => {
      setUser(updatedUser);
        queryClient.invalidateQueries({ queryKey: ["members"] });
        queryClient.invalidateQueries({ queryKey: ["messages"] });
        queryClient.invalidateQueries({ queryKey: ["dm_messages"] });
        queryClient.invalidateQueries({ queryKey: ["dm_conversations"] });
      onClose();
    },
    onError: (error: Error) => {
      showError(
        t(`errors.code.${error.message}`) !== `errors.code.${error.message}`
          ? t(`errors.code.${error.message}`)
          : t("profile.updateError")
      );
    },
  });

  if (!isOpen || !user) return null;

  const isUsernameInvalid = username.trim().length < 3 || username.trim().length > 32;

  const handleSubmit = () => {
    mutation.mutate({
      username: username.trim(),
      avatar_url: avatarUrl?.trim() || null,
      status:status == "donotdisturb" ? "dnd":status,
    });
  };

  return (
    <div className="fixed inset-0 bg-black/30 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-lg p-4 sm:p-6 w-full max-w-md shadow-xl border border-gray-200 max-h-[90vh] overflow-y-auto">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-lg sm:text-xl font-semibold text-gray-900">{t("profile.modalTitle")}</h2>
          <button onClick={onClose} className="text-gray-600 hover:text-bordeaux-hover hover:cursor-pointer">
            <X size={20} />
          </button>
        </div>

        <div className="space-y-4">
          <div className="flex items-center gap-3 rounded border border-gray-200 p-3 bg-gray-50">
            {avatarUrl ? (
              <img
                src={avatarUrl}
                alt={username || user.username}
                className="w-11 h-11 rounded-full object-cover border border-gray-200"
                onError={(e) => {
                  e.currentTarget.style.display = "none";
                }}
              />
            ) : (
              <div className="w-11 h-11 rounded-full bg-steel-blue text-white font-semibold flex items-center justify-center">
                {(username || user.username).charAt(0).toUpperCase()
                }
              </div>
            )}
            <div className="text-sm">
              <p className="text-gray-900 font-medium">{username || user.username}</p>
              <p className="text-gray-500">{t("profile.preview")}</p>
            </div>
          </div>

          <Input
            label={t("profile.usernameLabel")}
            value={username}
            onChange={setUsername}
            required
          />

          <div className="relative">
            <Input
              label={t("profile.avatarLabel")}
              value={avatarUrl ?? ""}
              onChange={setAvatarUrl}
              placeholder="https://example.com/avatar.png"
            />
            {avatarUrl && (
              <button
                type="button"
                onClick={() => setAvatarUrl(null)}
                className="absolute right-2 top-[30px] text-gray-400 hover:text-gray-600"
              >
                <X size={14} />
              </button>
            )}
          </div>

          <div>
            <label className="block text-xs font-semibold text-gray-700 uppercase tracking-wide mb-2">
              {t("profile.statusLabel")}
            </label>
            <select
              value={status}
              onChange={(e) => setStatus(e.target.value as StoredUserStatus)}
              className="w-full bg-white border border-gray-300 rounded px-3 py-2.5 text-gray-900 focus:outline-none focus:ring-2 focus:ring-steel-blue focus:border-transparent"
            >
              <option value="online">{t("members.online")}</option>
              <option value="away">{t("members.away")}</option>
              <option value="donotdisturb">{t("members.dnd")}</option>
              <option value="invisible">{t("members.invisible")}</option>
            </select>
          </div>

          <div className="flex gap-2 pt-2">
            <button
              onClick={onClose}
              className="flex-1 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded-lg transition-colors hover:text-bordeaux-hover"
            >
              {t("profile.cancel")}
            </button>
            <Button
              onClick={handleSubmit}
              className="flex-1 hover:cursor-pointer"
              disabled={isUsernameInvalid || mutation.isPending}
            >
              {mutation.isPending ? t("profile.saving") : t("profile.save")}
            </Button>
          </div>

          {isUsernameInvalid && (
            <p className="text-danger text-sm">{t("profile.usernameConstraint")}</p>
          )}
        </div>
      </div>
    </div>
  );
}

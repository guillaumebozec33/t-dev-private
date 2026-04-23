"use client";

import { useState } from "react";
import Button from "@/components/ui/button";
import Input from "@/components/ui/input";
import { useMutation } from "@tanstack/react-query";
import { addChannels, updateChannel } from "@/lib/api/endpoints/channels";
import { useQueryClient } from "@tanstack/react-query";
import { Channel } from "@/types";
import { X, Lock, Hash, Volume2, Bell, Star, Heart, Zap, Globe, Music, Video, Image, FileText, Code, Coffee, Gamepad2, BookOpen, Megaphone, Shield, Flame, Smile } from "lucide-react";
import { useTranslation } from "@/lib/i18n/language-context";
import { useSelectedServer } from "@/hooks";

const ICON_OPTIONS = [
  { name: "Hash", component: Hash },
  { name: "Volume2", component: Volume2 },
  { name: "Bell", component: Bell },
  { name: "Star", component: Star },
  { name: "Heart", component: Heart },
  { name: "Zap", component: Zap },
  { name: "Globe", component: Globe },
  { name: "Music", component: Music },
  { name: "Video", component: Video },
  { name: "Image", component: Image },
  { name: "FileText", component: FileText },
  { name: "Code", component: Code },
  { name: "Coffee", component: Coffee },
  { name: "Gamepad2", component: Gamepad2 },
  { name: "BookOpen", component: BookOpen },
  { name: "Megaphone", component: Megaphone },
  { name: "Shield", component: Shield },
  { name: "Flame", component: Flame },
  { name: "Smile", component: Smile },
];

interface CreateChannelModalProps {
  isOpen: boolean;
  onClose: () => void;
  channel: Channel | null;
  isEdit: boolean | null;
  onDelete?: (channelId: string) => void;
}

export default function ChannelModal({
  isOpen,
  onClose,
  channel,
  isEdit,
  onDelete,
}: CreateChannelModalProps) {
  const { selectedServer } = useSelectedServer();
  const [name, setName] = useState(isEdit ? channel?.name ?? "" : "");
  const [isPrivate, setIsPrivate] = useState(channel?.is_private ?? false);
  const [selectedIcon, setSelectedIcon] = useState<string | null>(channel?.icon ?? null);
  const [showIconPicker, setShowIconPicker] = useState(false);
  const { t } = useTranslation();
  const buttonText = isEdit ? t("channelModal.buttonEdit") : t("channelModal.buttonCreate");
  const titleLabel = isEdit ? t("channelModal.titleEdit") : t("channelModal.titleCreate");
  const MAX_LENGTH = 20;
  const isOverLimit = name.length > MAX_LENGTH;

  const queryClient = useQueryClient();

  const mutation = useMutation({
    mutationFn: addChannels,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["channels"] });
    },
  });

  const editMutation = useMutation({
    mutationFn: updateChannel,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["channels"] });
    },
  });

  function handleSubmit() {
    if (isEdit && channel) {
      editMutation.mutate({ name, id: channel.id, is_private: isPrivate, icon: selectedIcon });
    } else {
      if (!selectedServer) return;
      mutation.mutate({ name, server_id: selectedServer.id, is_private: isPrivate, icon: selectedIcon });
    }


  }

    const confirmDelete = async (channelId:string) => {
        let confirmed = false;

        if ("__TAURI__" in window) {
            const { confirm } = await import("@tauri-apps/plugin-dialog");
            confirmed = await confirm(t("channels.deleteConfirm"), {
                title: t("channels.deleteTitle"),
                kind: "warning",
            });
        } else {
            confirmed = window.confirm(t("channels.deleteConfirm"));
        }

        if (!confirmed) return;
        onDelete?.(channelId);
        onClose();
    };

  const CurrentIcon = selectedIcon
    ? ICON_OPTIONS.find((i) => i.name === selectedIcon)?.component ?? Hash
    : Hash;

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/30 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-lg p-4 sm:p-6 w-full max-w-md shadow-xl border border-gray-200">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-lg sm:text-xl font-semibold text-gray-900">{titleLabel}</h2>
          <button onClick={onClose} className="text-gray-600 hover:text-bordeaux-hover hover:cursor-pointer">
            <X size={20} />
          </button>
        </div>

        <div className="space-y-4">
          <div>
            <Input
              label={t("channelModal.nameLabel")}
              value={name}
              onChange={setName}
              required
            />
            {isOverLimit && (
              <p className="text-danger text-sm mt-1">
                {t("channelModal.tooLong")} ({name.length}/{MAX_LENGTH})
              </p>
            )}
          </div>

          {/* Icon picker */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              {t("channelModal.iconLabel")}
            </label>
            <button
              type="button"
              onClick={() => setShowIconPicker((v) => !v)}
              className="flex items-center gap-2 px-3 py-2 border border-gray-300 rounded-lg text-sm text-gray-700 hover:border-bordeaux transition-colors"
            >
              <CurrentIcon size={16} />
              <span>{selectedIcon ?? t("channelModal.iconDefault")}</span>
            </button>

            {showIconPicker && (
              <div className="mt-2 p-2 border border-gray-200 rounded-lg grid grid-cols-6 gap-1 max-h-36 overflow-y-auto">
                {ICON_OPTIONS.map(({ name: iconName, component: Icon }) => (
                  <button
                    key={iconName}
                    type="button"
                    title={iconName}
                    onClick={() => {
                      setSelectedIcon(iconName);
                      setShowIconPicker(false);
                    }}
                    className={`p-2 rounded hover:bg-sidebar-hover transition-colors flex items-center justify-center ${
                      selectedIcon === iconName ? "bg-bordeaux text-white" : "text-gray-600"
                    }`}
                  >
                    <Icon size={16} />
                  </button>
                ))}
              </div>
            )}
          </div>

          <div className="flex items-center space-x-3">
            <input
              type="checkbox"
              id="isPrivate"
              checked={isPrivate}
              onChange={(e) => setIsPrivate(e.target.checked)}
              className="w-4 h-4 text-bordeaux bg-gray-100 border-gray-300 rounded focus:ring-bordeaux focus:ring-2"
            />
            <label htmlFor="isPrivate" className="flex items-center text-sm text-gray-700 cursor-pointer">
              <Lock size={16} className="mr-2 text-gray-500" />
              {t("channelModal.privateLabel")}
            </label>
          </div>

          <div className="flex space-x-3 pt-4">
            {isEdit && channel && (
              <button
                onClick={() => {
                  // if (!window.confirm(t("channels.deleteConfirm"))) return;
                    confirmDelete(channel.id);
                  // onDelete?.(channel.id);
                }}
                className="flex-1 bg-white text-bordeaux hover:cursor-pointer hover:text-white hover:bg-bordeaux px-4 py-2 text-sm font-medium rounded-lg transition-colors"
              >
                {t("channels.delete")}
              </button>
            )}
            <Button
              onClick={() => { handleSubmit(); onClose(); }}
              className="flex-1 disabled:opacity-50 disabled:cursor-not-allowed hover:cursor-pointer"
              disabled={isOverLimit || !name.trim()}
            >
              {buttonText}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}

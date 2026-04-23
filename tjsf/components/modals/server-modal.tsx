"use client";

import { useState } from "react";
import Button from "@/components/ui/button";
import Input from "@/components/ui/input";
import { useMutation } from "@tanstack/react-query";
import { addServers, joinServer, deleteServer } from "@/lib/api/endpoints/servers";
import { useQueryClient } from "@tanstack/react-query";
import { Server } from "@/types";
import { updateServer } from "@/lib/api/endpoints/servers";
import { X, Trash2 } from "lucide-react";
import { useTranslation } from "@/lib/i18n/language-context";
import { useToast } from "@/components/ui/toast";

interface CreateServerModalProps {
  isOpen: boolean;
  onClose: () => void;
  server: Server | null;
  isEdit?: boolean;
  onServerJoined?: (serverId: string) => void;
  onDelete?: (serverId: string) => void;
}

export default function ServerModal({
  isOpen,
  onClose,
  server,
  isEdit,
  onServerJoined,
  onDelete,
}: CreateServerModalProps) {
  const [name, setName] = useState(server?.name ?? "");
  const [description, setDescription] = useState(server?.description ?? "");
  const [iconUrl, setIconUrl] = useState(server?.icon_url ?? "");
  const [joinCode, setJoinCode] = useState("");
  const { t } = useTranslation();
  const { showError } = useToast();
  const handleError = (error: Error) =>
    showError(
      t(`errors.code.${error.message}`) !== `errors.code.${error.message}`
        ? t(`errors.code.${error.message}`)
        : "Action impossible"
    );
  const queryClient = useQueryClient();
  const MAX_LENGTH = 20;
  const isOverLimit = name.length > MAX_LENGTH;

  const buttonText = isEdit ? t("serverModal.buttonEdit") : t("serverModal.buttonCreate");
  const labelTitle = isEdit ? t("serverModal.titleEdit") : t("serverModal.titleCreate");
  const labelSubTitle = isEdit ? t("serverModal.subtitleEdit") : t("serverModal.subtitleCreate");

  const mutation = useMutation({
    mutationFn: addServers,
    onSuccess: async (data: Server) => {
      await queryClient.invalidateQueries({ queryKey: ["servers"] });
      onClose();
      if (onServerJoined) onServerJoined(data.id);
    },
    onError: (error: Error) => handleError(error),
  });

  const joinServerMutation = useMutation({
    mutationFn: joinServer,
    onSuccess: async (data: Server) => {
      await queryClient.invalidateQueries({ queryKey: ["servers"] });
      onClose();
      if (data.id && onServerJoined) onServerJoined(data.id);
    },
    onError: (error: Error) => handleError(error),
  });

  const editMutation = useMutation({
    mutationFn: updateServer,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["servers"] });
    },
    onError: (error: Error) => handleError(error),
  });

  function handleSubmit() {
    if (isEdit && server) {
      editMutation.mutate({
        name,
        id: server.id,
        description,
        icon_url: iconUrl || "",
      });
      onClose();
    } else {
      mutation.mutate({ name, description, icon_url: iconUrl || undefined });
    }
  }

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/30 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-lg p-4 sm:p-6 w-full max-w-md shadow-xl border border-gray-200 max-h-[90vh] overflow-y-auto">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-lg sm:text-xl font-semibold text-gray-900">{labelTitle}</h2>
          <button onClick={onClose} className="text-gray-600 hover:text-bordeaux-hover hover:cursor-pointer">
            <X size={20} />
          </button>
        </div>

        <div className="space-y-6">
          <div className="space-y-4">
            <h3 className="text-lg font-medium text-gray-900">{labelSubTitle}</h3>

            {/* Icon preview + input */}
            <div className="flex items-center gap-4">
              <div className="w-14 h-14 rounded-full bg-sidebar-hover flex items-center justify-center flex-shrink-0 overflow-hidden">
                {iconUrl ? (
                  <img src={iconUrl} alt="icon" className="w-full h-full object-cover rounded-full" />
                ) : (
                  <span className="text-xl font-semibold text-gray-600">
                    {name.charAt(0).toUpperCase() || "?"}
                  </span>
                )}
              </div>
              <div className="flex-1 space-y-1">
                <Input
                  label={t("serverModal.iconLabel")}
                  value={iconUrl}
                  onChange={setIconUrl}
                  placeholder={t("serverModal.iconPlaceholder")}
                />
                {iconUrl && (
                  <button
                    type="button"
                    onClick={() => setIconUrl("")}
                    className="flex items-center gap-1 text-xs text-bordeaux hover:text-bordeaux-hover"
                  >
                    <Trash2 size={12} />
                    {t("serverModal.removeIcon")}
                  </button>
                )}
              </div>
            </div>

            <div>
              <Input
                label={t("serverModal.nameLabel")}
                value={name}
                onChange={setName}
                required
              />
              {isOverLimit && (
                <p className="text-danger text-sm mt-1">
                  {t("serverModal.tooLong")} ({name.length}/{MAX_LENGTH})
                </p>
              )}
            </div>

            <Input
              label={t("serverModal.descriptionLabel")}
              value={description}
              onChange={setDescription}
            />

            <div className="flex space-x-3">
              {isEdit && server && onDelete && (
                <button
                  onClick={() => {
                    if (!window.confirm(t("serverModal.deleteConfirm"))) return;
                    onDelete(server.id);
                    onClose();
                  }}
                  className="flex-1 bg-white text-bordeaux hover:cursor-pointer hover:text-white hover:bg-bordeaux px-4 py-2 text-sm font-medium rounded-lg transition-colors"
                >
                  {t("serverModal.delete")}
                </button>
              )}
              <Button
                onClick={handleSubmit}
                className="flex-1 hover:cursor-pointer"
                disabled={isOverLimit || !name.trim()}
              >
                {buttonText}
              </Button>
            </div>
          </div>

          {!isEdit && !server && (
            <>
              <div className="flex items-center">
                <div className="flex-1 h-px bg-gray-300"></div>
                <span className="px-4 text-gray-600 text-sm">{t("serverModal.or")}</span>
                <div className="flex-1 h-px bg-gray-300"></div>
              </div>

              <div className="space-y-4">
                <h3 className="text-lg font-medium text-gray-900">{t("serverModal.joinServer")}</h3>
                <Input
                  label={t("serverModal.inviteCode")}
                  value={joinCode}
                  onChange={setJoinCode}
                  placeholder={t("serverModal.inviteCodePlaceholder")}
                />
                <Button
                  onClick={() => joinServerMutation.mutate({ invite_code: joinCode })}
                  disabled={!joinCode.trim()}
                  className="w-full hover:cursor-pointer"
                >
                  {t("serverModal.joinButton")}
                </Button>
              </div>

              <div className="flex justify-center pt-4">
                <button
                  onClick={onClose}
                  className="flex-1 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded-lg transition-colors hover:text-bordeaux-hover"
                >
                  {t("serverModal.cancel")}
                </button>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

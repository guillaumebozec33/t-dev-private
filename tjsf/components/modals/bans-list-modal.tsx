"use client";

import { X, Shield, Ban as BanIcon } from "lucide-react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { getBans, unbanMember } from "@/lib/api/endpoints/bans";
import { Ban } from "@/types";
import { useTranslation } from "@/lib/i18n/language-context";
import {useSelectedServer} from "@/hooks";

interface BansListModalProps {
  onClose: () => void;
}

export default function BansListModal({
  onClose,
}: BansListModalProps) {
    const { selectedServer } = useSelectedServer();
  const queryClient = useQueryClient();
  const { t, language } = useTranslation();

  const { data: bans = [], isLoading } = useQuery<Ban[]>({
    queryKey: ["bans", selectedServer?.id],
    queryFn: () => getBans(selectedServer!.id),
      enabled: !!selectedServer?.id,
  });

  const unbanMutation = useMutation({
    mutationFn: unbanMember,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["bans", selectedServer?.id] });
    },
  });

  const handleUnban = (userId: string) => {
      if (!selectedServer?.id) return;
    if (confirm(t("bans.confirmUnban"))) {
      unbanMutation.mutate({ serverId:selectedServer.id, userId });
    }
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    const locale = language === "fr" ? "fr-FR" : "en-GB";
    return date.toLocaleDateString(locale, {
      day: "2-digit",
      month: "2-digit",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  const getRemainingTime = (expiresAt: string) => {
    const now = new Date();
    const expiry = new Date(expiresAt);
    const diff = expiry.getTime() - now.getTime();

    if (diff < 0) return t("bans.expired");

    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));

    if (days > 365) return t("bans.permanentLabel");
    if (days > 0) return t("bans.daysHoursRemaining", { days, hours });
    return t("bans.hoursRemaining", { hours });
  };

  return (
    <>
      <div
        className="fixed inset-0 bg-black/30 backdrop-blur-sm z-40"
        onClick={onClose}
      />
      <div className="fixed inset-0 flex items-center justify-center z-50 p-4">
        <div className="bg-white rounded-lg shadow-xl w-full max-w-2xl max-h-[80vh] flex flex-col">
          {/* Header */}
          <div className="flex items-center justify-between p-4 border-b border-gray-200">
            <div className="flex items-center gap-2">
              <Shield size={20} className="text-red-600" />
              <h2 className="text-lg font-semibold text-gray-900">
                {t("bans.title")}
              </h2>
            </div>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-bordeaux-hover hover:cursor-pointer transition-colors"
            >
              <X size={20} />
            </button>
          </div>

          {/* Content */}
          <div className="flex-1 overflow-y-auto p-4">
            {isLoading ? (
              <div className="text-center py-8 text-gray-500">
                {t("bans.loading")}
              </div>
            ) : bans.length === 0 ? (
              <div className="text-center py-8 text-gray-500">
                {t("bans.empty")}
              </div>
            ) : (
              <div className="space-y-3">
                {bans.map((ban) => (
                  <div
                    key={ban.id}
                    className="flex items-center justify-between p-4 bg-gray-50 rounded-lg border border-gray-200"
                  >
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <div className="w-10 h-10 bg-red-100 rounded-full flex items-center justify-center">
                          <BanIcon size={20} className="text-red-600" />
                        </div>
                        <div>
                          <p className="font-medium text-gray-900">
                              {ban.username}
                          </p>
                          <p className="text-xs text-gray-500">
                            {t("bans.bannedAt")} {formatDate(ban.banned_at)}
                          </p>
                        </div>
                      </div>
                      <div className="mt-2 ml-12">
                        <p className="text-sm text-gray-600">
                          {ban.is_permanent ? (
                            <span className="text-red-600 font-medium">
                              {t("bans.permanent")}
                            </span>
                          ) : (
                            <>
                              {t("bans.expiresAt")} {formatDate(ban.expires_at)}
                              <span className="ml-2 text-gray-500">
                                ({getRemainingTime(ban.expires_at)})
                              </span>
                            </>
                          )}
                        </p>
                      </div>
                    </div>

                    <button
                      onClick={() => handleUnban(ban.user_id)}
                      disabled={unbanMutation.isPending}
                      className="px-4 py-2 text-sm font-medium text-white bg-bordeaux hover:bg-bordeaux-hover disabled:bg-gray-400 rounded-lg transition-colors"
                    >
                      {t("bans.unban")}
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Footer */}
          <div className="flex items-center justify-end p-4 border-t border-gray-200">
            <button
              onClick={onClose}
              className="px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded-lg transition-colors hover:cursor-pointer hover:text-bordeaux-hover"
            >
              {t("bans.close")}
            </button>
          </div>
        </div>
      </div>
    </>
  );
}

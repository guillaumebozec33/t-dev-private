"use client";

import {Server } from "@/types";
import { useState } from "react";
import ServerModal from "@/components/modals/server-modal";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useMutation } from "@tanstack/react-query";
import { leaveServer, deleteServer } from "@/lib/api/endpoints/servers";
import { useQueryClient } from "@tanstack/react-query";
import { Plus, LogOut, MessageSquare } from "lucide-react";
import { useTranslation } from "@/lib/i18n/language-context";
import { useToast } from "@/components/ui/toast";
import {useServers, useSelectedServer,useUserRole} from '@/hooks'

interface ServerListProps {
  onServerSelect: (serverId: string) => void;
  onServerLeave?: () => void;
  onLogout?: () => void;
  onOpenDmInbox?: () => void;
  isDmInboxOpen?: boolean;
  hasUnreadDms?: boolean;
}

export default function ServerList({
  onServerSelect,
  onServerLeave,
  onLogout,
  onOpenDmInbox,
  isDmInboxOpen,
  hasUnreadDms,
}: ServerListProps) {
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isEdit, setIsEdit] = useState(false);
  const [serverToEdit, setServerToEdit] = useState<Server | null>(null);
  const queryClient = useQueryClient();
  const { t } = useTranslation();
  const { showError } = useToast();
  const handleError = (error: Error) => showError(t(`errors.code.${error.message}`) !== `errors.code.${error.message}` ? t(`errors.code.${error.message}`) : "Action impossible");
  const {servers} = useServers();
  const {selectedServerId} = useSelectedServer();
  const {isOwner} = useUserRole();



  const mutation = useMutation({
    mutationFn: leaveServer,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["servers"] });
      if (onServerLeave) onServerLeave();
    },
    onError: (error: Error) => handleError(error),
  });

  const deleteMutation = useMutation({
    mutationFn: deleteServer,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["servers"] });
      if (onServerLeave) onServerLeave();
    },
    onError: (error: Error) => handleError(error),
  });

  const handleLeaveServer = (server: Server) => {
    mutation.mutate(server.id);
  };

  const handleRemoveServer = (serverId: string) => {
    deleteMutation.mutate(serverId);
  };

  return (
    <>
      <div className="w-16 flex-shrink-0 bg-sidebar-bg flex flex-col items-center h-full">
        <ScrollArea className="flex-1 py-3" hideScrollbar>
          <div className="flex flex-col items-center space-y-2">
            <button
              onClick={onOpenDmInbox}
              className={`relative w-10 h-10 sm:w-12 sm:h-12 rounded-full flex items-center justify-center cursor-pointer transition-all ${
                isDmInboxOpen
                  ? "bg-bordeaux text-white"
                  : "bg-sidebar-hover hover:bg-bordeaux-hover hover:text-white text-gray-700"
              }`}
              title={t("dm.openInbox")}
              type="button"
              aria-label={t("dm.openInbox")}
            >
              <MessageSquare size={18} />
              {hasUnreadDms && !isDmInboxOpen && (
                <span className="absolute -top-0.5 -right-0.5 w-2.5 h-2.5 bg-bordeaux rounded-full border-2 border-gray-100" />
              )}
            </button>

            <div className="w-full h-px bg-gray-300 mx-2" />

            {servers.map((server) => (
              <div
                key={server.id}
                onClick={() => onServerSelect(server.id)}
                className={`relative w-10 h-10 sm:w-12 sm:h-12 rounded-2xl flex items-center justify-center cursor-pointer transition-all overflow-hidden group ${
                  selectedServerId === server.id
                    ? "bg-bordeaux text-white rounded-xl"
                    : "bg-sidebar-hover hover:bg-bordeaux-hover hover:text-white text-gray-700 hover:rounded-xl"
                }`}
                title={server.name + " - " + server.description}
              >
                {server.icon_url ? (
                  <img
                    src={server.icon_url}
                    alt={server.name}
                    className="w-full h-full object-cover"
                  />
                ) : (
                  <span className="text-sm font-semibold">
                    {server.name.charAt(0).toUpperCase()}
                  </span>
                )}
              </div>
            ))}

            <div className="w-full h-px bg-gray-300 mx-2" />

            <div
              onClick={() => setIsModalOpen(true)}
              className="w-12 h-12 rounded-full flex items-center justify-center cursor-pointer transition-all bg-bordeaux hover:bg-bordeaux-hover text-white hover:text-white"
              title={t("serverModal.subtitleCreate")}
            >
              <Plus size={20} />
            </div>
          </div>
        </ScrollArea>

        {onLogout && (
          <button
            onClick={onLogout}
            className="flex justify-center items-center p-3 w-full bg-sidebar-bg hover:bg-sidebar-hover transition-colors flex-shrink-0 cursor-pointer"
            title={t("chat.logout")}
          >
            <LogOut size={20} className="text-bordeaux" />
          </button>
        )}
      </div>

      <ServerModal
        key={`${isModalOpen}-${serverToEdit?.id}-${isEdit}`}
        isOpen={isModalOpen}
        isEdit={isEdit}
        server={serverToEdit || null}
        onClose={() => {
          setIsModalOpen(false);
          setIsEdit(false);
          setServerToEdit(null);
        }}
        onServerJoined={onServerSelect}
      />

    </>
  );
}

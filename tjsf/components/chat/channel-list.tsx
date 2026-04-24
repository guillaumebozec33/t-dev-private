"use client";

import React from "react";
import { Channel } from "@/types";
import { useState } from "react";
import ChannelModal from "@/components/modals/channel-modal";
import InviteUserModal from "@/components/modals/invite-user-modal";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { deleteChannel } from "@/lib/api/endpoints/channels";
import { deleteServer, leaveServer } from "@/lib/api/endpoints/servers";
import { Plus, UserPlus, Lock, Settings, Hash, Volume2, Bell, Star, Heart, Zap, Globe, Music, Video, Image, FileText, Code, Coffee, Gamepad2, BookOpen, Megaphone, Shield, Flame, Smile, LogOut } from "lucide-react";
import Profile from "@/components/chat/profile";
import { useTranslation } from "@/lib/i18n/language-context";
import { useToast } from "@/components/ui/toast";
import { checkLength } from "@/lib/utils/checkLength";
import { useUserRole,useChannels,useSelectedServer,useSelectedChannel } from "@/hooks";
import {useChannelStore} from "@/lib/store/channel-store";
import ServerModal from "@/components/modals/server-modal";

const CHANNEL_ICONS: Record<string, React.ComponentType<{ size?: number; className?: string }>> = {
  Hash, Volume2, Bell, Star, Heart, Zap, Globe, Music, Video, Image,
  FileText, Code, Coffee, Gamepad2, BookOpen, Megaphone, Shield, Flame, Smile,
};

function ChannelIcon({ icon, isPrivate }: { icon?: string | null; isPrivate: boolean }) {
  const IconComp = icon && CHANNEL_ICONS[icon] ? CHANNEL_ICONS[icon] : null;
  return (
    <span className="relative mr-2 flex-shrink-0 flex items-center justify-center w-4 h-4">
      {IconComp ? (
        <IconComp size={14} className="text-gray-500" />
      ) : (
        <span className="text-gray-500 text-sm leading-none">#</span>
      )}
      {isPrivate && (
        <Lock size={8} className="absolute -bottom-1 -right-1 text-gray-500" />
      )}
    </span>
  );
}

interface ChannelListProps {
  onChannelSelect: (channelId: string) => void;
}

export default function ChannelList({
  onChannelSelect,
}: ChannelListProps) {
  const [isChannelModalOpen, setIsChannelModalOpen] = useState(false);
  const [isEdit, setIsEdit] = useState(false);
  const [isInviteModalOpen, setIsInviteModalOpen] = useState(false);
  const [isServerModalOpen, setIsServerModalOpen] = useState(false);
  const queryClient = useQueryClient();
  const { isOwner, isAdmin } = useUserRole();
  const { t } = useTranslation();
  const { showError } = useToast();
  const {channels} = useChannels()
  const {selectedChannel} = useSelectedChannel()
  const {selectedServer} = useSelectedServer()
  const {setSelectedChannelId, resetSelectedChannelId} = useChannelStore()

  const handleError = (error: Error) =>
    showError(
      t(`errors.code.${error.message}`) !== `errors.code.${error.message}`
        ? t(`errors.code.${error.message}`)
        : "Action impossible"
    );

  const canSeePrivateChannels = isOwner || isAdmin;
  const visibleChannels = canSeePrivateChannels
    ? channels
    : channels.filter((channel) => !channel.is_private);
  const textChannels = visibleChannels.filter((channel) => channel.channel_type === "text");
  const voiceChannels = visibleChannels.filter((channel) => channel.channel_type === "voice");

    const renderChannelRow = (channel: Channel, index: number) => (
        <div
            key={channel.id}
            onClick={() => onChannelSelect(channel.id)}
            className={`group flex items-center px-2 py-1 rounded transition-colors cursor-pointer ${
                selectedChannel?.id === channel.id
                    ? "bg-sidebar-hover text-gray-900"
                    : "text-gray-600 hover:bg-sidebar-hover hover:text-gray-900"
            }`}
        >
            <ChannelIcon icon={channel.icon} isPrivate={channel.is_private} />
            <span className="text-sm truncate flex-1">{checkLength(channel.name)}</span>
            {(isOwner || isAdmin) && index !== 0 && (
                <button
                    onClick={(e) => { e.stopPropagation(); handleEditChannel(channel); }}
                    className="opacity-0 group-hover:opacity-100 ml-1 flex-shrink-0 text-gray-400 hover:text-gray-700 transition-opacity cursor-pointer"
                    title={t("channels.options")}
                >
                    <Settings size={14} className="hover:text-bordeaux-hover"/>
                </button>
            )}
        </div>
    );

  const deleteMutation = useMutation({
    mutationFn: deleteChannel,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["channels", selectedServer?.id] });
    },
    onError: (error: Error) => handleError(error),
  });
    const leaveServerMutation = useMutation({
        mutationFn: leaveServer,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["servers"] });
        },
        onError: (error: Error) => handleError(error),
    });

  const deleteServerMutation = useMutation({
    mutationFn: deleteServer,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["servers"] });
    },
    onError: (error: Error) => handleError(error),
  });

  const handleEditChannel = (channel: Channel) => {
    setIsEdit(true);
    setSelectedChannelId(channel.id);
    setIsChannelModalOpen(true);
  };
  const handleDeleteChannel = (channelId: string) => {
    resetSelectedChannelId();
    deleteMutation.mutate(channelId);
  };
  const handleCreateChannelMenu = () => {
    setIsEdit(false);
    setIsChannelModalOpen(true);
  };

  return (
    <>
      <div className="w-full md:w-60 bg-sidebar-bg flex flex-col min-w-0 h-full">
        <div className="p-4 border-b border-gray-300 max-h-[57px]">
          <div className="flex items-center justify-between ">
            <h2 className="text-gray-900 font-semibold truncate">
              {checkLength(selectedServer?.name) || t("channels.server")}
            </h2>
            <div className="flex items-center gap-1 flex-shrink-0">
                {!isOwner && (
                    <button
                        onClick={() => selectedServer && leaveServerMutation.mutate(selectedServer.id)}
                        className="text-gray-600 hover:text-bordeaux-hover cursor-pointer"
                        title={t("channels.leaveServer")}
                    >
                        <LogOut size={16} />
                    </button>
                )}
              {isOwner && (
                <button
                  onClick={() => setIsServerModalOpen(true)}
                  className="text-gray-600 hover:text-bordeaux-hover cursor-pointer"
                  title={t("serverList.editServer")}
                >
                  <Settings size={16} />
                </button>
              )}
              {(isOwner || isAdmin) && (
                <button
                  onClick={() => setIsInviteModalOpen(true)}
                  className="text-gray-600 hover:text-bordeaux-hover text-lg font-bold cursor-pointer"
                  title={t("channels.inviteUser")}
                >
                  <UserPlus size={16} />
                </button>
              )}
            </div>
          </div>
          <p className="text-xs text-gray-600 line-clamp-2" title={selectedServer?.description ? selectedServer?.description : "" }>
            {checkLength(
selectedServer?.description
            )}
          </p>
        </div>

        <ScrollArea className="flex-1">
          <div className="p-2 flex-1">
            <div className="mb-4">
              <div className="flex items-center justify-between px-2 mb-2">
                <h3 className="text-xs font-semibold text-gray-600 uppercase tracking-wide">
                  {t("channels.textChannels")}
                </h3>
                {(isOwner || isAdmin) && (
                  <button
                    onClick={() => handleCreateChannelMenu()}
                    className="text-gray-600 hover:text-bordeaux-hover text-lg font-bold cursor-pointer"
                    title={t("channels.createChannel")}
                  >
                    <Plus size={16} />
                  </button>
                )}
              </div>
              {textChannels.map((channel, index) => renderChannelRow(channel, index))}
            </div>

            <div className="mb-4">
              <div className="flex items-center justify-between px-2 mb-2">
                <h3 className="text-xs font-semibold text-gray-600 uppercase tracking-wide">
                  {t("channels.voiceChannels")}
                </h3>
              </div>
              {voiceChannels.map((channel, index) => renderChannelRow(channel, index))}
            </div>
          </div>
        </ScrollArea>

        <Profile />
      </div>

      <ChannelModal
        key={`${isChannelModalOpen}-${selectedChannel?.id}-${isEdit}`}
        isOpen={isChannelModalOpen}
        onClose={() => {
          setIsChannelModalOpen(false);
          setIsEdit(false);
        }}
        isEdit={isEdit}
        channel={selectedChannel || null}
        onDelete={handleDeleteChannel}
      />

      <InviteUserModal
        isOpen={isInviteModalOpen}
        onClose={() => setIsInviteModalOpen(false)}
      />

      <ServerModal
        key={`${isServerModalOpen}-${selectedServer?.id}`}
        isOpen={isServerModalOpen}
        isEdit={true}
        server={selectedServer || null}
        onClose={() => setIsServerModalOpen(false)}
        onDelete={(id) => { deleteServerMutation.mutate(id); setIsServerModalOpen(false); }}
      />

    </>
  );
}

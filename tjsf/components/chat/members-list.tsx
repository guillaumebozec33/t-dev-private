"use client";
import {Member} from "@/types";
import { useState } from "react";
import ContextMenu from "@/components/chat/context-menu";
import { useMutation } from "@tanstack/react-query";
import { useQueryClient } from "@tanstack/react-query";
import { changeRole, transferOwnership } from "@/lib/api/endpoints/members";
import { Crown, Shield, Ban as BanIcon, MessageSquare, UserMinus, ShieldPlus, ShieldMinus, GitFork } from "lucide-react";
import { useAuthStore } from "@/lib/store/auth-store";
import { kickMember } from "@/lib/api/endpoints/servers";
import { banMember } from "@/lib/api/endpoints/bans";
import BanDurationModal from "@/components/modals/ban-duration-modal";
import BansListModal from "@/components/modals/bans-list-modal";
import { useTranslation } from "@/lib/i18n/language-context";
import { useToast } from "@/components/ui/toast";
import { checkLength } from "@/lib/utils/checkLength";
import {useSelectedServer,useMembers,useMemberPresence,useUserRole} from '@/hooks'

interface MembersListProps {
  onDmOpen?: (userId: string) => void;
}

export default function MembersList({
  onDmOpen
}: MembersListProps) {
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    member: Member;
  } | null>(null);
  const [banModalMember, setBanModalMember] = useState<Member | null>(null);
  const [showBansList, setShowBansList] = useState(false);
  const queryClient = useQueryClient();
  const { t } = useTranslation();
  const { showError } = useToast();
  const {members} = useMembers();
  const {selectedServer} = useSelectedServer();




  const handleError = (error: Error) => showError(t(`errors.code.${error.message}`) !== `errors.code.${error.message}` ? t(`errors.code.${error.message}`) : "Action impossible");

  const mutation = useMutation({
    mutationFn: changeRole,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["members"] });
      queryClient.invalidateQueries({ queryKey: ["channels", selectedServer?.id] });
    },
    onError: (error: Error) => handleError(error),
  });
  const transferOwnershipMutation = useMutation({
    mutationFn: transferOwnership,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["members"] });
      queryClient.invalidateQueries({ queryKey: ["channels", selectedServer?.id] });
    },
    // onError: (error: Error) => handleError(error),
  });
  const kickMutation = useMutation({
    mutationFn: kickMember,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["members"] });
    },
    onError: (error: Error) => handleError(error),
  });

  const banMutation = useMutation({
    mutationFn: banMember,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["members"] });
      queryClient.invalidateQueries({ queryKey: ["bans", selectedServer?.id] });
    },
    onError: (error: Error) => handleError(error),
  });
  const { isOwner, isAdmin } = useUserRole();
  const me = useAuthStore.getState().user;

  const membersWithPresence = useMemberPresence(members);

  const onlineMembers = membersWithPresence.filter((member) => member.displayStatus !== "offline");
  const offlineMembers = membersWithPresence.filter((member) => member.displayStatus === "offline");

  const handleMemberClick = (e: React.MouseEvent | React.TouchEvent, member: Member) => {
    if (member.user_id === me?.id) return;
    const { clientX, clientY } = 'touches' in e ? e.touches[0] : e as React.MouseEvent;
    setContextMenu({ x: clientX, y: clientY, member });
  };

  const handleContextMenu = (e: React.MouseEvent, member: Member) => {
    e.preventDefault();
    handleMemberClick(e, member);
  };

  const closeContextMenu = () => {
    setContextMenu(null);
  };

  const handlePromoteAdmin = (member: Member) => {
      if(!selectedServer?.id) return;
      mutation.mutate({
      user_id: member.user_id,
      role: "admin",
      server_id: selectedServer.id,
    });
  };

  const handleDemoteToMember = (member: Member) => {
    if(!selectedServer?.id) return;
      mutation.mutate({
      user_id: member.user_id,
      role: "member",
      server_id: selectedServer.id,
    });
  };
  const handleKickMember = (member: Member) => {
      if(!selectedServer?.id) return;
      kickMutation.mutate({ serverId: selectedServer.id, memberId: member.user_id });
  };

  const handleBanMember = (member: Member) => {
    setBanModalMember(member);
    closeContextMenu();
  };

  const handleBanConfirm = (durationHours?: number) => {
      if(!selectedServer?.id) return;
      if (banModalMember) {
      banMutation.mutate({
        serverId:selectedServer.id,
        memberId: banModalMember.user_id,
        duration_hours: durationHours,
      });
      setBanModalMember(null);
    }
  };

  const handleTransferOwnership = (member: Member) => {
      if(!selectedServer?.id) return;
      transferOwnershipMutation.mutate({
      new_owner_id: member.user_id,
      server_id: selectedServer.id,
    });
  };

  const roleTag = (role: string) => {
    if (role === "owner") return <span className="flex items-center gap-0.5 text-[10px] font-semibold text-amber-600 bg-amber-100 px-1.5 py-0.5 rounded-full"><Crown size={10} />{role}</span>;
    if (role === "admin") return <span className="flex items-center gap-0.5 text-[10px] font-semibold text-blue-600 bg-blue-100 px-1.5 py-0.5 rounded-full"><Shield size={10} />{role}</span>;
    return null;
  };

  const renderMemberAvatar = (member: Member, offline = false) => {
    if (member.avatar_url) {
      return (
        <img
          src={member.avatar_url}
          alt={member.username}
          className="w-8 h-8 rounded-full object-cover"
        />
      );
    }
    return (
      <div
        className={`w-8 h-8 ${offline ? "bg-gray-400" : "bg-steel-blue"} rounded-full flex items-center justify-center text-sm font-semibold text-white`}
      >
        {member.username.charAt(0).toUpperCase()}
      </div>
    );
  };

  return (
    <div className="w-full md:w-60 bg-sidebar-bg text-gray-900 flex flex-col">
      <div className="p-4 pb-5 border-b border-gray-300 flex items-center justify-between">
        <h3 className="text-sm font-semibold text-gray-600">
            {t('members.title')} — {members.length}
        </h3>
        {(isOwner || isAdmin) && (
          <button
            onClick={() => setShowBansList(true)}
            className=" hover:bg-sidebar-hover rounded transition-colors cursor-pointer"
            title={t("members.viewBanned")}
          >
            <BanIcon size={16} className="text-black hover:text-bordeaux-hover" />
          </button>
        )}
      </div>

      <div className="flex-1 overflow-y-auto p-2">
        {onlineMembers.length > 0 && (
          <div className="mb-4">
            <h4 className="text-xs font-semibold text-gray-600 uppercase mb-2 px-2">
              {t("members.online")} — {onlineMembers.length}
            </h4>
            {onlineMembers.map((member) => (
              <div
                key={member.id}
                className={`flex items-center gap-2 px-2 py-1 rounded hover:bg-sidebar-hover ${member.user_id !== me?.id ? 'cursor-pointer' : 'cursor-default'}`}
                onContextMenu={(e) => handleContextMenu(e, member)}
                onClick={(e) => handleMemberClick(e, member)}
              >
                <div className="relative flex-shrink-0">
                  {renderMemberAvatar(member)}
                  {member.displayStatus === "online" ? (
                    <div className="absolute -bottom-1 -right-1 w-3 h-3 bg-green-500 rounded-full border-2 border-sidebar-bg"></div>
                  ) : member.displayStatus === "away" ? (
                    <div className="absolute -bottom-1 -right-1 w-3 h-3 bg-yellow-500 rounded-full border-2 border-sidebar-bg"></div>
                  ) : member.displayStatus === "donotdisturb" ? (
                    <div className="absolute -bottom-1 -right-1 w-3 h-3 bg-red-600 rounded-full border-2 border-sidebar-bg"></div>
                  ) : (
                    <div className="absolute -bottom-1 -right-1 w-3 h-3 bg-gray-400 rounded-full border-2 border-sidebar-bg"></div>
                  )}
                </div>
                <span className="ml-3 text-sm text-gray-900 truncate">
                  {checkLength(member.username)}
                </span>
                {roleTag(member.role)}
              </div>
            ))}
          </div>
        )}

        {offlineMembers.length > 0 && (
          <div>
            <h4 className="text-xs font-semibold text-gray-600 uppercase mb-2 px-2">
              {t("members.offline")} — {offlineMembers.length}
            </h4>
            {offlineMembers.map((member) => (
              <div
                key={member.id}
                className={`flex items-center gap-2 px-2 py-1 rounded hover:bg-sidebar-hover opacity-50 ${member.user_id !== me?.id ? 'cursor-pointer' : 'cursor-default'}`}
                onContextMenu={(e) => handleContextMenu(e, member)}
                onClick={(e) => handleMemberClick(e, member)}
              >
                <div className="relative flex-shrink-0">
                  {renderMemberAvatar(member, true)}
                  <div className="absolute -bottom-1 -right-1 w-3 h-3 bg-gray-400 rounded-full border-2 border-sidebar-bg"></div>
                </div>
                <span className="ml-3 text-sm text-gray-600 opacity-70 truncate">
                  {member.username}
                </span>
                {roleTag(member.role)}
              </div>
            ))}
          </div>
        )}
      </div>

      {contextMenu && contextMenu.member.user_id !== me?.id && (() => {
        const m = contextMenu.member;
        const items = [];
        if (onDmOpen)
          items.push({ label: t("dm.action"), icon: <MessageSquare size={15} />, onClick: () => { onDmOpen(m.user_id); closeContextMenu(); } });
        if (isOwner) {
          items.push({ label: t("members.transferOwnership"), icon: <GitFork size={15} />, onClick: () => handleTransferOwnership(m), separator: !!onDmOpen });
          items.push({ label: t("members.promoteAdmin"), icon: <ShieldPlus size={15} />, onClick: () => handlePromoteAdmin(m), disabled: m.role === "admin" || m.role === "owner" });
          items.push({ label: t("members.demoteMember"), icon: <ShieldMinus size={15} />, onClick: () => handleDemoteToMember(m), disabled: m.role === "member" || m.role === "owner" });
          items.push({ label: t("members.kick"), icon: <UserMinus size={15} />, onClick: () => handleKickMember(m), separator: true });
          items.push({ label: t("members.ban"), icon: <BanIcon size={15} />, onClick: () => handleBanMember(m), variant: "danger" as const });
        } else if (isAdmin && m.role === "member") {
          items.push({ label: t("members.kick"), icon: <UserMinus size={15} />, onClick: () => handleKickMember(m), separator: !!onDmOpen });
          items.push({ label: t("members.ban"), icon: <BanIcon size={15} />, onClick: () => handleBanMember(m), variant: "danger" as const });
        }
        if (items.length === 0) return null;
        return (
          <ContextMenu
            x={contextMenu.x}
            y={contextMenu.y}
            title={checkLength(m.username)}
            subtitle={m.role}
            avatarUrl={m.avatar_url? m.avatar_url:undefined}
            items={items}
            onClose={closeContextMenu}
          />
        );
      })()}

      {banModalMember && (
        <BanDurationModal
          memberName={checkLength(banModalMember.username)}
          onConfirm={handleBanConfirm}
          onClose={() => setBanModalMember(null)}
        />
      )}

      {showBansList && (
        <BansListModal
          onClose={() => setShowBansList(false)}
        />
      )}
    </div>
  );
}

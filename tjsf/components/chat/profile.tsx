import {useAuthStore} from "@/lib/store/auth-store";
import ContextMenu from "@/components/chat/context-menu";
import {useState} from "react";
import {MoreVertical} from "lucide-react";
import {useTranslation} from "@/lib/i18n/language-context";
import ProfileModal from "@/components/modals/profile-modal";

export default function Profile() {
    const {user} = useAuthStore();
    const {t} = useTranslation();

    const [contextMenu, setContextMenu] = useState<{ x: number; y: number } | null>(null);
    const [isProfileModalOpen, setIsProfileModalOpen] = useState(false);
    if (!user) return null;


    const handleMenuOpen = (e: React.MouseEvent) => {
        e.preventDefault();
        setContextMenu({ x: e.clientX, y: e.clientY });
    };

    return (
        <>
            <div className="flex items-center  w-full h-[89.5px] py-3 border-t-1 border-b-1 border-gray-300">
                <div className="w-full px-2 flex justify-between">
                    <div
                        key={user.id}
                        className="flex items-center w-full px-2 py-1 rounded hover:bg-sidebar-hover cursor-pointer"
                        onClick={() => setIsProfileModalOpen(true)}

                    >
                        <div className="relative">
                            {user.avatar_url ? (
                                <img
                                    src={user.avatar_url}
                                    alt={user.username}
                                    className="w-8 h-8 rounded-full object-cover"
                                />
                            ) : (
                            <div
                                className="w-8 h-8 bg-steel-blue rounded-full flex items-center justify-center text-sm font-semibold text-white">
                                {user.username.charAt(0).toUpperCase()}
                            </div>
                            )}
                            {user?.status === "online" ? (
                                <div
                                    className="absolute -bottom-1 -right-1 w-3 h-3 bg-green-500 rounded-full border-2 border-sidebar-bg"></div>
                            ) : user.status === "away" ? (
                                <div className="absolute -bottom-1 -right-1 w-3 h-3 bg-yellow-500 rounded-full border-2 border-sidebar-bg"></div>
                            ) : user.status === "donotdisturb" ? (
                                <div
                                    className="absolute -bottom-1 -right-1 w-3 h-3 bg-red-600 rounded-full border-2 border-sidebar-bg"></div>
                            ): (
                                <div
                                className="absolute -bottom-1 -right-1 w-3 h-3 bg-gray-400 rounded-full border-2 border-sidebar-bg"></div>
                                )}

                        </div>
                        <span className="ml-3 text-sm text-gray-900">
                            {user.username}
                        </span>
                    </div>
                    {/*<button*/}
                    {/*    onClick={handleMenuOpen}*/}
                    {/*    className="p-1 rounded hover:bg-gray-200 text-gray-600 hover:text-gray-900 cursor-pointer"*/}
                    {/*    title={t('profile.options')}*/}
                    {/*    aria-label={t('profile.options')}*/}
                    {/*>*/}
                    {/*    <MoreVertical size={16} />*/}
                    {/*</button>*/}
                </div>

                {contextMenu && (
                    <ContextMenu
                        x={contextMenu.x}
                        y={contextMenu.y}
                        title={t('profile.menuTitle')}
                        items={[
                            { label: t('profile.openEditor'), onClick: () => setIsProfileModalOpen(true) },
                        ]}
                        onClose={() => setContextMenu(null)}
                    />
                )}

            </div>

            {isProfileModalOpen && (
                <ProfileModal
                    isOpen={isProfileModalOpen}
                    onClose={() => setIsProfileModalOpen(false)}
                />
            )}
        </>

    );
}

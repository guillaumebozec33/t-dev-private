import { useAuthStore } from '@/lib/store/auth-store';
import {useMembers} from "@/hooks";

export function useUserRole() {
  const me = useAuthStore.getState().user;
  const {members} = useMembers()

  const currentMember = members.find(m => m.user_id === me?.id);
  const role = currentMember?.role;
  
  return {
    isOwner: role === 'owner',
    isAdmin: role === 'admin',
    isMember: role === 'member',
    role,
    hasPermission: (requiredRole: 'owner' | 'admin' | 'member') => {
      if (requiredRole === 'member') return true;
      if (requiredRole === 'admin') return role === 'admin' || role === 'owner';
      if (requiredRole === 'owner') return role === 'owner';
      return false;
    }
  };
}
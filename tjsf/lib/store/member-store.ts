import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import {Member} from '@/types';

type MemberStore = {
    members : Member[] | null;
    _hasHydrated : boolean;
    setMembers : (members:Member[]) => void
    setHasHydrated : (val:boolean) => void
};

export const useMemberStore = create<MemberStore>()(
    persist(
        (set)=>({
            members : null,
            _hasHydrated:false,
            setMembers:(members)=> set({members}),
            setHasHydrated:(val)=> set({_hasHydrated: val}),
        }),
        {name:'member-storage',
            partialize:(state)=>({
                members:state.members,
            }),
            onRehydrateStorage: ()=>(state) =>{
                state?.setHasHydrated(true);
            }
        }
    )
);


import { create } from 'zustand';
import { persist } from 'zustand/middleware';

type DmStore = {
    selectedDmConversationId : string | null;
    _hasHydrated : boolean;
    setSelectedDmConversationId : (convId:string) => void
    setHasHydrated : (val:boolean) => void
    resetSelectedDmConversationId : () => void
};

export const useDmStore = create<DmStore>()(
    persist(
        (set)=>({
            selectedDmConversationId : null,
            _hasHydrated:false,
            setSelectedDmConversationId:(convId)=> set({selectedDmConversationId: convId}),
            setHasHydrated:(val)=> set({_hasHydrated: val}),
            resetSelectedDmConversationId:() => set({selectedDmConversationId:null})
        }),
        {name:'dm-storage',
            partialize:(state)=>({
                selectedDmConversationId:state.selectedDmConversationId
            }),
            onRehydrateStorage: ()=>(state) =>{
                state?.setHasHydrated(true);
            }
        }
    )
);


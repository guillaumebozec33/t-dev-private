import { create } from 'zustand';
import { persist } from 'zustand/middleware';

type ServerStore = {
    selectedServerId : string | null;
    _hasHydrated : boolean;
    setSelectedServerId : (serverId:string) => void
    setHasHydrated : (val:boolean) => void
    resetSelectedServerId : () => void
};

export const useServerStore = create<ServerStore>()(
    persist(
        (set)=>({
            selectedServerId : null,
            _hasHydrated:false,
            setSelectedServerId:(serverId)=> set({selectedServerId: serverId}),
            setHasHydrated:(val)=> set({_hasHydrated: val}),
            resetSelectedServerId:() => set({selectedServerId:null})
        }),
        {name:'server-storage',
            partialize:(state)=>({
                selectedServerId:state.selectedServerId
            }),
            onRehydrateStorage: ()=>(state) =>{
                state?.setHasHydrated(true);
            }
        }
    )
);


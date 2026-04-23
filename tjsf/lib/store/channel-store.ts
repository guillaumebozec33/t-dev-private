import { create } from 'zustand';
import { persist } from 'zustand/middleware';

type ChannelStore = {
    selectedChannelId : string | null;
    _hasHydrated : boolean;
    setSelectedChannelId : (channelId:string) => void
    setHasHydrated : (val:boolean) => void
    resetSelectedChannelId : () => void
};

export const useChannelStore = create<ChannelStore>()(
    persist(
        (set)=>({
            selectedChannelId : null,
            _hasHydrated:false,
            setSelectedChannelId:(channelId)=> set({selectedChannelId: channelId}),
            setHasHydrated:(val)=> set({_hasHydrated: val}),
            resetSelectedChannelId:() => set({selectedChannelId:null})
        }),
        {name:'channel-storage',
            partialize:(state)=>({
                selectedChannelId:state.selectedChannelId
            }),
            onRehydrateStorage: ()=>(state) =>{
                state?.setHasHydrated(true);
            }
        }
    )
);


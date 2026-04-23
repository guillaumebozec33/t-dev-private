import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import {Message} from '@/types';

type MessageStore = {
    messages : Message[] | null;
    _hasHydrated : boolean;
    setMessages : (message:Message[]) => void
    setHasHydrated : (val:boolean) => void
};

export const useMessageStore = create<MessageStore>()(
    persist(
        (set)=>({
            messages : null,
            _hasHydrated:false,
            setMessages:(messages)=> set({messages}),
            setHasHydrated:(val)=> set({_hasHydrated: val}),
        }),
        {name:'message-storage',
            partialize:(state)=>({
                messages:state.messages,
            }),
            onRehydrateStorage: ()=>(state) =>{
                state?.setHasHydrated(true);
            }
        }
    )
);


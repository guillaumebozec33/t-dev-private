import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import {User} from '@/types';

type AuthStore = {
    token : string | null;
    user : User | null;
    isAuthenticated : boolean;
    _hasHydrated : boolean;
    setAuth:(token:string,user:User) => void;
    logout : () => void;
    setUser: (user:User) => void;
    setHasHydrated : (val:boolean) => void
};

export const useAuthStore = create<AuthStore>()(
  persist(
      (set)=>({
          token:null,
          user:null,
          isAuthenticated:false,
          _hasHydrated:false,
          setAuth:(token,user)=> set({token, user, isAuthenticated: true}),
          logout:()=> set({token:null,user:null,isAuthenticated:false}),
          setUser: (user) => set({user}),
          setHasHydrated:(val)=> set({_hasHydrated: val}),
      }),
      {name:'auth-storage',
      partialize:(state)=>({
          token:state.token,
          user:state.user,
          isAuthenticated:state.isAuthenticated,
      }),
          onRehydrateStorage: ()=>(state) =>{
          state?.setHasHydrated(true);
          }
      }
  )
);


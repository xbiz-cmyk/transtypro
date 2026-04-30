import { create } from "zustand";
import type { DictationMode } from "../lib/types";

interface ModesState {
  modes: DictationMode[];
  isLoading: boolean;
  error: string | null;
  setModes: (modes: DictationMode[]) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
}

export const useModesStore = create<ModesState>((set) => ({
  modes: [],
  isLoading: false,
  error: null,
  setModes: (modes) => set({ modes }),
  setLoading: (isLoading) => set({ isLoading }),
  setError: (error) => set({ error }),
}));

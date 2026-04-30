import { create } from "zustand";
import type { AppSettings } from "../lib/types";

interface SettingsState {
  settings: Partial<AppSettings>;
  isLoading: boolean;
  error: string | null;
  setSettings: (settings: Partial<AppSettings>) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  settings: {},
  isLoading: false,
  error: null,
  setSettings: (settings) => set({ settings }),
  setLoading: (isLoading) => set({ isLoading }),
  setError: (error) => set({ error }),
}));

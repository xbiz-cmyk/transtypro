import { create } from "zustand";
import type { HistoryEntry } from "../lib/types";

interface HistoryState {
  entries: HistoryEntry[];
  isLoading: boolean;
  error: string | null;
  setEntries: (entries: HistoryEntry[]) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
}

export const useHistoryStore = create<HistoryState>((set) => ({
  entries: [],
  isLoading: false,
  error: null,
  setEntries: (entries) => set({ entries }),
  setLoading: (isLoading) => set({ isLoading }),
  setError: (error) => set({ error }),
}));

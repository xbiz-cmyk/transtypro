import { create } from "zustand";

interface UiState {
  overlayOpen: boolean;
  toggleOverlay: () => void;
  activeMode: string;
  setActiveMode: (mode: string) => void;
}

export const useUiStore = create<UiState>((set) => ({
  overlayOpen: false,
  toggleOverlay: () => set((s) => ({ overlayOpen: !s.overlayOpen })),
  activeMode: "Smart",
  setActiveMode: (mode) => set({ activeMode: mode }),
}));

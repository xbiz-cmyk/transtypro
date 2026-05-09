import { create } from "zustand";

interface UiState {
  overlayOpen: boolean;
  toggleOverlay: () => void;
  openOverlay: () => void;
  closeOverlay: () => void;
  activeMode: string;
  setActiveMode: (mode: string) => void;
}

export const useUiStore = create<UiState>((set) => ({
  overlayOpen: false,
  toggleOverlay: () => set((s) => ({ overlayOpen: !s.overlayOpen })),
  openOverlay: () => set({ overlayOpen: true }),
  closeOverlay: () => set({ overlayOpen: false }),
  activeMode: "Smart",
  setActiveMode: (mode) => set({ activeMode: mode }),
}));

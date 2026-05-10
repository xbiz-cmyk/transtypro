import { create } from "zustand";

interface UiState {
  overlayOpen: boolean;
  toggleOverlay: () => void;
  openOverlay: () => void;
  closeOverlay: () => void;
  activeMode: string;
  setActiveMode: (mode: string) => void;
  /** Current PTT pipeline phase. "idle" when no pipeline is running. */
  pttPhase: string;
  /** Human-readable PTT status message. Never contains user-dictated content. */
  pttMessage: string;
  setPttStatus: (phase: string, message: string) => void;
}

export const useUiStore = create<UiState>((set) => ({
  overlayOpen: false,
  toggleOverlay: () => set((s) => ({ overlayOpen: !s.overlayOpen })),
  openOverlay: () => set({ overlayOpen: true }),
  closeOverlay: () => set({ overlayOpen: false }),
  activeMode: "Smart",
  setActiveMode: (mode) => set({ activeMode: mode }),
  pttPhase: "idle",
  pttMessage: "",
  setPttStatus: (phase, message) => set({ pttPhase: phase, pttMessage: message }),
}));

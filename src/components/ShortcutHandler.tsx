import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useNavigate } from "react-router-dom";
import { useUiStore } from "../stores/uiStore";

export default function ShortcutHandler() {
  const navigate = useNavigate();
  const openOverlay = useUiStore((s) => s.openOverlay);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<null>("dictation-shortcut-pressed", () => {
      openOverlay();
      navigate("/dictation");
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, [navigate, openOverlay]);

  return null;
}

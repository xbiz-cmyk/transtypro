import { BrowserRouter, Routes, Route } from "react-router-dom";
import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Sidebar from "./components/Sidebar";
import StatusBar from "./components/StatusBar";
import FloatingOverlay from "./components/FloatingOverlay";
import ShortcutHandler from "./components/ShortcutHandler";
import PttOverlay from "./components/PttOverlay";
import Home from "./pages/Home";
import Dictation from "./pages/Dictation";
import History from "./pages/History";
import Modes from "./pages/Modes";
import Vocabulary from "./pages/Vocabulary";
import Models from "./pages/Models";
import Providers from "./pages/Providers";
import Settings from "./pages/Settings";
import Privacy from "./pages/Privacy";
import Diagnostics from "./pages/Diagnostics";
import About from "./pages/About";
import { getAppVersion, getStatusSummary } from "./lib/api";

// Evaluated once at module load time (synchronous in Tauri v2).
// The ptt-overlay window renders only the small feedback overlay;
// the main window renders the full app shell unchanged.
const IS_PTT_OVERLAY = getCurrentWindow().label === "ptt-overlay";

function MainApp() {
  const [version, setVersion] = useState("...");
  const [privacyMode, setPrivacyMode] = useState("local-only");

  useEffect(() => {
    async function loadAppInfo() {
      try {
        const v = await getAppVersion();
        setVersion(v);
      } catch {
        setVersion("?.?.?");
      }

      try {
        const status = await getStatusSummary();
        setPrivacyMode(status.privacy_mode);
      } catch {
        // Keep default "local-only" on error
      }
    }
    loadAppInfo();
  }, []);

  return (
    <BrowserRouter>
      <ShortcutHandler />
      <div id="app-shell" className="min-h-screen bg-(--color-surface-base)">
        {/* Sidebar */}
        <Sidebar />

        {/* Main content area — offset by sidebar width */}
        <div className="ml-(--spacing-sidebar) flex flex-col min-h-screen">
          {/* Status bar */}
          <StatusBar privacyMode={privacyMode} version={version} />

          {/* Page content */}
          <main className="flex-1 overflow-y-auto">
            <Routes>
              <Route path="/" element={<Home />} />
              <Route path="/dictation" element={<Dictation />} />
              <Route path="/history" element={<History />} />
              <Route path="/modes" element={<Modes />} />
              <Route path="/vocabulary" element={<Vocabulary />} />
              <Route path="/models" element={<Models />} />
              <Route path="/providers" element={<Providers />} />
              <Route path="/settings" element={<Settings />} />
              <Route path="/privacy" element={<Privacy />} />
              <Route path="/diagnostics" element={<Diagnostics />} />
              <Route path="/about" element={<About />} />
            </Routes>
          </main>
        </div>

        {/* Floating dictation overlay */}
        <FloatingOverlay />
      </div>
    </BrowserRouter>
  );
}

export default function App() {
  if (IS_PTT_OVERLAY) return <PttOverlay />;
  return <MainApp />;
}

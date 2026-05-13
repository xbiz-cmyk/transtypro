import { BrowserRouter, Routes, Route } from "react-router-dom";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Sidebar from "./components/Sidebar";
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
// Evaluated once at module load time (synchronous in Tauri v2).
// The ptt-overlay window renders only the small feedback overlay;
// the main window renders the full app shell unchanged.
const IS_PTT_OVERLAY = getCurrentWindow().label === "ptt-overlay";

function MainApp() {
  return (
    <BrowserRouter>
      <ShortcutHandler />
      <div id="app-shell" className="min-h-screen bg-(--color-surface-base)">
        {/* Sidebar */}
        <Sidebar />

        {/* Main content area — offset by sidebar width */}
        <main className="ml-(--spacing-sidebar) min-h-screen overflow-y-auto">
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

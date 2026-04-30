/**
 * transtypro — Application shell.
 *
 * Layout with sidebar, status bar, and routed content area.
 * Uses react-router-dom for page routing.
 */
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { useEffect, useState } from "react";
import Sidebar from "./components/Sidebar";
import StatusBar from "./components/StatusBar";
import Home from "./pages/Home";
import { getAppVersion, getStatusSummary } from "./lib/api";

export default function App() {
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
            </Routes>
          </main>
        </div>
      </div>
    </BrowserRouter>
  );
}

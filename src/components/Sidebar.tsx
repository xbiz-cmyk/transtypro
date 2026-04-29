/**
 * transtypro — Sidebar navigation component.
 *
 * Shows navigation items with enabled/disabled state.
 * Disabled items show "Coming soon" instead of pretending to work.
 */
import { NavLink } from "react-router-dom";
import type { NavItem } from "../lib/types";

const navItems: NavItem[] = [
  { path: "/", label: "Home", icon: "🏠", enabled: true },
  { path: "/dictation", label: "Dictation", icon: "🎙️", enabled: false },
  { path: "/history", label: "History", icon: "📋", enabled: false },
  { path: "/modes", label: "Modes", icon: "⚡", enabled: false },
  { path: "/vocabulary", label: "Vocabulary", icon: "📖", enabled: false },
  { path: "/models", label: "Models", icon: "🧠", enabled: false },
  { path: "/providers", label: "Providers", icon: "☁️", enabled: false },
  { path: "/settings", label: "Settings", icon: "⚙️", enabled: false },
  { path: "/privacy", label: "Privacy", icon: "🔒", enabled: false },
  { path: "/diagnostics", label: "Diagnostics", icon: "🔧", enabled: false },
];

export default function Sidebar() {
  return (
    <aside
      id="sidebar"
      className="fixed top-0 left-0 h-screen w-(--spacing-sidebar) bg-(--color-surface-sidebar) border-r border-(--color-border-default) flex flex-col z-10"
    >
      {/* Brand header */}
      <div className="px-5 py-5 border-b border-(--color-border-subtle)">
        <h1 className="text-lg font-semibold tracking-tight text-(--color-text-primary)">
          transtypro
        </h1>
        <p className="text-xs text-(--color-text-muted) mt-1">
          Speak instead of type
        </p>
      </div>

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto py-3 px-2">
        <ul className="space-y-0.5">
          {navItems.map((item) => (
            <li key={item.path}>
              {item.enabled ? (
                <NavLink
                  to={item.path}
                  id={`nav-${item.label.toLowerCase()}`}
                  className={({ isActive }) =>
                    `flex items-center gap-3 px-3 py-2.5 rounded-(--radius-btn) text-sm transition-colors duration-150 ${
                      isActive
                        ? "bg-(--color-surface-overlay) text-(--color-brand-300) font-medium"
                        : "text-(--color-text-secondary) hover:bg-(--color-surface-raised) hover:text-(--color-text-primary)"
                    }`
                  }
                >
                  <span className="text-base w-5 text-center">{item.icon}</span>
                  <span>{item.label}</span>
                </NavLink>
              ) : (
                <div
                  className="flex items-center gap-3 px-3 py-2.5 rounded-(--radius-btn) text-sm text-(--color-text-muted) cursor-default select-none"
                  title={`${item.label} — coming in a later phase`}
                >
                  <span className="text-base w-5 text-center opacity-50">
                    {item.icon}
                  </span>
                  <span className="flex-1">{item.label}</span>
                  <span className="text-[0.625rem] uppercase tracking-wider opacity-60">
                    Soon
                  </span>
                </div>
              )}
            </li>
          ))}
        </ul>
      </nav>

      {/* Footer */}
      <div className="px-4 py-3 border-t border-(--color-border-subtle) text-xs text-(--color-text-muted)">
        Phase 0 — Bootstrap
      </div>
    </aside>
  );
}

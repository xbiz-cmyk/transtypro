import { NavLink } from "react-router-dom";
import type { NavItem } from "../lib/types";

const mainNavItems: NavItem[] = [
  { path: "/", label: "Home", icon: "🏠" },
  { path: "/dictation", label: "Dictation", icon: "🎙️" },
  { path: "/history", label: "History", icon: "📋" },
  { path: "/modes", label: "Modes", icon: "⚡" },
  { path: "/vocabulary", label: "Vocabulary", icon: "📖" },
  { path: "/models", label: "Models", icon: "🧠" },
  { path: "/providers", label: "Providers", icon: "☁️" },
];

const bottomNavItems: NavItem[] = [
  { path: "/privacy", label: "Privacy", icon: "🔒" },
  { path: "/diagnostics", label: "Diagnostics", icon: "🔧" },
  { path: "/settings", label: "Settings", icon: "⚙️" },
  { path: "/about", label: "About", icon: "ℹ️" },
];

function NavItemLink({ item }: { item: NavItem }) {
  return (
    <li key={item.path}>
      <NavLink
        to={item.path}
        end={item.path === "/"}
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
    </li>
  );
}

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

      {/* Main navigation */}
      <nav className="flex-1 overflow-y-auto py-3 px-2">
        <ul className="space-y-0.5">
          {mainNavItems.map((item) => (
            <NavItemLink key={item.path} item={item} />
          ))}
        </ul>
      </nav>

      {/* Bottom navigation (privacy, settings, about) */}
      <div className="py-3 px-2 border-t border-(--color-border-subtle)">
        <ul className="space-y-0.5">
          {bottomNavItems.map((item) => (
            <NavItemLink key={item.path} item={item} />
          ))}
        </ul>
      </div>

      {/* Footer */}
      <div className="px-4 py-3 border-t border-(--color-border-subtle) text-xs text-(--color-text-muted)">
        Phase 1 — UI shell
      </div>
    </aside>
  );
}

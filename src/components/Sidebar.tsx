import { NavLink } from "react-router-dom";
import type { NavItem } from "../lib/types";
import Logo from "./Logo";
import {
  HomeIcon,
  DictationIcon,
  HistoryIcon,
  ModesIcon,
  VocabularyIcon,
  ModelsIcon,
  ProvidersIcon,
  PrivacyIcon,
  DiagnosticsIcon,
  SettingsIcon,
  AboutIcon,
} from "./icons";

const mainNavItems: NavItem[] = [
  { path: "/", label: "Home", icon: "home" },
  { path: "/dictation", label: "Dictation", icon: "dictation" },
  { path: "/history", label: "History", icon: "history" },
  { path: "/modes", label: "Modes", icon: "modes" },
  { path: "/vocabulary", label: "Vocabulary", icon: "vocabulary" },
  { path: "/models", label: "Models", icon: "models" },
  { path: "/providers", label: "Providers", icon: "providers" },
];

const bottomNavItems: NavItem[] = [
  { path: "/privacy", label: "Privacy", icon: "privacy" },
  { path: "/diagnostics", label: "Diagnostics", icon: "diagnostics" },
  { path: "/settings", label: "Settings", icon: "settings" },
  { path: "/about", label: "About", icon: "about" },
];

const iconMap: Record<string, React.ReactElement> = {
  home:        <HomeIcon size={16} />,
  dictation:   <DictationIcon size={16} />,
  history:     <HistoryIcon size={16} />,
  modes:       <ModesIcon size={16} />,
  vocabulary:  <VocabularyIcon size={16} />,
  models:      <ModelsIcon size={16} />,
  providers:   <ProvidersIcon size={16} />,
  privacy:     <PrivacyIcon size={16} />,
  diagnostics: <DiagnosticsIcon size={16} />,
  settings:    <SettingsIcon size={16} />,
  about:       <AboutIcon size={16} />,
};

function NavIcon({ name }: { name: string }) {
  return (
    <span className="w-5 flex items-center justify-center shrink-0">
      {iconMap[name] ?? null}
    </span>
  );
}

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
        <NavIcon name={item.icon} />
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
      <div className="px-5 py-5 border-b border-(--color-border-subtle) flex items-center gap-3">
        <Logo size={22} />
        <div>
          <h1 className="text-lg font-semibold tracking-tight text-(--color-text-primary) leading-none">
            transtypro
          </h1>
          <p className="text-xs text-(--color-text-muted) mt-0.5">
            Speak instead of type
          </p>
        </div>
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
    </aside>
  );
}

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
  home:        <HomeIcon size={15} />,
  dictation:   <DictationIcon size={15} />,
  history:     <HistoryIcon size={15} />,
  modes:       <ModesIcon size={15} />,
  vocabulary:  <VocabularyIcon size={15} />,
  models:      <ModelsIcon size={15} />,
  providers:   <ProvidersIcon size={15} />,
  privacy:     <PrivacyIcon size={15} />,
  diagnostics: <DiagnosticsIcon size={15} />,
  settings:    <SettingsIcon size={15} />,
  about:       <AboutIcon size={15} />,
};

function NavIcon({ name }: { name: string }) {
  return (
    <span className="w-4 flex items-center justify-center shrink-0">
      {iconMap[name] ?? null}
    </span>
  );
}

function NavItemLink({ item }: { item: NavItem }) {
  return (
    <li>
      <NavLink
        to={item.path}
        end={item.path === "/"}
        id={`nav-${item.label.toLowerCase()}`}
        className={({ isActive }) =>
          `relative flex items-center gap-2.5 px-3 py-[7px] rounded-(--radius-btn) text-sm transition-colors duration-100 ${
            isActive
              ? "bg-(--color-brand-500)/15 text-(--color-brand-300) font-semibold"
              : "text-(--color-text-muted) hover:bg-(--color-surface-raised) hover:text-(--color-text-primary)"
          }`
        }
      >
        {({ isActive }: { isActive: boolean }) => (
          <>
            {isActive && (
              <span
                className="absolute left-0 top-1/2 -translate-y-1/2 h-5 w-[3px] bg-(--color-brand-400) rounded-r-full"
                aria-hidden="true"
              />
            )}
            <NavIcon name={item.icon} />
            <span>{item.label}</span>
          </>
        )}
      </NavLink>
    </li>
  );
}

export default function Sidebar() {
  return (
    <aside
      id="sidebar"
      className="fixed top-0 left-0 h-screen w-(--spacing-sidebar) bg-(--color-surface-sidebar) border-r border-(--color-border-subtle) flex flex-col z-10"
    >
      {/* Brand header */}
      <div className="px-4 py-[14px] border-b border-(--color-border-subtle) flex items-center gap-3">
        <Logo size={26} />
        <div>
          <h1 className="text-[13px] font-bold tracking-tight text-(--color-text-primary) leading-none">
            transtypro
          </h1>
          <p className="text-[10px] text-(--color-text-muted) mt-[3px] leading-none">
            Speak instead of type
          </p>
        </div>
      </div>

      {/* Main navigation */}
      <nav className="flex-1 overflow-y-auto py-2 px-2">
        <ul className="space-y-0.5">
          {mainNavItems.map((item) => (
            <NavItemLink key={item.path} item={item} />
          ))}
        </ul>
      </nav>

      {/* Bottom navigation */}
      <div className="py-2 px-2 border-t border-(--color-border-subtle)">
        <ul className="space-y-0.5">
          {bottomNavItems.map((item) => (
            <NavItemLink key={item.path} item={item} />
          ))}
        </ul>
      </div>

      {/* Version footer */}
      <div className="px-4 py-2.5 border-t border-(--color-border-subtle)">
        <p className="text-[10px] text-(--color-text-muted) tabular-nums">v0.11 preview</p>
      </div>
    </aside>
  );
}

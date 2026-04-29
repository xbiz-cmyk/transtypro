/**
 * transtypro — Shared TypeScript types.
 *
 * All types used across the frontend should be defined here.
 * Keep in sync with Rust models in src-tauri/src/models/.
 */

/** Application status summary returned by the backend. */
export interface StatusSummary {
  /** Current privacy mode: "local-only" or "cloud-enabled" */
  privacy_mode: string;
  /** Whether a transcription model is configured and ready */
  transcription_ready: boolean;
  /** Name of the active cleanup provider, if any */
  cleanup_provider: string | null;
  /** Current dictation mode name */
  active_mode: string;
  /** Total number of history entries */
  history_count: number;
}

/** Navigation item for the sidebar. */
export interface NavItem {
  /** Route path */
  path: string;
  /** Display label */
  label: string;
  /** Icon name or identifier */
  icon: string;
  /** Whether this feature is available (false = "coming soon") */
  enabled: boolean;
}

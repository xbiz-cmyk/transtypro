/**
 * transtypro — Tauri command wrappers.
 *
 * All frontend-to-backend communication goes through these wrappers.
 * Never call `invoke()` directly from components.
 */
import { invoke } from "@tauri-apps/api/core";
import type { StatusSummary } from "./types";

/** Verify frontend-backend IPC communication. */
export async function ping(): Promise<string> {
  return invoke<string>("ping");
}

/** Get the current application version. */
export async function getAppVersion(): Promise<string> {
  return invoke<string>("get_app_version");
}

/** Get the application status summary for the home page. */
export async function getStatusSummary(): Promise<StatusSummary> {
  return invoke<StatusSummary>("get_status_summary");
}

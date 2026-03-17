import { invoke } from "@tauri-apps/api/core";
import type {
  DashboardStats,
  EnrichedTrade,
  ImportOutcome,
  ImportPreview,
  ImportRecord,
} from "./types/bindings";

export async function loadSample(): Promise<void> {
  await invoke("load_sample");
}

export async function dashboardStats(quote: string): Promise<DashboardStats> {
  return invoke("dashboard_stats", { quote });
}

export async function trades(quote: string): Promise<EnrichedTrade[]> {
  return invoke("trades", { quote });
}

export async function syncCandles(): Promise<void> {
  await invoke("sync_candles");
}

export async function previewImport(
  path: string,
  quote: string,
): Promise<ImportPreview> {
  return invoke("preview_import", { path, quote });
}

export async function confirmImport(
  path: string,
  quote: string,
): Promise<ImportOutcome> {
  return invoke("confirm_import", { path, quote });
}

export async function listImports(): Promise<ImportRecord[]> {
  return invoke("list_imports");
}

export async function removeImport(importId: number): Promise<void> {
  await invoke("remove_import", { importId });
}

export async function nukeAllData(): Promise<void> {
  await invoke("nuke_all_data");
}

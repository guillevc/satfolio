import { invoke } from "@tauri-apps/api/core";
import type {
  DashboardStats,
  EnrichedTrade,
  TradesSummary,
} from "./types/bindings";

export async function loadSample(): Promise<void> {
  await invoke("load_sample");
}

export async function getDashboardStats(): Promise<DashboardStats> {
  return invoke("dashboard_stats");
}

export async function getTrades(): Promise<EnrichedTrade[]> {
  return invoke("trades");
}

export async function syncCandles(): Promise<void> {
  await invoke("sync_candles");
}

export async function previewImport(path: string): Promise<TradesSummary> {
  return invoke("preview_import", { path });
}

export async function confirmImport(path: string): Promise<TradesSummary> {
  return invoke("confirm_import", { path });
}

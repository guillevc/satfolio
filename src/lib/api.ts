import { invoke } from "@tauri-apps/api/core";
import type { Candle, EnrichedTrade, PositionSummary } from "./types/bindings";

export async function loadSample(): Promise<void> {
  await invoke("load_sample");
}

export async function getPositionSummary(): Promise<PositionSummary> {
  return invoke("position_summary");
}

export async function getTrades(): Promise<EnrichedTrade[]> {
  return invoke("trades");
}

export async function getCandles(): Promise<Candle[]> {
  return invoke("candles");
}

export async function syncCandles(): Promise<void> {
  await invoke("sync_candles");
}

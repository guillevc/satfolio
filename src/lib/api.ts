import { invoke } from "@tauri-apps/api/core";
import type {
  BepSnapshot,
  Candle,
  EnrichedTrade,
  PositionSummary,
} from "./types/bindings";

export async function loadSample(): Promise<void> {
  await invoke("load_sample");
}

export async function getPositionSummary(): Promise<PositionSummary> {
  return invoke("position_summary");
}

export async function getBepSnaps(): Promise<Record<string, BepSnapshot>> {
  return invoke("bep_snaps");
}

export async function getTrades(): Promise<EnrichedTrade[]> {
  return invoke("trades");
}

export async function getCandles(): Promise<Candle[]> {
  return invoke("candles");
}

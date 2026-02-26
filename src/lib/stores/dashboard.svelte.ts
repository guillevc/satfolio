import {
  loadSample,
  getPositionSummary,
  getBepSnaps,
  getCandles,
  syncCandles,
} from "$lib/api";
import type { BepSnapshot, Candle, PositionSummary } from "$lib/types/bindings";
import { tick } from "svelte";

export const dashboard = $state({
  summary: null as PositionSummary | null,
  bepSnaps: null as Record<string, BepSnapshot> | null,
  candles: null as Candle[] | null,
  loading: false,
  error: null as string | null,
});

export async function loadDashboard(): Promise<void> {
  dashboard.loading = true;
  dashboard.error = null;
  await tick();
  try {
    const [, summary, bepSnaps, candles] = await Promise.all([
      loadSample(),
      getPositionSummary(),
      getBepSnaps(),
      getCandles(),
    ]);
    dashboard.summary = summary;
    dashboard.bepSnaps = bepSnaps;
    dashboard.candles = candles;

    // Gap-fill candles from Kraken in background (fire-and-forget)
    syncCandles().catch(() => {});
  } catch (e) {
    dashboard.error = String(e);
  } finally {
    dashboard.loading = false;
  }
}

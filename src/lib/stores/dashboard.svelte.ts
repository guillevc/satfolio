import {
  loadSample,
  getPositionSummary,
  getBepSnaps,
  getCandles,
} from "$lib/api";
import type { BepSnapshot, Candle, PositionSummary } from "$lib/types/bindings";

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
  try {
    await loadSample();
    const [summary, bepSnaps, candles] = await Promise.all([
      getPositionSummary(),
      getBepSnaps(),
      getCandles(),
    ]);
    dashboard.summary = summary;
    dashboard.bepSnaps = bepSnaps;
    dashboard.candles = candles;
  } catch (e) {
    dashboard.error = String(e);
  } finally {
    dashboard.loading = false;
  }
}

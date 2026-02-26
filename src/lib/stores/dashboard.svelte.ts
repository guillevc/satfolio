import {
  getPositionSummary,
  getBepSnaps,
  getCandles,
  syncCandles,
} from "$lib/api";
import type { BepSnapshot, Candle, PositionSummary } from "$lib/types/bindings";

export const dashboard = $state({
  summary: null as PositionSummary | null,
  bepSnaps: null as Record<string, BepSnapshot> | null,
  candles: null as Candle[] | null,
  loading: false,
  syncing: false,
  error: null as string | null,
});

export async function loadDashboard(): Promise<void> {
  dashboard.error = null;
  dashboard.loading = true;
  try {
    const [summary, bepSnaps, candles] = await Promise.all([
      getPositionSummary(),
      getBepSnaps(),
      getCandles(),
    ]);
    dashboard.summary = summary;
    dashboard.bepSnaps = bepSnaps;
    dashboard.candles = candles;
  } catch (e) {
    dashboard.error =
      e && typeof e === "object" && "message" in e
        ? String((e as { message: string }).message)
        : String(e);
    return;
  } finally {
    dashboard.loading = false;
  }

  // Gap-fill candles from Kraken in background, then refresh
  dashboard.syncing = true;
  syncCandles()
    .then(() => getCandles())
    .then((fresh) => {
      const last = fresh[fresh.length - 1];
      console.log(`sync done: ${fresh.length} candles, last: ${last?.date}`);
      dashboard.candles = fresh;
    })
    .catch((e) => console.error("sync_candles failed:", e))
    .finally(() => {
      dashboard.syncing = false;
    });
}

/** Re-sync prices only (no trades/summary/bepSnaps reload). */
export async function refreshDashboard(): Promise<void> {
  dashboard.syncing = true;
  try {
    await syncCandles();
    dashboard.candles = await getCandles();
  } catch (e) {
    console.error("refresh failed:", e);
  } finally {
    dashboard.syncing = false;
  }
}

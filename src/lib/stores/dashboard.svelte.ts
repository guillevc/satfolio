import { getDashboardStats, syncCandles } from "$lib/api";
import type { DashboardStats } from "$lib/types/bindings";

export const dashboard = $state({
  stats: null as DashboardStats | null,
  loading: false,
  syncing: false,
  error: null as string | null,
});

export async function loadDashboard(): Promise<void> {
  dashboard.error = null;
  dashboard.loading = true;
  try {
    dashboard.stats = await getDashboardStats();
  } catch (e) {
    dashboard.error =
      e && typeof e === "object" && "message" in e
        ? String((e as { message: string }).message)
        : String(e);
    return;
  } finally {
    dashboard.loading = false;
  }

  // Gap-fill candles from Kraken in background, then refresh stats
  dashboard.syncing = true;
  syncCandles()
    .then(() => getDashboardStats())
    .then((fresh) => {
      const last = fresh.candles[fresh.candles.length - 1];
      console.log(
        `sync done: ${fresh.candles.length} candles, last: ${last?.date}`,
      );
      dashboard.stats = fresh;
    })
    .catch((e) => console.error("sync_candles failed:", e))
    .finally(() => {
      dashboard.syncing = false;
    });
}

/** Re-sync prices and stats. */
export async function refreshDashboard(): Promise<void> {
  dashboard.syncing = true;
  try {
    await syncCandles();
    dashboard.stats = await getDashboardStats();
  } catch (e) {
    console.error("refresh failed:", e);
  } finally {
    dashboard.syncing = false;
  }
}

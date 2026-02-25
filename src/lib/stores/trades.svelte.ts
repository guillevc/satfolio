import { loadSample, getTrades } from "$lib/api";
import type { EnrichedTrade } from "$lib/types/bindings";

export const trades = $state({
  rows: null as EnrichedTrade[] | null,
  loading: false,
  error: null as string | null,
});

export async function loadTrades(): Promise<void> {
  trades.loading = true;
  trades.error = null;
  try {
    await loadSample();
    trades.rows = await getTrades();
  } catch (e) {
    trades.error = String(e);
  } finally {
    trades.loading = false;
  }
}

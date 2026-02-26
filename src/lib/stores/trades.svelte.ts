import { loadSample, getTrades } from "$lib/api";
import type { EnrichedTrade } from "$lib/types/bindings";
import { tick } from "svelte";

export const trades = $state({
  rows: null as EnrichedTrade[] | null,
  loading: false,
  error: null as string | null,
});

export async function loadTrades(): Promise<void> {
  trades.loading = true;
  trades.error = null;
  await tick();
  try {
    const [, rows] = await Promise.all([loadSample(), getTrades()]);
    trades.rows = rows;
  } catch (e) {
    trades.error = String(e);
  } finally {
    trades.loading = false;
  }
}

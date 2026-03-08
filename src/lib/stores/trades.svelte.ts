import { trades as fetchTrades } from "$lib/api";
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
    trades.rows = await fetchTrades();
  } catch (e) {
    trades.error =
      e && typeof e === "object" && "message" in e
        ? String((e as { message: string }).message)
        : String(e);
  } finally {
    trades.loading = false;
  }
}

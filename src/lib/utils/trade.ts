import type { EnrichedTrade } from "$lib/types/bindings";

export const QUOTE_ASSET = "EUR";
export const FIAT_ASSETS = new Set(["EUR", "USD", "GBP"]);

export function isBuy(t: EnrichedTrade): boolean {
  return t.spent.asset === QUOTE_ASSET;
}

export function baseAmount(t: EnrichedTrade): string {
  return isBuy(t) ? t.received.amount : t.spent.amount;
}

export function quoteAmount(t: EnrichedTrade): string {
  return isBuy(t) ? t.spent.amount : t.received.amount;
}

export function pricePerUnit(t: EnrichedTrade): number {
  const units = parseFloat(baseAmount(t));
  if (units === 0) return 0;
  return parseFloat(quoteAmount(t)) / units;
}

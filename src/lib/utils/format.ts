import type { AssetAmount } from "$lib/types/bindings";

/** Convert AssetAmount decimal string to number for display formatting only.
 *  All financial math stays in Rust. */
export function displayAmount(a: AssetAmount): number {
  return parseFloat(a.amount);
}

// ── Intl.NumberFormat cache ─────────────────────────────────

const fmtCache = new Map<string, Intl.NumberFormat>();

function getCurrencyFmt(currency: string, decimals: number): Intl.NumberFormat {
  const key = `${currency}:${decimals}`;
  let fmt = fmtCache.get(key);
  if (!fmt) {
    fmt = new Intl.NumberFormat("en-US", {
      style: "currency",
      currency,
      minimumFractionDigits: decimals,
      maximumFractionDigits: decimals,
    });
    fmtCache.set(key, fmt);
  }
  return fmt;
}

// ── Public API ──────────────────────────────────────────────

/** Format a number as currency (e.g. "$84,210" or "€1,234.56"). */
export function formatCurrency(
  value: number,
  currency: string,
  decimals = 0,
): string {
  return getCurrencyFmt(currency, decimals).format(value);
}

/** Format a number as signed currency (e.g. "+$1,200" / "-$350"). */
export function formatSignedCurrency(
  value: number,
  currency: string,
  decimals = 0,
): string {
  const formatted = getCurrencyFmt(currency, decimals).format(Math.abs(value));
  return value >= 0 ? `+${formatted}` : `-${formatted}`;
}

import type { AssetAmount } from "$lib/types/bindings";
import { formattingLocale } from "$lib/utils/locale";

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
    fmt = new Intl.NumberFormat(formattingLocale, {
      style: "currency",
      currency,
      currencyDisplay: "narrowSymbol",
      minimumFractionDigits: decimals,
      maximumFractionDigits: decimals,
    });
    fmtCache.set(key, fmt);
  }
  return fmt;
}

function getBtcFmt(): Intl.NumberFormat {
  let fmt = fmtCache.get("BTC");
  if (!fmt) {
    fmt = new Intl.NumberFormat(formattingLocale, {
      minimumFractionDigits: 0,
      maximumFractionDigits: 8,
    });
    fmtCache.set("BTC", fmt);
  }
  return fmt;
}

function getDecimalFmt(decimals: number): Intl.NumberFormat {
  const key = `dec:${decimals}`;
  let fmt = fmtCache.get(key);
  if (!fmt) {
    fmt = new Intl.NumberFormat(formattingLocale, {
      minimumFractionDigits: decimals,
      maximumFractionDigits: decimals,
    });
    fmtCache.set(key, fmt);
  }
  return fmt;
}

// ── Public API ──────────────────────────────────────────────

/** Locale-aware number with exactly N fraction digits (e.g. "1.234,567800"). */
export function formatDecimal(value: number, decimals: number): string {
  return getDecimalFmt(decimals).format(value);
}

/** Format a BTC amount with locale-aware grouping (e.g. "1.234,5678 BTC"). */
export function formatBtc(value: number): string {
  return `${getBtcFmt().format(value)} BTC`;
}

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

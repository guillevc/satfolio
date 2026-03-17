import { describe, expect, test } from "vitest";
import { isBuy, baseAmount, pricePerUnit } from "./trade";
import type { EnrichedTrade, AssetAmount } from "$lib/types/bindings";

// ── Helpers ──────────────────────────────────────────────────

function amt(amount: string, asset: string): AssetAmount {
  return { amount, asset };
}

function trade(overrides: Partial<EnrichedTrade> = {}): EnrichedTrade {
  return {
    date: "2025-01-15T10:30:00Z",
    provider: "kraken",
    spent: amt("1000.00", "EUR"),
    received: amt("0.012345", "BTC"),
    fee: amt("1.50", "EUR"),
    side: "Buy",
    bep: null,
    pnl: null,
    ...overrides,
  };
}

// ── isBuy ────────────────────────────────────────────────────

describe("isBuy", () => {
  test("returns true when side is Buy", () => {
    expect(isBuy(trade())).toBe(true);
  });

  test("returns false when side is Sell", () => {
    const sell = trade({
      side: "Sell",
      spent: amt("0.5", "BTC"),
      received: amt("45000.00", "EUR"),
    });
    expect(isBuy(sell)).toBe(false);
  });

  test("returns false when side is null", () => {
    expect(isBuy(trade({ side: null }))).toBe(false);
  });
});

// ── baseAmount ───────────────────────────────────────────────

describe("baseAmount", () => {
  test("returns received amount for buys (BTC received)", () => {
    expect(baseAmount(trade())).toBe("0.012345");
  });

  test("returns spent amount for sells (BTC spent)", () => {
    const sell = trade({
      side: "Sell",
      spent: amt("0.5", "BTC"),
      received: amt("45000.00", "EUR"),
    });
    expect(baseAmount(sell)).toBe("0.5");
  });
});

// ── pricePerUnit ─────────────────────────────────────────────

describe("pricePerUnit", () => {
  test("computes EUR spent / BTC received for a buy", () => {
    const t = trade({
      spent: amt("1000.00", "EUR"),
      received: amt("0.02", "BTC"),
    });
    expect(pricePerUnit(t)).toBeCloseTo(50_000, 2);
  });

  test("computes EUR received / BTC spent for a sell", () => {
    const t = trade({
      side: "Sell",
      spent: amt("0.02", "BTC"),
      received: amt("1200.00", "EUR"),
    });
    expect(pricePerUnit(t)).toBeCloseTo(60_000, 2);
  });

  test("returns 0 when base amount is zero", () => {
    const t = trade({
      spent: amt("0", "EUR"),
      received: amt("0", "BTC"),
    });
    expect(pricePerUnit(t)).toBe(0);
  });
});

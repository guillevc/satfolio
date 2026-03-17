import { describe, expect, test, vi } from "vitest";

vi.mock("$lib/utils/locale", () => ({ systemLocale: "en-US" }));

import {
  displayAmount,
  formatBtc,
  formatCurrency,
  formatDecimal,
  formatPercent,
  formatSignedCurrency,
} from "./format";

// ── displayAmount ───────────────────────────────────────────

describe("displayAmount", () => {
  test("parses a decimal string to number", () => {
    expect(displayAmount({ amount: "1234.56", asset: "EUR" })).toBe(1234.56);
  });

  test("handles integer amounts", () => {
    expect(displayAmount({ amount: "42", asset: "BTC" })).toBe(42);
  });

  test("handles very small decimals", () => {
    expect(displayAmount({ amount: "0.00000001", asset: "BTC" })).toBe(1e-8);
  });

  test("handles zero", () => {
    expect(displayAmount({ amount: "0", asset: "EUR" })).toBe(0);
  });
});

// ── formatBtc ──────────────────────────────────────────────

describe("formatBtc", () => {
  test("formats with locale grouping", () => {
    expect(formatBtc(1234.5678)).toBe("1,234.5678 BTC");
  });

  test("trims trailing zeros", () => {
    expect(formatBtc(1)).toBe("1 BTC");
  });

  test("preserves up to 8 decimals", () => {
    expect(formatBtc(0.00000001)).toBe("0.00000001 BTC");
  });

  test("formats zero", () => {
    expect(formatBtc(0)).toBe("0 BTC");
  });
});

// ── formatDecimal ──────────────────────────────────────────

describe("formatDecimal", () => {
  test("formats with exact fraction digits", () => {
    expect(formatDecimal(0.1, 6)).toBe("0.100000");
  });

  test("uses locale grouping", () => {
    expect(formatDecimal(1234.5, 2)).toBe("1,234.50");
  });
});

// ── formatPercent ──────────────────────────────────────────

describe("formatPercent", () => {
  test("formats a 0–1 fraction as percent", () => {
    expect(formatPercent(0.293, 2)).toBe("29.30%");
  });

  test("formats negative fractions", () => {
    expect(formatPercent(-0.05, 1)).toBe("-5.0%");
  });

  test("formats zero", () => {
    expect(formatPercent(0, 1)).toBe("0.0%");
  });
});

// ── formatCurrency ──────────────────────────────────────────

describe("formatCurrency", () => {
  test("formats USD with no decimals by default", () => {
    expect(formatCurrency(84210, "USD")).toBe("$84,210");
  });

  test("formats USD with 2 decimals", () => {
    expect(formatCurrency(1234.5, "USD", 2)).toBe("$1,234.50");
  });

  test("formats EUR", () => {
    expect(formatCurrency(1000, "EUR")).toContain("1,000");
  });

  test("formats zero", () => {
    expect(formatCurrency(0, "USD")).toBe("$0");
  });

  test("formats negative values", () => {
    expect(formatCurrency(-500, "USD")).toBe("-$500");
  });
});

// ── formatSignedCurrency ────────────────────────────────────

describe("formatSignedCurrency", () => {
  test("prepends + for positive values", () => {
    expect(formatSignedCurrency(1200, "USD")).toBe("+$1,200");
  });

  test("prepends - for negative values", () => {
    expect(formatSignedCurrency(-350, "USD")).toBe("-$350");
  });

  test("treats zero as non-negative (+)", () => {
    expect(formatSignedCurrency(0, "USD")).toBe("+$0");
  });

  test("respects decimals parameter", () => {
    expect(formatSignedCurrency(99.5, "USD", 2)).toBe("+$99.50");
  });
});

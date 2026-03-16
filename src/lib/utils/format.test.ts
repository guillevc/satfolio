import { describe, expect, test } from "vitest";
import { displayAmount, formatCurrency, formatSignedCurrency } from "./format";

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

import { describe, expect, test } from "vitest";
import { displayAmount } from "./utils";

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

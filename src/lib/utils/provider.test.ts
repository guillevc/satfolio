import { describe, expect, test } from "vitest";
import { providerMeta } from "./provider";

describe("providerMeta", () => {
  test("has entry for kraken", () => {
    expect(providerMeta.kraken).toEqual({
      label: "Kraken",
      initial: "K",
      classes: expect.any(String),
    });
  });

  test("has entry for coinbase", () => {
    expect(providerMeta.coinbase).toEqual({
      label: "Coinbase",
      initial: "C",
      classes: expect.any(String),
    });
  });

  test("all entries have required fields", () => {
    for (const [key, meta] of Object.entries(providerMeta)) {
      expect(meta, `${key} missing label`).toHaveProperty("label");
      expect(meta, `${key} missing initial`).toHaveProperty("initial");
      expect(meta, `${key} missing classes`).toHaveProperty("classes");
      expect(meta.initial).toHaveLength(1);
    }
  });
});

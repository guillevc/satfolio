import { describe, expect, test } from "vitest";
import { formatDate } from "./columns";

// ── formatDate ───────────────────────────────────────────────

describe("formatDate", () => {
  test("splits ISO string into date and time", () => {
    const { date, time } = formatDate("2025-01-15T10:30:00Z");
    expect(date).toBe("2025-01-15");
    expect(time).toMatch(/^\d{2}:\d{2}$/);
  });

  test("handles midnight correctly", () => {
    const { time } = formatDate("2025-06-01T00:00:00Z");
    expect(time).toMatch(/^\d{2}:\d{2}$/);
  });
});

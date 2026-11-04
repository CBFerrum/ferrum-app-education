import { describe, expect, it } from "vitest";
import { todayIso } from "./Home";

describe("today", () => {
  it("is the local date, zero-padded", () => {
    expect(todayIso(new Date(2026, 0, 5, 23, 30))).toBe("2026-01-05");
  });
});

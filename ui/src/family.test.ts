import { describe, expect, it } from "vitest";
import { statusTone } from "@ferrum/kit";
import { APPLICATION_TONES } from "./family/Apply";
import { attended, statusLink } from "./family/api";

describe("the family's pages", () => {
  it("counts the late as present, and says nothing before any class", () => {
    expect(attended({ present: 1, absent: 1, late: 1, excused: 1 })).toBe(50);
    expect(attended({ present: 2, absent: 1, late: 0, excused: 0 })).toBe(66.67);
    expect(attended({ present: 0, absent: 0, late: 0, excused: 0 })).toBeNull();
    expect(attended(null)).toBeNull();
  });

  it("keeps an application's link under the application page", () => {
    expect(statusLink("hill school", "a/b")).toBe("/apply/hill%20school/status/a%2Fb");
  });

  it("tones an application by where it stands, and a state it lacks as neutral", () => {
    expect(statusTone("Admitted", APPLICATION_TONES)).toBe("green");
    expect(statusTone("Rejected", APPLICATION_TONES)).toBe("red");
    expect(statusTone("Applied", APPLICATION_TONES)).toBe("amber");
    expect(statusTone("Something new", APPLICATION_TONES)).toBe("neutral");
  });
});

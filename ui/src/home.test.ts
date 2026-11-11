import { describe, expect, it } from "vitest";
import { classLine, classRow } from "./Home";
import { rollChanges } from "./Roll";

describe("the school office", () => {
  it("names a class by its time, course, group, room and instructor", () => {
    expect(classLine({ id: "c", from_time: "09:00", to_time: "10:00", _titles: { course: "Mathematics", student_group: "6A", room: "Room 101", instructor: "Mr Weiss" } })).toBe(
      "09:00 to 10:00 · Mathematics · 6A · Room 101 · Mr Weiss",
    );
    expect(classLine({ id: "c", from_time: "09:00", to_time: "10:00" })).toBe("09:00 to 10:00");
  });

  it("saves a roll call's new and changed marks, and leaves the rest", () => {
    const kept = new Map([
      ["lea", { id: "a1", status: "Present" }],
      ["tom", { id: "a2", status: "Present" }],
    ]);
    const chosen = new Map([
      ["tom", "Late"],
      ["ana", "Absent"],
    ]);
    expect(rollChanges(["lea", "tom", "ana", "ben"], kept, chosen)).toEqual([
      { student: "tom", id: "a2", status: "Late" },
      { student: "ana", id: undefined, status: "Absent" },
      { student: "ben", id: undefined, status: "Present" },
    ]);
  });

  it("leads a class's row with its start time, and keeps the rest beneath", () => {
    expect(classRow({ id: "c", from_time: "09:00", to_time: "10:00", _titles: { course: "Mathematics", student_group: "6A", room: "Room 101", instructor: "Mr Weiss" } })).toEqual({ lead: "09:00", title: "Mathematics", meta: "09:00–10:00 · 6A · Room 101 · Mr Weiss" });
    expect(classRow({ id: "c", from_time: "09:00", to_time: "10:00" })).toEqual({ lead: "09:00", title: "", meta: "09:00–10:00" });
  });
});

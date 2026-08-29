import { describe, expect, it } from "vitest";

import { inQuietHours, type SoundPrefs, soundAllowed } from "./sound";

/** A local-time moment, built from the parts the functions actually read. */
function at(hour: number, minute = 30): Date {
  return new Date(2026, 7, 29, hour, minute, 0);
}

const LOUD: SoundPrefs = { muted: false, quietHours: false };
const DEFAULTS: SoundPrefs = { muted: false, quietHours: true };

describe("inQuietHours", () => {
  it("covers 22:00 through to 08:00, across midnight", () => {
    expect(inQuietHours(at(22))).toBe(true);
    expect(inQuietHours(at(23))).toBe(true);
    expect(inQuietHours(at(0))).toBe(true);
    expect(inQuietHours(at(3))).toBe(true);
    expect(inQuietHours(at(7))).toBe(true);
  });

  it("leaves the waking day alone", () => {
    expect(inQuietHours(at(8))).toBe(false);
    expect(inQuietHours(at(12))).toBe(false);
    expect(inQuietHours(at(21))).toBe(false);
  });

  it("turns exactly on the hour, both ends", () => {
    expect(inQuietHours(at(21, 59))).toBe(false);
    expect(inQuietHours(at(22, 0))).toBe(true);
    expect(inQuietHours(at(7, 59))).toBe(true);
    expect(inQuietHours(at(8, 0))).toBe(false);
  });
});

describe("soundAllowed", () => {
  it("lets a sound through in the middle of the day", () => {
    expect(soundAllowed(DEFAULTS, at(14))).toBe(true);
  });

  it("holds it at 3am, because quiet hours are on by default", () => {
    expect(soundAllowed(DEFAULTS, at(3))).toBe(false);
  });

  it("plays at 3am for somebody who turned quiet hours off", () => {
    expect(soundAllowed(LOUD, at(3))).toBe(true);
  });

  it("mute wins over everything, at any hour", () => {
    expect(soundAllowed({ muted: true, quietHours: false }, at(14))).toBe(false);
    expect(soundAllowed({ muted: true, quietHours: true }, at(3))).toBe(false);
  });
});

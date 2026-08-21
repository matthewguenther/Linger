/**
 * Who is in a room, as names and as a list of people.
 *
 * Occupancy is a list of people, never a number (SPEC §4.1, §4.2). The
 * header writes `#garage · Matt, Callie`; the rail draws a small stack of
 * the same people. Both read from here so they cannot disagree, and both
 * sort by name so a DashMap's iteration order cannot shuffle the line
 * while you look at it.
 */
import type { PresenceEntry } from "../generated/PresenceEntry";
import type { RoomId } from "../generated/RoomId";
import type { User } from "../generated/User";
import type { UserId } from "../generated/UserId";

/**
 * The people in a room, by display name.
 *
 * Takes both the occupancy map and the presence list because they are two
 * views of the same fact, arriving a frame apart. Either one is enough;
 * together they cannot drop somebody the other one still holds.
 */
export function occupantsOf(
  roomId: RoomId,
  occupancy: Readonly<Record<string, UserId[]>>,
  presence: readonly PresenceEntry[],
  users: readonly User[],
): User[] {
  const ids = new Set<string>(occupancy[roomId] ?? []);
  for (const entry of presence) {
    if (entry.room_id === roomId) ids.add(entry.user_id);
  }
  if (ids.size === 0) return [];
  return users
    .filter((person) => ids.has(person.id))
    .sort((a, b) =>
      a.display_name.localeCompare(b.display_name, undefined, { sensitivity: "base" }),
    );
}

/**
 * The occupancy clause of a room header: `Matt, Callie`. Empty when nobody
 * is in the room, so the header can stay just `#garage`.
 *
 * Commas, not "and". SPEC §4.1 draws `#garage · Matt, Callie`, and "and"
 * would make two people sound like a sentence rather than a list. A long
 * list is the header's problem to ellipsis, not this function's to count.
 */
export function occupancyLine(people: readonly User[]): string {
  if (people.length === 0) return "";
  return people.map((person) => person.display_name).join(", ");
}

/** How many faces the rail stack will actually draw. The rest live in the label. */
export const STACK_VISIBLE = 5;

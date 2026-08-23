/**
 * The sentences a new member hits before they know how the app works.
 *
 * T-411's empty-state job is mostly copy, not new screens. These live in one
 * place so a dead-end rewrite cannot drift from the tests that pin it.
 */

/** The stream column when this server has no rooms, or is still connecting. */
export function noRoomsBody(ready: boolean, isHost: boolean): string {
  if (!ready) return "Connecting to the server…";
  if (isHost) return "This server has no rooms yet.";
  return "This server has no rooms yet. The host has to make the first one — you'll see it here when they do.";
}

/** The rail under "rooms" when the list is empty. A dash said nothing. */
export function noRoomsRail(): string {
  return "no rooms yet";
}

/**
 * The roster when it has no cards. After `ready` you are on the list yourself,
 * so an empty stack here is "the people have not arrived yet", not "you are
 * missing". Before `ready` it is just the connection catching up.
 */
export function emptyRoster(ready: boolean): string {
  return ready ? "nobody here yet" : "finding who's around…";
}

/** A room you can stand in that nobody has spoken in. */
export function emptyRoom(): string {
  return "Nothing here yet. Type below to say the first thing.";
}

/**
 * The one thing in Linger allowed to interrupt you.
 *
 * SPEC §4.2 deletes the badge and replaces it with weight and a line in the
 * stream. The single exception is a message that names you, or one from
 * somebody you have explicitly asked to hear from — and that exception is
 * this file.
 *
 * Three rules shape it.
 *
 * **Nothing fires for something you are already looking at.** A notification
 * about a message on your screen, in the room you are reading, is noise.
 *
 * **Messages are batched per room.** A resume replays everything that happened
 * while the socket was down, which can be a burst; a burst must be one
 * notification, not thirty. The batch also means a conversation that mentions
 * you twice while you are making tea rings once.
 *
 * **No numbers, ever.** The notification names people and quotes the last
 * thing said. It never says how many messages are waiting, because that is the
 * badge wearing a different coat.
 */
import { isTauri } from "@tauri-apps/api/core";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

import type { RoomId } from "../generated/RoomId";
import type { ServerFrame } from "../generated/ServerFrame";
import type { GatewayState } from "../lib/gateway";
import { plainText } from "../stream/markdown";
import { notificationText, notifyReason } from "./rules";

/**
 * How long a room's messages are collected before one notification goes out.
 * Long enough to swallow a resume's replay, short enough that a message
 * arriving while you are away from the desk still feels prompt.
 */
const BATCH_MS = 1_200;

/** The room on screen. Set by the frame that is drawing it. */
let viewing: RoomId | null = null;

/** Tell the notifier which room you are looking at, if any. */
export function setViewing(roomId: RoomId | null): void {
  viewing = roomId;
}

interface Batch {
  slug: string;
  /** Distinct, in the order they first spoke. */
  names: string[];
  /** The most recent thing said, as plain words. */
  excerpt: string;
}

const batches = new Map<string, Batch>();
let flushIn: number | null = null;

/**
 * Look at one frame and decide whether it is worth saying something about.
 *
 * Takes the snapshot rather than reading the store, so the decision is made
 * against the state that includes this very frame and nothing later.
 */
export function considerFrame(frame: ServerFrame, snapshot: GatewayState): void {
  if (frame.op !== "message.create") return;
  const me = snapshot.me;
  if (me === null) return;

  const message = frame.d;
  if (notifyReason(message, me, snapshot.notifyRules) === null) return;
  // You are looking right at it.
  if (viewing === message.room_id && focused()) return;

  const slug = snapshot.rooms.find((room) => room.id === message.room_id)?.slug ?? "a room";
  const name =
    snapshot.users.find((person) => person.id === message.author_id)?.display_name ?? "someone";

  const held = batches.get(message.room_id);
  if (held === undefined) {
    batches.set(message.room_id, { slug, names: [name], excerpt: plainText(message.body) });
  } else {
    if (!held.names.includes(name)) held.names.push(name);
    held.excerpt = plainText(message.body);
  }
  if (flushIn === null) flushIn = window.setTimeout(flush, BATCH_MS);
}

function flush(): void {
  flushIn = null;
  const pending = [...batches.values()];
  batches.clear();
  for (const batch of pending) {
    const { title, body } = notificationText(batch.slug, batch.names, batch.excerpt);
    void show(title, body);
  }
}

/**
 * Whether the window has the user's attention. Not `document.visibilityState`:
 * a window can be fully visible on a second monitor while you are typing
 * somewhere else, and in that case you have not read anything.
 */
function focused(): boolean {
  return typeof document !== "undefined" && document.hasFocus();
}

/**
 * Asked for once and remembered. `null` means we have not asked yet.
 *
 * A refusal is final and silent: somebody who turned notifications off at the
 * OS level has already said what they want, and asking again every time
 * somebody says their name would be exactly the pestering this app exists to
 * not do.
 */
let allowed: boolean | null = null;

async function show(title: string, body: string): Promise<void> {
  if (!isTauri()) return;
  try {
    if (allowed === null) {
      allowed = (await isPermissionGranted()) || (await requestPermission()) === "granted";
    }
    if (!allowed) return;
    sendNotification({ title, body });
  } catch {
    // No notification daemon, a sandbox with no portal, a headless session.
    // The stream still shows the message; there is nothing to tell anyone.
  }
}

/** Drop everything pending. Used when the account changes. */
export function resetNotifications(): void {
  if (flushIn !== null) window.clearTimeout(flushIn);
  flushIn = null;
  batches.clear();
  viewing = null;
}

/**
 * Opening a link somebody put in a message.
 *
 * It goes to the system browser, never to this window. A WebView that navigates
 * itself to a URL out of a chat message has replaced the application with a
 * website, taking the signed-in session with it — so the app never follows a
 * link, it hands it over.
 *
 * The URL has already been through `safeHref` in `stream/markdown.ts`, which is
 * how it became a link at all, and the Tauri capability in
 * `src-tauri/capabilities/default.json` narrows the plugin to http and https on
 * the Rust side. Two locks on the door, because the input is a message body and
 * message bodies are hostile (ARCHITECTURE §7).
 */
import { isTauri } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";

export function openExternal(href: string): void {
  if (isTauri()) {
    // A refusal here is the capability doing its job, or a desktop with no
    // browser registered. Neither is worth a dialog over a link.
    void openUrl(href).catch(() => undefined);
    return;
  }
  // `pnpm dev` in a plain browser, where there is no shell to hand it to.
  window.open(href, "_blank", "noreferrer");
}

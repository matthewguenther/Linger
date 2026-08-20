/**
 * Sending a link somewhere that isn't the app.
 *
 * A WebView that follows an `href` replaces Linger with a web page, and there
 * is no back button to come home with. So every link in a message body has its
 * default prevented and comes through here, which hands it to the operating
 * system instead.
 *
 * The scheme is checked twice on purpose. `markdown.ts` refuses to build a link
 * node for anything outside the allowlist, `capabilities/default.json` narrows
 * what the Tauri command will accept, and this sits between them — because the
 * cost of a third check is one comparison and the cost of missing one is a
 * message body that can run code.
 */
import { isTauri } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";

const SAFE_SCHEMES = ["http:", "https:", "mailto:"];

/** Open a web or mail address outside the app. Anything else does nothing. */
export async function openExternal(raw: string): Promise<void> {
  let url: URL;
  try {
    url = new URL(raw);
  } catch {
    return;
  }
  if (!SAFE_SCHEMES.includes(url.protocol)) return;

  if (!isTauri()) {
    // `pnpm dev` in a plain browser: a new tab is the honest equivalent.
    window.open(url.toString(), "_blank", "noopener,noreferrer");
    return;
  }
  try {
    await openUrl(url.toString());
  } catch {
    // No browser configured, or the OS refused. There is nothing useful to say
    // and nothing to retry — the link is still there to copy.
  }
}

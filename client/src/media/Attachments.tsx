/**
 * Files, inside a message (SPEC §5.6, PROTOCOL §6).
 *
 * "Images render inline at true aspect ratio, capped at 400px height, click to
 * expand." Video gets its poster frame and a player; audio gets a player;
 * everything else gets one line and a way to save it. No cards, no thumbnails
 * in boxes, no drop shadows — a picture in a conversation is a picture.
 *
 * Two rules from the architecture show up here as code. The URLs are opaque and
 * absolute already: they point at the media origin, which is a different host
 * from the API on purpose (ARCHITECTURE §7), so nothing here builds one. And a
 * file that is not an image, a video or a sound is never rendered — it is
 * handed to the system browser to save, which is where a download belongs.
 */
import { useEffect, useState } from "react";

import type { Attachment } from "../generated/Attachment";
import { openExternal } from "../lib/external";
import { durationText, fileSize, inlineBox, renderAs } from "./media";
import "./media.css";

export default function Attachments({ files }: { files: Attachment[] }) {
  const [expanded, setExpanded] = useState<Attachment | null>(null);
  if (files.length === 0) return null;
  return (
    <div className="atts">
      {files.map((file) => (
        <One key={file.id} file={file} onExpand={() => setExpanded(file)} />
      ))}
      {expanded ? <Expanded file={expanded} onClose={() => setExpanded(null)} /> : null}
    </div>
  );
}

function One({ file, onExpand }: { file: Attachment; onExpand: () => void }) {
  switch (renderAs(file.mime)) {
    case "image": {
      // The box is set before the bytes arrive so the row is measured at its
      // real height once, rather than growing under everything below it when
      // the picture loads — the list is virtualized and that shifts the world.
      const box = inlineBox(file.width, file.height);
      return (
        <button type="button" className="att-image" onClick={onExpand} title="expand">
          <img
            src={file.url}
            alt={file.filename}
            width={box?.width}
            height={box?.height}
            loading="lazy"
            decoding="async"
          />
        </button>
      );
    }
    case "video":
      return (
        <video
          className="att-video"
          src={file.url}
          poster={file.poster_url ?? undefined}
          controls
          preload="metadata"
          aria-label={file.filename}
        />
      );
    case "audio":
      return (
        <div className="att-audio">
          <p className="att-line meta">
            <span className="att-name">{file.filename}</span>
            {file.duration_ms === null ? null : (
              <span className="att-size">{durationText(Number(file.duration_ms))}</span>
            )}
          </p>
          <audio src={file.url} controls preload="metadata" aria-label={file.filename} />
        </div>
      );
    default:
      return (
        <p className="att-line meta">
          <span className="att-name">{file.filename}</span>
          <span className="att-size">{fileSize(Number(file.size_bytes))}</span>
          {/* Handed to the system browser, never followed in this window: the
              server sends it as an attachment with `nosniff`, and a webview
              that navigates itself to somebody's upload has replaced the app
              with it (ARCHITECTURE §7, `lib/external.ts`). */}
          <button type="button" className="att-get" onClick={() => openExternal(file.url)}>
            save
          </button>
        </p>
      );
  }
}

/**
 * The expanded picture. Escape closes it, so does clicking anywhere — there is
 * nothing else on this layer and nothing to aim at.
 */
function Expanded({ file, onClose }: { file: Attachment; onClose: () => void }) {
  useEffect(() => {
    const onKey = (event: KeyboardEvent): void => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    <div
      className="att-expanded"
      role="dialog"
      aria-modal="true"
      aria-label={file.filename}
      onClick={onClose}
    >
      <img src={file.url} alt={file.filename} />
      <p className="att-expanded-name meta">{file.filename}</p>
    </div>
  );
}

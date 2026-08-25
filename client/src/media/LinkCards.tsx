/**
 * The restrained embed (SPEC §5.6).
 *
 * "A link renders as a one-line inline card: favicon, title, domain. Not a
 * 400px billboard." One line, and when the server could not find a title, one
 * line with a domain on it — which is still more than the raw URL said.
 *
 * The favicon is a `data:` URI that came down with the card. It is never a
 * remote address, because an `<img>` pointed at a linked site would tell that
 * site who had scrolled past the message (PROTOCOL §6, `lib/previews.ts`).
 */
import type { AuthedApi } from "../lib/api";
import { openExternal } from "../lib/external";
import { useCards } from "../lib/previews";
import "./media.css";

export default function LinkCards({ api, urls }: { api: AuthedApi; urls: string[] }) {
  const cards = useCards(api, urls);
  if (urls.length === 0) return null;
  return (
    <div className="cards">
      {urls.map((url) => {
        const card = cards[url];
        // Until the answer arrives there is nothing worth drawing: a row that
        // grows a card a moment later is better than one that grows a
        // placeholder and then a card.
        if (card === undefined) return null;
        return (
          <button
            key={url}
            type="button"
            className="card"
            onClick={() => openExternal(url)}
            title={url}
          >
            {card.icon === null ? (
              <span className="card-icon card-icon-blank" aria-hidden="true" />
            ) : (
              <img className="card-icon" src={card.icon} alt="" width={14} height={14} />
            )}
            {card.title === null ? null : <span className="card-title">{card.title}</span>}
            <span className="card-domain meta">{card.domain}</span>
          </button>
        );
      })}
    </div>
  );
}

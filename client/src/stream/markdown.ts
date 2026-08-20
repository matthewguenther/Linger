/**
 * Markdown for message bodies.
 *
 * The whole point of this file is what it *doesn't* produce. It turns a body
 * into a tree of typed nodes — never an HTML string — and the renderer turns
 * those nodes into React elements. There is no `dangerouslySetInnerHTML`
 * anywhere in the client, so raw HTML in a message is not sanitized so much as
 * never possible in the first place: `<img onerror=…>` matches no rule here, so
 * it falls through to a text node and renders as the characters somebody typed
 * (ARCHITECTURE §7, "allowlist-based sanitizer, no raw HTML passthrough, ever").
 *
 * Link destinations are the one place a string from a message could still turn
 * into behavior, so they go through `safeHref`: parsed with `URL` and checked
 * against three schemes. Anything else is left as the literal text of what was
 * written, which keeps the attempt visible instead of quietly swallowing it.
 *
 * The grammar is deliberately small — the subset people actually type in chat:
 *
 *   **bold**  *italic*  _italic_  ~~struck~~  `code`  ```fenced```
 *   > quoted     - bulleted     1. numbered
 *   [label](https://example.com)   and bare https:// links
 *
 * No headings: sizes in Console are fixed by the density mode (SPEC §5.2), so a
 * line that silently became 24px type would be a hole in the design system. No
 * images or tables either — an image is an upload (M6), and a table is a
 * document, not a thing you say.
 */

export type Inline =
  | { kind: "text"; text: string }
  /** A code span. Its text is literal: nothing inside is parsed further. */
  | { kind: "code"; text: string }
  | { kind: "strong"; children: Inline[] }
  | { kind: "em"; children: Inline[] }
  | { kind: "strike"; children: Inline[] }
  /** `href` has already passed `safeHref`; it is safe to put in an anchor. */
  | { kind: "link"; href: string; children: Inline[] };

export type Block =
  | { kind: "paragraph"; children: Inline[] }
  | { kind: "quote"; children: Block[] }
  | { kind: "codeblock"; lang: string | null; text: string }
  | { kind: "list"; ordered: boolean; start: number; items: Inline[][] };

/**
 * The only schemes a link may carry. `javascript:` and `data:` are the two that
 * turn a link into code, and leaving them off a list is how they get in — so
 * this is an allowlist, and everything not on it renders as plain text.
 */
const SAFE_SCHEMES = ["http:", "https:", "mailto:"];

/** How deep emphasis, quotes and links may nest before we stop recursing. */
const MAX_DEPTH = 6;

/** Characters a backslash can escape, so `\*not italic\*` works. */
const ESCAPABLE = /[\\`*_~[\]()>#+\-.!]/;

/**
 * Turn a message body into blocks. Total: every string is valid markdown, so
 * this never throws and never returns null — the worst case is one paragraph
 * of exactly what was typed.
 */
export function parseMarkdown(source: string): Block[] {
  return parseBlocks(source.split("\n"), 0);
}

/**
 * A link destination we are willing to put in an `href`, or null.
 *
 * Parsing with `URL` rather than a regex is the load-bearing choice: it settles
 * case (`JaVaScRiPt:`), percent-encoding, and embedded control characters
 * before the scheme is compared, which is where hand-rolled checks go wrong.
 */
export function safeHref(destination: string): string | null {
  const text = destination.trim();
  if (text.length === 0) return null;
  let url: URL;
  try {
    url = new URL(text);
  } catch {
    // Relative and scheme-less destinations land here. There is nothing for a
    // message to link to inside the app, so there is nothing to resolve against.
    return null;
  }
  return SAFE_SCHEMES.includes(url.protocol) ? url.toString() : null;
}

/**
 * A body flattened to one line of plain text.
 *
 * This is for the quote above a reply, which has one line to say which message
 * is being answered. Markers come off rather than being rendered small, because
 * a one-line quote showing `**` is a worse clue than one showing the words.
 */
export function plainText(source: string): string {
  const inline = (nodes: Inline[]): string =>
    nodes
      .map((node) => (node.kind === "text" || node.kind === "code" ? node.text : inline(node.children)))
      .join("");
  const blocks = (list: Block[]): string =>
    list
      .map((block) => {
        switch (block.kind) {
          case "paragraph":
            return inline(block.children);
          case "quote":
            return blocks(block.children);
          case "codeblock":
            return block.text;
          case "list":
            return block.items.map(inline).join(" ");
        }
      })
      .join(" ");
  return blocks(parseMarkdown(source)).replace(/\s+/g, " ").trim();
}

// ---------------------------------------------------------------------------
// Blocks
// ---------------------------------------------------------------------------

const FENCE = /^ {0,3}(`{3,})\s*([A-Za-z0-9_+#.-]*)\s*$/;
const QUOTE = /^ {0,3}>\s?(.*)$/;
const BULLET = /^ {0,3}[-*+][ \t]+(.*)$/;
const ORDERED = /^ {0,3}(\d{1,9})[.)][ \t]+(.*)$/;

function opensBlock(line: string): boolean {
  return FENCE.test(line) || QUOTE.test(line) || BULLET.test(line) || ORDERED.test(line);
}

function parseBlocks(lines: string[], depth: number): Block[] {
  const blocks: Block[] = [];
  let at = 0;

  while (at < lines.length) {
    const line = lines[at] ?? "";

    if (line.trim().length === 0) {
      at += 1;
      continue;
    }

    const fence = FENCE.exec(line);
    if (fence) {
      const closer = new RegExp(`^ {0,3}\`{${(fence[1] ?? "```").length},}\\s*$`);
      const body: string[] = [];
      at += 1;
      while (at < lines.length && !closer.test(lines[at] ?? "")) {
        body.push(lines[at] ?? "");
        at += 1;
      }
      // An unclosed fence runs to the end of the message rather than being
      // rejected: somebody pasting code and hitting send is the common case.
      at += 1;
      const lang = fence[2] ?? "";
      blocks.push({ kind: "codeblock", lang: lang.length > 0 ? lang : null, text: body.join("\n") });
      continue;
    }

    if (QUOTE.test(line) && depth < MAX_DEPTH) {
      const quoted: string[] = [];
      while (at < lines.length) {
        const inner = QUOTE.exec(lines[at] ?? "");
        if (!inner) break;
        quoted.push(inner[1] ?? "");
        at += 1;
      }
      blocks.push({ kind: "quote", children: parseBlocks(quoted, depth + 1) });
      continue;
    }

    const listStart = BULLET.exec(line) ?? ORDERED.exec(line);
    if (listStart) {
      const ordered = BULLET.exec(line) === null;
      const start = ordered ? Number(ORDERED.exec(line)?.[1] ?? "1") : 1;
      const items: Inline[][] = [];
      while (at < lines.length) {
        const current = lines[at] ?? "";
        const bullet = BULLET.exec(current);
        const numbered = ORDERED.exec(current);
        // A list ends when the marker changes shape, so a bulleted list
        // followed by a numbered one reads as two lists, which is what the
        // person typing them meant.
        if (ordered ? numbered === null : bullet === null) break;
        const text = ordered ? (numbered?.[2] ?? "") : (bullet?.[1] ?? "");
        items.push(parseInline(text, depth + 1, true));
        at += 1;
      }
      blocks.push({ kind: "list", ordered, start, items });
      continue;
    }

    const paragraph: string[] = [];
    while (at < lines.length) {
      const current = lines[at] ?? "";
      if (current.trim().length === 0 || opensBlock(current)) break;
      paragraph.push(current);
      at += 1;
    }
    // Lines inside a paragraph keep their newlines. In a document a single
    // newline is a space; in a chat message it is a line the person chose to
    // break, and the renderer keeps the whitespace.
    blocks.push({ kind: "paragraph", children: parseInline(paragraph.join("\n"), depth, true) });
  }

  return blocks;
}

// ---------------------------------------------------------------------------
// Inline
// ---------------------------------------------------------------------------

interface Taken {
  node: Inline;
  end: number;
}

function isWordChar(char: string | undefined): boolean {
  return char !== undefined && /[\p{L}\p{N}]/u.test(char);
}

function runLength(source: string, at: number, char: string): number {
  let length = 0;
  while (source[at + length] === char) length += 1;
  return length;
}

/**
 * Parse a run of text into inline nodes.
 *
 * Order matters and is the security-relevant part: code spans are taken before
 * anything else, so `` `[x](javascript:…)` `` is literal text, and links are
 * taken before emphasis, so a destination can never be split in half by a `_`
 * in the middle of a URL.
 */
export function parseInline(source: string, depth: number, allowLinks: boolean): Inline[] {
  const nodes: Inline[] = [];
  let plain = "";
  let at = 0;
  /**
   * Delimiters already known to have nothing to close against, keyed by the
   * delimiter and its length.
   *
   * Whether a run *can* close depends only on what surrounds it, never on where
   * the opener was — so once a search from one position runs off the end of the
   * string, every later opener of the same shape runs off the end too.
   * Remembering that turns a line of `*a *a *a …` from one scan per asterisk
   * into one scan, which is the difference between a body somebody typed and a
   * body somebody sent to make the app think about it (ARCHITECTURE §7).
   */
  const hopeless = new Map<string, number>();

  const flush = (): void => {
    if (plain.length > 0) {
      nodes.push({ kind: "text", text: plain });
      plain = "";
    }
  };
  const take = (taken: Taken | null): boolean => {
    if (!taken) return false;
    flush();
    nodes.push(taken.node);
    at = taken.end;
    return true;
  };

  while (at < source.length) {
    const char = source[at] ?? "";

    if (char === "\\") {
      const next = source[at + 1];
      if (next !== undefined && ESCAPABLE.test(next)) {
        plain += next;
        at += 2;
        continue;
      }
    }

    if (char === "`" && take(codeSpanAt(source, at, hopeless))) continue;
    if (allowLinks && char === "[" && take(linkAt(source, at, depth))) continue;
    if (allowLinks && (char === "h" || char === "H") && take(autolinkAt(source, at))) continue;
    if (
      (char === "*" || char === "_" || char === "~") &&
      take(emphasisAt(source, at, depth, allowLinks, hopeless))
    )
      continue;

    plain += char;
    at += 1;
  }

  flush();
  return nodes;
}

/**
 * A code span, delimited by a run of backticks of matching length so that a
 * body containing a backtick can still be quoted: `` `a` `` is one span.
 */
function codeSpanAt(source: string, at: number, hopeless: Map<string, number>): Taken | null {
  const ticks = runLength(source, at, "`");
  const fence = "`".repeat(ticks);
  const from = at + ticks;

  // Same shortcut as emphasis, same reason: a run of this length that finds no
  // partner from here finds none from any later position either.
  const shape = `\`${ticks}`;
  const given = hopeless.get(shape);
  if (given !== undefined && from >= given) return null;

  let scan = from;
  while (scan < source.length) {
    if (source.startsWith(fence, scan) && runLength(source, scan, "`") === ticks) {
      const text = source.slice(from, scan);
      if (text.length === 0) return null;
      return {
        // One space either side is the fence's own padding, not content:
        // `` ` a ` `` is the way to write a span that starts with a backtick.
        node: { kind: "code", text: text.replace(/^ (.*) $/s, "$1") },
        end: scan + ticks,
      };
    }
    scan += 1;
  }
  hopeless.set(shape, from);
  return null;
}

/**
 * The longest a link's two halves may be before we stop looking.
 *
 * Unlike emphasis and code spans, "no closer from here" does not carry forward
 * for brackets — `[[a]` has no link starting at the first bracket and one
 * starting at the second — so the guard is a length instead. Nobody writes a
 * five-hundred-character link label, and a body that does is a body asking the
 * parser to scan to the end once per bracket.
 */
const MAX_LABEL = 512;
const MAX_DESTINATION = 2048;

/** `[label](destination)`, with a destination we are willing to link to. */
function linkAt(source: string, at: number, depth: number): Taken | null {
  if (depth >= MAX_DEPTH) return null;

  let scan = at + 1;
  let nesting = 1;
  const labelEnds = Math.min(source.length, at + 1 + MAX_LABEL);
  while (scan < labelEnds && nesting > 0) {
    const char = source[scan];
    if (char === "\\") {
      scan += 2;
      continue;
    }
    if (char === "[") nesting += 1;
    if (char === "]") nesting -= 1;
    if (nesting === 0) break;
    scan += 1;
  }
  if (nesting !== 0 || source[scan + 1] !== "(") return null;

  const label = source.slice(at + 1, scan);
  const open = scan + 2;
  let close = open;
  let parens = 1;
  const destinationEnds = Math.min(source.length, open + MAX_DESTINATION);
  while (close < destinationEnds && parens > 0) {
    const char = source[close];
    if (char === "\n") return null;
    if (char === "(") parens += 1;
    if (char === ")") parens -= 1;
    if (parens === 0) break;
    close += 1;
  }
  if (parens !== 0) return null;

  const raw = source.slice(open, close).trim();
  // `<…>` is markdown's way of writing a destination containing spaces.
  const destination = raw.startsWith("<") && raw.endsWith(">") ? raw.slice(1, -1) : raw;
  const href = safeHref(destination);
  if (href === null) return null;

  // Links do not nest, so the label is parsed with them switched off.
  const children = parseInline(label, depth + 1, false);
  if (children.length === 0) return null;
  return { node: { kind: "link", href, children }, end: close + 1 };
}

/** Trailing characters that are almost always sentence punctuation, not URL. */
const TRAILING = new Set([".", ",", ";", ":", "!", "?", "'", '"', "”", "’"]);

/** A bare `https://…` in the middle of a sentence. */
function autolinkAt(source: string, at: number): Taken | null {
  if (isWordChar(source[at - 1])) return null;
  const rest = source.slice(at);
  const scheme = /^https?:\/\/\S/i.exec(rest);
  if (!scheme) return null;

  let end = at;
  while (end < source.length && !/\s/.test(source[end] ?? "")) end += 1;

  // Walk back over punctuation the sentence owns rather than the URL. A closing
  // paren only comes off if it has no opener inside the link, so the Wikipedia
  // shape — /wiki/Thing_(disambiguation) — survives.
  while (end > at) {
    const last = source[end - 1] ?? "";
    if (TRAILING.has(last)) {
      end -= 1;
      continue;
    }
    if (last === ")") {
      const inside = source.slice(at, end);
      const opens = (inside.match(/\(/g) ?? []).length;
      const closes = (inside.match(/\)/g) ?? []).length;
      if (closes > opens) {
        end -= 1;
        continue;
      }
    }
    break;
  }

  const text = source.slice(at, end);
  const href = safeHref(text);
  if (href === null) return null;
  return { node: { kind: "link", href, children: [{ kind: "text", text }] }, end };
}

/**
 * `*em*`, `**strong**`, `***both***`, `~~struck~~`.
 *
 * Two rules keep ordinary prose out of it. A delimiter only opens if the
 * character after it isn't a space and only closes if the character before it
 * isn't — so `2 * 3 * 4` is arithmetic. And `_` additionally has to sit at a
 * word boundary, which is what leaves `snake_case_names` alone.
 */
function emphasisAt(
  source: string,
  at: number,
  depth: number,
  allowLinks: boolean,
  hopeless: Map<string, number>,
): Taken | null {
  if (depth >= MAX_DEPTH) return null;
  const char = source[at] ?? "";
  const run = Math.min(runLength(source, at, char), 3);
  if (char === "~" && run < 2) return null;
  if (char === "_" && isWordChar(source[at - 1])) return null;

  const from = at + run;
  const opener = source[from];
  if (opener === undefined || /\s/.test(opener)) return null;

  const shape = `${char}${run}`;
  const given = hopeless.get(shape);
  if (given !== undefined && from >= given) return null;

  let scan = from;
  while (scan < source.length) {
    if (source[scan] === "\\") {
      scan += 2;
      continue;
    }
    if (source[scan] === char && runLength(source, scan, char) >= run) {
      const before = source[scan - 1] ?? "";
      const after = source[scan + run];
      const closes =
        !/\s/.test(before) && scan > from && !(char === "_" && isWordChar(after));
      if (closes) {
        const children = parseInline(source.slice(from, scan), depth + 1, allowLinks);
        const wrapped: Inline =
          char === "~"
            ? { kind: "strike", children }
            : run === 1
              ? { kind: "em", children }
              : run === 2
                ? { kind: "strong", children }
                : { kind: "strong", children: [{ kind: "em", children }] };
        return { node: wrapped, end: scan + run };
      }
    }
    scan += 1;
  }
  hopeless.set(shape, from);
  return null;
}

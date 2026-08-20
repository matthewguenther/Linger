/**
 * Markdown parsing, and the part that matters more: what it refuses to build.
 *
 * The renderer only knows how to draw the node kinds this parser emits, and
 * none of them carry markup. So "does an XSS attempt render inert" is answered
 * here, at the parser, by checking that hostile input comes out as text nodes
 * and that no link ever carries a scheme outside the allowlist.
 */
import { describe, expect, it } from "vitest";

import { type Block, type Inline, parseMarkdown, safeHref } from "./markdown";

/** Every string in a tree of blocks, in order — what a reader would see. */
function textOf(blocks: Block[]): string {
  const inline = (nodes: Inline[]): string =>
    nodes
      .map((node) => {
        switch (node.kind) {
          case "text":
          case "code":
            return node.text;
          default:
            return inline(node.children);
        }
      })
      .join("");
  return blocks
    .map((block) => {
      switch (block.kind) {
        case "paragraph":
          return inline(block.children);
        case "quote":
          return textOf(block.children);
        case "codeblock":
          return block.text;
        case "list":
          return block.items.map(inline).join("\n");
      }
    })
    .join("\n");
}

/** Every link in a tree, however deeply nested. */
function linksOf(blocks: Block[]): string[] {
  const found: string[] = [];
  const walk = (nodes: Inline[]): void => {
    for (const node of nodes) {
      if (node.kind === "link") {
        found.push(node.href);
        walk(node.children);
      } else if (node.kind !== "text" && node.kind !== "code") {
        walk(node.children);
      }
    }
  };
  const blockWalk = (list: Block[]): void => {
    for (const block of list) {
      if (block.kind === "paragraph") walk(block.children);
      if (block.kind === "quote") blockWalk(block.children);
      if (block.kind === "list") block.items.forEach(walk);
    }
  };
  blockWalk(blocks);
  return found;
}

function firstParagraph(source: string): Inline[] {
  const block = parseMarkdown(source)[0];
  if (block?.kind !== "paragraph") throw new Error(`not a paragraph: ${block?.kind ?? "nothing"}`);
  return block.children;
}

describe("hostile input", () => {
  const attempts = [
    `<img src=x onerror="alert(1)">`,
    `<script>alert(document.cookie)</script>`,
    `<a href="javascript:alert(1)">click</a>`,
    `<svg/onload=alert(1)>`,
    `<iframe src="https://evil.example"></iframe>`,
    `<div onmouseover=alert(1)>hover</div>`,
    `<style>body{display:none}</style>`,
  ];

  for (const attempt of attempts) {
    it(`keeps ${attempt.slice(0, 24)}… as text`, () => {
      const blocks = parseMarkdown(attempt);
      // Nothing but text: no node kind here can carry markup, and the string
      // survives whole, so the reader sees exactly what was typed.
      expect(blocks).toEqual([{ kind: "paragraph", children: [{ kind: "text", text: attempt }] }]);
      expect(linksOf(blocks)).toEqual([]);
    });
  }

  it("refuses a javascript: destination and shows the source instead", () => {
    const blocks = parseMarkdown("[click me](javascript:alert(1))");
    expect(linksOf(blocks)).toEqual([]);
    expect(textOf(blocks)).toBe("[click me](javascript:alert(1))");
  });

  it("refuses schemes however they are spelled", () => {
    for (const destination of [
      "javascript:alert(1)",
      "JaVaScRiPt:alert(1)",
      "  javascript:alert(1)",
      "data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==",
      "vbscript:msgbox(1)",
      "file:///etc/passwd",
      "linger://join",
      "//evil.example/x",
      "/relative",
    ]) {
      expect(safeHref(destination)).toBeNull();
    }
  });

  it("allows the three schemes a message may link to", () => {
    expect(safeHref("https://example.com/a?b=c#d")).toBe("https://example.com/a?b=c#d");
    expect(safeHref("http://127.0.0.1:8080/")).toBe("http://127.0.0.1:8080/");
    expect(safeHref("mailto:someone@example.com")).toBe("mailto:someone@example.com");
  });

  it("does not read markup inside a code span", () => {
    expect(firstParagraph("`<img onerror=alert(1)>`")).toEqual([
      { kind: "code", text: "<img onerror=alert(1)>" },
    ]);
  });
});

describe("bodies built to be expensive", () => {
  // A delimiter with no closer means scanning to the end of the string, and
  // eight thousand of them in a row — the longest body the server accepts —
  // used to mean eight thousand of those scans. Measured on the machine this was
  // written on: 143ms before the shortcut in `parseInline`, 3.2ms after. The
  // budget sits between the two with room for a slower runner, because the
  // shape of the output is fine here and the cost is the whole bug.
  const budgetMs = 50;

  it("shrugs off a body that is all unclosed emphasis", () => {
    const body = "*a ".repeat(2600);
    const started = performance.now();
    parseMarkdown(body);
    expect(performance.now() - started).toBeLessThan(budgetMs);
  });

  it("shrugs off unclosed code spans and brackets too", () => {
    for (const body of [
      "`a ".repeat(2600),
      "[a ".repeat(2600),
      "_a ".repeat(2600),
      // Brackets that do have a partner, just never the right one.
      "[".repeat(3999) + "]",
      "[a](".repeat(2000),
    ]) {
      const started = performance.now();
      parseMarkdown(body);
      expect(performance.now() - started).toBeLessThan(budgetMs);
    }
  });

  it("still reads the emphasis in a body full of failed openers", () => {
    // Giving up on one delimiter must not give up on another: the shortcut is
    // keyed by the delimiter and its length, so twenty asterisks that never
    // close say nothing about the underscores after them.
    const nodes = firstParagraph(`${"*a\n".repeat(20)}_italic_`);
    expect(nodes[nodes.length - 1]).toEqual({
      kind: "em",
      children: [{ kind: "text", text: "italic" }],
    });
  });
});

describe("emphasis", () => {
  it("reads bold, italic, both, and struck", () => {
    expect(firstParagraph("**b**")).toEqual([
      { kind: "strong", children: [{ kind: "text", text: "b" }] },
    ]);
    expect(firstParagraph("*i*")).toEqual([{ kind: "em", children: [{ kind: "text", text: "i" }] }]);
    expect(firstParagraph("_i_")).toEqual([{ kind: "em", children: [{ kind: "text", text: "i" }] }]);
    expect(firstParagraph("~~gone~~")).toEqual([
      { kind: "strike", children: [{ kind: "text", text: "gone" }] },
    ]);
    expect(firstParagraph("***both***")).toEqual([
      { kind: "strong", children: [{ kind: "em", children: [{ kind: "text", text: "both" }] }] },
    ]);
  });

  it("nests", () => {
    expect(firstParagraph("**bold with *italic* inside**")).toEqual([
      {
        kind: "strong",
        children: [
          { kind: "text", text: "bold with " },
          { kind: "em", children: [{ kind: "text", text: "italic" }] },
          { kind: "text", text: " inside" },
        ],
      },
    ]);
  });

  it("leaves arithmetic and snake_case alone", () => {
    expect(textOf(parseMarkdown("2 * 3 * 4"))).toBe("2 * 3 * 4");
    expect(firstParagraph("2 * 3 * 4")).toEqual([{ kind: "text", text: "2 * 3 * 4" }]);
    expect(firstParagraph("some_variable_name")).toEqual([
      { kind: "text", text: "some_variable_name" },
    ]);
  });

  it("leaves an unclosed delimiter as the character it is", () => {
    expect(firstParagraph("*unfinished")).toEqual([{ kind: "text", text: "*unfinished" }]);
    expect(firstParagraph("a ** b")).toEqual([{ kind: "text", text: "a ** b" }]);
  });

  it("honours a backslash escape", () => {
    expect(firstParagraph("\\*not italic\\*")).toEqual([{ kind: "text", text: "*not italic*" }]);
  });
});

describe("links", () => {
  it("reads a labelled link", () => {
    expect(firstParagraph("[the docs](https://example.com/docs)")).toEqual([
      {
        kind: "link",
        href: "https://example.com/docs",
        children: [{ kind: "text", text: "the docs" }],
      },
    ]);
  });

  it("reads a bare url in a sentence, without the full stop", () => {
    const nodes = firstParagraph("see https://example.com/page.");
    expect(nodes).toEqual([
      { kind: "text", text: "see " },
      {
        kind: "link",
        href: "https://example.com/page",
        children: [{ kind: "text", text: "https://example.com/page" }],
      },
      { kind: "text", text: "." },
    ]);
  });

  it("keeps a balanced paren that belongs to the url", () => {
    expect(linksOf(parseMarkdown("https://ex.example/wiki/Thing_(disambiguation)"))).toEqual([
      "https://ex.example/wiki/Thing_(disambiguation)",
    ]);
  });

  it("does not autolink inside a word", () => {
    expect(linksOf(parseMarkdown("xhttps://example.com"))).toEqual([]);
  });

  it("does not nest links inside a link label", () => {
    const nodes = firstParagraph("[a [b](https://inner.example) c](https://outer.example)");
    expect(nodes).toHaveLength(1);
    expect(linksOf([{ kind: "paragraph", children: nodes }])).toEqual(["https://outer.example/"]);
  });
});

describe("blocks", () => {
  it("keeps line breaks inside one paragraph", () => {
    expect(parseMarkdown("one\ntwo")).toEqual([
      { kind: "paragraph", children: [{ kind: "text", text: "one\ntwo" }] },
    ]);
  });

  it("splits paragraphs on a blank line", () => {
    expect(parseMarkdown("one\n\ntwo")).toHaveLength(2);
  });

  it("reads a fenced code block, language and all", () => {
    expect(parseMarkdown("```rust\nfn main() {}\n```")).toEqual([
      { kind: "codeblock", lang: "rust", text: "fn main() {}" },
    ]);
  });

  it("runs an unclosed fence to the end rather than dropping it", () => {
    expect(parseMarkdown("```\nhalf a paste")).toEqual([
      { kind: "codeblock", lang: null, text: "half a paste" },
    ]);
  });

  it("does not parse anything inside a fence", () => {
    const blocks = parseMarkdown("```\n**not bold** <script>x</script>\n```");
    expect(blocks).toEqual([
      { kind: "codeblock", lang: null, text: "**not bold** <script>x</script>" },
    ]);
  });

  it("reads quotes, including nested ones", () => {
    expect(parseMarkdown("> outer\n> > inner")).toEqual([
      {
        kind: "quote",
        children: [
          { kind: "paragraph", children: [{ kind: "text", text: "outer" }] },
          {
            kind: "quote",
            children: [{ kind: "paragraph", children: [{ kind: "text", text: "inner" }] }],
          },
        ],
      },
    ]);
  });

  it("reads both kinds of list and keeps a numbered list's start", () => {
    expect(parseMarkdown("- one\n- two")).toEqual([
      {
        kind: "list",
        ordered: false,
        start: 1,
        items: [[{ kind: "text", text: "one" }], [{ kind: "text", text: "two" }]],
      },
    ]);
    const numbered = parseMarkdown("3. three\n4. four")[0];
    expect(numbered).toMatchObject({ kind: "list", ordered: true, start: 3 });
  });

  it("does not turn an asterisk emphasis at the start of a line into a list", () => {
    expect(firstParagraph("*emphasis* at the start")).toEqual([
      { kind: "em", children: [{ kind: "text", text: "emphasis" }] },
      { kind: "text", text: " at the start" },
    ]);
  });

  it("survives an empty body", () => {
    expect(parseMarkdown("")).toEqual([]);
  });
});

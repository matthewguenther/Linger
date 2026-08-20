/**
 * Drawing a parsed message body.
 *
 * Every element here is written as JSX, so the text inside it is React's to
 * escape. There is no `dangerouslySetInnerHTML` in this file and there must
 * never be one: that prop is the only way a message body could become markup,
 * and its absence is what makes `markdown.ts`'s allowlist hold
 * (ARCHITECTURE §7).
 *
 * Links get their default click prevented and are handed to the operating
 * system instead — see `lib/open.ts` for why a WebView must never follow one.
 */
import { type MouseEvent, type ReactNode, useMemo } from "react";

import { openExternal } from "../lib/open";
import { type Block, type Inline, parseMarkdown } from "./markdown";

export default function MessageBody({
  source,
  trailing,
}: {
  source: string;
  /**
   * Something to hang off the end of the last line — the "edited" marker.
   * It goes *inside* the final paragraph rather than after the body, because a
   * block element after a block element is a new line, and "edited" sitting on
   * a line of its own reads like part of the message.
   */
  trailing?: ReactNode;
}) {
  // A row re-renders whenever the gateway store changes — somebody's presence,
  // somebody typing — and none of that changes what a message says. Parsing on
  // every one of those would be thirty parses a frame for nothing.
  const blocks = useMemo(() => parseMarkdown(source), [source]);
  const last = blocks.length - 1;
  return (
    <>
      {blocks.map((block, index) => (
        <BlockNode key={index} block={block} trailing={index === last ? trailing : undefined} />
      ))}
      {blocks.length === 0 ? trailing : null}
    </>
  );
}

function BlockNode({ block, trailing }: { block: Block; trailing?: ReactNode }) {
  switch (block.kind) {
    case "paragraph":
      return (
        <p className="md-p">
          <Inlines nodes={block.children} />
          {trailing}
        </p>
      );
    case "quote":
      return (
        <>
          <blockquote className="md-quote">
            {block.children.map((child, index) => (
              <BlockNode key={index} block={child} />
            ))}
          </blockquote>
          {trailing}
        </>
      );
    case "codeblock":
      return (
        <>
          <pre className="md-pre">
            <code>{block.text}</code>
          </pre>
          {trailing}
        </>
      );
    case "list": {
      const items = block.items.map((item, index) => (
        <li key={index}>
          <Inlines nodes={item} />
        </li>
      ));
      return (
        <>
          {block.ordered ? (
            <ol className="md-list" start={block.start}>
              {items}
            </ol>
          ) : (
            <ul className="md-list">{items}</ul>
          )}
          {trailing}
        </>
      );
    }
  }
}

/**
 * Index keys are right here and only here: this list is rebuilt from the same
 * string every time and has no identity of its own to preserve.
 */
function Inlines({ nodes }: { nodes: Inline[] }) {
  return (
    <>
      {nodes.map((node, index) => (
        <InlineNode key={index} node={node} />
      ))}
    </>
  );
}

function InlineNode({ node }: { node: Inline }) {
  switch (node.kind) {
    case "text":
      return <>{node.text}</>;
    case "code":
      return <code className="md-code">{node.text}</code>;
    case "strong":
      return (
        <strong>
          <Inlines nodes={node.children} />
        </strong>
      );
    case "em":
      return (
        <em>
          <Inlines nodes={node.children} />
        </em>
      );
    case "strike":
      return (
        <s>
          <Inlines nodes={node.children} />
        </s>
      );
    case "link":
      return (
        <a
          className="md-link"
          href={node.href}
          title={node.href}
          // Both handlers matter: a plain click and a middle click are two
          // different ways for the WebView to try to navigate.
          onClick={leave(node.href)}
          onAuxClick={leave(node.href)}
        >
          <Inlines nodes={node.children} />
        </a>
      );
  }
}

function leave(href: string): (event: MouseEvent) => void {
  return (event) => {
    event.preventDefault();
    void openExternal(href);
  };
}

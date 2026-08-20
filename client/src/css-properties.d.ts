/**
 * Lets `style={{ "--age": 0.88 }}` typecheck.
 *
 * React's `CSSProperties` has no index signature for custom properties, and the
 * alternative is an `as` cast on every element that sets one. All of Linger's
 * theming runs through custom properties, so that would be a lot of casts in
 * exchange for nothing.
 */
import "react";

declare module "react" {
  interface CSSProperties {
    [key: `--${string}`]: string | number | undefined;
  }
}

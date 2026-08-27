import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./styles/tokens.css";
// The 16 palette colors, generated from linger-core::PALETTE by
// `cargo test -p linger-core`. It comes after the tokens so `[data-theme]`
// stays one continuous block, and before anything that draws a name.
import "./generated/palette.generated.css";
import "./styles/base.css";
import "./styles/names.css";
import App from "./App";

const root = document.getElementById("root");
if (!root) throw new Error("missing #root");

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);

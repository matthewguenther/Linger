# Bundled fonts

The curated set from SPEC §5.7 — open-licensed, subset, bundled with the client.
**No CDN, no remote font loading, ever** (fingerprinting vector + remote dependency).

| Key | Face | License | Source |
|---|---|---|---|
| `geist-sans` | Geist Sans | OFL-1.1 | https://vercel.com/font |
| `geist-mono` | Geist Mono | OFL-1.1 | https://vercel.com/font |
| `ibm-plex-sans` | IBM Plex Sans | OFL-1.1 | https://github.com/IBM/plex |
| `ibm-plex-mono` | IBM Plex Mono | OFL-1.1 | https://github.com/IBM/plex |
| `jetbrains-mono` | JetBrains Mono | OFL-1.1 | https://github.com/JetBrains/JetBrainsMono |
| `inter` | Inter | OFL-1.1 | https://github.com/rsms/inter |
| `space-grotesk` | Space Grotesk | OFL-1.1 | https://github.com/floriankarsten/space-grotesk |
| `commit-mono` | Commit Mono | OFL-1.1 | https://github.com/eigilnikolajsen/commit-mono |
| `newsreader` | Newsreader | OFL-1.1 | https://github.com/productiontype/Newsreader |
| `instrument-serif` | Instrument Serif | OFL-1.1 | https://github.com/Instrument/instrument-serif |
| `departure-mono` | Departure Mono | OFL-1.1 | https://departuremono.com |
| `silkscreen` | Silkscreen | OFL-1.1 | https://kottke.org/plus/type/silkscreen |

The font keys are defined once in `linger-core::FONTS`; this directory must stay
in one-to-one correspondence with that list.

## What is where

- `OFL-<key>.txt` — each face's license text, downloaded with the face.
- `client/src/fonts/*.woff2` — the subset faces the app actually ships.
- `client/src/fonts/fonts.css` — the generated `@font-face` rules. Do not edit;
  the next run overwrites it.

All of it is **committed**. The app must build with no network and must never
fetch a glyph at runtime, so the binaries live in the repo. The whole set is
about 800 KB across 30 faces, which is what subsetting buys: unsubset, the same
twelve faces are roughly six megabytes.

## Rebuilding

```bash
scripts/fetch-fonts.sh
```

You do not need this to build Linger — run it when a face is added to
`linger-core::FONTS`, or to pull an upstream fix, and commit what changes. It
needs python3 and network access, and keeps its virtualenv under `target/`.

The manifest — where each face comes from, which weights, which axes get pinned
— is at the top of `scripts/fetch_fonts.py`. The script refuses to run if the
manifest and `linger-core::FONTS` disagree.

## What subsetting keeps

Latin, Latin Extended-A and -B, general punctuation, currency, and arrows
(the reply mark is U+21A9). Weights 400/500/700 and italics where the face has
them — several of these publish a variable font, so one file covers the whole
400–700 range and the `@font-face` rule declares it as a range.

Three faces are one weight or have no italic, because that is what the designer
drew: Instrument Serif (400 + italic), Departure Mono (400), Silkscreen
(400/700, no italic). Nothing is faked in the pipeline; the browser slants or
emboldens if somebody asks for what does not exist.

Ten of the twelve are downloaded from `google/fonts`, which is where those
projects publish and which carries the OFL text next to the binary. Commit Mono
and Departure Mono come from their own repos. The Source column above is each
face's home, not the download URL — the URLs live in the script.

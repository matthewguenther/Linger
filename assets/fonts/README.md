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

Acquisition and subsetting happen in task T-604 (see TASKS.md): woff2, weights
400/500/700 + italic where the face has them, Latin + Latin-Extended subset,
each face's OFL license text kept alongside it in this directory.

The font keys are defined once in `linger-core::FONTS`; this directory must stay
in one-to-one correspondence with that list.

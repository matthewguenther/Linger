#!/usr/bin/env python3
"""Fetch, subset and bundle the twelve curated faces (SPEC §5.7, T-604).

Run it through `scripts/fetch-fonts.sh`, which builds the virtualenv this needs.

What comes out:

  client/src/fonts/*.woff2   the subset faces, committed
  client/src/fonts/fonts.css the @font-face rules, generated from MANIFEST below
  assets/fonts/OFL-*.txt     each face's license text, kept with the sources

Three rules this script exists to keep:

  * **No CDN, ever.** A remote font URL is a fingerprinting vector and a
    dependency on somebody else's uptime. Everything is downloaded once, here,
    and committed. Nothing the app ships reaches the network for a glyph.
  * **The keys are `linger-core::FONTS`.** MANIFEST is keyed by them and the
    script refuses to run if the two lists disagree, so a face can never be
    added on one side of the wire only.
  * **Subset to Latin.** A full face is 150-900 KB; the app needs Latin,
    Latin Extended, punctuation and a handful of arrows. That is the difference
    between a font directory of a few hundred KB and one of ten megabytes.

Sources are the canonical upstream repos where the face lives, which for ten of
the twelve is `google/fonts` — it is where those projects publish, it carries
the OFL text next to the binary, and its paths are stable. `assets/fonts/README.md`
records each face's own home page.
"""

from __future__ import annotations

import io
import re
import shutil
import subprocess
import sys
import tempfile
import urllib.request
import zipfile
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT_DIR = ROOT / "client" / "src" / "fonts"
LICENSE_DIR = ROOT / "assets" / "fonts"
CORE_LIB = ROOT / "crates" / "linger-core" / "src" / "lib.rs"

GF = "https://raw.githubusercontent.com/google/fonts/main/ofl"
CM = "https://raw.githubusercontent.com/eigilnikolajsen/commit-mono/main/src/fonts/fontlab"

# Latin + Latin Extended, punctuation, currency, and the arrows the UI draws
# (the reply mark is U+21A9). Anything outside this is not a glyph this app has
# a use for, and every one of them costs bytes in twelve faces at once.
UNICODES = ",".join(
    [
        "U+0000-00FF",  # Basic Latin + Latin-1 Supplement
        "U+0100-017F",  # Latin Extended-A
        "U+0180-024F",  # Latin Extended-B
        "U+0259",  # schwa, used by a few Latin orthographies
        "U+02BB-02BC",
        "U+02C6",
        "U+02DA",
        "U+02DC",
        "U+0304",
        "U+0308",
        "U+0329",
        "U+2000-206F",  # General Punctuation: quotes, dashes, the ellipsis
        "U+20A0-20BF",  # currency
        "U+2122",  # trademark
        "U+2190-21FF",  # arrows
        "U+2212",  # minus
        "U+2215",
        "U+FEFF",
        "U+FFFD",
    ]
)


@dataclass
class Face:
    """One `@font-face` rule and the file behind it."""

    url: str
    style: str = "normal"
    #: A single weight (`"400"`) or a variable font's range (`"400 700"`).
    weight: str = "400"
    #: Axes to pin before subsetting, e.g. `{"wdth": 100}`. A two-axis variable
    #: font is enormous; pinning the axis nobody varies is most of the saving.
    pin: dict[str, float] = field(default_factory=dict)
    #: Keep this axis as a range when instancing, e.g. `("wght", 400, 700)`.
    keep: tuple[str, float, float] | None = None
    #: Set when the download is a zip; names the member to pull out of it.
    member: str | None = None


@dataclass
class Bundle:
    """One key from `linger-core::FONTS`, with the CSS family it draws as."""

    family: str
    license_url: str
    faces: list[Face]


MANIFEST: dict[str, Bundle] = {
    "geist-sans": Bundle(
        family="Geist Sans",
        license_url=f"{GF}/geist/OFL.txt",
        faces=[
            Face(f"{GF}/geist/Geist%5Bwght%5D.ttf", weight="400 700", keep=("wght", 400, 700)),
            Face(
                f"{GF}/geist/Geist-Italic%5Bwght%5D.ttf",
                style="italic",
                weight="400 700",
                keep=("wght", 400, 700),
            ),
        ],
    ),
    "geist-mono": Bundle(
        family="Geist Mono",
        license_url=f"{GF}/geistmono/OFL.txt",
        faces=[
            Face(
                f"{GF}/geistmono/GeistMono%5Bwght%5D.ttf",
                weight="400 700",
                keep=("wght", 400, 700),
            ),
            Face(
                f"{GF}/geistmono/GeistMono-Italic%5Bwght%5D.ttf",
                style="italic",
                weight="400 700",
                keep=("wght", 400, 700),
            ),
        ],
    ),
    "ibm-plex-sans": Bundle(
        family="IBM Plex Sans",
        license_url=f"{GF}/ibmplexsans/OFL.txt",
        faces=[
            Face(
                f"{GF}/ibmplexsans/IBMPlexSans%5Bwdth,wght%5D.ttf",
                weight="400 700",
                pin={"wdth": 100},
                keep=("wght", 400, 700),
            ),
            Face(
                f"{GF}/ibmplexsans/IBMPlexSans-Italic%5Bwdth,wght%5D.ttf",
                style="italic",
                weight="400 700",
                pin={"wdth": 100},
                keep=("wght", 400, 700),
            ),
        ],
    ),
    # Plex Mono ships as statics, so the three weights are three files.
    "ibm-plex-mono": Bundle(
        family="IBM Plex Mono",
        license_url=f"{GF}/ibmplexmono/OFL.txt",
        faces=[
            Face(f"{GF}/ibmplexmono/IBMPlexMono-Regular.ttf", weight="400"),
            Face(f"{GF}/ibmplexmono/IBMPlexMono-Medium.ttf", weight="500"),
            Face(f"{GF}/ibmplexmono/IBMPlexMono-Bold.ttf", weight="700"),
            Face(f"{GF}/ibmplexmono/IBMPlexMono-Italic.ttf", style="italic", weight="400"),
            Face(f"{GF}/ibmplexmono/IBMPlexMono-MediumItalic.ttf", style="italic", weight="500"),
            Face(f"{GF}/ibmplexmono/IBMPlexMono-BoldItalic.ttf", style="italic", weight="700"),
        ],
    ),
    "jetbrains-mono": Bundle(
        family="JetBrains Mono",
        license_url=f"{GF}/jetbrainsmono/OFL.txt",
        faces=[
            Face(
                f"{GF}/jetbrainsmono/JetBrainsMono%5Bwght%5D.ttf",
                weight="400 700",
                keep=("wght", 400, 700),
            ),
            Face(
                f"{GF}/jetbrainsmono/JetBrainsMono-Italic%5Bwght%5D.ttf",
                style="italic",
                weight="400 700",
                keep=("wght", 400, 700),
            ),
        ],
    ),
    # Inter's optical-size axis is pinned at the size the app actually sets
    # text at; leaving it variable would roughly double the file for nothing.
    "inter": Bundle(
        family="Inter",
        license_url=f"{GF}/inter/OFL.txt",
        faces=[
            Face(
                f"{GF}/inter/Inter%5Bopsz,wght%5D.ttf",
                weight="400 700",
                pin={"opsz": 14},
                keep=("wght", 400, 700),
            ),
            Face(
                f"{GF}/inter/Inter-Italic%5Bopsz,wght%5D.ttf",
                style="italic",
                weight="400 700",
                pin={"opsz": 14},
                keep=("wght", 400, 700),
            ),
        ],
    ),
    # No italic drawn for this one, so none is bundled — the browser slants it
    # if somebody asks, which is the honest outcome for a face without one.
    "space-grotesk": Bundle(
        family="Space Grotesk",
        license_url=f"{GF}/spacegrotesk/OFL.txt",
        faces=[
            Face(
                f"{GF}/spacegrotesk/SpaceGrotesk%5Bwght%5D.ttf",
                weight="400 700",
                keep=("wght", 400, 700),
            )
        ],
    ),
    "commit-mono": Bundle(
        family="Commit Mono",
        license_url="https://raw.githubusercontent.com/eigilnikolajsen/commit-mono/main/LICENSE-FONT",
        faces=[
            Face(f"{CM}/CommitMonoV143-400Regular.otf", weight="400"),
            Face(f"{CM}/CommitMonoV143-500Regular.otf", weight="500"),
            Face(f"{CM}/CommitMonoV143-700Regular.otf", weight="700"),
            Face(f"{CM}/CommitMonoV143-400Italic.otf", style="italic", weight="400"),
            Face(f"{CM}/CommitMonoV143-500Italic.otf", style="italic", weight="500"),
            Face(f"{CM}/CommitMonoV143-700Italic.otf", style="italic", weight="700"),
        ],
    ),
    "newsreader": Bundle(
        family="Newsreader",
        license_url=f"{GF}/newsreader/OFL.txt",
        faces=[
            Face(
                f"{GF}/newsreader/Newsreader%5Bopsz,wght%5D.ttf",
                weight="400 700",
                pin={"opsz": 16},
                keep=("wght", 400, 700),
            ),
            Face(
                f"{GF}/newsreader/Newsreader-Italic%5Bopsz,wght%5D.ttf",
                style="italic",
                weight="400 700",
                pin={"opsz": 16},
                keep=("wght", 400, 700),
            ),
        ],
    ),
    # One weight exists, and it is a display face — 400 is the whole of it.
    "instrument-serif": Bundle(
        family="Instrument Serif",
        license_url=f"{GF}/instrumentserif/OFL.txt",
        faces=[
            Face(f"{GF}/instrumentserif/InstrumentSerif-Regular.ttf", weight="400"),
            Face(f"{GF}/instrumentserif/InstrumentSerif-Italic.ttf", style="italic", weight="400"),
        ],
    ),
    # A bitmap-styled face from a zip, with one weight and no italic.
    "departure-mono": Bundle(
        family="Departure Mono",
        license_url="https://raw.githubusercontent.com/rektdeckard/departure-mono/main/LICENSE",
        faces=[
            Face(
                "https://github.com/rektdeckard/departure-mono/releases/download/v1.500/DepartureMono-1.500.zip",
                weight="400",
                member="DepartureMono-Regular.otf",
            )
        ],
    ),
    "silkscreen": Bundle(
        family="Silkscreen",
        license_url=f"{GF}/silkscreen/OFL.txt",
        faces=[
            Face(f"{GF}/silkscreen/Silkscreen-Regular.ttf", weight="400"),
            Face(f"{GF}/silkscreen/Silkscreen-Bold.ttf", weight="700"),
        ],
    ),
}


def core_font_keys() -> list[str]:
    """The keys from `linger-core::FONTS`, read rather than copied."""
    source = CORE_LIB.read_text(encoding="utf-8")
    block = re.search(r"pub const FONTS: \[&str; \d+\] = \[(.*?)\];", source, re.S)
    if block is None:
        sys.exit("could not find FONTS in crates/linger-core/src/lib.rs")
    return re.findall(r'"([^"]+)"', block.group(1))


def fetch(url: str, member: str | None) -> bytes:
    request = urllib.request.Request(url, headers={"User-Agent": "linger-fetch-fonts"})
    with urllib.request.urlopen(request, timeout=120) as response:  # noqa: S310 - fixed URLs
        raw = response.read()
    if member is None:
        return raw
    with zipfile.ZipFile(io.BytesIO(raw)) as archive:
        for name in archive.namelist():
            if Path(name).name == member:
                return archive.read(name)
    sys.exit(f"{member} is not in {url}")


def build_face(key: str, index: int, face: Face, work: Path) -> tuple[str, int]:
    """Download one face, pin its extra axes, subset it, write the woff2."""
    from fontTools import subset
    from fontTools.ttLib import TTFont
    from fontTools.varLib import instancer

    raw = work / f"{key}-{index}.src"
    raw.write_bytes(fetch(face.url, face.member))

    source = raw
    if face.pin or face.keep:
        font = TTFont(raw)
        axes: dict[str, object] = dict(face.pin)
        if face.keep is not None:
            name, low, high = face.keep
            axes[name] = (low, high)
        font = instancer.instantiateVariableFont(font, axes, updateFontNames=False)
        source = work / f"{key}-{index}.pinned.ttf"
        font.save(source)

    out = OUT_DIR / f"{key}-{index}.woff2"
    subset.main(
        [
            str(source),
            f"--unicodes={UNICODES}",
            "--layout-features=kern,liga,calt,ccmp,locl,mark,mkmk,frac,tnum",
            "--flavor=woff2",
            "--no-hinting",
            "--desubroutinize",
            # `meta` is design/script metadata the subsetter cannot rewrite and
            # nothing renders from; dropping it silences a warning per face.
            "--drop-tables+=DSIG,meta",
            f"--output-file={out}",
        ]
    )
    return out.name, out.stat().st_size


def css_header() -> str:
    return """/*
 * The twelve bundled faces (SPEC §5.7). Generated by `scripts/fetch-fonts.sh`
 * from the manifest in `scripts/fetch_fonts.py` — do not edit.
 *
 * No CDN and no remote font URL, ever: a remote face is a fingerprinting vector
 * and somebody else's uptime. Every file here is subset to Latin, Latin
 * Extended, punctuation and arrows, and lives in the repo.
 *
 * The `--font-<key>` custom properties in `styles/tokens.css` name these
 * families, and `lib/fonts.ts` decides which key a person's style resolves to.
 */
"""


def main() -> None:
    keys = core_font_keys()
    if sorted(keys) != sorted(MANIFEST):
        missing = sorted(set(keys) - set(MANIFEST))
        extra = sorted(set(MANIFEST) - set(keys))
        sys.exit(f"MANIFEST disagrees with linger-core::FONTS (missing {missing}, extra {extra})")

    if OUT_DIR.exists():
        shutil.rmtree(OUT_DIR)
    OUT_DIR.mkdir(parents=True)
    LICENSE_DIR.mkdir(parents=True, exist_ok=True)

    rules: list[str] = []
    total = 0
    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp)
        for key in keys:
            bundle = MANIFEST[key]
            LICENSE_DIR.joinpath(f"OFL-{key}.txt").write_bytes(fetch(bundle.license_url, None))
            for index, face in enumerate(bundle.faces):
                name, size = build_face(key, index, face, work)
                total += size
                print(f"  {name:<28} {size / 1024:6.1f} KB  {bundle.family} {face.weight} {face.style}")
                rules.append(
                    "@font-face {\n"
                    f'  font-family: "{bundle.family}";\n'
                    f"  font-style: {face.style};\n"
                    f"  font-weight: {face.weight};\n"
                    "  font-display: swap;\n"
                    f'  src: url("./{name}") format("woff2");\n'
                    "}\n"
                )

    (OUT_DIR / "fonts.css").write_text(css_header() + "\n" + "\n".join(rules), encoding="utf-8")
    print(f"\n{len(rules)} faces, {total / 1024 / 1024:.2f} MB total in {OUT_DIR}")


if __name__ == "__main__":
    main()

#!/usr/bin/env bash
# Fetch, subset and bundle the twelve curated faces (SPEC §5.7, T-604).
#
# You do not need to run this to build Linger. The output is committed —
# `client/src/fonts/` and the OFL texts in `assets/fonts/` — because the app
# must build with no network and must never load a font from a CDN. Run it when
# a face is added to `linger-core::FONTS`, or to pull upstream fixes, and commit
# what changes.
#
# It needs python3 and network access. The virtualenv lives under `target/`,
# which is already gitignored, so nothing is installed on your machine.
set -euo pipefail
cd "$(dirname "$0")/.."

VENV=target/fonts-venv

if [ ! -x "$VENV/bin/python" ]; then
  echo "== making $VENV =="
  # Debian and Ubuntu ship python3 without `ensurepip`, so the ordinary venv
  # fails on exactly the machines this project is developed on. Fall back to a
  # venv with no pip in it and bootstrap pip into that, which needs the network
  # this script already needs.
  if ! python3 -m venv "$VENV" 2>/dev/null; then
    echo "   (no ensurepip here — bootstrapping pip)"
    rm -rf "$VENV"
    python3 -m venv --without-pip "$VENV"
    curl -fsSL https://bootstrap.pypa.io/get-pip.py -o "$VENV/get-pip.py"
    "$VENV/bin/python" "$VENV/get-pip.py" --quiet
  fi
  # `brotli` is what fontTools compresses woff2 with; without it the subsetter
  # can read a font and not write one, which is a confusing way to fail.
  "$VENV/bin/python" -m pip install --quiet --upgrade pip
  "$VENV/bin/python" -m pip install --quiet "fonttools[woff]>=4.55" brotli
fi

echo "== fetching and subsetting =="
"$VENV/bin/python" scripts/fetch_fonts.py

echo
echo "Commit client/src/fonts/ and assets/fonts/ with your change."

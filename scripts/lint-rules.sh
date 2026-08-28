#!/usr/bin/env bash
# Mechanical checks for the AGENTS.md hard rules that a grep can catch:
#   rule 1 — no AI attribution in commits, identities, or the tree
#   rule 6 — the dropped vocabulary stays dropped in code
# Usage: scripts/lint-rules.sh [base-ref]
#   With a base-ref (e.g. origin/main), commits on base-ref..HEAD are checked too.
set -euo pipefail
cd "$(dirname "$0")/.."

fail=0
AI_NAMES='claude|anthropic|openai|chatgpt|gpt-[0-9]|codex|copilot|gemini|qwen|grok|devin|aider|windsurf|deepseek|sonnet|opus 4|opus 5|fable'

# ---- commit checks (only when a base ref is given) --------------------------
if [ -n "${1:-}" ]; then
  range="$1..HEAD"
  if git log --format='%B' "$range" | grep -qiE "(co-authored-by|generated (with|by)).{0,60}(${AI_NAMES})"; then
    echo "FAIL: AI attribution in a commit message on ${range}."
    echo "      The author is the person who ran the session — nothing else."
    echo "      Rewrite the commits (git rebase); do not add a fixup on top."
    git log --format='%h %s' "$range" | head -20
    fail=1
  fi
  if git log --format='%an <%ae>%n%cn <%ce>' "$range" | grep -iE "${AI_NAMES}" | grep -viq 'noreply@github.com'; then
    echo "FAIL: a commit on ${range} has an AI author or committer identity."
    echo "      Fix your git user.name / user.email and rewrite the commits."
    fail=1
  fi
fi

# ---- tree checks ------------------------------------------------------------
CODE_DIRS=(crates client/src client/src-tauri/src deploy)

if grep -rniE "(co-authored-by|generated (with|by)).{0,60}(${AI_NAMES})" "${CODE_DIRS[@]}"; then
  echo "FAIL: AI attribution string in the tree (rule 1)."
  fail=1
fi

# Lines that *forbid* a word ("never ChannelId") are documentation, not use.
if grep -rniE '\bchannelid\b|\bstoops?\b|\bshelf\b|\bsitting in\b' "${CODE_DIRS[@]}" | grep -viE '\bnever\b|\bnot\b'; then
  echo "FAIL: dropped vocabulary in the tree (rule 6, SPEC §1)."
  fail=1
fi

if [ "$fail" -eq 0 ]; then echo "rules lint: clean"; fi
exit "$fail"

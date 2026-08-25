#!/usr/bin/env bash
# Run the S3 backend's integration tests against a real S3 API (T-502).
#
# `cargo test --workspace` skips crates/linger-server/tests/s3.rs, because most
# machines have no bucket. This script gives it one: it starts a throwaway MinIO
# on 127.0.0.1:9000, runs the tests, and stops it again. CI does the same thing
# with a service container.
#
# MinIO is a single binary. Put it on your PATH, or set MINIO_BIN to it:
#   curl -o minio https://dl.min.io/server/minio/release/linux-amd64/minio
#   chmod +x minio
#
# Docker works too, if you would rather not have the binary around:
#   docker run --rm -p 9000:9000 -e MINIO_ROOT_USER=lingertest \
#     -e MINIO_ROOT_PASSWORD=lingertestsecret minio/minio server /data
# then run this script with LINGER_TEST_S3_ENDPOINT already set, and it will
# use the server you started instead of launching one.
set -euo pipefail
cd "$(dirname "$0")/.."

export LINGER_TEST_S3_BUCKET="${LINGER_TEST_S3_BUCKET:-linger-test}"
export LINGER_TEST_S3_REGION="${LINGER_TEST_S3_REGION:-us-east-1}"
export LINGER_TEST_S3_ACCESS_KEY_ID="${LINGER_TEST_S3_ACCESS_KEY_ID:-lingertest}"
export LINGER_TEST_S3_SECRET_ACCESS_KEY="${LINGER_TEST_S3_SECRET_ACCESS_KEY:-lingertestsecret}"

if [ -n "${LINGER_TEST_S3_ENDPOINT:-}" ]; then
  echo "== using the S3 endpoint you already have: $LINGER_TEST_S3_ENDPOINT =="
else
  MINIO_BIN="${MINIO_BIN:-$(command -v minio || true)}"
  if [ -z "$MINIO_BIN" ]; then
    echo "minio not found. Put it on your PATH or set MINIO_BIN — see the top of this script." >&2
    exit 1
  fi

  workdir="$(mktemp -d)"
  # Kill by exact pid, never by pattern: pkill -f matches this script's own
  # command line (AGENTS.md).
  cleanup() {
    kill "$minio_pid" 2>/dev/null || true
    wait "$minio_pid" 2>/dev/null || true
    rm -rf "$workdir"
  }
  trap cleanup EXIT

  echo "== starting minio on 127.0.0.1:9000 =="
  MINIO_ROOT_USER="$LINGER_TEST_S3_ACCESS_KEY_ID" \
  MINIO_ROOT_PASSWORD="$LINGER_TEST_S3_SECRET_ACCESS_KEY" \
    "$MINIO_BIN" server "$workdir/data" --address 127.0.0.1:9000 >"$workdir/minio.log" 2>&1 &
  minio_pid=$!

  for _ in $(seq 1 60); do
    if curl -fsS -o /dev/null http://127.0.0.1:9000/minio/health/live; then break; fi
    sleep 0.5
  done
  curl -fsS -o /dev/null http://127.0.0.1:9000/minio/health/live || {
    echo "minio never came up:" >&2
    cat "$workdir/minio.log" >&2
    exit 1
  }
  export LINGER_TEST_S3_ENDPOINT="http://127.0.0.1:9000"
fi

echo "== cargo test -p linger-server --test s3 =="
cargo test -p linger-server --test s3 -- --nocapture

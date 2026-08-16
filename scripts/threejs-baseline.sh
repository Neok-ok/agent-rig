#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
mkdir -p artifacts
OUT="$ROOT/artifacts/threejs-frame.png"
CHROME="${CHROME:-/usr/bin/google-chrome}"

if [[ ! -f "$ROOT/compare/three.module.min.js" ]]; then
  echo "missing $ROOT/compare/three.module.min.js" >&2
  exit 1
fi

PORT="${THREEJS_PORT:-8765}"
python3 -m http.server "$PORT" --bind 127.0.0.1 >/tmp/agent-rig-threejs-http.log 2>&1 &
PID=$!
cleanup() { kill "$PID" 2>/dev/null || true; }
trap cleanup EXIT
sleep 0.4

URL="http://127.0.0.1:${PORT}/compare/threejs-baseline.html"
rm -f "$OUT"

"$CHROME" \
  --headless=new \
  --disable-gpu \
  --hide-scrollbars \
  --window-size=800,450 \
  --use-angle=swiftshader \
  --enable-unsafe-swiftshader \
  --ignore-gpu-blocklist \
  --virtual-time-budget=10000 \
  --screenshot="$OUT" \
  "$URL"

if [[ ! -f "$OUT" ]]; then
  echo "chrome did not write $OUT" >&2
  exit 1
fi

BYTES=$(stat -c%s "$OUT")
if [[ "$BYTES" -lt 1024 ]]; then
  echo "png too small: $BYTES" >&2
  exit 1
fi
echo "wrote $OUT ($BYTES bytes)"

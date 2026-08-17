#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
mkdir -p artifacts/increment63
OUT="$ROOT/artifacts/increment63/threejs-frame.png"
CHROME="${CHROME:-/usr/bin/google-chrome}"

if [[ ! -f "$ROOT/compare/three.module.min.js" ]]; then
  echo "missing $ROOT/compare/three.module.min.js" >&2
  exit 1
fi
if [[ ! -f "$ROOT/compare/threejs-from-scene.html" ]]; then
  echo "missing $ROOT/compare/threejs-from-scene.html" >&2
  exit 1
fi
if [[ ! -f "$ROOT/artifacts/increment63/scene.json" ]]; then
  echo "missing artifacts/increment63/scene.json (run increment63 first)" >&2
  exit 1
fi
if [[ ! -f "$ROOT/artifacts/increment63/physics.json" ]]; then
  echo "missing artifacts/increment63/physics.json (run increment63 first)" >&2
  exit 1
fi

PORT="${THREEJS_PORT:-8807}"
python3 -m http.server "$PORT" --bind 127.0.0.1 >/tmp/agent-rig-threejs-inc63-http.log 2>&1 &
PID=$!
cleanup() { kill "$PID" 2>/dev/null || true; }
trap cleanup EXIT
sleep 0.4

URL="http://127.0.0.1:${PORT}/compare/threejs-from-scene.html?scene=/artifacts/increment63/scene.json&physics=/artifacts/increment63/physics.json"
rm -f "$OUT"

"$CHROME" \
  --headless=new \
  --disable-gpu \
  --hide-scrollbars \
  --window-size=800,450 \
  --use-angle=swiftshader \
  --enable-unsafe-swiftshader \
  --ignore-gpu-blocklist \
  --virtual-time-budget=20000 \
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

if [[ -f "$ROOT/artifacts/increment63/next-physics.json" ]]; then
  NEXT_SCENE="$ROOT/artifacts/increment63/next-scene.json"
  if [[ ! -f "$NEXT_SCENE" ]]; then
    echo "missing artifacts/increment63/next-scene.json (needed for next-threejs)" >&2
    exit 1
  fi
  NEXT_OUT="$ROOT/artifacts/increment63/next-threejs-frame.png"
  NEXT_URL="http://127.0.0.1:${PORT}/compare/threejs-from-scene.html?scene=/artifacts/increment63/next-scene.json&physics=/artifacts/increment63/next-physics.json"
  rm -f "$NEXT_OUT"
  "$CHROME" \
    --headless=new \
    --disable-gpu \
    --hide-scrollbars \
    --window-size=800,450 \
    --use-angle=swiftshader \
    --enable-unsafe-swiftshader \
    --ignore-gpu-blocklist \
    --virtual-time-budget=20000 \
    --screenshot="$NEXT_OUT" \
    "$NEXT_URL"
  if [[ ! -f "$NEXT_OUT" ]]; then
    echo "chrome did not write $NEXT_OUT" >&2
    exit 1
  fi
  NEXT_BYTES=$(stat -c%s "$NEXT_OUT")
  if [[ "$NEXT_BYTES" -lt 1024 ]]; then
    echo "png too small: $NEXT_BYTES" >&2
    exit 1
  fi
  echo "wrote $NEXT_OUT ($NEXT_BYTES bytes)"
fi

#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
mkdir -p artifacts/increment12
CHROME="${CHROME:-/usr/bin/google-chrome}"

if [[ ! -f "$ROOT/compare/three.module.min.js" ]]; then
  echo "missing $ROOT/compare/three.module.min.js" >&2
  exit 1
fi
if [[ ! -f "$ROOT/compare/threejs-from-scene.html" ]]; then
  echo "missing $ROOT/compare/threejs-from-scene.html" >&2
  exit 1
fi
if [[ ! -f "$ROOT/artifacts/increment12/scene.json" ]]; then
  echo "missing artifacts/increment12/scene.json (run increment12 first)" >&2
  exit 1
fi
if [[ ! -f "$ROOT/artifacts/increment12/physics.json" ]]; then
  echo "missing artifacts/increment12/physics.json (run increment12 first)" >&2
  exit 1
fi

PORT="${THREEJS_PORT:-8776}"
python3 -m http.server "$PORT" --bind 127.0.0.1 >/tmp/agent-rig-threejs-inc12-http.log 2>&1 &
PID=$!
cleanup() { kill "$PID" 2>/dev/null || true; }
trap cleanup EXIT
sleep 0.4

mapfile -t CAMS < <(python3 - <<'PY'
import json, math
with open("artifacts/increment12/scene.json") as f:
    spec = json.load(f)
cam = spec["camera"]
look = cam["look_at"]
pos = cam["position"]
dx = pos[0] - look[0]
dz = pos[2] - look[2]
radius = (dx * dx + dz * dz) ** 0.5
height = pos[1]
for i in range(8):
    yaw = i * math.tau / 8.0
    x = look[0] + radius * math.sin(yaw)
    z = look[2] + radius * math.cos(yaw)
    print(f"{x},{height},{z}")
PY
)

for i in $(seq 0 7); do
  printf -v idx "%02d" "$i"
  OUT="$ROOT/artifacts/increment12/threejs_${idx}.png"
  CAM="${CAMS[$i]}"
  URL="http://127.0.0.1:${PORT}/compare/threejs-from-scene.html?scene=/artifacts/increment12/scene.json&physics=/artifacts/increment12/physics.json&cam=${CAM}"
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
    echo "png too small: $BYTES ($OUT)" >&2
    exit 1
  fi
  echo "wrote $OUT ($BYTES bytes) cam=${CAM}"
done

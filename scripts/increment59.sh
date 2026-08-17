#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment59
cargo run --release -- increment59 --out artifacts/increment59
cargo run --release -- scenes --out artifacts/increment59/scenes.json
./scripts/increment59-threejs.sh

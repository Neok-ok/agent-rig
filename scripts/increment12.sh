#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment12
cargo run --release -- increment12 --out artifacts/increment12
./scripts/increment12-threejs.sh

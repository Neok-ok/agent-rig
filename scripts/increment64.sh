#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment64
cargo run --release -- increment64 --out artifacts/increment64
./scripts/increment64-threejs.sh

#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment58
cargo run --release -- increment58 --out artifacts/increment58
./scripts/increment58-threejs.sh

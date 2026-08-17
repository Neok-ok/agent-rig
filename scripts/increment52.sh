#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment52
cargo run --release -- increment52 --out artifacts/increment52
./scripts/increment52-threejs.sh

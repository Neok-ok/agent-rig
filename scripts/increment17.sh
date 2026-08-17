#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment17
cargo run --release -- increment17 --out artifacts/increment17
./scripts/increment17-threejs.sh

#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment50
cargo run --release -- increment50 --out artifacts/increment50
./scripts/increment50-threejs.sh

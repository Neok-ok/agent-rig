#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment18
cargo run --release -- increment18 --out artifacts/increment18
./scripts/increment18-threejs.sh

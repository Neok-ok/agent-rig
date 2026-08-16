#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment11
cargo run --release -- increment11 --out artifacts/increment11
./scripts/increment11-threejs.sh

#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment10
cargo run --release -- increment10 --out artifacts/increment10
./scripts/increment10-threejs.sh

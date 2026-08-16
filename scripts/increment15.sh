#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment15
cargo run --release -- increment15 --out artifacts/increment15
./scripts/increment15-threejs.sh

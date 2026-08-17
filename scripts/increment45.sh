#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment45
cargo run --release -- increment45 --out artifacts/increment45
./scripts/increment45-threejs.sh

#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment20
cargo run --release -- increment20 --out artifacts/increment20
./scripts/increment20-threejs.sh

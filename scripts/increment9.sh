#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment9
cargo run --release -- increment9 --out artifacts/increment9
./scripts/increment9-threejs.sh

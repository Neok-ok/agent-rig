#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment30
cargo run --release -- increment30 --out artifacts/increment30
./scripts/increment30-threejs.sh

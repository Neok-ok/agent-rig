#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment8
cargo run --release -- increment8 --out artifacts/increment8
./scripts/increment8-threejs.sh

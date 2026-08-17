#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment19
cargo run --release -- increment19 --out artifacts/increment19
./scripts/increment19-threejs.sh

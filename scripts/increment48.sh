#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment48
cargo run --release -- increment48 --out artifacts/increment48
./scripts/increment48-threejs.sh

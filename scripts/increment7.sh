#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment7
cargo run --release -- increment7 --out artifacts/increment7
./scripts/increment7-threejs.sh

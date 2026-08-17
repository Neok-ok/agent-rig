#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment56
cargo run --release -- increment56 --out artifacts/increment56
./scripts/increment56-threejs.sh

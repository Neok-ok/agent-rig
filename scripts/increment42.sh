#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment42
cargo run --release -- increment42 --out artifacts/increment42
./scripts/increment42-threejs.sh

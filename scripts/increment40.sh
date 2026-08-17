#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment40
cargo run --release -- increment40 --out artifacts/increment40
./scripts/increment40-threejs.sh

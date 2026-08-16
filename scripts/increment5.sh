#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment5
cargo run --release -- increment5 --out artifacts/increment5
./scripts/increment5-threejs.sh

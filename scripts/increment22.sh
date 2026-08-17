#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment22
cargo run --release -- increment22 --out artifacts/increment22
./scripts/increment22-threejs.sh

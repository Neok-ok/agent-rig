#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment27
cargo run --release -- increment27 --out artifacts/increment27
./scripts/increment27-threejs.sh

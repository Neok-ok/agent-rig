#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment31
cargo run --release -- increment31 --out artifacts/increment31
./scripts/increment31-threejs.sh

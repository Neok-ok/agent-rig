#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment28
cargo run --release -- increment28 --out artifacts/increment28
./scripts/increment28-threejs.sh

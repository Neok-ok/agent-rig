#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment51
cargo run --release -- increment51 --out artifacts/increment51
./scripts/increment51-threejs.sh

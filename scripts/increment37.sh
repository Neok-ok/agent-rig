#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment37
cargo run --release -- increment37 --out artifacts/increment37
./scripts/increment37-threejs.sh

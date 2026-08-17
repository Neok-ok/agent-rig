#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment57
cargo run --release -- increment57 --out artifacts/increment57
./scripts/increment57-threejs.sh

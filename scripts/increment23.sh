#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment23
cargo run --release -- increment23 --out artifacts/increment23
./scripts/increment23-threejs.sh

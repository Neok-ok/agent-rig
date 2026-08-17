#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment33
cargo run --release -- increment33 --out artifacts/increment33
./scripts/increment33-threejs.sh

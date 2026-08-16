#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment4
cargo run --release -- increment4 --out artifacts/increment4
./scripts/increment4-threejs.sh

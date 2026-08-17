#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment26
cargo run --release -- increment26 --out artifacts/increment26
./scripts/increment26-threejs.sh

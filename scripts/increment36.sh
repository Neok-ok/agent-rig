#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment36
cargo run --release -- increment36 --out artifacts/increment36
./scripts/increment36-threejs.sh

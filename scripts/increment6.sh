#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment6
cargo run --release -- increment6 --out artifacts/increment6
./scripts/increment6-threejs.sh

#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment21
cargo run --release -- increment21 --out artifacts/increment21
./scripts/increment21-threejs.sh

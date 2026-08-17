#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment34
cargo run --release -- increment34 --out artifacts/increment34
./scripts/increment34-threejs.sh

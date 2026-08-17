#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment32
cargo run --release -- increment32 --out artifacts/increment32
./scripts/increment32-threejs.sh

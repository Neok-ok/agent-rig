#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment62
cargo run --release -- increment62 --out artifacts/increment62
./scripts/increment62-threejs.sh

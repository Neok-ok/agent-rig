#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment14
cargo run --release -- increment14 --out artifacts/increment14
./scripts/increment14-threejs.sh

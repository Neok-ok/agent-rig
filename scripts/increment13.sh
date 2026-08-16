#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment13
cargo run --release -- increment13 --out artifacts/increment13
./scripts/increment13-threejs.sh

#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment39
cargo run --release -- increment39 --out artifacts/increment39
./scripts/increment39-threejs.sh

#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment65
cargo run --release -- increment65 --out artifacts/increment65
./scripts/increment65-threejs.sh

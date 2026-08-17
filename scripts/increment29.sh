#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment29
cargo run --release -- increment29 --out artifacts/increment29
./scripts/increment29-threejs.sh

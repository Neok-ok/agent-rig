#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment49
cargo run --release -- increment49 --out artifacts/increment49
./scripts/increment49-threejs.sh

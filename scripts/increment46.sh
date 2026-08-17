#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment46
cargo run --release -- increment46 --out artifacts/increment46
./scripts/increment46-threejs.sh

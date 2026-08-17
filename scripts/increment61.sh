#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment61
cargo run --release -- increment61 --out artifacts/increment61
./scripts/increment61-threejs.sh

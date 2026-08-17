#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment55
cargo run --release -- increment55 --out artifacts/increment55
./scripts/increment55-threejs.sh

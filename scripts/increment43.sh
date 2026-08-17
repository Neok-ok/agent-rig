#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment43
cargo run --release -- increment43 --out artifacts/increment43
./scripts/increment43-threejs.sh

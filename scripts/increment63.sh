#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment63
cargo run --release -- increment63 --out artifacts/increment63
./scripts/increment63-threejs.sh

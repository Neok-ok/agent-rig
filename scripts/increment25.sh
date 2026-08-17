#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment25
cargo run --release -- increment25 --out artifacts/increment25
./scripts/increment25-threejs.sh

#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment41
cargo run --release -- increment41 --out artifacts/increment41
./scripts/increment41-threejs.sh

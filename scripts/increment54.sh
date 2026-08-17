#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment54
cargo run --release -- increment54 --out artifacts/increment54
./scripts/increment54-threejs.sh

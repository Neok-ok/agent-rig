#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment35
cargo run --release -- increment35 --out artifacts/increment35
./scripts/increment35-threejs.sh

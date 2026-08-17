#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment53
cargo run --release -- increment53 --out artifacts/increment53
./scripts/increment53-threejs.sh

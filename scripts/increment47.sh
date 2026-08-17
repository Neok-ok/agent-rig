#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment47
cargo run --release -- increment47 --out artifacts/increment47
./scripts/increment47-threejs.sh

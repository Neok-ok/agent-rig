#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment16
cargo run --release -- increment16 --out artifacts/increment16
./scripts/increment16-threejs.sh

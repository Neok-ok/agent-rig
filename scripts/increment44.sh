#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment44
cargo run --release -- increment44 --out artifacts/increment44
./scripts/increment44-threejs.sh

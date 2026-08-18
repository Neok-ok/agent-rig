#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment66
cargo run --release -- increment66 --out artifacts/increment66
./scripts/increment66-threejs.sh

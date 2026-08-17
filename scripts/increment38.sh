#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment38
cargo run --release -- increment38 --out artifacts/increment38
./scripts/increment38-threejs.sh

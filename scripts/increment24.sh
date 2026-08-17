#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment24
cargo run --release -- increment24 --out artifacts/increment24
./scripts/increment24-threejs.sh

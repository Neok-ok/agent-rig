#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment60
cargo run --release -- increment60 --out artifacts/increment60
./scripts/increment60-threejs.sh

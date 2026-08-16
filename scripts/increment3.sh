#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment3
cargo run --release -- increment3 --out artifacts/increment3

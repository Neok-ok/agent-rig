#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts/increment2
cargo run --release -- increment2 --out artifacts/increment2

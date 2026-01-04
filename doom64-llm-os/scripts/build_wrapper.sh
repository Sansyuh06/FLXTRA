#!/bin/bash
export PATH="$HOME/.cargo/bin:$PATH"
cd "$(dirname "$0")/.."
./scripts/build_iso.sh

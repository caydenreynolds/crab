#!/bin/sh
set -e

echo "Building c builtins"
cd ../c/ || exit
./scripts/rtccrabc.sh

echo "Building rust project -- parser and compiler"
cd ../rust || exit
cargo build "$@"

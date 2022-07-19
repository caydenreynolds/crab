#!/bin/bash
set -e

script_dir=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
echo "Script directory: ${script_dir}"

echo "Building c builtins"
pushd "${script_dir}/../c/"
./scripts/rtccrabc.sh
popd

echo "Building rust project -- parser and compiler"
pushd "${script_dir}/../rust/"
cargo build "$@"
popd

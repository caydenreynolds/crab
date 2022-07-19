#!/bin/sh

script_dir=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
echo "Script directory: ${script_dir}"

mkdir "${script_dir}/target/"
set -e
"${script_dir}/../rust/target/debug/print_ast.exe" "$@" ../std/ > "${script_dir}/target/out.ast"

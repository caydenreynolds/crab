#!/bin/sh

mkdir target/
set -e
../rust/target/debug/print_ast.exe "$@" ../std/ > target/out.ast

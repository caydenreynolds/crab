#!/bin/sh

mkdir target/

set -e

../rust/target/debug/blue.exe "$@" ../std/
# FIXME: blue just outputs whatever, wherever, so we gotta move it ourselves
mv out.ll target/out.ll

llc \
  --stats \
  -O0 \
  -o target/out.s \
  target/out.ll

clang \
  -v \
  -L ../c/target/ \
  -l crabcbuiltins \
  -o target/out.exe \
  target/out.s

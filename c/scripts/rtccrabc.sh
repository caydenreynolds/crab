#!/bin/sh

mkdir target

set -e

# FIXME: Hardcoded .lib extension because I'm on windows right now
echo "Building crabcbuiltins.ll"
clang \
  -c \
  -v \
  -Wall \
  -o target/crabcbuiltins.ll \
  -S \
  -emit-llvm \
  src/*

echo "Building crabcbuiltins.lib"
clang \
  -v \
  -c \
  -Wall \
  -O3 \
  -o target/crabcbuiltins.lib \
  src/*

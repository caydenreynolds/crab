#!/bin/sh

mkdir target

set -e

# FIXME: Hardcoded .lib extension because I'm on windows right now
echo "Building crabbuiltins.ll"
clang \
  -c \
  -v \
  -Wall \
  -o target/crabbuiltins.ll \
  -S \
  -emit-llvm \
  src/*

echo "Building crabbuiltins.lib"
clang \
  -v \
  -c \
  -Wall \
  -O3 \
  -o target/crabbuiltins.lib \
  src/*

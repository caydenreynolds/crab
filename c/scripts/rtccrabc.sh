#!/bin/sh

mkdir target

set -e

# FIXME: Hardcoded .lib extension because I'm on windows right now
clang \
  -v \
  -c \
  -Wall \
  -o target/crabcbuiltins.lib \
  src/*

clang \
  -c \
  -v \
  -Wall \
  -o target/crabcbuiltins.ll \
  -S \
  -emit-llvm \
  src/*

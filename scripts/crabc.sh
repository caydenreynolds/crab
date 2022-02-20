#!/bin/sh

../rust/target/debug/blue.exe $@
llc --stats out.ll -O0 -o out.s
clang -v out.s -o out.exe

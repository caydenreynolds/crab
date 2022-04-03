# Crab Programming Language

Crab is a programming language I've been developing in my spare time.
This project is in a very early stage of development, but plans to include a complete set of modern programming language features.

## Features

To see a demonstration of all the currently implemented features, check out the test/ directory

### Why name it Crab?

The project is named Crab because crabs are the most evolved creature on our beautiful blue planet.
The Crab programming language wishes to continue this legacy by becoming the most evolved programming language on the face of the Earth.

## LLVM Windows Build (For Future Cayden)

* Don't bother with llvmenv
* Build source from llvm website
* Version 13.0.0
* Make sure clang is also built, using the `cmake -DLLVM_ENABLE_PROJECTS=clang -DLLVM_ENABLE_PROJECTS=lld ../llvm` command
  * Followed by `cmake --build .`
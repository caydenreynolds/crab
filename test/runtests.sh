#!/bin/bash

# Usage: ./runtest.sh [options]
# Options:
#    -v --verbose: More output to console

# Parse inputs
verbose="FALSE"
while [[ $# -gt 0 ]]; do
  case $1 in
    -v|--verbose)
      verbose="TRUE"
      shift # past argument
      ;;
  esac
done


# Clean out any old binaries there may have been
rm -rf target
mkdir target

source crab_env/Scripts/activate
./../scripts/buildall.sh --release

# FIXME: Shell is hardcoded for windows
declare -A rel_paths=(["CBUILTINS_DIR"]="../c/target" ["CRABC"]="../rust/target/release/crabc.exe" ["CRAB_SRC"]="crab" ["TARGET_DIR"]="target" ["CRAB_STD"]="../std")
declare -A actual_paths
for key in "${!rel_paths[@]}"
    do
        rel="${rel_paths[$key]}"
        real=$(realpath "$rel")
        win=$(cygpath -d "$real")
        actual_paths["$key"]="$win"
    done

robot \
  -d target/ \
  --variable CBUILTINS_DIR:"${actual_paths[CBUILTINS_DIR]}" \
  --variable CRABC:"${actual_paths[CRABC]}" \
  --variable CRAB_SRC:"${actual_paths[CRAB_SRC]}" \
  --variable TARGET_DIR:"${actual_paths[TARGET_DIR]}" \
  --variable CRAB_STD:"${actual_paths[CRAB_STD]}" \
  --variable VERBOSE:"$verbose" \
  robot
start target/report.html

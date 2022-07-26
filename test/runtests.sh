#!/bin/bash

# Usage: ./runtest.sh [options]
# Options:
#    -v --verbose: More output to console
#    -n --no-display: Don't immediately display generated html file
# Assumes your cwd is the same directory as this script is located in

# Parse inputs
verbose="FALSE"
display_after="TRUE"
while [[ $# -gt 0 ]]; do
  case $1 in
    -v|--verbose)
      verbose="TRUE"
      shift # past argument
      ;;
    -n|--no-display)
      display_after="FALSE"
      shift
      ;;
  esac
done

# Clean out any old binaries there may have been
rm -rf target
mkdir target

source crab_env/Scripts/activate
./../tools/buildall.sh --release

# FIXME: Shell is hardcoded for windows
declare -A rel_paths=(
  ["CBUILTINS_DIR"]="../c/target"
  ["CRABC"]="../rust/target/release/crabc.exe"
  ["CRAB_SRC"]="crab"
  ["TARGET_DIR"]="target"
  ["CRAB_STD"]="../std"
  ["RESOURCES"]="resources"
)
declare -A actual_paths
for key in "${!rel_paths[@]}"
    do
        rel="${rel_paths[$key]}"
        real=$(realpath "$rel")
        win=$(cygpath -d "$real")
        actual_paths["$key"]="$win"
    done

export PYTHONPATH="robot/libraries"

robot \
  -d target/ \
  --variable CBUILTINS_DIR:"${actual_paths[CBUILTINS_DIR]}" \
  --variable CRABC:"${actual_paths[CRABC]}" \
  --variable CRAB_SRC:"${actual_paths[CRAB_SRC]}" \
  --variable TARGET_DIR:"${actual_paths[TARGET_DIR]}" \
  --variable CRAB_STD:"${actual_paths[CRAB_STD]}" \
  --variable VERBOSE:"$verbose" \
  robot

if [[ "$display_after" == "TRUE" ]]; then
  start target/report.html
fi

#!/bin/bash

# Usage:
#  remote_build.sh [options]
#
#  Options:
#  -t --test : Run the test scripts
#
#  This script requires that the environment variable CRAB_BUILD_HOST is set

if [[ "$1" == "-t" ]] || [[ "$1" == "--test" ]]; then
  test="true"
else
  test="false"
fi

script_dir=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
echo "Script directory: ${script_dir}"
pushd "$script_dir"

# I'm looking for this literal asterisk character, not trying to glob
# shellcheck disable=SC2063
branch="$(git branch | grep "*")"
branch="${branch:2}"
echo "On branch $branch"

if [[ "$branch" == "main" ]]; then
  echo "Skipping push because we're on main"
else
  echo "Pushing"
  pushd "../"
  git add .
  git commit -m "Autopush from remote_build.sh"
  git push --set-upstream origin "$branch"
  echo "Wait for the push to complete"
  sleep 5
  popd
fi

# Yes, this is extremely specific to me
# No, I won't fix it
host_cmd="
              pushd ~/projects/crab
              git checkout $branch
              git pull
            "

if [[ "$test" == "true" ]]; then
  echo "Running with robot tests"
  host_cmd="
                ${host_cmd}
                pushd test
                ./runtests.sh -n
                tar -ac -f target.tar.gz target/
                popd
              "
else
  echo "Running without robot tests"
  host_cmd="
                ${host_cmd}
                ./tools/buildall.sh
              "
fi
ssh "$CRAB_BUILD_HOST" "$host_cmd"

if [[ "$test" == "true" ]]; then
  sftp "${CRAB_BUILD_HOST}:projects/crab/test/target.tar.gz"
  tar -xf target.tar.gz
  start target/report.html
fi

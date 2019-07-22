#!/usr/bin/env bash

ROOT=`dirname "$0"`

# A list of directories which contain wasm projects.
SRCS=(

)

# Make pushd/popd silent.

pushd () {
	command pushd "$@" > /dev/null
}

popd () {
	command popd "$@" > /dev/null
}

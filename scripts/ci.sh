#!/usr/bin/env bash

set -eux

sudo apt-get -y update
sudo apt-get install -y cmake pkg-config libssl-dev

# Install rustup and the specified rust toolchain.
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain=stable -y

# Load cargo environment. Specifically, put cargo into PATH.
source ~/.cargo/env

rustc --version
rustup --version
cargo --version

./scripts/init.sh
./scripts/build.sh

cargo test --all --locked

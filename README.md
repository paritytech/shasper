# Substrate Shasper

[![Build Status](https://travis-ci.org/paritytech/shasper.svg?branch=master)](https://travis-ci.org/paritytech/shasper)

*Note: This is an experimental project. Everything will break, and it may disappear without any notice!*

This is an implementation of [Shasper](https://github.com/ethereum/eth2.0-specs) beacon chain using the [Substrate framework](https://github.com/paritytech/substrate).

## Status

Currently we have a (mostly complete, but untested) implementation of Shasper state transition validation algorithms. This is then combined with Substrate's embedded consensus engine to provide a simple Substrate node implementation. In the future, this consensus engine will be replaced to comply with Shasper's fork choice rule specification.

## Get Started

To build the project, first install [rustup](https://rustup.rs/) and [wasm-gc](https://github.com/alexcrichton/wasm-gc):

```
rustup update stable
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
cargo +nightly install --git https://github.com/alexcrichton/wasm-gc
```

After that, compile the WebAssembly runtime:

```
cd runtime/wasm && ./build.sh && cd ../..
```

You can then execute the client:

```
cargo run -- --chain dev
```

However, before the block authoring logic is added, there's probably not much you can do!

## License

Licensed under GPLv3.



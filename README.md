# Parity Shasper

[![crates.io](https://img.shields.io/crates/v/beacon.svg)](https://crates.io/crates/beacon)
[![Documentation](https://docs.rs/beacon/badge.svg)](https://docs.rs/beacon)

This is an implementation of
[Serenity](https://github.com/ethereum/eth2.0-specs) beacon chain by Parity
Technologies. To learn more about Serenity and Ethereum's sharding plan, see the
[sharding FAQ](https://github.com/ethereum/wiki/wiki/Sharding-FAQ) and the
[research compendium](https://notes.ethereum.org/s/H1PGqDhpm).

Parity Shasper consists of a core library `beacon` which handles all beacon
chain state transition logic, a client built on
[Substrate](https://github.com/paritytech/substrate) framework (in `substrate`
folder), and a lightweight client built from ground up (in `blockchain`
folder). The `substrate` client and the `blockchain` client shares the core
library, but operates independently.

To build the client, you need to have [Rust](https://www.rust-lang.org/)
installed. Other dependencies required including `pkgconfig`, `libudev`,
`openssl`, `cmake`, `clang`.

## `substrate` client

The `substrate` client uses Aura consensus and Serenity `beacon` v0.6
runtime. The client also inherents the complete networking stack from
Substrate. Currently, the client can only validate blocks and cannot be a
validator.

To build the `blockchain` client, it is recommended that you use
[rustup](https://rustup.rs) as we need both stable and nightly Rust to build the
project. In addition, you need to have
[wasm-gc](https://github.com/alexcrichton/wasm-gc) installed for the WebAssembly
runtime.

```bash
rustup update stable
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
cargo +nightly install --git https://github.com/alexcrichton/wasm-gc
```

To compile the WebAssembly runtime and run the client:

```bash
cd ./substrate && ./build.sh
cargo run --release -- --dev -k Alice
```

## `blockchain` client

The `blockchain` client uses spec archive LMD-GHOST consensus and Serenity
`beacon` v0.6 runtime. The client implements a basic in-memory backend and
networking stack based on `libp2p`. It also contains basic validator logic and
can participate in beacon chain proposing and attestation.

To build the `blockchain` client:

```bash
cd ./blockchain && cargo run --release -- --author
```

## FAQ

**I remember the `substrate` client has LMD-GHOST consensus with proposing and
attestation logic, why isn't it there now?**

The previous LMD-GHOST consensus logic and block proposer was no longer suitable
for current Serenity beacon chain specification. As a result we temporarily fall
back to use Aura consensus. It will be added back in the short-term future.

**Why common caching strategies for `beacon` are not yet implemented?**

Internally we made the decision that we will strictly follow the beacon chain
specification for now, and implement optimizations after the specification is
frozen. This is because the specification still changes a lot, and we worry that
optimizations we make right now will make upgrading to a newer version of the
specification much harder.

## License

Licensed under GPLv3.

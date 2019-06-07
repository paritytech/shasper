#!/usr/bin/env bash

set -euo pipefail

cargo run --release -- --config full -r attestation res/spectests/tests/operations/attestation/attestation_mainnet.yaml
cargo run --release -- --config full -r attester_slashing res/spectests/tests/operations/attester_slashing/attester_slashing_mainnet.yaml
cargo run --release -- --config full -r block_header res/spectests/tests/operations/block_header/block_header_mainnet.yaml
cargo run --release -- --config full -r deposit res/spectests/tests/operations/deposit/deposit_mainnet.yaml
cargo run --release -- --config full -r proposer_slashing res/spectests/tests/operations/proposer_slashing/proposer_slashing_mainnet.yaml
cargo run --release -- --config full -r transfer res/spectests/tests/operations/transfer/transfer_mainnet.yaml

cargo run --release -- --config full -r crosslinks res/spectests/tests/epoch_processing/crosslinks/crosslinks_mainnet.yaml
cargo run --release -- --config full -r registry_updates res/spectests/tests/epoch_processing/registry_updates/registry_updates_mainnet.yaml

cargo run --release -- --config full -r blocks res/spectests/tests/sanity/blocks/blocksanity_s_mainnet.yaml
cargo run --release -- --config full -r slots res/spectests/tests/sanity/slots/slotsanity_s_mainnet.yaml

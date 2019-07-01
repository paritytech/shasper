//! Non-configurable constants used throughout the specification.

use crate::primitives::Uint;

/// Far future epoch.
pub const FAR_FUTURE_EPOCH: Uint = Uint::max_value();
/// Base rewards per epoch.
pub const BASE_REWARDS_PER_EPOCH: Uint = 5;
/// Deposit contract tree depth.
pub const DEPOSIT_CONTRACT_TREE_DEPTH: Uint = 32;
/// Seconds per day.
pub const SECONDS_PER_DAY: Uint = 86400;
/// Justification bits length;
pub const JUSTIFICATION_BITS_LENGTH: Uint = 4;

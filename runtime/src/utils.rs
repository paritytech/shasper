use primitives::H256;
use runtime_primitives::traits::{DigestItem as DigestItemT};
use rstd::prelude::*;

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum Never { }

impl DigestItemT for Never {
	type Hash = H256;
	type AuthorityId = Never;
}

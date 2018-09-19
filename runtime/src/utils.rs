use primitives::H256;
use runtime_primitives::traits::{DigestItem as DigestItemT};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Never { }

impl DigestItemT for Never {
	type Hash = H256;
	type AuthorityId = Never;
}

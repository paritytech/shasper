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

pub fn split(list: Vec<usize>, n: usize) -> Vec<Vec<usize>> {
	let mut ret = Vec::new();
	for i in 0..n {
		let cur = list[(list.len() * i / n)..(list.len() * (i + 1) / n)]
			.iter().cloned().collect();
		ret.push(cur);
	}
	ret
}

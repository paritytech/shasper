macro_rules! impl_beacon_fixed_hash {
	( $t:tt, $size:expr, $size_t:ty ) => {
		#[cfg(feature = "serde")]
		use serde::{Serialize, Serializer, Deserialize, Deserializer};
		use fixed_hash::construct_fixed_hash;
		#[cfg(feature = "serde")]
		use impl_serde::serialize as bytes;
		use bm_le::{FixedVec, FixedVecRef, ElementalFixedVecRef, ElementalFixedVec,
					IntoVectorTree, FromVectorTree};
		use core::marker::PhantomData;

		const SIZE: usize = $size;

		construct_fixed_hash! {
			/// Fixed 384-bit hash.
			pub struct $t(SIZE);
		}

		impl Serialize for $t {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
				bytes::serialize(&self.0, serializer)
			}
		}

		impl<'de> Deserialize<'de> for $t {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
				bytes::deserialize_check_len(deserializer, bytes::ExpectedLen::Exact(SIZE))
					.map(|x| <$t>::from_slice(&x))
			}
		}

		ssz::impl_composite_known_size!($t, Some(SIZE));
		ssz::impl_decode_with_empty_config!($t);

		impl ssz::Encode for $t {
			fn encode(&self) -> Vec<u8> {
				FixedVecRef::<u8, $size_t>(&self.0, PhantomData).encode()
			}
		}

		impl ssz::Decode for $t {
			fn decode(value: &[u8]) -> Result<Self, ssz::Error> {
				Ok(<$t>::from_slice(&FixedVec::<u8, $size_t>::decode(value)?.0))
			}
		}

		impl Into<primitive_types::H256> for $t {
			fn into(self) -> primitive_types::H256 {
				primitive_types::H256::from_slice(&self[0..32])
			}
		}

		impl bm_le::Composite for $t { }

		impl<DB> bm_le::IntoTree<DB> for $t where
			DB: bm_le::Backend<Intermediate=bm_le::Intermediate, End=bm_le::End>
		{
			fn into_tree(&self, db: &mut DB) -> Result<bm_le::ValueOf<DB>, bm_le::Error<DB::Error>> {
				ElementalFixedVecRef(&self.0.as_ref()).into_vector_tree(db, None)
			}
		}

		bm_le::impl_from_tree_with_empty_config!($t);
		impl<DB> bm_le::FromTree<DB> for $t where
			DB: bm_le::Backend<Intermediate=bm_le::Intermediate, End=bm_le::End>
		{
			fn from_tree(root: &bm_le::ValueOf<DB>, db: &DB) -> Result<Self, bm_le::Error<DB::Error>> {
				let value = ElementalFixedVec::<u8>::from_vector_tree(root, db, SIZE, None)?;
				Ok(Self::from_slice(value.0.as_ref()))
			}
		}

	}
}

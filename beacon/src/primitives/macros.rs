macro_rules! impl_beacon_fixed_hash {
	( $t:tt, $size:expr, $size_t:ty ) => {
		#[cfg(feature = "serde")]
		use serde::{Serialize, Serializer, Deserialize, Deserializer};
		use fixed_hash::construct_fixed_hash;
		#[cfg(feature = "serde")]
		use impl_serde::serialize as bytes;
		use bm_le::{ElementalFixedVecRef, ElementalFixedVec,
					IntoCompactVectorTree, FromCompactVectorTree, Compact, CompactRef};
		use generic_array::GenericArray;

		const SIZE: usize = $size;

		construct_fixed_hash! {
			/// Fixed 384-bit hash.
			pub struct $t(SIZE);
		}

		#[cfg(feature = "serde")]
		impl Serialize for $t {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
				bytes::serialize(&self.0, serializer)
			}
		}

		#[cfg(feature = "serde")]
		impl<'de> Deserialize<'de> for $t {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
				bytes::deserialize_check_len(deserializer, bytes::ExpectedLen::Exact(SIZE))
					.map(|x| <$t>::from_slice(&x))
			}
		}

		#[cfg(feature = "parity-codec")]
		impl parity_codec::Encode for $t {
			fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
				self.0.using_encoded(f)
			}
		}

		#[cfg(feature = "parity-codec")]
		impl parity_codec::Decode for $t {
			fn decode<I: parity_codec::Input>(input: &mut I) -> Option<Self> {
				<[u8; SIZE] as parity_codec::Decode>::decode(input).map($t)
			}
		}

		impl ssz::Codec for $t {
			type Size = $size_t;
		}

		impl ssz::Encode for $t {
			fn encode(&self) -> Vec<u8> {
				CompactRef(GenericArray::<u8, $size_t>::from_slice(&self.0)).encode()
			}
		}

		impl ssz::Decode for $t {
			fn decode(value: &[u8]) -> Result<Self, ssz::Error> {
				Ok(<$t>::from_slice(Compact::<GenericArray<u8, $size_t>>::decode(value)?.0.as_slice()))
			}
		}

		impl Into<primitive_types::H256> for $t {
			fn into(self) -> primitive_types::H256 {
				primitive_types::H256::from_slice(&self[0..32])
			}
		}

		impl bm_le::IntoTree for $t {
			fn into_tree<DB: bm_le::WriteBackend>(&self, db: &mut DB) -> Result<bm_le::ValueOf<DB::Construct>, bm_le::Error<DB::Error>> where
				DB::Construct: bm_le::CompatibleConstruct
			{
				ElementalFixedVecRef(&self.0.as_ref()).into_compact_vector_tree(db, None)
			}
		}

		impl bm_le::FromTree for $t {
			fn from_tree<DB: bm_le::ReadBackend>(root: &bm_le::ValueOf<DB::Construct>, db: &mut DB) -> Result<Self, bm_le::Error<DB::Error>> where
				DB::Construct: bm_le::CompatibleConstruct
			{
				let value = ElementalFixedVec::<u8>::from_compact_vector_tree(root, db, SIZE, None)?;
				Ok(Self::from_slice(value.0.as_ref()))
			}
		}

	}
}

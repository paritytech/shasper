use crate::{Encode, Decode, Error, LengthOffset};
use alloc::vec::Vec;
use alloc::collections::VecDeque;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum SeriesItem {
	Fixed(Vec<u8>),
	Variable(Vec<u8>),
}

#[derive(Default, Eq, PartialEq, Clone, Debug)]
pub struct Series(pub Vec<SeriesItem>);

impl Series {
	pub fn encode(&self) -> Vec<u8> {
		let mut ret = Vec::new();

		let fixed_parts_size = self.0.iter().fold(0, |acc, part| {
			acc + match part {
				SeriesItem::Fixed(ref fixed) => fixed.len(),
				SeriesItem::Variable(_) => LengthOffset::default()
					.using_encoded(|buf| buf.len()),
			}
		});

		let mut offset = fixed_parts_size;

		for part in &self.0 {
			match part {
				SeriesItem::Fixed(ref fixed) => {
					ret.extend_from_slice(fixed);
				},
				SeriesItem::Variable(ref variable) => {
					(offset as LengthOffset).using_encoded(|buf| ret.extend_from_slice(buf));
					offset += variable.len();
				},
			}
		}

		for part in &self.0 {
			match part {
				SeriesItem::Fixed(_) => (),
				SeriesItem::Variable(ref variable) => {
					ret.extend_from_slice(variable);
				},
			}
		}

		ret
	}

	pub fn decode_vector(value: &[u8], typs: &[Option<usize>]) -> Result<Self, Error> {
		let mut ret = Vec::new();
		let mut variable_offsets = VecDeque::new();

		let mut pos = 0;
		for typ in typs {
			match typ {
				Some(fixed_len) => {
					ret.push(
						SeriesItem::Fixed(value[pos..(pos + fixed_len)].to_vec())
					);
					pos += fixed_len;
				},
				None => {
					ret.push(SeriesItem::Variable(Default::default()));
					let len = LengthOffset::default().using_encoded(|buf| buf.len());
					variable_offsets.push_back(LengthOffset::decode(&value[pos..(pos + len)])? as usize);
					pos += len;
				},
			}
		}

		for part in &mut ret {
			match part {
				SeriesItem::Fixed(_) => (),
				SeriesItem::Variable(ref mut part) => {
					let offset = variable_offsets.pop_front().expect(
						"One variable offset is pushed with one variable item inserted; qed"
					) as usize;
					let next_offset = variable_offsets.front().map(|v| *v as usize)
						.unwrap_or(value.len());

					part.extend_from_slice(&value[offset..next_offset]);
				},
			}
		}

		Ok(Self(ret))
	}

	pub fn decode_list(value: &[u8], typ: Option<usize>) -> Result<Self, Error> {
		let mut ret = Vec::new();

		match typ {
			Some(fixed_len) => {
				let mut pos = 0;

				while pos + fixed_len <= value.len() {
					ret.push(
						SeriesItem::Fixed(value[pos..(pos + fixed_len)].to_vec())
					);
					pos += fixed_len;
				}
			},
			None => {
				let mut pos = 0;
				let mut variable_offsets = VecDeque::new();
				let fixed_len = LengthOffset::default().using_encoded(|buf| buf.len());

				while pos + fixed_len <= value.len() &&
					variable_offsets.front().map(|f| pos + fixed_len <= *f).unwrap_or(true)
				{
					variable_offsets.push_back(LengthOffset::decode(&value[pos..(pos + fixed_len)])? as usize);
					pos += fixed_len;
				}

				while let Some(offset) = variable_offsets.pop_front() {
					let offset = offset as usize;
					let next_offset = variable_offsets.front().map(|v| *v as usize)
						.unwrap_or(value.len());

					ret.push(
						SeriesItem::Variable(value[offset..next_offset].to_vec())
					);
				}
			},
		}

		Ok(Self(ret))
	}
}

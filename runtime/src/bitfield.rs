use std::ops::BitOr;

pub struct BitField(Vec<u8>, usize);

impl BitField {
	pub fn has_voted(&self, index: usize) -> bool {
		assert!(index < self.1);
		self.0[index / 8] & (128 >> (index % 8)) == 1
	}

	pub fn set_voted(&mut self, index: usize) {
		assert!(index < self.1);
		let byte_index = index / 8;
		let bit_index = index % 8;
		self.0[byte_index] = self.0[byte_index] | (128 >> bit_index);
	}

	pub fn new(count: usize) -> Self {
		let byte_len = (count + 7) / 8;
		let mut payload = Vec::with_capacity(byte_len);
		payload.resize(byte_len, 0);
		BitField(payload, count)
	}

	pub fn count(&self) -> usize {
		self.1
	}

	pub fn vote_count(&self) -> usize {
		let mut votes = 0;
		for i in 0..self.1 {
			if self.has_voted(i) {
				votes += 1;
			}
		}
		votes
	}
}

impl BitOr for BitField {
	type Output = Self;

	fn bitor(self, rhs: Self) -> Self {
		let mut new = BitField::new(::std::cmp::max(self.count(), rhs.count()));
		for i in 0..::std::cmp::max(self.count(), rhs.count()) {
			let mut voted = false;
			if i < self.count() {
				voted = voted || self.has_voted(i);
			}
			if i < rhs.count() {
				voted = voted || rhs.has_voted(i);
			}
			if voted {
				new.set_voted(i);
			}
		}
		new
	}
}

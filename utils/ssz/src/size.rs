use core::ops::Add;
use typenum::Unsigned;

/// Indicate the size type of the current ssz value.
pub trait Size {
	/// Whether the value is fixed sized.
	fn is_fixed() -> bool { !Self::is_variable() }
	/// Whether the value is variable sized.
	fn is_variable() -> bool { !Self::is_fixed() }
	/// The actual size of the value.
	fn size() -> Option<usize>;
}

impl<U: Unsigned> Size for U {
	fn is_fixed() -> bool { true }
	fn size() -> Option<usize> { Some(Self::to_usize()) }
}

/// A plain variable sized value.
pub struct VariableSize;

impl Size for VariableSize {
	fn is_variable() -> bool { true }
	fn size() -> Option<usize> { None }
}

pub trait Sum<Rhs> {
	type Output: Size;
}

impl<U: Unsigned> Sum<VariableSize> for U {
	type Output = VariableSize;
}

impl<U: Unsigned> Sum<U> for VariableSize {
	type Output = VariableSize;
}

impl<U: Unsigned, V: Unsigned + Add<U>> Sum<U> for V where
	<V as Add<U>>::Output: Size
{
	type Output = <V as Add<U>>::Output;
}

impl Sum<VariableSize> for VariableSize {
	type Output = VariableSize;
}

#[macro_export]
macro_rules! sum {
	( $one: ty ) => ( $one );
	( $first: ty, $( $rest:ty ),* ) => (
		<$first as $crate::Sum<$crate::sum!($( $rest ),*)>>::Output
	);
}

#[cfg(test)]
mod tests {
	use crate::{VariableSize, Size, Codec};
	use typenum::*;

	type Simple1 = crate::sum!(U1);
	type Simple2 = crate::sum!(U1, U2);
	type Simple3 = crate::sum!(U1, VariableSize, U2);

	#[test]
	fn test_simple() {
		assert!(Simple1::is_fixed());
		assert_eq!(Simple1::size(), Some(1));
		assert!(Simple2::is_fixed());
		assert_eq!(Simple2::size(), Some(3));
		assert!(Simple3::is_variable());
		assert_eq!(Simple3::size(), None);
	}

	type Combined = crate::sum!(<u8 as Codec>::Size,
								<u16 as Codec>::Size,
								<u32 as Codec>::Size);

	#[test]
	fn test_combined() {
		assert!(Combined::is_fixed());
		assert_eq!(Combined::size(), Some(7));
	}
}

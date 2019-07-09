use core::marker::PhantomData;

/// Indicate the size type of the current ssz value.
pub trait Size {
	/// Whether the value is fixed sized.
	fn is_fixed() -> bool { !Self::is_variable() }
	/// Whether the value is variable sized.
	fn is_variable() -> bool { !Self::is_fixed() }
}

pub trait Eval {
	type Output: Size;
}

/// A plain fixed sized value.
pub struct FixedSize;

impl Size for FixedSize {
	fn is_fixed() -> bool { true }
}

impl Eval for FixedSize {
	type Output = FixedSize;
}

/// A plain variable sized value.
pub struct VariableSize;

impl Size for VariableSize {
	fn is_variable() -> bool { true }
}

impl Eval for VariableSize {
	type Output = VariableSize;
}

pub struct OrVariable<A: Size, B: Size>(PhantomData<(A, B)>);

impl Eval for OrVariable<FixedSize, FixedSize> {
	type Output = FixedSize;
}

impl Eval for OrVariable<FixedSize, VariableSize> {
	type Output = VariableSize;
}

impl Eval for OrVariable<VariableSize, FixedSize> {
	type Output = VariableSize;
}

impl Eval for OrVariable<VariableSize, VariableSize> {
	type Output = FixedSize;
}

#[macro_export]
macro_rules! or_variable {
	( $one: ty ) => ( $one );
	( $first: ty, $( $rest:ty ),* ) => (
		<$crate::size::OrVariable<$first, $crate::or_variable!($( $rest ),*)> as
			$crate::size::Eval>::Output
	);
}

#[cfg(test)]
mod tests {
	use crate::{FixedSize, VariableSize, Size, Codec};

	type Simple1 = crate::or_variable!(FixedSize);
	type Simple2 = crate::or_variable!(FixedSize, FixedSize);
	type Simple3 = crate::or_variable!(FixedSize, VariableSize, FixedSize);

	#[test]
	fn test_simple() {
		assert!(Simple1::is_fixed());
		assert!(Simple2::is_fixed());
		assert!(Simple3::is_variable());
	}

	type Combined = crate::or_variable!(<u8 as Codec>::Size,
										<u16 as Codec>::Size,
										<u32 as Codec>::Size);

	#[test]
	fn test_combined() {
		assert!(Combined::is_fixed());
	}
}

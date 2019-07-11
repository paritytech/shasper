use typenum::Unsigned;

/// Indicate the size type of the current ssz value.
pub trait Size {
	/// Whether the value is fixed sized.
	fn is_fixed() -> bool { Self::size().is_some() }
	/// Whether the value is variable sized.
	fn is_variable() -> bool { Self::size().is_none() }
	/// The actual size of the value.
	fn size() -> Option<usize>;
}

impl<U: Unsigned> Size for U {
	fn size() -> Option<usize> { Some(Self::to_usize()) }
}

/// A plain variable sized value.
pub struct VariableSize;

impl Size for VariableSize {
	fn size() -> Option<usize> { None }
}

pub trait Add<Rhs> {
	type Output: Size;
}

impl<U: Unsigned> Add<VariableSize> for U {
	type Output = VariableSize;
}

impl<U: Unsigned> Add<U> for VariableSize {
	type Output = VariableSize;
}

impl<U: Unsigned, V: Unsigned + core::ops::Add<U>> Add<U> for V where
	<V as core::ops::Add<U>>::Output: Size
{
	type Output = <V as core::ops::Add<U>>::Output;
}

impl Add<VariableSize> for VariableSize {
	type Output = VariableSize;
}

pub trait Mul<Rhs> {
	type Output: Size;
}

impl<U: Unsigned> Mul<VariableSize> for U {
	type Output = VariableSize;
}

impl<U: Unsigned> Mul<U> for VariableSize {
	type Output = VariableSize;
}

impl<U: Unsigned, V: Unsigned + core::ops::Mul<U>> Mul<U> for V where
	<V as core::ops::Mul<U>>::Output: Size
{
	type Output = <V as core::ops::Mul<U>>::Output;
}

impl Mul<VariableSize> for VariableSize {
	type Output = VariableSize;
}

pub trait Div<Rhs> {
	type Output: Size;
}

impl<U: Unsigned> Div<VariableSize> for U {
	type Output = VariableSize;
}

impl<U: Unsigned> Div<U> for VariableSize {
	type Output = VariableSize;
}

impl<U: Unsigned, V: Unsigned + core::ops::Div<U>> Div<U> for V where
	<V as core::ops::Div<U>>::Output: Size
{
	type Output = <V as core::ops::Div<U>>::Output;
}

impl Div<VariableSize> for VariableSize {
	type Output = VariableSize;
}

#[macro_export]
macro_rules! sum {
	( $one: ty ) => ( $one );
	( $first: ty, $( $rest:ty ),* ) => (
		<$first as $crate::Add<$crate::sum!($( $rest ),*)>>::Output
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

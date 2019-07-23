use core::marker::PhantomData;
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

// The Rust's type system is not powerful enough at this moment to carry out
// calculations, so we temporarily just calculate stuff like this. Note that
// unless we have const generics, this is probably the most efficient we can
// get.

/// Add `A` and `B`, where `A` and `B` are both `Size`.
pub struct Add<A: Size, B: Size>(PhantomData<(A, B)>);

impl<A: Size, B: Size> Size for Add<A, B> {
	fn size() -> Option<usize> {
		match (A::size(), B::size()) {
			(Some(a), Some(b)) => Some(a + b),
			_ => None,
		}
	}
}

/// Multiply `A` and `B`, where `A` and `B` are both `Size`.
pub struct Mul<A: Size, B: Size>(PhantomData<(A, B)>);

impl<A: Size, B: Size> Size for Mul<A, B> {
	fn size() -> Option<usize> {
		match (A::size(), B::size()) {
			(Some(a), Some(b)) => Some(a * b),
			_ => None,
		}
	}
}

/// Divide `A` by `B`, where `A` and `B` are both `Size`.
pub struct Div<A: Size, B: Size>(PhantomData<(A, B)>);

impl<A: Size, B: Size> Size for Div<A, B> {
	fn size() -> Option<usize> {
		match (A::size(), B::size()) {
			(Some(a), Some(b)) => Some(a / b),
			_ => None,
		}
	}
}

#[macro_export]
/// Shortcut for adding more than two sizes together.
macro_rules! sum {
	( $one:ty ) => ( $one );
	( $first:ty, $( $rest:ty ),* ) => (
		$crate::Add<$first, $crate::sum!($( $rest ),*)>
	);
}

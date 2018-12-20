pub extern crate bls_aggregates;

pub mod bls {
	pub use bls_aggregates::AggregatePublicKey as AggregatePublic;
	pub use bls_aggregates::AggregateSignature;
	pub use bls_aggregates::Keypair as Pair;
	pub use bls_aggregates::PublicKey as Public;
	pub use bls_aggregates::SecretKey as Secret;
	pub use bls_aggregates::Signature;
}

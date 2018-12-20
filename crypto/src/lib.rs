pub extern crate bls_aggregates;

mod bls {
	pub use bls_aggregates::AggregatePublicKey;
	pub use bls_aggregates::AggregateSignature;
	pub use bls_aggregates::Keypair as Pair;
	pub use bls_aggregates::PublicKey;
	pub use bls_aggregates::SecretKey;
	pub use bls_aggregates::Signature;
}

pub extern crate bls;
pub extern crate pairing;

use pairing::bls12_381::Bls12;

pub type BlsPair = bls::Keypair<Bls12>;
pub type BlsPublicKey = bls::PublicKey<Bls12>;
pub type BlsSignature = bls::Signature<Bls12>;
pub type BlsAggregateSignature = bls::AggregateSignature<Bls12>;

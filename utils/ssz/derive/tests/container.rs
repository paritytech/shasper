use ssz_derive::Ssz;

#[derive(Ssz)]
pub struct A {
	a: u64,
	b: u64,
}

pub trait Config {

}

#[derive(Ssz)]
#[bm(config_trait = "Config")]
pub struct B {
	a: u64,
	b: u64,
}

extern crate shasper_runtime as runtime;
extern crate substrate_primitives as primitives;
extern crate substrate_client as client;
extern crate substrate_service as service;

use runtime::Block;
use primitives::Blake2Hasher;

pub trait Components: service::Components {
	type Backend: 'static + client::backend::Backend<Block, Blake2Hasher>;
	type Executor: 'static + client::CallExecutor<Block, Blake2Hasher> + Send + Sync;
}

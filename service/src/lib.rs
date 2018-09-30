extern crate shasper_runtime as runtime;
extern crate substrate_service as service;

use runtime::Block;

pub trait Components: serivce::Components {
	type Backend: 'static + client::backend::Backend<Block, Blake2Hasher>;
	type Executor: 'static + client::CallExecutor<Block, Blake2Hasher> + Send + Sync;
}

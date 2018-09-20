extern crate substrate_client as client;
#[macro_use]
extern crate substrate_executor as executor;
extern crate substrate_primitives as primitives;

pub extern crate shasper_runtime as runtime;

use runtime::KeccakHasher;
use primitives::RlpCodec;

mod local_executor {
	use super::runtime;
	native_executor_instance!(pub LocalExecutor, runtime::api::dispatch, runtime::VERSION, include_bytes!("../../runtime/wasm/target/wasm32-unknown-unknown/release/shasper_runtime.compact.wasm"));
}

pub use local_executor::LocalExecutor;
pub type Backend = client::in_mem::Backend<runtime::Block, KeccakHasher, RlpCodec>;
pub type Executor = client::LocalCallExecutor<Backend, executor::NativeExecutor<LocalExecutor>>;
pub type Client = client::Client<Backend, Executor, runtime::Block>;

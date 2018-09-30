#[macro_use]
extern crate substrate_executor as executor;
extern crate substrate_primitives as primitives;

pub extern crate shasper_runtime as runtime;

pub use executor::NativeExecutor;
native_executor_instance!(pub Executor, runtime::api::dispatch, runtime::native_version, include_bytes!("../../runtime/wasm/target/wasm32-unknown-unknown/release/shasper_runtime.compact.wasm"));

// Copyright 2018 Parity Technologies (UK) Ltd.
// This file is part of Substrate Shasper.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

#[macro_use]
extern crate substrate_executor as executor;
extern crate substrate_primitives as primitives;

pub extern crate shasper_runtime as runtime;

pub use executor::NativeExecutor;
native_executor_instance!(pub Executor, runtime::api::dispatch, runtime::native_version, include_bytes!("../../runtime/wasm/target/wasm32-unknown-unknown/release/shasper_runtime.compact.wasm"));

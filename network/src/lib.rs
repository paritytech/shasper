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

extern crate shasper_runtime as runtime;

#[macro_use]
extern crate log;
extern crate substrate_network as network;
extern crate substrate_primitives as primitives;

use network::{NodeIndex, Context, Severity};
use network::consensus_gossip::ConsensusGossip;
use network::{message, generic_message};
use network::specialization::Specialization;
use network::StatusMessage as GenericFullStatus;
use runtime::{Header, Block, Hash};

type FullStatus = GenericFullStatus<Block>;

pub struct Protocol(ConsensusGossip<Block>);

impl Protocol {
	pub fn new() -> Self {
		Protocol(ConsensusGossip::new())
	}
}

impl Specialization<Block> for Protocol {
	fn status(&self) -> Vec<u8> {
		Vec::new()
	}

	fn on_connect(&mut self, _ctx: &mut Context<Block>, _who: NodeIndex, _status: FullStatus) { }

	fn on_disconnect(&mut self, _ctx: &mut Context<Block>, _who: NodeIndex) { }

	fn on_message(&mut self, ctx: &mut Context<Block>, who: NodeIndex, message: message::Message<Block>) {
		match message {
			generic_message::Message::BftMessage(msg) => {
				trace!(target: "node-network", "BFT message from {}: {:?}", who, msg);
			}
			generic_message::Message::ChainSpecific(_) => {
				trace!(target: "node-network", "Bad message from {}", who);
				ctx.report_peer(who, Severity::Bad("Invalid node protocol message format"));
			}
			_ => {}
		}
	}

	fn on_abort(&mut self) { }

	fn maintain_peers(&mut self, _ctx: &mut Context<Block>) { }

	fn on_block_imported(&mut self, _ctx: &mut Context<Block>, _hash: Hash, _header: &Header) { }
}

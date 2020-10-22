#[macro_use]
extern crate lazy_static;

extern crate chain;
extern crate primitives;

mod consensus;
mod network;

pub use primitives::{compact, hash};

pub use consensus::{ConsensusFork, ConsensusParams, TransactionOrdering};
pub use network::{Magic, Network};

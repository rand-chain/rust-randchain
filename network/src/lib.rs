#[macro_use]
extern crate lazy_static;
extern crate rug;

extern crate bitcrypto as crypto;
extern crate chain;
extern crate primitives;

mod network;

pub use primitives::{compact, hash};

pub use network::{Magic, Network};

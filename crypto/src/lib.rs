extern crate crypto as rcrypto;
extern crate curve25519_dalek;
extern crate primitives;
extern crate rand;
extern crate rand_core;
extern crate schnorrkel;
extern crate sha3;
extern crate siphasher;

pub mod ecvrf;
mod hash;
pub mod sr25519;

pub use hash::{checksum, dhash160, dhash256, siphash24, DHash160, DHash256};
pub use rcrypto::digest::Digest;

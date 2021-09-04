extern crate crypto as rcrypto;
extern crate curve25519_dalek;
extern crate primitives;
extern crate rand;
extern crate rand_core;
extern crate schnorrkel;
extern crate sha3;
extern crate siphasher;

#[macro_use]
extern crate hex_literal;

mod hash;
mod sr25519;

pub use hash::{checksum, dhash160, dhash256, siphash24, DHash160, DHash256};
pub use rcrypto::digest::Digest;
pub use sr25519::{create_keypair, sign, verify, vrf_eval, vrf_verify, PK, SK};

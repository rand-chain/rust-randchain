extern crate crypto as rcrypto;
extern crate curve25519_dalek;
extern crate primitives;
extern crate rand;
extern crate rand_core;
extern crate sha3;
extern crate siphasher;

mod ed25519;
mod hash;

pub use ed25519::{keygen, prove, verify, VrfProof, PK, SK};
pub use hash::{checksum, dhash160, dhash256, siphash24, DHash160, DHash256};
pub use rcrypto::digest::Digest;

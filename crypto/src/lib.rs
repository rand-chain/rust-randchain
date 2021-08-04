extern crate crypto as rcrypto;
extern crate primitives;
extern crate siphasher;

mod hash;

pub use hash::{checksum, dhash160, dhash256, siphash24, DHash160, DHash256};
pub use rcrypto::digest::Digest;

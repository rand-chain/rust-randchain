//! Various chain-specific test dummies

extern crate rug;
extern crate time;

extern crate chain;
extern crate primitives;
extern crate serialization as ser;
extern crate spow;

use chain::Block;

pub mod block;
pub mod chain_builder;
pub mod invoke;

pub use block::{
    block_builder, block_hash_builder, build_n_empty_blocks, build_n_empty_blocks_from,
    build_n_empty_blocks_from_genesis,
};
pub use chain_builder::ChainBuilder;

// TODO:
pub fn block1() -> Block {
    "01000000ba8b9cda965dd8e536670f9ddec10e53aab14b20bacad27b9137190000000000190760b278fe7b8565fda3b968b918d5fd997f993b23674c0af3b6fde300b38f33a5914ce6ed5b1b01e32f57".into()
}

pub fn genesis() -> Block {
    block_h0()
}

pub fn block_h0() -> Block {
    "0100000000000000000000000000000000000000000000000000000000000000000000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a29ab5f49ffff001d1dac2b7c".into()
}

// height 1
pub fn block_h1() -> Block {
    "010000006fe28c0ab6f1b372c1a6a246ae63f74f931e8365e15a089c68d6190000000000982051fd1e4ba744bbbe680e1fee14677ba1a3c3540bf7b1cdb606e857233e0e61bc6649ffff001d01e36299".into()
}

// height 2
pub fn block_h2() -> Block {
    "010000004860eb18bf1b1620e37e9490fc8a427514416fd75159ab86688e9a8300000000d5fdcc541e25de1c7a5addedf24858b8bb665c9f36ef744ee42c316022c90f9bb0bc6649ffff001d08d2bd61".into()
}

// height 3
pub fn block_h3() -> Block {
    "01000000bddd99ccfda39da1b108ce1a5d70038d0a967bacb68b6b63065f626a0000000044f672226090d85db9a9f2fbfe5f0f9609b387af7be5b7fbb7a1767c831c9e995dbe6649ffff001d05e0ed6d".into()
}

// block with the first transaction
// also is the source for 181
pub fn block_h170() -> Block {
    "0100000055bd840a78798ad0da853f68974f3d183e2bd1db6a842c1feecf222a00000000ff104ccb05421ab93e63f8c3ce5c2c2e9dbb37de2764b3a3175c8166562cac7d51b96a49ffff001d283e9e70".into()
}

// block 169
pub fn block_h169() -> Block {
    "01000000696aa63f0f22d9189c8536bb83b18737ae8336c25a67937f79957e5600000000982db9870a5e30d8f0b2a4ebccc5852b5a1e2413e9274c4947bfec6bdaa9b9d75bb76a49ffff001d2b719fdd".into()
}

// block with the source funds for the first transaction
pub fn block_h9() -> Block {
    "01000000c60ddef1b7618ca2348a46e868afc26e3efc68226c78aa47f8488c4000000000c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd37047fca6649ffff001d28404f53".into()
}

// block with some interesting test case
pub fn block_h221() -> Block {
    "01000000581d2b080bc47372c06cc5de8eb40386b00c72d4bdfecdd239c56ab600000000079f89b6e0f19f8c29d6c648ff390c9af2cf7c1da8eab6ae168cd208c745f467cc516b49ffff001d0171a069".into()
}

// block for with transaction source for 221
pub fn block_h182() -> Block {
    "01000000e5c6af65c46bd826723a83c1c29d9efa189320458dc5298a0c8655dc0000000030c2a0d34bfb4a10d35e8166e0f5a37bce02fc1b85ff983739a191197f010f2f40df6a49ffff001d2ce7ac9e".into()
}

// block for with transaction source for 182
pub fn block_h181() -> Block {
    "01000000f2c8a8d2af43a9cd05142654e56f41d159ce0274d9cabe15a20eefb500000000366c2a0915f05db4b450c050ce7165acd55f823fee51430a8c993e0bdbb192ede5dc6a49ffff001d192d3f2f".into()
}

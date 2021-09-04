//! Various chain-specific test dummies

extern crate rug;
extern crate time;

extern crate bitcrypto as crypto;
extern crate chain;
extern crate network;
extern crate primitives;
extern crate serialization as ser;
extern crate verification;

use chain::Block;
use crypto::PK;
use network::Network::Mainnet;
use primitives::compact::Compact;

mod block;
mod chain_builder;
mod invoke;

pub use block::{
    block_builder, block_hash_builder, build_n_empty_blocks, build_n_empty_blocks_from,
    build_n_empty_blocks_from_genesis,
};
pub use chain_builder::ChainBuilder;

pub fn genesis() -> Block {
    block_h0()
}

pub fn block_h0() -> Block {
    block::block_builder()
        .header()
        .parent(0.into())
        .bits(Compact::max_value())
        .version(1)
        .pubkey(PK::from_bytes(&[0; 32]).unwrap())
        .iterations(1)
        .evaluated()
        .build()
        .proved()
        .build()
}

pub fn block_h1() -> Block {
    block::block_builder()
        .header()
        .parent(block_h0().hash())
        .bits(Compact::max_value())
        .version(1)
        .pubkey(PK::from_bytes(&[0; 32]).unwrap())
        .iterations(1)
        .evaluated()
        .build()
        .proved()
        .build()
}

pub fn block_h2() -> Block {
    block::block_builder()
        .header()
        .parent(block_h1().hash())
        .bits(Compact::max_value())
        .version(1)
        .pubkey(PK::from_bytes(&[0; 32]).unwrap())
        .iterations(1)
        .evaluated()
        .build()
        .proved()
        .build()
}

pub fn block_h3() -> Block {
    block::block_builder()
        .header()
        .parent(block_h2().hash())
        .bits(Compact::max_value())
        .version(1)
        .pubkey(PK::from_bytes(&[0; 32]).unwrap())
        .iterations(1)
        .evaluated()
        .build()
        .proved()
        .build()
}

pub fn block_h169() -> Block {
    block::block_builder()
        .header()
        .parent("6868686868686868686868686868686868686868686868686868686868686868".into())
        .bits(Compact::from_u256(Mainnet.max_bits()))
        .version(1)
        .pubkey(PK::from_bytes(&[0; 32]).unwrap())
        .iterations(1)
        .evaluated()
        .build()
        .proved()
        .build()
}

pub fn block_h170() -> Block {
    block::block_builder()
        .header()
        .parent(block_h169().hash())
        .bits(Compact::from_u256(Mainnet.max_bits()))
        .version(1)
        .pubkey(PK::from_bytes(&[0; 32]).unwrap())
        .iterations(1)
        .evaluated()
        .build()
        .proved()
        .build()
}

pub fn block_h181() -> Block {
    block::block_builder()
        .header()
        .parent("8080808080808080808080808080808080808080808080808080808080808080".into())
        .bits(Compact::from_u256(Mainnet.max_bits()))
        .version(1)
        .pubkey(PK::from_bytes(&[0; 32]).unwrap())
        .iterations(4)
        .evaluated()
        .build()
        .proved()
        .build()
}

pub fn block_h182() -> Block {
    block::block_builder()
        .header()
        .parent(block_h181().hash())
        .bits(Compact::from_u256(Mainnet.max_bits()))
        .version(1)
        .pubkey(PK::from_bytes(&[0; 32]).unwrap())
        .iterations(4)
        .evaluated()
        .build()
        .proved()
        .build()
}

pub fn block_h221() -> Block {
    block::block_builder()
        .header()
        .parent("2020202020202020202020202020202020202020202020202020202020202020".into())
        .bits(Compact::from_u256(Mainnet.max_bits()))
        .version(1)
        .pubkey(PK::from_bytes(&[0; 32]).unwrap())
        .iterations(4)
        .evaluated()
        .build()
        .proved()
        .build()
}

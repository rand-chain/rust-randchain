//! Various chain-specific test dummies

extern crate ecvrf;
extern crate rug;
extern crate time;

extern crate chain;
extern crate primitives;
extern crate serialization as ser;
extern crate vdf;
extern crate verification;

use ecvrf::VrfPk;

use chain::Block;
use primitives::compact::Compact;

pub mod block;
pub mod chain_builder;
pub mod invoke;

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
        .time(1000)
        .bits(Compact::max_value())
        .version(1)
        .pubkey(VrfPk::from_bytes(&[0; 32]).unwrap())
        .iterations(1024)
        .evaluated()
        .build()
        .proved()
        .build()
}

pub fn block_h1() -> Block {
    "010000000484d17b4bd9a0afcf5a9dd53743c48e26a1eeb8f6b053004b7af774ca7dbaa13ba3edfd7a7b12b20600000001370201380138".into()
}

pub fn block_h2() -> Block {
    "01000000c6235208c895dbfd487d3c760194b77b5e0633835a0482fe6df049fc35b282773ba3edfd7a7b12b20600000001370201380138".into()
}

pub fn block_h3() -> Block {
    "01000000b6d94e340f618ec8f11682fe8eef6fdf19cbfdd0a67aad15907d88294cc961ae3ba3edfd7a7b12b20600000001370201380138".into()
}

pub fn block_h169() -> Block {
    "0100000069696969696969696969696969696969696969696969696969696969696969693ba3edfd7a7b12b20600000001370201380138".into()
}

pub fn block_h170() -> Block {
    "0100000012bc7f0860ef556a071363e72b862aa839b98093e681948dfd13a3bbf76904563ba3edfd7a7b12b20600000001370201380138".into()
}

pub fn block_h181() -> Block {
    "010000002405eed65d493e68cbe8045858a9b8a3db202d5eeec94c8ab8c3c85befabae0f3ba3edfd7a7b12b20600000001370201380138".into()
}

pub fn block_h182() -> Block {
    "0100000072db6cf01a23a2b797e7300f4943b31978b814fffad350fc1314a8bdcfa717063ba3edfd7a7b12b20600000001370201380138".into()
}

pub fn block_h221() -> Block {
    "0100000021212121212121212121212121212121212121212121212121212121212121213ba3edfd7a7b12b20600000001370201380138".into()
}

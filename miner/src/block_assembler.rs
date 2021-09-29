use crypto::sr25519::PK;
use network::Network;
use primitives::bytes::Bytes;
use primitives::compact::Compact;
use primitives::hash::H256;
use ser::{serialize, Stream};
use storage::SharedStore;
use verification::work_required;

const BLOCK_VERSION: u32 = 0x00000001;
// TODO:
// const BLOCK_HEADER_SIZE: u32 = 4 + 32 + 32 + 4 + 4 + 4;

#[derive(Copy, Clone)]
pub struct BlockTemplate {
    /// Version
    pub version: u32,
    /// The hash of previous block
    pub previous_header_hash: H256,
    /// The compressed difficulty
    pub bits: Compact,
    /// Block height
    pub height: u32,
}

impl BlockTemplate {
    pub fn to_bytes(&self, pk: &PK) -> Bytes {
        let mut stream = Stream::default();
        stream
            .append(&self.version)
            .append(&self.previous_header_hash)
            .append(&self.bits)
            .append(&Bytes::from(pk.to_bytes().to_vec()));
        stream.out()
    }
}

/// Block assembler
pub struct BlockAssembler {}

impl BlockAssembler {
    pub fn create_new_block(&self, store: &SharedStore, network: &Network) -> BlockTemplate {
        // get best block
        // take it's hash && height
        let best_block = store.best_block();
        let previous_header_hash = best_block.hash;
        let height = best_block.number + 1;
        let bits = work_required(
            previous_header_hash.clone(),
            height,
            store.as_block_header_provider(),
            network,
        );
        let version = BLOCK_VERSION;

        BlockTemplate {
            version: version,
            previous_header_hash: previous_header_hash,
            bits: bits,
            height: height,
        }
    }
}

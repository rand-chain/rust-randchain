use block::Block;
use hash::H256;
use hex::FromHex;
use indexed_header::IndexedBlockHeader;
use rug::Integer;
use ser::{deserialize, Serializable};
use std::cmp;

#[derive(Debug, Clone, Deserializable)]
pub struct IndexedBlock {
    pub header: IndexedBlockHeader,
}

#[cfg(feature = "test-helpers")]
impl From<Block> for IndexedBlock {
    fn from(block: Block) -> Self {
        Self::from_raw(block)
    }
}
impl cmp::PartialEq for IndexedBlock {
    fn eq(&self, other: &Self) -> bool {
        self.header.hash == other.header.hash
    }
}

impl IndexedBlock {
    pub fn new(header: IndexedBlockHeader) -> Self {
        IndexedBlock { header: header }
    }

    /// Explicit conversion of the raw Block into IndexedBlock.
    ///
    /// Hashes block header + transactions.
    pub fn from_raw(block: Block) -> Self {
        let Block { block_header } = block;
        Self::new(IndexedBlockHeader::from_raw(block_header))
    }

    pub fn hash(&self) -> &H256 {
        &self.header.hash
    }

    pub fn to_raw_block(self) -> Block {
        Block::new(self.header.raw)
    }

    pub fn size(&self) -> usize {
        let header_size = self.header.raw.serialized_size();
        header_size
    }

    pub fn randomness(&self) -> &Integer {
        &self.header.raw.spow.randomness
    }
}

impl From<&'static str> for IndexedBlock {
    fn from(s: &'static str) -> Self {
        deserialize(&s.from_hex::<Vec<u8>>().unwrap() as &[u8]).unwrap()
    }
}

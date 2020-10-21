use block::Block;
use hash::H256;
use hex::FromHex;
use indexed_header::IndexedBlockHeader;
// use merkle_root::merkle_root;
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

    pub fn size_with_witness(&self) -> usize {
        let header_size = self.header.raw.serialized_size();
        header_size
    }

    // TODO:
    pub fn merkle_root(&self) -> H256 {
        unimplemented!()
        // merkle_root(
        //     &self
        //         .transactions
        //         .iter()
        //         .map(|tx| &tx.hash)
        //         .collect::<Vec<&H256>>(),
        // )
    }

    // TODO:
    pub fn witness_merkle_root(&self) -> H256 {
        unimplemented!()
        // let hashes = match self.transactions.split_first() {
        //     None => vec![],
        //     Some((_, rest)) => {
        //         let mut hashes = vec![H256::from(0)];
        //         hashes.extend(rest.iter().map(|tx| tx.raw.witness_hash()));
        //         hashes
        //     }
        // };
        // merkle_root(&hashes)
    }

    // TODO:
    pub fn is_final(&self, _height: u32) -> bool {
        true
    }
}

impl From<&'static str> for IndexedBlock {
    fn from(s: &'static str) -> Self {
        deserialize(&s.from_hex::<Vec<u8>>().unwrap() as &[u8]).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::IndexedBlock;

    #[test]
    fn size_with_witness_not_equal_to_size() {
        let block_without_witness: IndexedBlock = "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".into();
        assert_eq!(
            block_without_witness.size(),
            block_without_witness.size_with_witness()
        );

        // bip143 block
        let block_with_witness: IndexedBlock = "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000010100000000000000000000000000000000000000000000000000000000000000000000000000000000000001010000000000".into();
        assert!(block_with_witness.size() != block_with_witness.size_with_witness());
    }
}

use chain::IndexedBlock;
use error::{Error, TransactionError};
use network::ConsensusFork;

pub struct BlockVerifier<'a> {
    pub serialized_size: BlockSerializedSize<'a>,
    pub merkle_root: BlockMerkleRoot<'a>,
}

impl<'a> BlockVerifier<'a> {
    pub fn new(block: &'a IndexedBlock) -> Self {
        BlockVerifier {
            serialized_size: BlockSerializedSize::new(
                block,
                ConsensusFork::absolute_maximum_block_size(),
            ),
            merkle_root: BlockMerkleRoot::new(block),
        }
    }

    pub fn check(&self) -> Result<(), Error> {
        self.serialized_size.check()?;
        self.extra_coinbases.check()?;
        self.merkle_root.check()?;
        Ok(())
    }
}

pub struct BlockSerializedSize<'a> {
    block: &'a IndexedBlock,
    max_size: usize,
}

impl<'a> BlockSerializedSize<'a> {
    fn new(block: &'a IndexedBlock, max_size: usize) -> Self {
        BlockSerializedSize {
            block: block,
            max_size: max_size,
        }
    }

    fn check(&self) -> Result<(), Error> {
        let size = self.block.size();
        if size > self.max_size {
            Err(Error::Size(size))
        } else {
            Ok(())
        }
    }
}

pub struct BlockMerkleRoot<'a> {
    block: &'a IndexedBlock,
}

impl<'a> BlockMerkleRoot<'a> {
    fn new(block: &'a IndexedBlock) -> Self {
        BlockMerkleRoot { block: block }
    }

    fn check(&self) -> Result<(), Error> {
        if self.block.merkle_root() == self.block.header.raw.merkle_root_hash {
            Ok(())
        } else {
            Err(Error::MerkleRoot)
        }
    }
}

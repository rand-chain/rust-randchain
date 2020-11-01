use chain::IndexedBlock;
use error::Error;

pub struct BlockVerifier<'a> {
    pub serialized_size: BlockSerializedSize<'a>,
    pub merkle_root: BlockMerkleRoot<'a>,
}

impl<'a> BlockVerifier<'a> {
    pub fn new(block: &'a IndexedBlock) -> Self {
        BlockVerifier {
            serialized_size: BlockSerializedSize::new(block),
            merkle_root: BlockMerkleRoot::new(block),
        }
    }

    pub fn check(&self) -> Result<(), Error> {
        self.serialized_size.check()?;
        self.merkle_root.check()?;
        Ok(())
    }
}

pub struct BlockSerializedSize<'a> {
    block: &'a IndexedBlock,
}

impl<'a> BlockSerializedSize<'a> {
    fn new(block: &'a IndexedBlock) -> Self {
        BlockSerializedSize { block: block }
    }

    // TODO:
    fn check(&self) -> Result<(), Error> {
        Ok(())
    }
}

pub struct BlockMerkleRoot<'a> {
    block: &'a IndexedBlock,
}

impl<'a> BlockMerkleRoot<'a> {
    fn new(block: &'a IndexedBlock) -> Self {
        BlockMerkleRoot { block: block }
    }

    // TODO:
    fn check(&self) -> Result<(), Error> {
        Ok(())
        // if self.block.merkle_root() == self.block.header.raw.merkle_root_hash {
        // } else {
        //     Err(Error::MerkleRoot)
        // }
    }
}

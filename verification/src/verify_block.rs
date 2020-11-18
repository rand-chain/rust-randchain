use chain::IndexedBlock;
use error::Error;

pub struct BlockVerifier<'a> {
    pub block: &'a IndexedBlock,
}

impl<'a> BlockVerifier<'a> {
    pub fn new(block: &'a IndexedBlock) -> Self {
        BlockVerifier { block: block }
    }

    pub fn check(&self) -> Result<(), Error> {
        todo!()
    }
}

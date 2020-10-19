use chain::IndexedBlock;
use error::{Error, TransactionError};
use network::ConsensusFork;

pub struct BlockVerifier<'a> {
    pub coinbase: BlockCoinbase<'a>,
    pub serialized_size: BlockSerializedSize<'a>,
    pub extra_coinbases: BlockExtraCoinbases<'a>,
    pub merkle_root: BlockMerkleRoot<'a>,
}

impl<'a> BlockVerifier<'a> {
    pub fn new(block: &'a IndexedBlock) -> Self {
        BlockVerifier {
            coinbase: BlockCoinbase::new(block),
            serialized_size: BlockSerializedSize::new(
                block,
                ConsensusFork::absolute_maximum_block_size(),
            ),
            extra_coinbases: BlockExtraCoinbases::new(block),
            merkle_root: BlockMerkleRoot::new(block),
        }
    }

    pub fn check(&self) -> Result<(), Error> {
        self.coinbase.check()?;
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

pub struct BlockCoinbase<'a> {
    block: &'a IndexedBlock,
}

impl<'a> BlockCoinbase<'a> {
    fn new(block: &'a IndexedBlock) -> Self {
        BlockCoinbase { block: block }
    }

    fn check(&self) -> Result<(), Error> {
        if self
            .block
            .transactions
            .first()
            .map(|tx| tx.raw.is_coinbase())
            .unwrap_or(false)
        {
            Ok(())
        } else {
            Err(Error::Coinbase)
        }
    }
}

pub struct BlockExtraCoinbases<'a> {
    block: &'a IndexedBlock,
}

impl<'a> BlockExtraCoinbases<'a> {
    fn new(block: &'a IndexedBlock) -> Self {
        BlockExtraCoinbases { block: block }
    }

    fn check(&self) -> Result<(), Error> {
        let misplaced = self
            .block
            .transactions
            .iter()
            .skip(1)
            .position(|tx| tx.raw.is_coinbase());

        match misplaced {
            Some(index) => Err(Error::Transaction(
                index + 1,
                TransactionError::MisplacedCoinbase,
            )),
            None => Ok(()),
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

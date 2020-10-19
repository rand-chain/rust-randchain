use canon::CanonBlock;
use deployments::BlockDeployments;
use error::Error;
use network::{ConsensusFork, ConsensusParams};
use storage::BlockHeaderProvider;

/// Flexible verification of ordered block
pub struct BlockAcceptor<'a> {
    pub finality: BlockFinality<'a>,
    pub serialized_size: BlockSerializedSize<'a>,
    pub witness: BlockWitness<'a>,
}

impl<'a> BlockAcceptor<'a> {
    pub fn new(
        consensus: &'a ConsensusParams,
        block: CanonBlock<'a>,
        height: u32,
        median_time_past: u32,
        deployments: &'a BlockDeployments<'a>,
        headers: &'a dyn BlockHeaderProvider,
    ) -> Self {
        BlockAcceptor {
            finality: BlockFinality::new(block, height, deployments, headers),
            serialized_size: BlockSerializedSize::new(
                block,
                consensus,
                deployments,
                height,
                median_time_past,
            ),
            witness: BlockWitness::new(block, deployments),
        }
    }

    pub fn check(&self) -> Result<(), Error> {
        self.finality.check()?;
        self.serialized_size.check()?;
        self.witness.check()?;
        Ok(())
    }
}

pub struct BlockFinality<'a> {
    block: CanonBlock<'a>,
    height: u32,
    headers: &'a dyn BlockHeaderProvider,
}

impl<'a> BlockFinality<'a> {
    fn new(
        block: CanonBlock<'a>,
        height: u32,
        _deployments: &'a BlockDeployments<'a>,
        headers: &'a dyn BlockHeaderProvider,
    ) -> Self {
        BlockFinality {
            block: block,
            height: height,
            headers: headers,
        }
    }

    fn check(&self) -> Result<(), Error> {
        // TODO:
        Ok(())
    }
}

pub struct BlockSerializedSize<'a> {
    block: CanonBlock<'a>,
    consensus: &'a ConsensusParams,
    height: u32,
    median_time_past: u32,
    segwit_active: bool,
}

impl<'a> BlockSerializedSize<'a> {
    fn new(
        block: CanonBlock<'a>,
        consensus: &'a ConsensusParams,
        deployments: &'a BlockDeployments<'a>,
        height: u32,
        median_time_past: u32,
    ) -> Self {
        let segwit_active = deployments.segwit();

        BlockSerializedSize {
            block: block,
            consensus: consensus,
            height: height,
            median_time_past: median_time_past,
            segwit_active: segwit_active,
        }
    }

    fn check(&self) -> Result<(), Error> {
        let size = self.block.size();

        // block size (without witness) is valid for all forks:
        // before SegWit: it is main check for size
        // after SegWit: without witness data, block size should be <= 1_000_000
        // after BitcoinCash fork: block size is increased to 8_000_000
        if size < self.consensus.fork.min_block_size(self.height)
            || size
                > self
                    .consensus
                    .fork
                    .max_block_size(self.height, self.median_time_past)
        {
            return Err(Error::Size(size));
        }

        // there's no need to define weight for pre-SegWit blocks
        if self.segwit_active {
            let size_with_witness = self.block.size_with_witness();
            let weight = size * (ConsensusFork::witness_scale_factor() - 1) + size_with_witness;
            if weight > self.consensus.fork.max_block_weight(self.height) {
                return Err(Error::Weight);
            }
        }
        Ok(())
    }
}

pub struct BlockWitness<'a> {
    block: CanonBlock<'a>,
}

impl<'a> BlockWitness<'a> {
    fn new(block: CanonBlock<'a>, _deployments: &'a BlockDeployments<'a>) -> Self {
        BlockWitness { block: block }
    }

    // TODO:
    fn check(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate test_data;

    use super::{BlockCoinbaseScript, BlockTransactionOrdering};
    use chain::{IndexedBlock, Transaction};
    use network::{BitcoinCashConsensusParams, ConsensusFork, ConsensusParams, Network};
    use {CanonBlock, Error};

    #[test]
    fn test_block_coinbase_script() {
        // transaction from block 461373
        // https://blockchain.info/rawtx/7cf05175ce9c8dbfff9aafa8263edc613fc08f876e476553009afcf7e3868a0c?format=hex
        let tx = "01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff3f033d0a070004b663ec58049cba630608733867a0787a02000a425720537570706f727420384d200a666973686572206a696e78696e092f425720506f6f6c2fffffffff01903d9d4e000000001976a914721afdf638d570285d02d3076d8be6a03ee0794d88ac00000000".into();
        let block_number = 461373;
        let block = test_data::block_builder()
            .with_transaction(tx)
            .header()
            .build()
            .build()
            .into();

        let coinbase_script_validator = BlockCoinbaseScript {
            block: CanonBlock::new(&block),
            bip34_active: true,
            height: block_number,
        };

        assert_eq!(coinbase_script_validator.check(), Ok(()));

        let coinbase_script_validator2 = BlockCoinbaseScript {
            block: CanonBlock::new(&block),
            bip34_active: true,
            height: block_number - 1,
        };

        assert_eq!(
            coinbase_script_validator2.check(),
            Err(Error::CoinbaseScript)
        );
    }

    #[test]
    fn block_transaction_ordering_works() {
        let tx1: Transaction = test_data::TransactionBuilder::with_output(1).into();
        let tx2: Transaction = test_data::TransactionBuilder::with_output(2).into();
        let tx3: Transaction = test_data::TransactionBuilder::with_output(3).into();
        let bad_block: IndexedBlock = test_data::block_builder()
            .with_transaction(tx1.clone())
            .with_transaction(tx2.clone())
            .with_transaction(tx3.clone())
            .header()
            .build()
            .build()
            .into();
        let good_block: IndexedBlock = test_data::block_builder()
            .with_transaction(tx1)
            .with_transaction(tx3)
            .with_transaction(tx2)
            .header()
            .build()
            .build()
            .into();

        let bad_block = CanonBlock::new(&bad_block);
        let good_block = CanonBlock::new(&good_block);

        // when topological ordering is used => we don't care about tx ordering
        let consensus = ConsensusParams::new(Network::Unitest, ConsensusFork::BitcoinCore);
        let checker = BlockTransactionOrdering::new(bad_block, &consensus, 0);
        assert_eq!(checker.check(), Ok(()));

        // when topological ordering is used => we care about tx ordering
        let mut bch = BitcoinCashConsensusParams::new(Network::Unitest);
        bch.magnetic_anomaly_time = 0;
        let consensus = ConsensusParams::new(Network::Unitest, ConsensusFork::BitcoinCash(bch));
        let checker = BlockTransactionOrdering::new(bad_block, &consensus, 0);
        assert_eq!(checker.check(), Err(Error::NonCanonicalTransactionOrdering));
        let checker = BlockTransactionOrdering::new(good_block, &consensus, 0);
        assert_eq!(checker.check(), Ok(()));
    }
}

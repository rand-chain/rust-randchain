use canon::CanonBlock;
use crypto::dhash256;
use deployments::BlockDeployments;
use error::{Error, TransactionError};
use network::{ConsensusFork, ConsensusParams, TransactionOrdering};
use script;
use ser::Stream;
use storage::{
    transaction_index_for_output_check, BlockHeaderProvider, DuplexTransactionOutputProvider,
    TransactionOutputProvider,
};
use work::block_reward_satoshi;

/// Flexible verification of ordered block
pub struct BlockAcceptor<'a> {
    pub finality: BlockFinality<'a>,
    pub serialized_size: BlockSerializedSize<'a>,
    pub witness: BlockWitness<'a>,
    pub ordering: BlockTransactionOrdering<'a>,
}

impl<'a> BlockAcceptor<'a> {
    pub fn new(
        store: &'a dyn TransactionOutputProvider,
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
            ordering: BlockTransactionOrdering::new(block, consensus, median_time_past),
        }
    }

    pub fn check(&self) -> Result<(), Error> {
        self.finality.check()?;
        self.serialized_size.check()?;
        self.witness.check()?;
        self.ordering.check()?;
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
    segwit_active: bool,
}

impl<'a> BlockWitness<'a> {
    fn new(block: CanonBlock<'a>, deployments: &'a BlockDeployments<'a>) -> Self {
        let segwit_active = deployments.segwit();

        BlockWitness {
            block: block,
            segwit_active: segwit_active,
        }
    }

    fn check(&self) -> Result<(), Error> {
        if !self.segwit_active {
            return Ok(());
        }

        // check witness from coinbase transaction
        let mut has_witness = false;
        if let Some(coinbase) = self.block.transactions.first() {
            let commitment = coinbase
                .raw
                .outputs
                .iter()
                .rev()
                .find(|output| script::is_witness_commitment_script(&output.script_pubkey));
            if let Some(commitment) = commitment {
                let witness_merkle_root = self.block.witness_merkle_root();
                if coinbase
                    .raw
                    .inputs
                    .get(0)
                    .map(|i| i.script_witness.len())
                    .unwrap_or_default()
                    != 1
                    || coinbase.raw.inputs[0].script_witness[0].len() != 32
                {
                    return Err(Error::WitnessInvalidNonceSize);
                }

                let mut stream = Stream::new();
                stream.append(&witness_merkle_root);
                stream.append_slice(&coinbase.raw.inputs[0].script_witness[0]);
                let hash_witness = dhash256(&stream.out());

                if hash_witness != commitment.script_pubkey[6..].into() {
                    return Err(Error::WitnessMerkleCommitmentMismatch);
                }

                has_witness = true;
            }
        }

        // witness commitment is required when block contains transactions with witness
        if !has_witness
            && self
                .block
                .transactions
                .iter()
                .any(|tx| tx.raw.has_witness())
        {
            return Err(Error::UnexpectedWitness);
        }

        Ok(())
    }
}

pub struct BlockTransactionOrdering<'a> {
    block: CanonBlock<'a>,
    transaction_ordering: TransactionOrdering,
}

impl<'a> BlockTransactionOrdering<'a> {
    fn new(block: CanonBlock<'a>, consensus: &'a ConsensusParams, median_time_past: u32) -> Self {
        BlockTransactionOrdering {
            block,
            transaction_ordering: consensus.fork.transaction_ordering(median_time_past),
        }
    }

    fn check(&self) -> Result<(), Error> {
        match self.transaction_ordering {
            // topological transaction ordering is checked in TransactionMissingInputs
            TransactionOrdering::Topological => Ok(()),
            // canonical transaction ordering means that transactions are ordered by
            // their id (i.e. hash) in ascending order
            TransactionOrdering::Canonical => {
                if self
                    .block
                    .transactions
                    .windows(2)
                    .skip(1)
                    .all(|w| w[0].hash < w[1].hash)
                {
                    Ok(())
                } else {
                    Err(Error::NonCanonicalTransactionOrdering)
                }
            }
        }
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

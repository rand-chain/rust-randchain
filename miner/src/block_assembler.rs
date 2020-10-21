use memory_pool::MemoryPool;
use network::ConsensusParams;
use primitives::compact::Compact;
use primitives::hash::H256;
use storage::SharedStore;
use verification::{block_reward_satoshi, work_required};

const BLOCK_VERSION: u32 = 0x20000000;
const BLOCK_HEADER_SIZE: u32 = 4 + 32 + 32 + 4 + 4 + 4;

/// Block template as described in [BIP0022](https://github.com/bitcoin/bips/blob/master/bip-0022.mediawiki#block-template-request)
pub struct BlockTemplate {
    /// Version
    pub version: u32,
    /// The hash of previous block
    pub previous_header_hash: H256,
    /// The current time as seen by the server
    pub time: u32,
    /// The compressed difficulty
    pub bits: Compact,
    /// Block height
    pub height: u32,
    /// Total funds available for the coinbase (in Satoshis)
    pub coinbase_value: u64,
    /// Number of bytes allowed in the block
    pub size_limit: u32,
}

/// Block size and number of signatures opcodes is limited
/// This structure should be used for storing this values.
struct SizePolicy {
    /// Current size
    current_size: u32,
    /// Max size
    max_size: u32,
    /// When current_size + size_buffer > max_size
    /// we need to start finishing the block
    size_buffer: u32,
    /// Number of transactions checked since finishing started
    finish_counter: u32,
    /// Number of transactions to check when finishing the block
    finish_limit: u32,
}

/// When appending transaction, opcode count and block size policies
/// must agree on appending the transaction to the block
#[derive(Debug, PartialEq, Copy, Clone)]
enum NextStep {
    /// Append the transaction, check the next one
    Append,
    /// Append the transaction, do not check the next one
    FinishAndAppend,
    /// Ignore transaction, check the next one
    Ignore,
    /// Ignore transaction, do not check the next one
    FinishAndIgnore,
}

impl NextStep {
    fn and(self, other: NextStep) -> Self {
        match (self, other) {
            (_, NextStep::FinishAndIgnore)
            | (NextStep::FinishAndIgnore, _)
            | (NextStep::FinishAndAppend, NextStep::Ignore)
            | (NextStep::Ignore, NextStep::FinishAndAppend) => NextStep::FinishAndIgnore,

            (NextStep::Ignore, _) | (_, NextStep::Ignore) => NextStep::Ignore,

            (_, NextStep::FinishAndAppend) | (NextStep::FinishAndAppend, _) => {
                NextStep::FinishAndAppend
            }

            (NextStep::Append, NextStep::Append) => NextStep::Append,
        }
    }
}

impl SizePolicy {
    fn new(current_size: u32, max_size: u32, size_buffer: u32, finish_limit: u32) -> Self {
        SizePolicy {
            current_size: current_size,
            max_size: max_size,
            size_buffer: size_buffer,
            finish_counter: 0,
            finish_limit: finish_limit,
        }
    }

    fn decide(&mut self, size: u32) -> NextStep {
        let finishing = self.current_size + self.size_buffer > self.max_size;
        let fits = self.current_size + size <= self.max_size;
        let finish = self.finish_counter + 1 >= self.finish_limit;

        if finishing {
            self.finish_counter += 1;
        }

        match (fits, finish) {
            (true, true) => NextStep::FinishAndAppend,
            (true, false) => NextStep::Append,
            (false, true) => NextStep::FinishAndIgnore,
            (false, false) => NextStep::Ignore,
        }
    }

    fn apply(&mut self, size: u32) {
        self.current_size += size;
    }
}

/// Block assembler
pub struct BlockAssembler {
    /// Maximal block size.
    pub max_block_size: u32,
    /// Maximal # of sigops in the block.
    pub max_block_sigops: u32,
}

impl BlockAssembler {
    pub fn create_new_block(
        &self,
        store: &SharedStore,
        mempool: &MemoryPool,
        time: u32,
        median_timestamp: u32,
        consensus: &ConsensusParams,
    ) -> BlockTemplate {
        // get best block
        // take it's hash && height
        let best_block = store.best_block();
        let previous_header_hash = best_block.hash;
        let height = best_block.number + 1;
        let bits = work_required(
            previous_header_hash.clone(),
            time,
            height,
            store.as_block_header_provider(),
            consensus,
        );
        let version = BLOCK_VERSION;

        let mut coinbase_value = block_reward_satoshi(height);

        BlockTemplate {
            version: version,
            previous_header_hash: previous_header_hash,
            time: time,
            bits: bits,
            height: height,
            coinbase_value: coinbase_value,
            size_limit: self.max_block_size,
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate test_data;

    use self::test_data::{ChainBuilder, TransactionBuilder};
    use super::{BlockAssembler, BlockTemplate, NextStep, SizePolicy};
    use chain::IndexedTransaction;
    use db::BlockChainDatabase;
    use fee::{FeeCalculator, NonZeroFeeCalculator};
    use memory_pool::MemoryPool;
    use network::{BitcoinCashConsensusParams, ConsensusFork, ConsensusParams, Network};
    use primitives::hash::H256;
    use std::sync::Arc;
    use storage::SharedStore;
    use verification::block_reward_satoshi;

    #[test]
    fn test_size_policy() {
        let mut size_policy = SizePolicy::new(0, 1000, 200, 3);
        assert_eq!(size_policy.decide(100), NextStep::Append);
        size_policy.apply(100);
        assert_eq!(size_policy.decide(500), NextStep::Append);
        size_policy.apply(500);
        assert_eq!(size_policy.decide(600), NextStep::Ignore);
        assert_eq!(size_policy.decide(200), NextStep::Append);
        size_policy.apply(200);
        assert_eq!(size_policy.decide(300), NextStep::Ignore);
        assert_eq!(size_policy.decide(300), NextStep::Ignore);
        // this transaction will make counter + buffer > max size
        assert_eq!(size_policy.decide(1), NextStep::Append);
        size_policy.apply(1);
        // so now only 3 more transactions may accepted / ignored
        assert_eq!(size_policy.decide(1), NextStep::Append);
        size_policy.apply(1);
        assert_eq!(size_policy.decide(1000), NextStep::Ignore);
        assert_eq!(size_policy.decide(1), NextStep::FinishAndAppend);
        size_policy.apply(1);
        // we should not call decide again after it returned finish...
        // but we can, let's check if result is ok
        assert_eq!(size_policy.decide(1000), NextStep::FinishAndIgnore);
    }

    #[test]
    fn test_next_step_and() {
        assert_eq!(NextStep::Append.and(NextStep::Append), NextStep::Append);
        assert_eq!(NextStep::Ignore.and(NextStep::Append), NextStep::Ignore);
        assert_eq!(
            NextStep::FinishAndIgnore.and(NextStep::Append),
            NextStep::FinishAndIgnore
        );
        assert_eq!(
            NextStep::Ignore.and(NextStep::FinishAndIgnore),
            NextStep::FinishAndIgnore
        );
        assert_eq!(
            NextStep::FinishAndAppend.and(NextStep::FinishAndIgnore),
            NextStep::FinishAndIgnore
        );
        assert_eq!(
            NextStep::FinishAndAppend.and(NextStep::Ignore),
            NextStep::FinishAndIgnore
        );
        assert_eq!(
            NextStep::FinishAndAppend.and(NextStep::Append),
            NextStep::FinishAndAppend
        );
    }

    #[test]
    fn test_fitting_transactions_iterator_max_block_size_reached() {}

    #[test]
    fn test_fitting_transactions_iterator_ignored_parent() {
        // TODO
    }

    #[test]
    fn test_fitting_transactions_iterator_locked_transaction() {
        // TODO
    }

    #[test]
    fn block_assembler_transaction_order() {
        fn construct_block(consensus: ConsensusParams) -> (BlockTemplate, H256, H256) {
            let chain = &mut ChainBuilder::new();
            TransactionBuilder::with_default_input(0)
                .set_output(30)
                .store(chain) // transaction0
                .into_input(0)
                .set_output(50)
                .store(chain); // transaction0 -> transaction1
            let hash0 = chain.at(0).hash();
            let hash1 = chain.at(1).hash();

            let mut pool = MemoryPool::new();
            let storage: SharedStore = Arc::new(BlockChainDatabase::init_test_chain(vec![
                test_data::genesis().into(),
            ]));
            pool.insert_verified(chain.at(0).into(), &NonZeroFeeCalculator);
            pool.insert_verified(chain.at(1).into(), &NonZeroFeeCalculator);

            (
                BlockAssembler {
                    max_block_size: 0xffffffff,
                    max_block_sigops: 0xffffffff,
                }
                .create_new_block(&storage, &pool, 0, 0, &consensus),
                hash0,
                hash1,
            )
        }

        // when topological consensus is used
        let topological_consensus =
            ConsensusParams::new(Network::Mainnet, ConsensusFork::BitcoinCore);
        let (block, hash0, hash1) = construct_block(topological_consensus);
        assert!(hash1 < hash0);
        assert_eq!(block.transactions[0].hash, hash0);
        assert_eq!(block.transactions[1].hash, hash1);

        // when canonocal consensus is used
        let mut canonical_fork = BitcoinCashConsensusParams::new(Network::Mainnet);
        canonical_fork.magnetic_anomaly_time = 0;
        let canonical_consensus =
            ConsensusParams::new(Network::Mainnet, ConsensusFork::BitcoinCash(canonical_fork));
        let (block, hash0, hash1) = construct_block(canonical_consensus);
        assert!(hash1 < hash0);
        assert_eq!(block.transactions[0].hash, hash1);
        assert_eq!(block.transactions[1].hash, hash0);
    }

    #[test]
    fn block_assembler_miner_fee() {
        let input_tx = test_data::genesis().transactions[0].clone();
        let tx0: IndexedTransaction = TransactionBuilder::with_input(&input_tx, 0)
            .set_output(100_000)
            .into();
        let expected_tx0_fee = input_tx.total_spends() - tx0.raw.total_spends();

        let storage: SharedStore = Arc::new(BlockChainDatabase::init_test_chain(vec![
            test_data::genesis().into(),
        ]));
        let mut pool = MemoryPool::new();
        pool.insert_verified(
            tx0,
            &FeeCalculator(storage.as_transaction_output_provider()),
        );

        let consensus = ConsensusParams::new(Network::Mainnet, ConsensusFork::BitcoinCore);
        let block = BlockAssembler {
            max_block_size: 0xffffffff,
            max_block_sigops: 0xffffffff,
        }
        .create_new_block(&storage, &pool, 0, 0, &consensus);

        let expected_coinbase_value = block_reward_satoshi(1) + expected_tx0_fee;
        assert_eq!(block.coinbase_value, expected_coinbase_value);
    }
}

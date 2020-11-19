use byteorder::{ByteOrder, LittleEndian};
use chain::IndexedBlock;
use db::BlockChainDatabase;
use network::Network;
use std::sync::Arc;
use test_data;
use verification::{BackwardsCompatibleChainVerifier as ChainVerifier, VerificationLevel, Verify};

use super::Benchmark;

// 1. write BLOCKS_INITIAL blocks with 1 transaction each
// 2. verify <BLOCKS> blocks that has <TRANSACTIONS> transaction each with <OUTPUTS> output each,
//    spending outputs from last <BLOCKS*TRANSACTIONS*OUTPUTS> blocks
pub fn main(benchmark: &mut Benchmark) {
    // params
    const BLOCKS_INITIAL: usize = 200200;
    const BLOCKS: usize = 10;
    const TRANSACTIONS: usize = 2000;
    const OUTPUTS: usize = 10;

    benchmark.samples(BLOCKS);

    assert!(
        BLOCKS_INITIAL - 100 > BLOCKS * OUTPUTS * TRANSACTIONS,
        "There will be not enough initial blocks to continue this bench"
    );

    // test setup
    let genesis = test_data::genesis();

    let mut rolling_hash = genesis.hash();
    let mut blocks: Vec<IndexedBlock> = Vec::new();

    for x in 0..BLOCKS_INITIAL {
        let mut iterations = [0u8; 8];
        LittleEndian::write_u64(&mut iterations[..], x as u64);
        let next_block = test_data::block_builder()
            // TODO:
            .header()
            .parent(rolling_hash.clone())
            .iterations(x as u32)
            .build()
            .build();
        rolling_hash = next_block.hash();
        blocks.push(next_block.into());
    }

    let store = Arc::new(BlockChainDatabase::init_test_chain(vec![genesis
        .clone()
        .into()]));
    for block in blocks.iter() {
        let hash = block.hash().clone();
        store.insert(block.clone()).unwrap();
        store.canonize(&hash).unwrap();
    }

    let mut verification_blocks: Vec<IndexedBlock> = Vec::new();
    for b in 0..BLOCKS {
        let mut iterations = [0u8; 8];
        LittleEndian::write_u64(&mut iterations[..], (b + BLOCKS_INITIAL) as u64);
        let builder = test_data::block_builder();

        verification_blocks.push(
            builder
                // TODO:
                .header()
                .parent(rolling_hash.clone())
                .build()
                .build()
                .into(),
        );
    }

    assert_eq!(store.best_block().hash, rolling_hash);

    let chain_verifier = ChainVerifier::new(store.clone(), Network::Unitest);

    // bench
    benchmark.start();
    for block in verification_blocks.iter() {
        chain_verifier
            .verify(VerificationLevel::Full, block)
            .unwrap();
    }
    benchmark.stop();
}

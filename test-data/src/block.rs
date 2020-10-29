//! Block builder

use super::genesis;
use chain;
use invoke::{Identity, Invoke};
use primitives::bytes::Bytes;
use primitives::compact::Compact;
use primitives::hash::H256;
use ser::{serialized_list_size, Serializable};
use std::cell::Cell;

thread_local! {
    pub static TIMESTAMP_COUNTER: Cell<u32> = Cell::new(0);
}

pub struct BlockHashBuilder<F = Identity> {
    callback: F,
    block: Option<chain::Block>,
}

impl BlockHashBuilder {
    pub fn new() -> Self {
        BlockHashBuilder::with_callback(Identity)
    }
}

impl<F> BlockHashBuilder<F>
where
    F: Invoke<(H256, chain::Block)>,
{
    pub fn with_callback(callback: F) -> Self {
        BlockHashBuilder {
            block: None,
            callback: callback,
        }
    }

    pub fn block(self) -> BlockBuilder<Self> {
        BlockBuilder::with_callback(self)
    }

    pub fn with_block(mut self, block: chain::Block) -> Self {
        self.block = Some(block);
        self
    }

    pub fn build(self) -> F::Result {
        let block = self
            .block
            .expect("Block is supposed to be build here to get hash");
        self.callback.invoke((block.hash(), block))
    }
}

impl<F> Invoke<chain::Block> for BlockHashBuilder<F>
where
    F: Invoke<(H256, chain::Block)>,
{
    type Result = Self;

    fn invoke(self, block: chain::Block) -> Self {
        self.with_block(block)
    }
}

pub struct BlockBuilder<F = Identity> {
    callback: F,
    header: Option<chain::BlockHeader>,
    transactions: Vec<chain::Transaction>,
}

impl BlockBuilder {
    pub fn new() -> Self {
        BlockBuilder::with_callback(Identity)
    }
}

impl<F> BlockBuilder<F>
where
    F: Invoke<chain::Block>,
{
    pub fn with_callback(callback: F) -> Self {
        BlockBuilder {
            callback: callback,
            header: None,
            transactions: Vec::new(),
        }
    }

    pub fn with_header(mut self, header: chain::BlockHeader) -> Self {
        self.header = Some(header);
        self
    }

    pub fn with_transaction(mut self, transaction: chain::Transaction) -> Self {
        self.transactions.push(transaction);
        self
    }

    pub fn with_transactions<I>(mut self, txs: I) -> Self
    where
        I: IntoIterator<Item = chain::Transaction>,
    {
        self.transactions.extend(txs);
        self
    }

    pub fn with_raw(mut self, raw: &'static str) -> Self {
        let raw_block: chain::Block = raw.into();
        self.transactions = raw_block.transactions.to_vec();
        self.header = Some(raw_block.header().clone());
        self
    }

    pub fn header(self) -> BlockHeaderBuilder<Self> {
        BlockHeaderBuilder::with_callback(self)
    }

    pub fn merkled_header(self) -> BlockHeaderBuilder<Self> {
        let hashes: Vec<H256> = self.transactions.iter().map(|t| t.hash()).collect();
        let builder = self.header().merkle_root(chain::merkle_root(&hashes));
        builder
    }

    pub fn transaction(self) -> TransactionBuilder<Self> {
        TransactionBuilder::with_callback(self)
    }

    pub fn transaction_with_sigops(self, sigops: usize) -> TransactionBuilder<Self> {
        // calling `index` creates previous output
        TransactionBuilder::with_callback(self)
            .input()
            .index(0)
            .signature_with_sigops(sigops)
            .build()
    }

    pub fn transaction_with_size(self, size: usize) -> TransactionBuilder<Self> {
        let builder = TransactionBuilder::with_callback(self);
        let current_size = builder.size();
        assert!(size > current_size, "desired transaction size is too low");
        // calling `index` creates previous output
        // let's remove current size and 1 (size of 0 script len)
        builder
            .input_with_size(size - current_size - 1)
            .index(0)
            .build()
    }

    pub fn derived_transaction(self, tx_idx: usize, output_idx: u32) -> TransactionBuilder<Self> {
        let tx = self
            .transactions
            .get(tx_idx)
            .expect(&format!(
                "using derive_transaction with the wrong index ({})",
                tx_idx
            ))
            .clone();
        TransactionBuilder::with_callback(self)
            .input()
            .hash(tx.hash())
            .index(output_idx)
            .build()
    }

    // use vec![(0, 1), (0, 2), (1, 1)]
    pub fn derived_transactions<I>(self, outputs: I) -> TransactionBuilder<Self>
    where
        I: IntoIterator<Item = (usize, u32)>,
    {
        let mut derives = Vec::new();
        for (tx_idx, output_idx) in outputs {
            derives.push((
                self.transactions
                    .get(tx_idx)
                    .expect(&format!(
                        "using derive_transaction with the wrong index ({})",
                        tx_idx
                    ))
                    .hash(),
                output_idx,
            ));
        }

        let mut builder = TransactionBuilder::with_callback(self);
        for (tx_hash, output_idx) in derives {
            builder = builder.input().hash(tx_hash).index(output_idx).build();
        }
        builder
    }

    pub fn build(self) -> F::Result {
        self.callback
            .invoke(chain::Block::new(self.header.unwrap(), self.transactions))
    }
}

impl<F> Invoke<chain::BlockHeader> for BlockBuilder<F>
where
    F: Invoke<chain::Block>,
{
    type Result = Self;

    fn invoke(self, header: chain::BlockHeader) -> Self {
        self.with_header(header)
    }
}

impl<F> Invoke<chain::Transaction> for BlockBuilder<F>
where
    F: Invoke<chain::Block>,
{
    type Result = Self;

    fn invoke(self, tx: chain::Transaction) -> Self {
        self.with_transaction(tx)
    }
}

pub struct BlockHeaderBuilder<F = Identity> {
    callback: F,
    time: u32,
    parent: H256,
    nonce: u32,
    bits: Compact,
    version: u32,
    merkle_root: H256,
}

impl<F> BlockHeaderBuilder<F>
where
    F: Invoke<chain::BlockHeader>,
{
    pub fn with_callback(callback: F) -> Self {
        BlockHeaderBuilder {
            callback: callback,
            time: TIMESTAMP_COUNTER.with(|counter| {
                let val = counter.get();
                counter.set(val + 1);
                val
            }),
            nonce: 0,
            merkle_root: 0.into(),
            parent: 0.into(),
            bits: Compact::max_value(),
            // set to 4 to allow creating long test chains
            version: 4,
        }
    }

    pub fn parent(mut self, parent: H256) -> Self {
        self.parent = parent;
        self
    }

    pub fn time(mut self, time: u32) -> Self {
        self.time = time;
        self
    }

    pub fn merkle_root(mut self, merkle_root: H256) -> Self {
        self.merkle_root = merkle_root;
        self
    }

    pub fn bits(mut self, bits: Compact) -> Self {
        self.bits = bits;
        self
    }

    pub fn nonce(mut self, nonce: u32) -> Self {
        self.nonce = nonce;
        self
    }

    pub fn build(self) -> F::Result {
        self.callback.invoke(chain::BlockHeader {
            time: self.time,
            previous_header_hash: self.parent,
            bits: self.bits,
            nonce: self.nonce,
            merkle_root_hash: self.merkle_root,
            version: self.version,
        })
    }
}

pub fn block_builder() -> BlockBuilder {
    BlockBuilder::new()
}
pub fn block_hash_builder() -> BlockHashBuilder {
    BlockHashBuilder::new()
}

pub fn build_n_empty_blocks_from(
    n: u32,
    start_nonce: u32,
    previous: &chain::BlockHeader,
) -> Vec<chain::Block> {
    let mut result = Vec::new();
    let mut previous_hash = previous.hash();
    let end_nonce = start_nonce + n;
    for i in start_nonce..end_nonce {
        let block = block_builder()
            .header()
            .nonce(i)
            .parent(previous_hash)
            .build()
            .build();
        previous_hash = block.hash();
        result.push(block);
    }
    result
}

pub fn build_n_empty_blocks_from_genesis(n: u32, start_nonce: u32) -> Vec<chain::Block> {
    build_n_empty_blocks_from(n, start_nonce, &genesis().block_header)
}

pub fn build_n_empty_blocks(n: u32, start_nonce: u32) -> Vec<chain::Block> {
    assert!(n != 0);
    let previous = block_builder().header().nonce(start_nonce).build().build();
    let mut result = vec![previous];
    let children = build_n_empty_blocks_from(n, start_nonce + 1, &result[0].block_header);
    result.extend(children);
    result
}

#[test]
fn example1() {
    let block = BlockBuilder::new().header().time(1000).build().build();
    assert_eq!(block.header().time, 1000);
}

#[test]
fn example2() {
    let block = BlockBuilder::new()
        .header()
        .build()
        .transaction()
        .lock_time(100500)
        .build()
        .build();

    assert_eq!(block.transactions().len(), 1);
}

#[test]
fn example3() {
    let block = block_builder()
        .header()
        .build()
        .transaction()
        .coinbase()
        .build()
        .build();

    assert!(block.transactions()[0].is_coinbase());
}

#[test]
fn example4() {
    let block = block_builder()
        .header()
        .build()
        .transaction()
        .coinbase()
        .output()
        .value(10)
        .build()
        .build()
        .transaction()
        .input()
        .hash(H256::from(1))
        .index(1)
        .build()
        .build()
        .build();

    assert_eq!(block.transactions().len(), 2);
    assert_eq!(
        block.transactions()[1].inputs[0].previous_output.hash,
        H256::from(1)
    );
}

#[test]
fn example5() {
    let (hash, block) = block_hash_builder()
        .block()
        .header()
        .parent(H256::from(0))
        .build()
        .build()
        .build();

    assert_eq!(
        hash,
        "3e24319d69a77c58e2da8c7331a21729482835c96834dafb3e1793c1253847c7".into()
    );
    assert_eq!(
        block.header().previous_header_hash,
        "0000000000000000000000000000000000000000000000000000000000000000".into()
    );
}

#[test]
fn transaction_with_size() {
    let block = block_builder()
        .header()
        .build()
        .transaction()
        .coinbase()
        .output()
        .value(10)
        .build()
        .build()
        .transaction_with_size(100)
        .build()
        .transaction_with_size(2000)
        .build()
        .transaction_with_size(50000)
        .build()
        .build();

    assert_eq!(block.transactions[1].serialized_size(), 100);
    assert_eq!(block.transactions[2].serialized_size(), 2000);
    assert_eq!(block.transactions[3].serialized_size(), 50000);
}

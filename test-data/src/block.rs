//! Block builder

use super::genesis;
use chain;
use invoke::{Identity, Invoke};
use primitives::compact::Compact;
use primitives::hash::H256;
use rug::Integer;
use spow::vdf;
use std::cell::Cell;
use VrfPk;

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
        }
    }

    pub fn with_header(mut self, header: chain::BlockHeader) -> Self {
        self.header = Some(header);
        self
    }

    pub fn with_raw(mut self, raw: &'static str) -> Self {
        let raw_block: chain::Block = raw.into();
        self.header = Some(raw_block.header().clone());
        self
    }

    pub fn header(self) -> BlockHeaderBuilder<Self> {
        BlockHeaderBuilder::with_callback(self)
    }

    pub fn build(self) -> F::Result {
        self.callback
            .invoke(chain::Block::new(self.header.unwrap()))
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

pub struct BlockHeaderBuilder<F = Identity> {
    callback: F,
    time: u32,
    parent: H256,
    bits: Compact,
    version: u32,
    pubkey: VrfPk,
    nonce: u32,
    randomness: Integer,
    proof: vdf::Proof,
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
            parent: 0.into(),
            bits: Compact::max_value(),
            // set to 4 to allow creating long test chains
            version: 4,
            pubkey: VrfPk::from_bytes(&[0; 32]).unwrap(),
            nonce: 0u32,
            randomness: Integer::from(0),
            proof: vec![],
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
            version: self.version,
            pubkey: self.pubkey,
            nonce: self.nonce,
            randomness: self.randomness,
            proof: self.proof,
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
        "cdad13c50f352946307fda1ec0614625bf1fb7263a2e66cad160eff552c35f19".into()
    );
    assert_eq!(
        block.header().previous_header_hash,
        "0000000000000000000000000000000000000000000000000000000000000000".into()
    );
}

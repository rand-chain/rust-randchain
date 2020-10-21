use chain::{IndexedBlock, IndexedBlockHeader};
use primitives::hash::H256;
use std::ops;

/// Blocks whose parents are known to be in the chain
#[derive(Clone, Copy)]
pub struct CanonBlock<'a> {
    block: &'a IndexedBlock,
}

impl<'a> CanonBlock<'a> {
    pub fn new(block: &'a IndexedBlock) -> Self {
        CanonBlock { block: block }
    }

    pub fn hash<'b>(&'b self) -> &'a H256
    where
        'a: 'b,
    {
        &self.block.header.hash
    }

    pub fn raw<'b>(&'b self) -> &'a IndexedBlock
    where
        'a: 'b,
    {
        self.block
    }

    pub fn header<'b>(&'b self) -> CanonHeader<'a>
    where
        'a: 'b,
    {
        CanonHeader::new(&self.block.header)
    }
}

impl<'a> ops::Deref for CanonBlock<'a> {
    type Target = IndexedBlock;

    fn deref(&self) -> &Self::Target {
        self.block
    }
}

#[derive(Clone, Copy)]
pub struct CanonHeader<'a> {
    header: &'a IndexedBlockHeader,
}

impl<'a> CanonHeader<'a> {
    pub fn new(header: &'a IndexedBlockHeader) -> Self {
        CanonHeader { header: header }
    }
}

impl<'a> ops::Deref for CanonHeader<'a> {
    type Target = IndexedBlockHeader;

    fn deref(&self) -> &Self::Target {
        self.header
    }
}

use canon::CanonBlock;
use error::Error;
use network::Network;
use storage::BlockHeaderProvider;

/// Flexible verification of ordered block
pub struct BlockAcceptor<'a> {
    pub finality: BlockFinality<'a>,
    pub serialized_size: BlockSerializedSize<'a>,
}

impl<'a> BlockAcceptor<'a> {
    pub fn new(
        network: &'a Network,
        block: CanonBlock<'a>,
        height: u32,
        median_time_past: u32,
        headers: &'a dyn BlockHeaderProvider,
    ) -> Self {
        BlockAcceptor {
            finality: BlockFinality::new(block, height, headers),
            serialized_size: BlockSerializedSize::new(block, network, height, median_time_past),
        }
    }

    pub fn check(&self) -> Result<(), Error> {
        self.finality.check()?;
        self.serialized_size.check()?;
        Ok(())
    }
}

pub struct BlockFinality<'a> {
    block: CanonBlock<'a>,
    height: u32,
    headers: &'a dyn BlockHeaderProvider,
}

impl<'a> BlockFinality<'a> {
    fn new(block: CanonBlock<'a>, height: u32, headers: &'a dyn BlockHeaderProvider) -> Self {
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
    network: &'a Network,
    height: u32,
    median_time_past: u32,
}

impl<'a> BlockSerializedSize<'a> {
    fn new(
        block: CanonBlock<'a>,
        network: &'a Network,
        height: u32,
        median_time_past: u32,
    ) -> Self {
        BlockSerializedSize {
            block: block,
            network: network,
            height: height,
            median_time_past: median_time_past,
        }
    }

    // TODO:
    fn check(&self) -> Result<(), Error> {
        Ok(())
    }
}

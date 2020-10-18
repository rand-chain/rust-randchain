use accept_block::BlockAcceptor;
use accept_header::HeaderAcceptor;
use canon::CanonBlock;
use deployments::BlockDeployments;
use error::Error;
use network::ConsensusParams;
use storage::{BlockHeaderProvider, TransactionMetaProvider, TransactionOutputProvider};

pub struct ChainAcceptor<'a> {
    pub block: BlockAcceptor<'a>,
    pub header: HeaderAcceptor<'a>,
}

impl<'a> ChainAcceptor<'a> {
    pub fn new(
        tx_out_provider: &'a dyn TransactionOutputProvider,
        tx_meta_provider: &'a dyn TransactionMetaProvider,
        header_provider: &'a dyn BlockHeaderProvider,
        consensus: &'a ConsensusParams,
        block: CanonBlock<'a>,
        height: u32,
        median_time_past: u32,
        deployments: &'a BlockDeployments,
    ) -> Self {
        trace!(target: "verification", "Block verification {}", block.hash().to_reversed_str());

        ChainAcceptor {
            block: BlockAcceptor::new(
                tx_out_provider,
                consensus,
                block,
                height,
                median_time_past,
                deployments,
                header_provider,
            ),
            header: HeaderAcceptor::new(
                header_provider,
                consensus,
                block.header(),
                height,
                deployments,
            ),
        }
    }

    pub fn check(&self) -> Result<(), Error> {
        self.block.check()?;
        self.header.check()?;
        Ok(())
    }
}

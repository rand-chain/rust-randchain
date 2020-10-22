use canon::CanonHeader;
use error::Error;
use network::ConsensusParams;
use storage::BlockHeaderProvider;
use timestamp::median_timestamp;
use work::work_required;

pub struct HeaderAcceptor<'a> {
    pub version: HeaderVersion<'a>,
    pub work: HeaderWork<'a>,
    pub median_timestamp: HeaderMedianTimestamp<'a>,
}

// TODO:
impl<'a> HeaderAcceptor<'a> {
    pub fn new(
        store: &'a dyn BlockHeaderProvider,
        consensus: &'a ConsensusParams,
        header: CanonHeader<'a>,
        height: u32,
    ) -> Self {
        HeaderAcceptor {
            work: HeaderWork::new(header, store, height, consensus),
            median_timestamp: HeaderMedianTimestamp::new(header, store),
            version: HeaderVersion::new(header, height, consensus),
        }
    }

    pub fn check(&self) -> Result<(), Error> {
        self.version.check()?;
        self.work.check()?;
        self.median_timestamp.check()?;
        Ok(())
    }
}

/// Conforms to BIP90
/// https://github.com/bitcoin/bips/blob/master/bip-0090.mediawiki
pub struct HeaderVersion<'a> {
    header: CanonHeader<'a>,
    height: u32,
    consensus_params: &'a ConsensusParams,
}

impl<'a> HeaderVersion<'a> {
    fn new(header: CanonHeader<'a>, height: u32, consensus_params: &'a ConsensusParams) -> Self {
        HeaderVersion {
            header: header,
            height: height,
            consensus_params: consensus_params,
        }
    }

    // TODO: can add more rules here
    fn check(&self) -> Result<(), Error> {
        Ok(())
    }
}

pub struct HeaderWork<'a> {
    header: CanonHeader<'a>,
    store: &'a dyn BlockHeaderProvider,
    height: u32,
    consensus: &'a ConsensusParams,
}

impl<'a> HeaderWork<'a> {
    fn new(
        header: CanonHeader<'a>,
        store: &'a dyn BlockHeaderProvider,
        height: u32,
        consensus: &'a ConsensusParams,
    ) -> Self {
        HeaderWork {
            header: header,
            store: store,
            height: height,
            consensus: consensus,
        }
    }

    fn check(&self) -> Result<(), Error> {
        let previous_header_hash = self.header.raw.previous_header_hash.clone();
        let time = self.header.raw.time;
        let work = work_required(
            previous_header_hash,
            time,
            self.height,
            self.store,
            self.consensus,
        );
        if work == self.header.raw.bits {
            Ok(())
        } else {
            Err(Error::Difficulty {
                expected: work,
                actual: self.header.raw.bits,
            })
        }
    }
}

pub struct HeaderMedianTimestamp<'a> {
    header: CanonHeader<'a>,
    store: &'a dyn BlockHeaderProvider,
}

impl<'a> HeaderMedianTimestamp<'a> {
    fn new(header: CanonHeader<'a>, store: &'a dyn BlockHeaderProvider) -> Self {
        HeaderMedianTimestamp {
            header: header,
            store: store,
        }
    }

    // TODO:
    fn check(&self) -> Result<(), Error> {
        // if csv_active && self.header.raw.time <= median_timestamp(&self.header.raw, self.store) {
        if self.header.raw.time <= median_timestamp(&self.header.raw, self.store) {
            Err(Error::Timestamp)
        } else {
            Ok(())
        }
    }
}

use primitives::bytes::Bytes;
use primitives::hash::H256;
use ser::Serializable;

#[derive(Debug, Default, Clone)]
pub struct ChainBuilder {}

impl ChainBuilder {
    pub fn new() -> ChainBuilder {
        ChainBuilder {}
    }
}

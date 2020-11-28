use hex::FromHex;
use ser::deserialize;
use BlockHeader;

#[cfg(any(test, feature = "test-helpers"))]
use hash::H256;

#[derive(Debug, PartialEq, Clone, Serializable, Deserializable)]
pub struct Block {
    pub block_header: BlockHeader,
    pub proof: vdf::Proof,
}

impl From<&'static str> for Block {
    fn from(s: &'static str) -> Self {
        deserialize(&s.from_hex::<Vec<u8>>().unwrap() as &[u8]).unwrap()
    }
}

impl Block {
    pub fn new(header: BlockHeader, proof: vdf::Proof) -> Self {
        Block {
            block_header: header,
            proof: proof,
        }
    }

    pub fn header(&self) -> &BlockHeader {
        &self.block_header
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn hash(&self) -> H256 {
        self.block_header.hash()
    }
}

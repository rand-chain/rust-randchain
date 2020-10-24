use bytes::Bytes;
use chain::BlockHeader;
use hash::H256;
use ser::{Reader, Stream};
use std::io;
use {MessageResult, Payload};

#[derive(Debug, PartialEq)]
pub struct MerkleBlock {
    pub block_header: BlockHeader,
    // TODO:
    pub hashes: Vec<H256>,
    pub flags: Bytes,
}

impl Payload for MerkleBlock {
    fn version() -> u32 {
        70014
    }

    fn command() -> &'static str {
        "merkleblock"
    }

    fn deserialize_payload<T>(reader: &mut Reader<T>, _version: u32) -> MessageResult<Self>
    where
        T: io::Read,
    {
        let merkle_block = MerkleBlock {
            block_header: reader.read()?,
            hashes: reader.read_list()?,
            flags: reader.read()?,
        };

        Ok(merkle_block)
    }

    fn serialize_payload(&self, stream: &mut Stream, _version: u32) -> MessageResult<()> {
        stream
            .append(&self.block_header)
            .append_list(&self.hashes)
            .append(&self.flags);
        Ok(())
    }
}

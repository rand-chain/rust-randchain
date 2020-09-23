use chain::BlockHeader;

#[derive(Debug, PartialEq, Serializable, Deserializable)]
pub struct BlockHeaderAndIDs {
    pub header: BlockHeader,
    pub nonce: u64,
}

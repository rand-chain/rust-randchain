use hash::H256;

// TODO:
#[derive(Debug, PartialEq, Serializable, Deserializable)]
pub struct BlockTransactions {
    pub blockhash: H256,
}

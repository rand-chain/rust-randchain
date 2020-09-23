use hash::H256;
use ser::{Deserializable, Error as ReaderError, Reader, Serializable, Stream};
use std::io;

#[derive(Debug, PartialEq)]
pub struct BlockTransactionsRequest {
    pub blockhash: H256,
}

impl Serializable for BlockTransactionsRequest {
    fn serialize(&self, stream: &mut Stream) {
        stream.append(&self.blockhash);
    }
}

impl Deserializable for BlockTransactionsRequest {
    fn deserialize<T>(reader: &mut Reader<T>) -> Result<Self, ReaderError>
    where
        T: io::Read,
    {
        let blockhash = reader.read()?;

        let request = BlockTransactionsRequest {
            blockhash: blockhash,
        };

        Ok(request)
    }
}

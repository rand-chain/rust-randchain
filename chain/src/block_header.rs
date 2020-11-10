use compact::Compact;
use crypto::dhash256;
use hash::H256;
use hex::FromHex;
use ser::{deserialize, serialize};
use spow::SPoWResult;
use std::fmt;

#[derive(PartialEq, Clone, Serializable, Deserializable)]
pub struct BlockHeader {
    pub version: u32,
    pub previous_header_hash: H256,
    pub time: u32,
    pub bits: Compact,
    pub spow: SPoWResult,
}

impl BlockHeader {
    /// Compute hash of the block header.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn hash(&self) -> H256 {
        block_header_hash(self)
    }
}

impl fmt::Debug for BlockHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BlockHeader")
            .field("version", &self.version)
            .field(
                "previous_header_hash",
                &self.previous_header_hash.reversed(),
            )
            .field("time", &self.time)
            .field("bits", &self.bits)
            .field("nonce", &self.spow.iterations)
            .field("randomness", &self.spow.randomness)
            .field("vdf_proof", &self.spow.proof)
            .finish()
    }
}

impl From<&'static str> for BlockHeader {
    fn from(s: &'static str) -> Self {
        deserialize(&s.from_hex::<Vec<u8>>().unwrap() as &[u8]).unwrap()
    }
}

/// Compute hash of the block header.
pub(crate) fn block_header_hash(block_header: &BlockHeader) -> H256 {
    dhash256(&serialize(block_header))
}

#[cfg(test)]
mod tests {
    use super::BlockHeader;
    use rug::Integer;
    use ser::{Error as ReaderError, Reader, Stream};
    use spow::SPoWResult;

    #[test]
    fn test_block_header_stream() {
        let block_header = BlockHeader {
            version: 1,
            previous_header_hash: [2; 32].into(),
            time: 4,
            bits: 5.into(),
            spow: SPoWResult {
                iterations: 6,
                randomness: Integer::from(7),
                proof: vec![Integer::from(8); 2],
            },
        };

        let mut stream = Stream::default();
        stream.append(&block_header);

        let expected = vec![
            0x01, 0x00, 0x00, 0x00, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
            0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
            0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x04, 0x00, 0x00, 0x00, 0x05, 0x00,
            0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x37, 0x02, 0x01, 0x38, 0x01, 0x38,
        ]
        .into();

        assert_eq!(stream.out(), expected);
    }

    #[test]
    fn test_block_header_reader() {
        let buffer = vec![
            0x01, 0x00, 0x00, 0x00, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
            0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
            0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x04, 0x00, 0x00, 0x00, 0x05, 0x00,
            0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x37, 0x02, 0x01, 0x38, 0x01, 0x38,
        ];

        let mut reader = Reader::new(&buffer);

        let expected = BlockHeader {
            version: 1,
            previous_header_hash: [2; 32].into(),
            time: 4,
            bits: 5.into(),
            spow: SPoWResult {
                iterations: 6,
                randomness: Integer::from(7),
                proof: vec![Integer::from(8), Integer::from(8)],
            },
        };

        assert_eq!(expected, reader.read().unwrap());
        assert_eq!(
            ReaderError::UnexpectedEnd,
            reader.read::<BlockHeader>().unwrap_err()
        );
    }
}

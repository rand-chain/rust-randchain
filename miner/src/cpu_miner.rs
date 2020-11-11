// TODO: merkle_root_hash seems useless
// TODO: implement randomness miner
use block_assembler::BlockTemplate;
use byteorder::{LittleEndian, WriteBytesExt};
use crypto::dhash256;
use primitives::bigint::{Uint, U256};
use primitives::bytes::Bytes;
use primitives::compact::Compact;
use primitives::hash::H256;
use ser::Stream;
use verification::is_valid_proof_of_work_hash;

/// Instead of serializing `BlockHeader` from scratch over and over again,
/// let's keep it serialized in memory and replace needed bytes
struct BlockHeaderBytes {
    data: Bytes,
}

impl BlockHeaderBytes {
    /// Creates new instance of block header bytes.
    fn new(version: u32, previous_header_hash: H256, bits: Compact) -> Self {
        let merkle_root_hash = H256::default();
        let time = 0u32;
        let nonce = 0u32;

        let mut stream = Stream::default();
        stream
            .append(&version)
            .append(&previous_header_hash)
            .append(&merkle_root_hash)
            .append(&time)
            .append(&bits)
            .append(&nonce);

        BlockHeaderBytes { data: stream.out() }
    }

    /// Set block header time
    fn set_time(&mut self, time: u32) {
        let mut time_bytes: &mut [u8] = &mut self.data[4 + 32 + 32..];
        time_bytes.write_u32::<LittleEndian>(time).unwrap();
    }

    /// Set block header nonce
    fn set_nonce(&mut self, nonce: u32) {
        let mut nonce_bytes: &mut [u8] = &mut self.data[4 + 32 + 32 + 4 + 4..];
        nonce_bytes.write_u32::<LittleEndian>(nonce).unwrap();
    }

    /// Returns block header hash
    fn hash(&self) -> H256 {
        dhash256(&self.data)
    }
}

/// Cpu miner solution.
pub struct Solution {
    /// Block header nonce.
    pub nonce: u32,
    /// Coinbase transaction extra nonce (modyfiable by miner).
    pub extranonce: U256,
    /// Block header time.
    pub time: u32,
}

/// Simple randchain cpu miner.
///
/// First it tries to find solution by changing block header nonce.
/// Once all nonce values have been tried, it increases extranonce.
/// Once all of them have been tried (quite unlikely on cpu ;),
/// and solution still hasn't been found it returns None.
/// It's possible to also experiment with time, but I find it pointless
/// to implement on CPU.
pub fn find_solution(block: &BlockTemplate, max_extranonce: U256) -> Option<Solution> {
    let mut extranonce = U256::default();
    let mut extranonce_bytes = [0u8; 32];

    let mut header_bytes = BlockHeaderBytes::new(
        block.version,
        block.previous_header_hash.clone(),
        block.bits,
    );
    // update header with time
    header_bytes.set_time(block.time);

    while extranonce < max_extranonce {
        extranonce.to_little_endian(&mut extranonce_bytes);

        for nonce in 0..(u32::max_value() as u64 + 1) {
            // update §
            header_bytes.set_nonce(nonce as u32);
            let hash = header_bytes.hash();
            if is_valid_proof_of_work_hash(block.bits, &hash) {
                let solution = Solution {
                    nonce: nonce as u32,
                    extranonce: extranonce,
                    time: block.time,
                };

                return Some(solution);
            }
        }

        extranonce = extranonce + 1.into();
    }

    None
}

#[cfg(test)]
mod tests {
    use super::find_solution;
    use block_assembler::BlockTemplate;
    use primitives::bigint::{Uint, U256};

    #[test]
    fn test_cpu_miner_low_difficulty() {
        let block_template = BlockTemplate {
            version: 0,
            previous_header_hash: 0.into(),
            time: 0,
            bits: U256::max_value().into(),
            height: 0,
        };

        let solution = find_solution(&block_template, U256::max_value());
        assert!(solution.is_some());
    }
}

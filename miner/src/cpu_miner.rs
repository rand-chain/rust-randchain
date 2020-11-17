use block_assembler::BlockTemplate;
use ecvrf::VrfPk;
use rug::Integer;
use spow::vdf;
// use byteorder::{LittleEndian, WriteBytesExt};
use crypto::dhash256;
use primitives::bigint::{Uint, U256};
use primitives::bytes::Bytes;
use primitives::compact::Compact;
use primitives::hash::H256;
use ser::Stream;
use verification::is_valid_proof_of_work_hash;

/// Same sequence as chain/block_header for hashing
struct BlockHeaderDraft {
    version: u32,
    previous_header_hash: H256,
    time: u32,
    bits: Compact,
    // pubkey: VrfPk,
    // nonce: Compact,
    // randomness: Integer,
    // proof: vdf::Proof,
}

impl BlockHeaderDraft {
    fn new(version: u32, previous_header_hash: H256, time: u32, bits: Compact) -> BlockHeaderDraft {
        BlockHeaderDraft {
            version: version,
            previous_header_hash: previous_header_hash,
            time: time,
            bits: bits,
            // pubkey: pubkey,
        }
    }

    fn fill_and_hash(
        &self,
        pubkey: VrfPk,
        nonce: u32,
        randomness: Integer,
        proof: vdf::Proof,
    ) -> H256 {
        let mut stream = Stream::default();
        stream
            .append(&self.version)
            .append(&self.previous_header_hash)
            .append(&self.time)
            .append(&self.bits)
            .append(&Bytes::from(pubkey.to_bytes().to_vec()))
            .append(&nonce)
            .append(&randomness)
            .append_vector(&proof);

        let data = stream.out();
        dhash256(&data)
    }
}

// TODO:
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
// TODO: load key
pub fn find_solution(
    block: &BlockTemplate,
    pubkey: VrfPk,
    max_extranonce: U256,
) -> Option<Solution> {
    let mut extranonce = U256::default();
    let mut extranonce_bytes = [0u8; 32];

    let mut header_bytes = BlockHeaderDraft::new(
        block.version,
        block.previous_header_hash.clone(),
        block.time,
        block.bits,
    );

    while extranonce < max_extranonce {
        extranonce.to_little_endian(&mut extranonce_bytes);

        for nonce in 0..(u32::max_value() as u64 + 1) {
            // update §
            // header_bytes.set_nonce(nonce as u32);
            let hash = header_bytes.fill_and_hash();
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
    use ecvrf::VrfPk;
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

        let pubkey: VrfPk = VrfPk::from_bytes(&[0; 32]).unwrap();
        let solution = find_solution(&block_template, pubkey, U256::max_value());
        assert!(solution.is_some());
    }
}

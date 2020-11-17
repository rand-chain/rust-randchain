use block_assembler::BlockTemplate;
use ecvrf::VrfPk;
use rug::Integer;
use spow::vdf;
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
    pubkey: VrfPk,
}

impl BlockHeaderDraft {
    fn new(
        version: u32,
        previous_header_hash: H256,
        time: u32,
        bits: Compact,
        pubkey: VrfPk,
    ) -> BlockHeaderDraft {
        BlockHeaderDraft {
            version: version,
            previous_header_hash: previous_header_hash,
            time: time,
            bits: bits,
            pubkey: pubkey,
        }
    }

    fn fill_and_hash(&self, nonce: u32, randomness: Integer, proof: vdf::Proof) -> H256 {
        let mut stream = Stream::default();
        stream
            .append(&self.version)
            .append(&self.previous_header_hash)
            .append(&self.time)
            .append(&self.bits)
            .append(&Bytes::from(self.pubkey.to_bytes().to_vec()))
            .append(&nonce)
            .append(&randomness)
            .append_vector(&proof);

        let data = stream.out();
        dhash256(&data)
    }
}

/// Cpu miner solution.
pub struct Solution {
    pub nonce: u32,
    pub randomness: Integer,
    pub proof: vdf::Proof,
}

/// Simple randchain cpu miner.
///
/// First it tries to find solution by changing block header nonce.
/// Once all nonce values have been tried, it increases extranonce.
/// Once all of them have been tried (quite unlikely on cpu ;),
/// and solution still hasn't been found it returns None.
/// It's possible to also experiment with time, but I find it pointless
/// to implement on CPU.
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
        pubkey,
    );

    while extranonce < max_extranonce {
        // extranonce.to_little_endian(&mut extranonce_bytes);

        for nonce in 0..(u32::max_value() as u64 + 1) {
            // update §
            // header_bytes.set_nonce(nonce as u32);
            let hash = header_bytes.fill_and_hash(nonce, Integer::from(0), vec![]);
            if is_valid_proof_of_work_hash(block.bits, &hash) {
                let solution = Solution {
                    nonce: nonce as u32,
                    randomness: randomness,
                    proof: proof,
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

        // generate or load key
        let pubkey: VrfPk = VrfPk::from_bytes(&[0; 32]).unwrap();
        let solution = find_solution(&block_template, pubkey, U256::max_value());
        assert!(solution.is_some());
    }
}

use block_assembler::BlockTemplate;
use crypto::dhash256;
use ecvrf::VrfPk;
// use primitives::bigint::{Uint, U256};
use primitives::bytes::Bytes;
use primitives::compact::Compact;
use primitives::hash::H256;
use rug::Integer;
use ser::Stream;
use spow::vdf;
use verification::is_valid_proof_of_work_hash;

const STEP: u64 = 1024;

/// Same sequence as chain/block_header for hashing
struct BlockHeaderDraft {
    version: u32,
    previous_header_hash: H256,
    time: u32,
    bits: Compact,
}

impl BlockHeaderDraft {
    fn new(version: u32, previous_header_hash: H256, time: u32, bits: Compact) -> BlockHeaderDraft {
        BlockHeaderDraft {
            version: version,
            previous_header_hash: previous_header_hash,
            time: time,
            bits: bits,
        }
    }

    // Same sequence as chain/block_header for hashing
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

/// Cpu miner solution.
pub struct Solution {
    pub nonce: u32,
    pub randomness: Integer,
    pub proof: vdf::Proof,
}

/// Simple randchain cpu miner.
pub fn find_solution(block: &BlockTemplate, pubkey: VrfPk) -> Option<Solution> {
    let header_bytes = BlockHeaderDraft::new(
        block.version,
        block.previous_header_hash.clone(),
        block.time,
        block.bits,
    );

    let mut y = Integer::from(0);
    let ini_state = Integer::from(0);

    let mut cur_state = Integer::from(0);

    for nonce in 0..u32::max_value() {
        // update §

        // let y = vdf::eval(state, STEP);

        y = Integer::from(0);

        let proof = vdf::prove(&ini_state, &cur_state, u64::from(nonce) * STEP);

        let hash = header_bytes.fill_and_hash(pubkey.clone(), nonce, y.clone(), proof.clone());

        if is_valid_proof_of_work_hash(block.bits, &hash) {
            let solution = Solution {
                nonce: nonce as u32,
                randomness: y,
                proof: proof,
            };

            return Some(solution);
        }

        cur_state = y;
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
        let solution = find_solution(&block_template, pubkey);
        assert!(solution.is_some());
    }
}

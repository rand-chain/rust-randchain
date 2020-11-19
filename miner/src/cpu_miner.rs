use block_assembler::BlockTemplate;
use crypto::dhash256;
use ecvrf::VrfPk;
use hash::H256;
use primitives::bytes::Bytes;
use rug::Integer;
use ser::Stream;
use verification::is_valid_proof_of_work_hash;

const STEP: u32 = 1024;

// consistent with verification/src/verify_block.rs
fn h_g(block: &BlockTemplate, pubkey: &VrfPk) -> Integer {
    let mut stream = Stream::default();
    stream
        .append(&block.version)
        .append(&block.previous_header_hash)
        .append(&block.time)
        .append(&block.bits)
        .append(&Bytes::from(pubkey.to_bytes().to_vec()));
    let data = stream.out();
    let h = dhash256(&data);
    let result = Integer::from_str_radix(&h.to_string(), 16).unwrap();

    // invert to get enough security bits
    match result.invert(&vdf::MODULUS) {
        Ok(inverse) => inverse,
        Err(unchanged) => unchanged,
    }
}

// consistent with chain/src/block_header.rs
fn randomness_hash(pubkey: &VrfPk, randomness: &Integer) -> H256 {
    let mut stream = Stream::default();
    stream
        .append(&Bytes::from(pubkey.to_bytes().to_vec()))
        .append(randomness);
    let data = stream.out();
    dhash256(&data)
}

/// Cpu miner solution.
pub struct Solution {
    pub iterations: u32,
    pub randomness: Integer,
    pub proof: vdf::Proof,
}

/// Simple randchain cpu miner.
pub fn find_solution(block: &BlockTemplate, pubkey: &VrfPk) -> Option<Solution> {
    let g = h_g(block, pubkey);
    let mut cur_y = g.clone();
    let mut iterations = 0u64;
    loop {
        iterations += STEP as u64;
        if iterations > (u32::max_value() as u64) {
            return None;
        }

        let new_y = vdf::eval(&cur_y, STEP);
        if is_valid_proof_of_work_hash(block.bits, &randomness_hash(pubkey, &new_y)) {
            let solution = Solution {
                iterations: iterations as u32,
                randomness: new_y.clone(),
                proof: vdf::prove(&g, &new_y, iterations as u32),
            };

            return Some(solution);
        }

        cur_y = new_y;
    }
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
        let solution = find_solution(&block_template, &pubkey);
        assert!(solution.is_some());
    }
}

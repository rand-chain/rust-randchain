use block_assembler::BlockTemplate;
use crypto::dhash256;
use ecvrf::VrfPk;
use primitives::bytes::Bytes;
use primitives::compact::Compact;
// use primitives::hash::H256;
use bigint::U256;
use rug::Integer;
use ser::Stream;
use spow::vdf;
use verification::is_valid_proof_of_work_hash;

const STEP: u64 = 1024;

fn h_g(block: &BlockTemplate, pubkey: VrfPk) -> Integer {
    let mut stream = Stream::default();
    stream
        .append(&block.version)
        .append(&block.previous_header_hash)
        .append(&block.time)
        .append(&block.bits)
        .append(&Bytes::from(pubkey.to_bytes().to_vec()));
    let data = stream.out();
    let h = dhash256(&data);
    // TODO:
    let s = U256::from(&*h.reversed() as &[u8]).to_string();
    Integer::from_str_radix(&s, 10).unwrap()
    // TODO: inverse
}

/// Cpu miner solution.
pub struct Solution {
    pub nonce: u32,
    pub randomness: Integer,
    pub proof: vdf::Proof,
}

/// Simple randchain cpu miner.
pub fn find_solution(block: &BlockTemplate, pubkey: VrfPk) -> Option<Solution> {
    let g = h_g(block, pubkey);
    let mut cur_y = g.clone();
    for nonce in 1..u32::max_value() {
        let new_y = vdf::eval(&cur_y, STEP);
        if is_valid_proof_of_work_hash(block.bits, &dhash256(new_y.to_string_radix(16).as_ref())) {
            let solution = Solution {
                nonce: nonce as u32,
                randomness: new_y.clone(),
                proof: vdf::prove(&g, &new_y, u64::from(nonce) * STEP),
            };

            return Some(solution);
        }

        cur_y = new_y;
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

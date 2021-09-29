use block_assembler::BlockTemplate;
use chain::BlockHeader;
use crypto::sr25519::PK;
use crypto::{dhash256, seqpow};
use network::Network;
use primitives::bytes::Bytes;
use primitives::compact::Compact;
use rug::{integer::Order, Integer};
use ser::{serialize, Stream};
use sha2::{Digest, Sha256};
use verification::is_valid_proof_of_work_hash;

/// Simple randchain cpu miner.
pub fn find_solution(pk: &PK, block: &BlockTemplate) -> seqpow::Solution {
    // let mut stream = Stream::default();
    // stream
    //     .append(&block.version)
    //     .append(&block.previous_header_hash)
    //     .append(&block.bits)
    //     .append(&Bytes::from(pk.to_bytes().to_vec()));
    // let data = stream.out();
    let data = block.to_bytes(pk);

    let mut solution = seqpow::init(&data, pk);
    loop {
        let (new_solution, result) = seqpow::solve(pk, &solution, block.bits);

        if result {
            return seqpow::prove(pk, &data, &new_solution);
        }
        solution = new_solution;
    }
}

/// Dry run miner
pub fn find_solution_dry(pk: &PK, block: &BlockTemplate) -> seqpow::Solution {
    // let mut stream = Stream::default();
    // stream
    //     .append(&block.version)
    //     .append(&block.previous_header_hash)
    //     .append(&block.bits)
    //     .append(&Bytes::from(pk.to_bytes().to_vec()));
    // let data = stream.out();
    let data = block.to_bytes(pk);
    seqpow::init(&data, pk)
}

pub fn verify_solution(pk: &PK, block: &BlockTemplate, solution: &seqpow::Solution) -> bool {
    // let mut stream = Stream::default();
    // stream
    //     .append(&block.version)
    //     .append(&block.previous_header_hash)
    //     .append(&block.bits)
    //     .append(&Bytes::from(pk.to_bytes().to_vec()));
    // let data = stream.out();
    let data = block.to_bytes(pk);

    if !seqpow::verify(pk, &data, solution, block.bits) {
        return false;
    }

    let block_header_hash = dhash256(&serialize(&BlockHeader {
        version: block.version,
        previous_header_hash: block.previous_header_hash,
        bits: block.bits,
        pubkey: pk.clone(),
        iterations: solution.iterations as u32,
        solution: solution.element.clone(),
    }));
    // if PoW verification fails, then fail
    if !is_valid_proof_of_work_hash(block.bits, &block_header_hash) {
        return false;
    }
    return true;
}

#[cfg(test)]
mod tests {
    use super::*;
    use block_assembler::BlockTemplate;
    use crypto::sr25519::PK;
    use primitives::bigint::{Uint, U256};

    #[test]
    fn test_cpu_miner_low_difficulty() {
        let pk: PK = PK::from_bytes(&[0; 32]).unwrap();
        let block_template = BlockTemplate {
            version: 0,
            previous_header_hash: 0.into(),
            bits: U256::max_value().into(),
            height: 0,
        };

        let solution = find_solution(&pk, &block_template);

        let r = verify_solution(&pk, &block_template, &solution);
        assert!(r);
    }
}

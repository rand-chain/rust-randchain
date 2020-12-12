use block_assembler::BlockTemplate;
use byteorder::BigEndian;
use chain::BlockHeader;
use crypto::dhash256;
use ecvrf::VrfPk;
use primitives::bytes::Bytes;
use rand::prelude::*;
use rug::{integer::Order, Integer};
use ser::{serialize, Stream};
use sha2::{Digest, Sha256};
use std::{thread, time};
use verification::is_valid_proof_of_work_hash;

const STEP: u32 = 1024;

// consistent with verification/src/verify_block.rs
fn h_g_blk(block: &BlockTemplate, pubkey: &VrfPk) -> Integer {
    let mut stream = Stream::default();
    stream
        .append(&block.version)
        .append(&block.previous_header_hash)
        .append(&block.time)
        .append(&block.bits)
        .append(&Bytes::from(pubkey.to_bytes().to_vec()));

    h_g(&stream.out(), pubkey)
}

// consistent with verification/src/verify_block.rs
fn h_g(data: &Bytes, pubkey: &VrfPk) -> Integer {
    let seed = dhash256(&data);
    let prefix = "residue_part_".as_bytes();
    // concat 8 sha256 to a 2048-bit hash
    let all_2048: Vec<u8> = (0..((2048 / 256) as u8))
        .map(|index| {
            let mut hasher = Sha256::new();
            hasher.update(prefix);
            hasher.update(vec![index]);
            hasher.update(<[u8; 32]>::from(seed));
            hasher.finalize()
        })
        .flatten()
        .collect();
    let result = Integer::from_digits(&all_2048, Order::Lsf);
    result.div_rem_floor(vdf::MODULUS.clone()).1
}

/// Cpu miner solution.
pub struct Solution {
    pub iterations: u32,
    pub randomness: Integer,
    pub proof: vdf::Proof,
}

/// Simple randchain cpu miner.
pub fn find_solution_mock(
    block: &BlockTemplate,
    pubkey: &VrfPk,
    mut iterations: u64,
    num_nodes: u16,
    blocktime: u16,
) -> Option<Solution> {
    // INJECT find_solution to somewhere
    thread::sleep(time::Duration::from_secs(1));
    let g = h_g_blk(block, pubkey);
    iterations += STEP as u64;
    if iterations > (u32::max_value() as u64) {
        return None;
    }

    let mut rng = rand::thread_rng();
    let r: f32 = rng.gen(); // generates a float between 0 and 1

    if r * (num_nodes as f32) * (blocktime as f32) <= 1f32 {
        let mut r_bytes = Bytes::from(r.to_ne_bytes().to_vec());
        let y = h_g(&r_bytes, pubkey);
        let block_header_hash = dhash256(&serialize(&BlockHeader {
            version: block.version,
            previous_header_hash: block.previous_header_hash,
            time: block.time,
            bits: block.bits,
            pubkey: pubkey.clone(),
            iterations: iterations as u32,
            randomness: y.clone(),
        }));

        let solution = Solution {
            iterations: iterations as u32,
            randomness: y.clone(),
            proof: vdf::prove(&g, &y, iterations as u32),
        };

        return Some(solution);
    }
    return None;
}

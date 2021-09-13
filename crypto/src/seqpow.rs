use dhash256;
use primitives::bigint::U256;
use primitives::bytes::Bytes;
use primitives::compact::Compact;
use primitives::hash::H256;
use rug::{integer::Order, Integer};
use ser::serialize;
use sha2::{Digest, Sha256};
use sr25519::PK;
use vdf;

const STEP: u64 = 100_000;

fn diff_check(bits: Compact, hash: &H256) -> bool {
    let target = match bits.to_u256() {
        Ok(target) => target,
        _err => return false,
    };

    let value = U256::from(&*hash.reversed() as &[u8]);
    value <= target
}

// consistent with verification/src/verify_block.rs
fn h_g(x: &[u8]) -> Integer {
    let prefix = "residue_part_".as_bytes();
    // concat 8 sha256 to a 2048-bit hash
    let all_2048: Vec<u8> = (0..((2048 / 256) as u8))
        .map(|index| {
            let mut hasher = Sha256::new();
            hasher.update(prefix);
            hasher.update(vec![index]);
            hasher.update(x);
            hasher.finalize()
        })
        .flatten()
        .collect();
    let result = Integer::from_digits(&all_2048, Order::Lsf);
    result.div_rem_floor(vdf::MODULUS.clone()).1
}

/// Cpu miner solution.
pub struct Solution {
    pub iterations: u64,
    pub element: Integer,
    pub proof: vdf::Proof,
}

/// SeqPoW.Init()
#[allow(dead_code)]
pub fn init(x: &[u8], pk: &PK) -> Solution {
    Solution {
        iterations: 0u64,
        element: h_g(x),
        proof: vec![], // placeholder
    }
}

/// SeqPoW.Solve()
pub fn solve(pk: &PK, solution: &Solution, target: Compact) -> (Solution, bool) {
    let new_y = vdf::eval(&solution.element, STEP);
    let new_solution = Solution {
        iterations: solution.iterations + STEP,
        element: new_y.clone(),
        proof: vec![],
    };

    // concat y and pk
    let mut bytes = serialize(&new_y);
    let mut pk_bytes = Bytes::from(pk.to_bytes().to_vec());
    bytes.append(&mut pk_bytes);
    let hash = dhash256(&bytes);

    if diff_check(target, &hash) {
        (new_solution, true)
    } else {
        (new_solution, false)
    }
}

/// SeqPoW.Prove()
pub fn prove(pk: &PK, x: &[u8], solution: &Solution) -> Solution {
    let first_solution = init(x, pk);
    Solution {
        iterations: solution.iterations,
        element: solution.element.clone(),
        proof: vdf::prove(
            &first_solution.element,
            &solution.element,
            solution.iterations,
        ),
    }
}

/// SeqPoW.Verify()
pub fn verify(pk: &PK, x: &[u8], solution: &Solution, target: Compact) -> bool {
    let first_solution = init(x, pk);
    // if not comply STEP parameter
    if solution.iterations % STEP != 0 {
        return false;
    }
    // if VDF verification fails, then fail
    if !vdf::verify(
        &first_solution.element,
        &solution.element,
        solution.iterations,
        &solution.proof,
    ) {
        return false;
    }
    // concat solution and pk
    let mut bytes = serialize(&solution.element);
    let mut pk_bytes = Bytes::from(pk.to_bytes().to_vec());
    bytes.append(&mut pk_bytes);
    let hash = dhash256(&bytes);
    // if PoW verification fails, then fail
    if !diff_check(target, &hash) {
        return false;
    }
    return true;
}

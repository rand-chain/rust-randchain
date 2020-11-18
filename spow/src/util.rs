use rug::Integer;
use sha2::{Digest, Sha256};

use super::config::MODULUS;

pub fn hash_to_prime(inputs: &[&Integer]) -> Integer {
    let mut hasher = Sha256::new();
    for input in inputs {
        hasher.update(input.to_digits::<u8>(Order::Lsf));
        hasher.update("\n".as_bytes());
    }
    let hashed = Integer::from_digits(&hasher.finalize(), Order::Lsf);

    // invert to get enough security bits
    let inverse = match hashed.invert(&MODULUS) {
        Ok(inverse) => inverse,
        Err(unchanged) => unchanged,
    };

    inverse.next_prime().div_rem_floor(MODULUS.clone()).1
}

/// Fiat–Shamir heuristic non-iterative signature
pub fn hash_fs(inputs: &[&Integer]) -> Integer {
    let mut hasher = Sha256::new();
    for input in inputs {
        hasher.update(input.to_digits::<u8>(Order::Lsf));
        hasher.update("\n".as_bytes());
    }
    let hashed = Integer::from_digits(&hasher.finalize(), Order::Lsf);

    // invert to get enough security bits
    match hashed.invert(&MODULUS) {
        Ok(inverse) => inverse,
        Err(unchanged) => unchanged,
    }
}

use rug::Integer;
use sha2::{Digest, Sha256};

use super::config::MODULUS;

pub fn hash_to_prime(inputs: &[&Integer]) -> Integer {
    let mut hasher = Sha256::new();
    for input in inputs {
        hasher.update(input.to_string_radix(16).as_bytes());
        hasher.update("\n".as_bytes());
    }
    let hashed_hex = hasher.finalize();
    let hashed_hex_str = format!("{:#x}", hashed_hex);
    let hashed_int = Integer::from_str_radix(&hashed_hex_str, 16).unwrap();

    // invert to get enough security bits
    let inverse = match hashed_int.invert(&MODULUS) {
        Ok(inverse) => inverse,
        Err(unchanged) => unchanged,
    };

    inverse.next_prime().div_rem_floor(MODULUS.clone()).1
}

/// Fiat–Shamir heuristic non-iterative signature
pub fn hash_fs(inputs: &[&Integer]) -> Integer {
    let mut hasher = Sha256::new();
    for input in inputs {
        hasher.update(input.to_string_radix(16).as_bytes());
        hasher.update("\n".as_bytes());
    }
    let hashed_hex = hasher.finalize();
    let hashed_hex_str = format!("{:#x}", hashed_hex);
    let hashed_int = Integer::from_str_radix(&hashed_hex_str, 16).unwrap();

    // invert to get enough security bits
    match hashed_int.invert(&MODULUS) {
        Ok(inverse) => inverse,
        Err(unchanged) => unchanged,
    }
}

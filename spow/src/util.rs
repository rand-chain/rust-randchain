use ecvrf;
use rug::Integer;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;

use super::config;

/// state & target should already be modulo
pub fn validate_difficulty(state: &Integer, target: &Integer) -> bool {
    let mut hasher = Sha256::new();
    let hash_input: String = state.clone().to_string_radix(16);
    // only hash state for demo purpose, in real-world case, we may need to add other block metadata
    hasher.update(hash_input.as_bytes());
    let hash_result = hasher.finalize();
    let hash_result_str = format!("{:#x}", hash_result);
    let hashed_int = Integer::from_str_radix(&hash_result_str, 16).unwrap();
    (hashed_int.cmp(target) == Ordering::Less) || (hashed_int.cmp(target) == Ordering::Equal)
}

/// int(H("pubkey"||pubkey||"residue"||x)) mod N
pub fn h_g(pubkey: &ecvrf::VrfPk, seed: &Integer) -> Integer {
    let mut hasher = Sha256::new();
    hasher.update("pubkey".as_bytes());
    hasher.update(pubkey.to_bytes());
    hasher.update("residue".as_bytes());
    hasher.update(seed.to_string_radix(16).as_bytes());
    let result_hex = hasher.finalize();
    let result_hex_str = format!("{:#x}", result_hex);
    let result_int = Integer::from_str_radix(&result_hex_str, 16).unwrap();

    // invert to get enough security bits
    match result_int.invert(&config::MODULUS) {
        Ok(inverse) => inverse,
        Err(unchanged) => unchanged,
    }
}

/// int(H("pubkey"||pubkey||"state"||state)) mod N
pub fn h_state(pubkey: &ecvrf::VrfPk, state: &Integer) -> Integer {
    let mut hasher = Sha256::new();
    hasher.update("pubkey".as_bytes());
    hasher.update(pubkey.to_bytes());
    hasher.update("state".as_bytes());
    hasher.update(state.to_string_radix(16).as_bytes());
    let result_hex = hasher.finalize();
    let result_hex_str = format!("{:#x}", result_hex);
    let result_int = Integer::from_str_radix(&result_hex_str, 16).unwrap();

    // invert to get enough security bits
    match result_int.invert(&config::MODULUS) {
        Ok(inverse) => inverse,
        Err(unchanged) => unchanged,
    }
}

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
    let inverse = match hashed_int.invert(&config::MODULUS) {
        Ok(inverse) => inverse,
        Err(unchanged) => unchanged,
    };

    inverse
        .next_prime()
        .div_rem_floor(config::MODULUS.clone())
        .1
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
    match hashed_int.invert(&config::MODULUS) {
        Ok(inverse) => inverse,
        Err(unchanged) => unchanged,
    }
}

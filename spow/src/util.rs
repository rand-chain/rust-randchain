use ecvrf;
use rug::Integer;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::str::FromStr;

///
/// modulus
///

/// RSA-2048 modulus, taken from [Wikipedia](https://en.wikipedia.org/wiki/RSA_numbers#RSA-2048).
const RSA2048_MODULUS_DECIMAL: &str =
  "251959084756578934940271832400483985714292821262040320277771378360436620207075955562640185258807\
  8440691829064124951508218929855914917618450280848912007284499268739280728777673597141834727026189\
  6375014971824691165077613379859095700097330459748808428401797429100642458691817195118746121515172\
  6546322822168699875491824224336372590851418654620435767984233871847744479207399342365848238242811\
  9816381501067481045166037730605620161967625613384414360383390441495263443219011465754445417842402\
  0924616515723350778707749817125772467962926386356373289912154831438167899885040445364023527381951\
  378636564391212010397122822120720357";

lazy_static! {
    pub static ref RSA2048_MODULUS: Integer = Integer::from_str(RSA2048_MODULUS_DECIMAL).unwrap();
}

///
/// helper functions
///

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
    match result_int.invert(&RSA2048_MODULUS) {
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
    match result_int.invert(&RSA2048_MODULUS) {
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
    let inverse = match hashed_int.invert(&RSA2048_MODULUS) {
        Ok(inverse) => inverse,
        Err(unchanged) => unchanged,
    };

    inverse
        .next_prime()
        .div_rem_floor(RSA2048_MODULUS.clone())
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
    match hashed_int.invert(&RSA2048_MODULUS) {
        Ok(inverse) => inverse,
        Err(unchanged) => unchanged,
    }
}

use ecvrf::VrfPk;
use rug::Integer;
use ser::{Deserializable, Error as ReaderError, Reader, Serializable, Stream};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::io;

use super::config::{MODULUS, STEP};
use super::vdf;

///
/// Sequetial Proof-of-Work
///

#[derive(Debug)]
pub struct SPoW<'a> {
    pubkey: &'a VrfPk,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SPoWResult {
    pub iterations: u32,
    pub randomness: Integer,
    pub proof: vdf::Proof,
}

impl Serializable for SPoWResult {
    fn serialize(&self, stream: &mut Stream) {
        stream
            .append(&self.iterations)
            .append(&self.randomness)
            .append_vector(&self.proof);
    }

    // TODO:
    // fn serialized_size(&self) -> usize
}

impl Deserializable for SPoWResult {
    fn deserialize<T>(reader: &mut Reader<T>) -> Result<Self, ReaderError>
    where
        T: io::Read,
    {
        let res = SPoWResult {
            iterations: reader.read()?,
            randomness: reader.read()?,
            proof: reader.read_vector()?,
        };

        Ok(res)
    }
}

impl SPoW<'_> {
    pub fn new(pubkey: &'static VrfPk) -> Self {
        SPoW { pubkey: pubkey }
    }

    pub fn mine(&mut self, ini_state: &Integer, target: &Integer) -> SPoWResult {
        let mut cur_state = ini_state.clone();
        let mut iters: u32 = 0;

        loop {
            iters += 1;
            let (new_state, diff_valid) = self.solve(&cur_state, target);
            cur_state = new_state;
            if diff_valid {
                break;
            }
        }

        SPoWResult {
            iterations: iters,
            randomness: cur_state.clone(),
            proof: vdf::prove(ini_state, &cur_state, u64::from(iters) * STEP),
        }
    }

    fn solve(&mut self, state: &Integer, target: &Integer) -> (Integer, bool) {
        let y = vdf::eval(state, STEP);
        let hstate = self.h_state(&y);
        (y, validate_difficulty(&hstate, target))
    }

    pub fn verify(
        &mut self,
        g: &Integer,
        y: &Integer,
        result: &SPoWResult,
        target: &Integer,
    ) -> bool {
        let hstate = self.h_state(y);
        if !validate_difficulty(&hstate, target) {
            return false;
        }

        vdf::verify(g, y, u64::from(result.iterations) * STEP, &result.proof)
    }

    /// int(H("pubkey"||pubkey||"state"||state)) mod N
    fn h_state(&mut self, state: &Integer) -> Integer {
        let mut hasher = Sha256::new();
        hasher.update("pubkey".as_bytes());
        hasher.update(self.pubkey.to_bytes());
        hasher.update("state".as_bytes());
        hasher.update(state.to_string_radix(16).as_bytes());
        let result_hex = hasher.finalize();
        // TODO: to_digits from_digits
        let result_hex_str = format!("{:#x}", result_hex);
        let result_int = Integer::from_str_radix(&result_hex_str, 16).unwrap();

        // invert to get enough security bits
        match result_int.invert(&MODULUS) {
            Ok(inverse) => inverse,
            Err(unchanged) => unchanged,
        }
    }
}

///
/// helper functions
///

/// state & target should already be modulo
pub fn validate_difficulty(state: &Integer, target: &Integer) -> bool {
    let mut hasher = Sha256::new();
    let hash_input: String = state.clone().to_string_radix(16);
    // TODO:
    // only hash state for demo purpose, in real-world case, we may need to add other block metadata
    hasher.update(hash_input.as_bytes());
    let hash_result = hasher.finalize();
    // TODO: to_digits from_digits
    let hash_result_str = format!("{:#x}", hash_result);
    let hashed_int = Integer::from_str_radix(&hash_result_str, 16).unwrap();
    (hashed_int.cmp(target) == Ordering::Less) || (hashed_int.cmp(target) == Ordering::Equal)
}

/// int(H("pubkey"||pubkey||"residue"||x)) mod N
pub fn h_g(pubkey: &VrfPk, seed: &Integer) -> Integer {
    let mut hasher = Sha256::new();
    hasher.update("pubkey".as_bytes());
    hasher.update(pubkey.to_bytes());
    hasher.update("residue".as_bytes());
    hasher.update(seed.to_string_radix(16).as_bytes());
    let result_hex = hasher.finalize();
    // TODO: to_digits from_digits
    let result_hex_str = format!("{:#x}", result_hex);
    let result_int = Integer::from_str_radix(&result_hex_str, 16).unwrap();

    // invert to get enough security bits
    match result_int.invert(&MODULUS) {
        Ok(inverse) => inverse,
        Err(unchanged) => unchanged,
    }
}

#[cfg(test)]
mod tests {
    use rug::{integer::Order, Integer};

    #[test]
    fn test_vec_integer_serialize_deserialize() {
        let i = Integer::from(0x0000_0001_0000_0002u64);
        let digits1 = i.to_digits::<u32>(Order::MsfLe);
        let digits2 = i.to_digits::<u32>(Order::MsfBe);

        println!("{:?}", digits1);
        println!("{:?}", digits2);
        // assert_eq!(digits, [0x1234_5678u32.to_be(), 0x9abc_def0u32.to_be()]);

        let zero = Integer::new();
        let digits_zero = zero.to_digits::<u32>(Order::MsfBe);
        println!("{:?}", digits_zero);
        assert!(digits_zero.is_empty());
    }
}

use ecvrf::VrfPk;
use rug::Integer;
use ser::{Deserializable, Serializable};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::vec::Vec;

use super::config::{MODULUS, STEP};
use super::util;

///
/// Sequetial Proof-of-Work
///

#[derive(Debug)]
pub struct SPoW<'a> {
    pubkey: &'a VrfPk,
}

pub type Proof = Vec<Integer>;

#[derive(Debug, Clone, PartialEq)]
pub struct SPoWResult {
    iterations: u64,
    randomness: Integer,
    proof: Proof,
}

impl Serializable for SPoWResult {
    fn serialize(&self, _: &mut ser::Stream) {
        todo!()
    }
}

impl Deserializable for SPoWResult {
    fn deserialize<T, Self>(_: &mut ser::Reader<T>) -> std::result::Result<Self, ser::Error>
    where
        T: std::io::Read,
    {
        todo!()
    }
}

impl SPoW<'_> {
    pub fn new(pubkey: &'static VrfPk) -> Self {
        SPoW { pubkey: pubkey }
    }

    pub fn mine(&mut self, ini_state: &Integer, target: &Integer) -> SPoWResult {
        let mut cur_state = ini_state.clone();
        let mut iters: u64 = 0;

        loop {
            iters += STEP;
            let (new_state, diff_valid) = self.solve(&cur_state, target);
            cur_state = new_state;
            if diff_valid {
                break;
            }
        }

        SPoWResult {
            iterations: iters,
            randomness: cur_state.clone(),
            proof: prove(ini_state, &cur_state, iters),
        }
    }

    fn solve(&mut self, state: &Integer, target: &Integer) -> (Integer, bool) {
        let mut y = state.clone();
        for _ in 0..STEP {
            y = y.clone() * y.clone();
            y = y.div_rem_floor(MODULUS.clone()).1;
        }

        let hstate = self.h_state(&y);
        (y, validate_difficulty(&hstate, target))
    }

    pub fn verify(
        &mut self,
        g: &Integer,
        y: &Integer,
        iterations: u64,
        proof: &Proof,
        target: &Integer,
    ) -> bool {
        let hstate = self.h_state(y);
        if !validate_difficulty(&hstate, target) {
            return false;
        }

        let (mut x_i, mut y_i) = (g.clone(), y.clone());
        let mut t = iterations;
        let two: Integer = 2u64.into();
        for mu_i in proof {
            let r_i = util::hash_fs(&[&x_i, &y_i, &mu_i]);

            let xi_ri = x_i.clone().pow_mod(&r_i, &MODULUS).unwrap();
            x_i = (xi_ri * mu_i.clone()).div_rem_floor(MODULUS.clone()).1;

            let mui_ri = mu_i.clone().pow_mod(&r_i, &MODULUS).unwrap();
            y_i = (mui_ri * y_i.clone()).div_rem_floor(MODULUS.clone()).1;

            t = t / 2;
            if (t % 2 != 0) && (t != 1) {
                t += 1;
                y_i = y_i.clone().pow_mod(&two, &MODULUS).unwrap();
            }
        }

        y_i == x_i.pow_mod(&two, &MODULUS).unwrap()
    }

    /// int(H("pubkey"||pubkey||"state"||state)) mod N
    fn h_state(&mut self, state: &Integer) -> Integer {
        let mut hasher = Sha256::new();
        hasher.update("pubkey".as_bytes());
        hasher.update(self.pubkey.to_bytes());
        hasher.update("state".as_bytes());
        hasher.update(state.to_string_radix(16).as_bytes());
        let result_hex = hasher.finalize();
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
    let hash_result_str = format!("{:#x}", hash_result);
    let hashed_int = Integer::from_str_radix(&hash_result_str, 16).unwrap();
    (hashed_int.cmp(target) == Ordering::Less) || (hashed_int.cmp(target) == Ordering::Equal)
}

fn prove(g: &Integer, y: &Integer, iterations: u64) -> Proof {
    let (mut x_i, mut y_i) = (g.clone(), y.clone());
    let mut proof = Proof::new();

    let mut t = iterations;
    let two: Integer = 2u64.into();
    while t >= 2 {
        let two_exp = Integer::from(1) << ((t / 2) as u32); // 2^(t/2)
        let mu_i = x_i.clone().pow_mod(&two_exp, &MODULUS).unwrap();

        let r_i = util::hash_fs(&[&x_i, &y_i, &mu_i]);

        let xi_ri = x_i.clone().pow_mod(&r_i, &MODULUS).unwrap();
        x_i = (xi_ri * mu_i.clone()).div_rem_floor(MODULUS.clone()).1;

        let mui_ri = mu_i.clone().pow_mod(&r_i, &MODULUS).unwrap();
        y_i = (mui_ri * y_i.clone()).div_rem_floor(MODULUS.clone()).1;

        t = t / 2;
        if (t % 2 != 0) && (t != 1) {
            t += 1;
            y_i = y_i.clone().pow_mod(&two, &MODULUS).unwrap();
        }

        proof.push(mu_i);
    }

    proof
}

/// int(H("pubkey"||pubkey||"residue"||x)) mod N
pub fn h_g(pubkey: &VrfPk, seed: &Integer) -> Integer {
    let mut hasher = Sha256::new();
    hasher.update("pubkey".as_bytes());
    hasher.update(pubkey.to_bytes());
    hasher.update("residue".as_bytes());
    hasher.update(seed.to_string_radix(16).as_bytes());
    let result_hex = hasher.finalize();
    let result_hex_str = format!("{:#x}", result_hex);
    let result_int = Integer::from_str_radix(&result_hex_str, 16).unwrap();

    // invert to get enough security bits
    match result_int.invert(&MODULUS) {
        Ok(inverse) => inverse,
        Err(unchanged) => unchanged,
    }
}

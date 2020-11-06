use ecvrf::VrfPk;
use rug::Integer;
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

#[derive(Debug)]
pub struct SPoWResult {
    iterations: u64,
    randomness: Integer,
    // TODO: serialize & unserialize
    proof: Proof,
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

    pub fn solve(&mut self, state: &Integer, target: &Integer) -> (Integer, bool) {
        let mut y = state.clone();
        for _ in 0..STEP {
            y = y.clone() * y.clone();
            y = y.div_rem_floor(MODULUS.clone()).1;
        }

        let hstate = util::h_state(self.pubkey, &y);
        (y, util::validate_difficulty(&hstate, target))
    }

    pub fn verify(
        &mut self,
        g: &Integer,
        y: &Integer,
        iterations: u64,
        proof: &Proof,
        target: &Integer,
    ) -> bool {
        let hstate = util::h_state(self.pubkey, y);
        if !util::validate_difficulty(&hstate, target) {
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
}

///
/// helper function
///
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

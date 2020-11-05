use super::util;
use ecvrf;
use rug::Integer;
use std::vec::Vec;

pub fn mine(
    step: u64,
    pubkey: &ecvrf::VrfPk,
    modulus: &Integer,
    ini_state: &Integer,
    target: &Integer,
) -> (Integer, Vec<Integer>, u64) {
    let mut cur_state = ini_state.clone();
    let mut iters: u64 = 0;

    loop {
        iters += step;
        let (new_state, diff_valid) = solve(modulus, &cur_state, step, pubkey, target);
        cur_state = new_state;
        if diff_valid {
            break;
        }
    }

    let pi = prove(modulus, ini_state, &cur_state, iters);

    (cur_state.clone(), pi, iters)
}

pub fn verify(
    modulus: &Integer,
    g: &Integer,
    y: &Integer,
    iterations: u64,
    pi_list: &Vec<Integer>,
    pubkey: &ecvrf::VrfPk,
    target: &Integer,
) -> bool {
    let hstate = util::h_state(modulus, pubkey, y);
    if !util::validate_difficulty(&hstate, target) {
        return false;
    }

    let (mut x_i, mut y_i) = (g.clone(), y.clone());
    let mut t = iterations;
    let two: Integer = 2u64.into();
    for mu_i in pi_list {
        let r_i = util::hash_fs(modulus, &[&x_i, &y_i, &mu_i]);

        let xi_ri = x_i.clone().pow_mod(&r_i, modulus).unwrap();
        x_i = (xi_ri * mu_i.clone()).div_rem_floor(modulus.clone()).1;

        let mui_ri = mu_i.clone().pow_mod(&r_i, modulus).unwrap();
        y_i = (mui_ri * y_i.clone()).div_rem_floor(modulus.clone()).1;

        t = t / 2;
        if (t % 2 != 0) && (t != 1) {
            t += 1;
            y_i = y_i.clone().pow_mod(&two, modulus).unwrap();
        }
    }

    y_i == x_i.pow_mod(&two, modulus).unwrap()
}

pub fn solve(
    modulus: &Integer,
    state: &Integer,
    step: u64,
    pubkey: &ecvrf::VrfPk,
    target: &Integer,
) -> (Integer, bool) {
    let mut y = state.clone();
    for _ in 0..step {
        y = y.clone() * y.clone();
        y = y.div_rem_floor(modulus.clone()).1;
    }

    let hstate = util::h_state(modulus, pubkey, &y);
    (y, util::validate_difficulty(&hstate, target))
}

pub fn prove(modulus: &Integer, g: &Integer, y: &Integer, iterations: u64) -> Vec<Integer> {
    let (mut x_i, mut y_i) = (g.clone(), y.clone());
    let mut pi_list = Vec::<Integer>::new();

    let mut t = iterations;
    let two: Integer = 2u64.into();
    while t >= 2 {
        let two_exp = Integer::from(1) << ((t / 2) as u32); // 2^(t/2)
        let mu_i = x_i.clone().pow_mod(&two_exp, modulus).unwrap();

        let r_i = util::hash_fs(modulus, &[&x_i, &y_i, &mu_i]);

        let xi_ri = x_i.clone().pow_mod(&r_i, modulus).unwrap();
        x_i = (xi_ri * mu_i.clone()).div_rem_floor(modulus.clone()).1;

        let mui_ri = mu_i.clone().pow_mod(&r_i, modulus).unwrap();
        y_i = (mui_ri * y_i.clone()).div_rem_floor(modulus.clone()).1;

        t = t / 2;
        if (t % 2 != 0) && (t != 1) {
            t += 1;
            y_i = y_i.clone().pow_mod(&two, modulus).unwrap();
        }

        pi_list.push(mu_i);
    }

    pi_list
}

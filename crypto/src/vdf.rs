use rug::{integer::Order, Integer};
use sha2::{Digest, Sha256};
use std::str::FromStr;
use std::vec::Vec;

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
    pub static ref MODULUS: Integer = Integer::from_str(RSA2048_MODULUS_DECIMAL).unwrap();
}

/// Fiat–Shamir heuristic non-iterative signature
pub fn hash_fs(inputs: &[&Integer]) -> Integer {
    let mut hasher = Sha256::new();
    for input in inputs {
        hasher.update(input.to_digits::<u8>(Order::Lsf));
        hasher.update("\n".as_bytes());
    }
    let seed = hasher.finalize();
    let prefix = "fs_part_".as_bytes();
    // concat 8 sha256 to a 2048-bit hash
    let all_2048: Vec<u8> = (0..((2048 / 256) as u8))
        .map(|index| {
            let mut hasher = Sha256::new();
            hasher.update(prefix);
            hasher.update(vec![index]);
            hasher.update(seed.clone());
            hasher.finalize()
        })
        .flatten()
        .collect();
    let result = Integer::from_digits(&all_2048, Order::Lsf);
    result.div_rem_floor(MODULUS.clone()).1
}

pub type Proof = Vec<Integer>;

pub fn eval(g: &Integer, t: u64) -> Integer {
    let mut y = g.clone();
    for _ in 0..t {
        y = y.clone() * y.clone();
        y = y.div_rem_floor(MODULUS.clone()).1;
    }

    y
}

pub fn prove(g: &Integer, y: &Integer, iterations: u64) -> Proof {
    let (mut x_i, mut y_i) = (g.clone(), y.clone());
    let mut proof = Proof::new();

    let mut t = iterations;
    let two = Integer::from(2);
    while t >= 2 {
        let two_exp = Integer::from(1) << (t / 2) as u32; // 2^(t/2)
        let mu_i = x_i.clone().pow_mod(&two_exp, &MODULUS).unwrap();

        let r_i = hash_fs(&[&x_i, &y_i, &mu_i]);

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

pub fn verify(g: &Integer, y: &Integer, iterations: u64, proof: &Proof) -> bool {
    let (mut x_i, mut y_i) = (g.clone(), y.clone());
    let mut t = iterations;
    let two = Integer::from(2);
    for mu_i in proof {
        let r_i = hash_fs(&[&x_i, &y_i, &mu_i]);

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

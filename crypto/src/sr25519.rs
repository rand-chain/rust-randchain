use schnorrkel::{ExpansionMode, Keypair, MiniSecretKey, PublicKey, SecretKey, Signature};

/// Generate a key pair.
///
/// * seed: UIntArray with 32 element
///
/// returned vector is the concatenation of
/// 1. the secret key (64 bytes)
/// 2. the public key (32 bytes)
pub fn keypair_from_seed(seed: &[u8]) -> (SecretKey, PublicKey) {
    match MiniSecretKey::from_bytes(seed) {
        Ok(mini) => {
            let keypair = mini.expand_to_keypair(ExpansionMode::Ed25519);
            (keypair.secret.clone(), keypair.public.clone())
        }
        Err(_) => panic!("Provided seed is invalid."),
    }
}

/// Keypair helper function.
fn create_from_pair(pair: &[u8]) -> Keypair {
    match Keypair::from_bytes(pair) {
        Ok(pair) => return pair,
        Err(_) => panic!("Provided pair is invalid."),
    }
}

/// SecretKey helper
fn create_secret(secret_key: &[u8]) -> SecretKey {
    match SecretKey::from_bytes(secret_key) {
        Ok(secret) => return secret,
        Err(_) => panic!("Provided private key is invalid."),
    }
}

/// PublicKey helper
fn create_public(public_key: &[u8]) -> PublicKey {
    match PublicKey::from_bytes(public_key) {
        Ok(public) => return public,
        Err(_) => panic!("Provided public key is invalid."),
    }
}

/// Sign a message
///
/// The combination of both public and private key must be provided.
/// This is effectively equivalent to a keypair.
///
/// * public: UIntArray with 32 element
/// * private: UIntArray with 64 element
/// * message: Arbitrary length UIntArray
///
/// * returned vector is the signature consisting of 64 bytes.
pub fn sign(pk: &PublicKey, sk: &SecretKey, message: &[u8]) -> Vec<u8> {
    let context = b"";
    sk.sign_simple(context, message, pk).to_bytes().to_vec()
}

/// Verify a message and its corresponding against a public key;
///
/// * signature: UIntArray with 64 element
/// * message: Arbitrary length UIntArray
/// * pubkey: UIntArray with 32 element
pub fn verify(signature: &[u8], message: &[u8], pk: &PublicKey) -> bool {
    let context = b"";
    let signature = match Signature::from_bytes(signature) {
        Ok(signature) => signature,
        Err(_) => return false,
    };

    pk.verify_simple(context, message, &signature).is_ok()
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use hex_literal::hex;
    use schnorrkel::{KEYPAIR_LENGTH, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, SIGNATURE_LENGTH};

    fn generate_random_seed() -> Vec<u8> {
        (0..32).map(|_| rand::random::<u8>()).collect()
    }

    #[test]
    fn can_create_keypair() {
        let seed = generate_random_seed();
        let (sk, pk) = keypair_from_seed(seed.as_slice());

        assert!(sk.to_bytes().len() == SECRET_KEY_LENGTH);
        assert!(pk.to_bytes().len() == PUBLIC_KEY_LENGTH);
    }

    #[test]
    fn creates_pair_from_known() {
        let seed = hex!("fac7959dbfe72f052e5a0c3c8d6530f202b02fd8f9f5ca3580ec8deb7797479e");
        let expected = hex!("46ebddef8cd9bb167dc30878d7113b7e168e6f0646beffd77d69d39bad76b47a");
        let (sk, pk) = keypair_from_seed(&seed);

        assert_eq!(pk.to_bytes(), expected);
    }

    #[test]
    fn can_sign_message() {
        let seed = generate_random_seed();
        let (sk, pk) = keypair_from_seed(seed.as_slice());
        let message = b"this is a message";
        let signature = sign(&pk, &sk, message);

        assert!(signature.len() == SIGNATURE_LENGTH);
    }

    #[test]
    fn can_verify_message() {
        let seed = generate_random_seed();
        let (sk, pk) = keypair_from_seed(seed.as_slice());
        let message = b"this is a message";
        let signature = sign(&pk, &sk, message);

        assert!(verify(&signature[..], message, &pk));
    }
}

use schnorrkel::{ExpansionMode, Keypair, MiniSecretKey, PublicKey, SecretKey, Signature};

/// Generate a key pair.
///
/// * seed: UIntArray with 32 element
///
/// returned vector is the concatenation of
/// 1. the secret key (64 bytes)
/// 2. the public key (32 bytes)
pub fn keypair_from_seed(seed: &[u8]) -> Vec<u8> {
    match MiniSecretKey::from_bytes(seed) {
        Ok(mini) => {
            let keypair = mini.expand_to_keypair(ExpansionMode::Ed25519);
            return keypair.to_bytes().to_vec();
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
pub fn sign(public_key: &[u8], secret_key: &[u8], message: &[u8]) -> Vec<u8> {
    let context = b"";
    create_secret(secret_key)
        .sign_simple(context, message, &create_public(public_key))
        .to_bytes()
        .to_vec()
}

/// Verify a message and its corresponding against a public key;
///
/// * signature: UIntArray with 64 element
/// * message: Arbitrary length UIntArray
/// * pubkey: UIntArray with 32 element
pub fn verify(signature: &[u8], message: &[u8], public_key: &[u8]) -> bool {
    let context = b"";
    let signature = match Signature::from_bytes(signature) {
        Ok(signature) => signature,
        Err(_) => return false,
    };

    create_public(public_key)
        .verify_simple(context, message, &signature)
        .is_ok()
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use hex_literal::hex;
    use schnorrkel::{KEYPAIR_LENGTH, SECRET_KEY_LENGTH, SIGNATURE_LENGTH};

    fn generate_random_seed() -> Vec<u8> {
        (0..32).map(|_| rand::random::<u8>()).collect()
    }

    #[test]
    fn can_create_keypair() {
        let seed = generate_random_seed();
        let keypair = keypair_from_seed(seed.as_slice());

        assert!(keypair.len() == KEYPAIR_LENGTH);
    }

    #[test]
    fn creates_pair_from_known() {
        let seed = hex!("fac7959dbfe72f052e5a0c3c8d6530f202b02fd8f9f5ca3580ec8deb7797479e");
        let expected = hex!("46ebddef8cd9bb167dc30878d7113b7e168e6f0646beffd77d69d39bad76b47a");
        let keypair = keypair_from_seed(&seed);
        let public = &keypair[SECRET_KEY_LENGTH..KEYPAIR_LENGTH];

        assert_eq!(public, expected);
    }

    #[test]
    fn can_sign_message() {
        let seed = generate_random_seed();
        let keypair = keypair_from_seed(seed.as_slice());
        let private = &keypair[0..SECRET_KEY_LENGTH];
        let public = &keypair[SECRET_KEY_LENGTH..KEYPAIR_LENGTH];
        let message = b"this is a message";
        let signature = sign(public, private, message);

        assert!(signature.len() == SIGNATURE_LENGTH);
    }

    #[test]
    fn can_verify_message() {
        let seed = generate_random_seed();
        let keypair = keypair_from_seed(seed.as_slice());
        let private = &keypair[0..SECRET_KEY_LENGTH];
        let public = &keypair[SECRET_KEY_LENGTH..KEYPAIR_LENGTH];
        let message = b"this is a message";
        let signature = sign(public, private, message);

        assert!(verify(&signature[..], message, public));
    }
}

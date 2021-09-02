use schnorrkel::{ExpansionMode, Keypair, MiniSecretKey, PublicKey, SecretKey, Signature};

/// SecretKey helper
fn create_sk(sk_bytes: &[u8]) -> SecretKey {
    match SecretKey::from_bytes(sk_bytes) {
        Ok(sk) => return sk,
        Err(_) => panic!("Provided private key is invalid."),
    }
}

/// PublicKey helper
fn create_pk(pk_bytes: &[u8]) -> PublicKey {
    match PublicKey::from_bytes(pk_bytes) {
        Ok(pk) => return pk,
        Err(_) => panic!("Provided public key is invalid."),
    }
}

/// Generate a key pair.
/// In:
/// - seed: UIntArray with 32 element
///
/// Out:
/// - sk: SecretKey
/// - pk: PublicKey
pub fn create_keypair(seed: &[u8]) -> (SecretKey, PublicKey) {
    match MiniSecretKey::from_bytes(seed) {
        Ok(mini) => {
            let keypair = mini.expand_to_keypair(ExpansionMode::Ed25519);
            (keypair.secret.clone(), keypair.public.clone())
        }
        Err(_) => panic!("Provided seed is invalid."),
    }
}

/// Sign a message
///
/// The combination of both public and private key must be provided.
/// This is effectively equivalent to a keypair.
///
/// * sk: SecretKey
/// * message: Arbitrary length UIntArray
///
/// * returned vector is the signature consisting of 64 bytes.
pub fn sign(sk: &SecretKey, message: &[u8]) -> Vec<u8> {
    let context = b"";
    let pk = sk.to_public();
    sk.sign_simple(context, message, &pk).to_bytes().to_vec()
}

/// Verify a message and its corresponding against a public key;
///
/// * signature: UIntArray with 64 element
/// * message: Arbitrary length UIntArray
/// * pk: UIntArray with 32 element
pub fn verify(pk: &PublicKey, message: &[u8], signature: &[u8]) -> bool {
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
        let (sk, pk) = create_keypair(seed.as_slice());

        assert!(sk.to_bytes().len() == SECRET_KEY_LENGTH);
        assert!(pk.to_bytes().len() == PUBLIC_KEY_LENGTH);
    }

    #[test]
    fn creates_pair_from_known() {
        let seed = hex!("fac7959dbfe72f052e5a0c3c8d6530f202b02fd8f9f5ca3580ec8deb7797479e");
        let expected = hex!("46ebddef8cd9bb167dc30878d7113b7e168e6f0646beffd77d69d39bad76b47a");
        let (sk, pk) = create_keypair(&seed);

        assert_eq!(pk.to_bytes(), expected);
    }

    #[test]
    fn can_sign_message() {
        let seed = generate_random_seed();
        let (sk, pk) = create_keypair(seed.as_slice());
        let message = b"this is a message";
        let signature = sign(&sk, message);

        assert!(signature.len() == SIGNATURE_LENGTH);
    }

    #[test]
    fn can_verify_message() {
        let seed = generate_random_seed();
        let (sk, pk) = create_keypair(seed.as_slice());
        let message = b"this is a message";
        let signature = sign(&sk, message);

        assert!(verify(&pk, message, &signature[..]));
    }
}

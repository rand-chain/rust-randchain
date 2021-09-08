use crypto::sr25519::{PK, SK};
use std::{fs, path::Path};

pub struct KeyPair {
    pub sk: SK,
    pub pk: PK,
}

impl KeyPair {
    pub fn from_sk(sk: SK) -> KeyPair {
        KeyPair {
            sk: sk.clone(),
            pk: sk.to_public(),
        }
    }

    pub fn load<P>(path: &P) -> Result<Self, String>
    where
        P: AsRef<Path>,
    {
        let sk_string = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => return Err("read sk file error".to_owned()),
        };
        let sk_bytes = sk_string.as_bytes();

        let sk = SK::from_bytes(sk_bytes).unwrap();
        Ok(KeyPair::from_sk(sk))
    }

    pub fn save<P>(&self, path: &P) -> Result<(), String>
    where
        P: AsRef<Path>,
    {
        let sk_bytes = self.sk.to_bytes();
        match fs::write(path, sk_bytes) {
            Ok(_) => Ok(()),
            Err(_) => Err("save sk file error".to_owned()),
        }
    }
}

use crypto::sr25519::{PK, SK};
use std::fs::File;
use std::io::prelude::*;
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
        // open key file
        let mut f = match File::open(path) {
            Ok(s) => s,
            Err(err) => return Err(format!("open sk file error: {:?}", err)),
        };
        // read key file
        let mut sk_bytes = Vec::new();
        match f.read_to_end(&mut sk_bytes) {
            Ok(_) => (),
            Err(err) => return Err(format!("read sk file error: {:?}", err)),
        };
        // parse key file to keypair
        let sk = match SK::from_bytes(&sk_bytes) {
            Ok(s) => s,
            Err(err) => return Err(format!("parse sk file error: {:?}", err)),
        };
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

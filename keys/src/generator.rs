use error::Error;
use keypair::KeyPair;
use network::Network;
use SECP256K1;

pub trait Generator {
	fn generate(&self) -> Result<KeyPair, Error>;
}

pub struct Random {
	network: Network,
}

impl Random {
	pub fn new(network: Network) -> Self {
		Random { network: network }
	}
}

impl Generator for Random {
	fn generate(&self) -> Result<KeyPair, Error> {
		let context = &SECP256K1;
		let mut rng = rand::thread_rng();
		let (secret, public) = context.generate_keypair(&mut rng)?;
		Ok(KeyPair::from_keypair(secret, public, self.network))
	}
}

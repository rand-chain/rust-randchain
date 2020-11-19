use chain::IndexedBlock;
use crypto::dhash256;
use error::Error;
use primitives::bytes::Bytes;
use rug::Integer;
use ser::Stream;

fn h_g(block: &IndexedBlock) -> Integer {
    let mut stream = Stream::default();
    stream
        .append(&block.header.raw.version)
        .append(&block.header.raw.previous_header_hash)
        .append(&block.header.raw.time)
        .append(&block.header.raw.bits)
        .append(&Bytes::from(block.header.raw.pubkey.to_bytes().to_vec()));
    let data = stream.out();
    let h = dhash256(&data);
    let result = Integer::from_str_radix(&h.to_string(), 16).unwrap();

    // invert to get enough security bits
    match result.invert(&vdf::MODULUS) {
        Ok(inverse) => inverse,
        Err(unchanged) => unchanged,
    }
}

pub struct BlockVerifier<'a> {
    pub block: &'a IndexedBlock,
}

impl<'a> BlockVerifier<'a> {
    pub fn new(block: &'a IndexedBlock) -> Self {
        BlockVerifier { block: block }
    }

    pub fn check(&self) -> Result<(), Error> {
        let g = h_g(self.block);

        match vdf::verify(
            &g,
            &self.block.header.raw.randomness,
            self.block.header.raw.iterations,
            &self.block.header.raw.proof,
        ) {
            true => Ok(()),
            false => Err(Error::Vdf),
        }
    }
}

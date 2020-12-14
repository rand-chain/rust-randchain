#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct SubmitBlockRequest {
	// TODO: use indexed_block?
    // pub version: u32,
    // pub previous_header_hash: H256,
    // pub time: u32,
    // pub bits: Compact,
    // pub pubkey: VrfPk,
    // pub iterations: u32,
    // pub randomness: Integer,
    // pub proof: vdf::Proof,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct SubmitBlockResponse {}

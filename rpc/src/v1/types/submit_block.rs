use super::block::RawBlock;

/// SubmitBlock Request
/// See https://github.com/btcsuite/btcd/blob/2d7825cf709fc6ac15921cecfd4d62ec78ccbba2/docs/json_rpc_api.md#submitblock and https://en.bitcoin.it/wiki/BIP_0022 for the specification
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct SubmitBlockRequest {
    // data (string, required) serialized, hex-encoded block
    pub data: RawBlock,
    // params (json object, optional, default=nil) this parameter is currently **ignored**. Some miner pools provide `workid` here
}

/// SubmitBlock Response
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct SubmitBlockResponse {}

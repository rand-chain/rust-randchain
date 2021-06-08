use chain::{Block, IndexedBlock};
use jsonrpc_core::Error;
use miner;
use ser::{deserialize, serialize};
use sync;
use v1::traits::Miner;
use v1::types::{
    BlockTemplate, BlockTemplateRequest, Bytes, SubmitBlockRequest, SubmitBlockResponse,
};

pub struct MinerClient<T: MinerClientCoreApi> {
    core: T,
}

pub trait MinerClientCoreApi: Send + Sync + 'static {
    fn get_block_template(&self) -> Result<miner::BlockTemplate, Error>;

    fn submit_block(
        &self,
        submit_block_req: SubmitBlockRequest,
    ) -> Result<SubmitBlockResponse, Error>;
}

pub struct MinerClientCore {
    local_sync_node: sync::LocalNodeRef,
}

impl MinerClientCore {
    pub fn new(local_sync_node: sync::LocalNodeRef) -> Self {
        MinerClientCore { local_sync_node }
    }
}

impl MinerClientCoreApi for MinerClientCore {
    // when receiving getblocktemplate request
    fn get_block_template(&self) -> Result<miner::BlockTemplate, Error> {
        Ok(self.local_sync_node.get_block_template())
    }

    // when receiving submitblock request
    fn submit_block(
        &self,
        submit_block_req: SubmitBlockRequest,
    ) -> Result<SubmitBlockResponse, Error> {
        // Deserialise to Block
        let data_vec: Vec<u8> = submit_block_req.data.into();
        let blk: Block = match deserialize(&data_vec[..]) {
            Ok(block) => block,
            Err(_) => return Err(Error::parse_error()),
        };
        // Convert Block to IndexedBlock
        let indexed_blk = IndexedBlock::from_raw(blk);
        trace!(
            "received submitblock request with block hash = {:?}",
            indexed_blk.hash()
        );
        // commit IndexedBlock locally
        // TODO RH check why on_block does not work
        self.local_sync_node.on_block(0, indexed_blk.clone());
        Ok(SubmitBlockResponse {})
    }
}

impl<T> MinerClient<T>
where
    T: MinerClientCoreApi,
{
    pub fn new(core: T) -> Self {
        MinerClient { core }
    }
}

impl<T> Miner for MinerClient<T>
where
    T: MinerClientCoreApi,
{
    fn get_block_template(&self, _request: BlockTemplateRequest) -> Result<BlockTemplate, Error> {
        let tpl: BlockTemplate = match self.core.get_block_template() {
            Ok(tpl) => {
                trace!(
                    "getblocktemplate OK: previous_header_hash = {:?}",
                    tpl.previous_header_hash
                );
                tpl.into()
            }
            Err(err) => {
                return {
                    error!("error upon getblocktemplate: {:?}", err);
                    Err(err)
                }
            }
        };
        Ok(tpl)
    }

    fn submit_block(
        &self,
        submit_block_req: SubmitBlockRequest,
    ) -> Result<SubmitBlockResponse, Error> {
        let resp: SubmitBlockResponse = match self.core.submit_block(submit_block_req) {
            Ok(resp) => {
                trace!("submitblock OK");
                resp
            }
            Err(err) => {
                error!("error upon submitblock: {:?}", err);
                return Err(err);
            }
        };
        Ok(resp)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use jsonrpc_core::IoHandler;
    use miner;
    use primitives::hash::H256;
    use v1::traits::Miner;

    #[derive(Default)]
    struct SuccessMinerClientCore;

    impl MinerClientCoreApi for SuccessMinerClientCore {
        fn get_block_template(&self) -> Result<miner::BlockTemplate, Error> {
            Ok(miner::BlockTemplate {
                version: 777,
                previous_header_hash: H256::from(1),
                bits: 44.into(),
                height: 55,
            })
        }

        fn submit_block(
            &self,
            submit_block_req: SubmitBlockRequest,
        ) -> Result<SubmitBlockResponse, Error> {
            Ok(SubmitBlockResponse {})
        }
    }

    #[test]
    fn getblocktemplate_accepted() {
        let client = MinerClient::new(SuccessMinerClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler
            .handle_request_sync(
                &(r#"
            {
                "jsonrpc": "2.0",
                "method": "getblocktemplate",
                "params": [{}],
                "id": 1
            }"#),
            )
            .unwrap();

        // direct hash is 0100000000000000000000000000000000000000000000000000000000000000
        // but client expects reverse hash
        assert_eq!(
            &sample,
            r#"{"jsonrpc":"2.0","result":{"bits":44,"coinbaseaux":null,"height":55,"mutable":null,"previousblockhash":"0000000000000000000000000000000000000000000000000000000000000001","rules":null,"target":"0000000000000000000000000000000000000000000000000000000000000000","vbavailable":null,"vbrequired":null,"version":777,"weightlimit":null},"id":1}"#
        );
    }
}

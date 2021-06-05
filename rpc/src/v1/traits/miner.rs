use jsonrpc_core::Error;

use v1::types::{BlockTemplate, BlockTemplateRequest, SubmitBlockRequest, SubmitBlockResponse};

build_rpc_trait! {
    /// Parity-randchain miner data interface.
    pub trait Miner {
        /// Get block template for mining.
        /// @curl-example: curl --data-binary '{"jsonrpc": "2.0", "method": "getblocktemplate", "params": [{"capabilities": ["coinbasetxn", "workid", "coinbase/append"]}], "id":1 }' -H 'content-type: application/json' http://127.0.0.1:8332/
        #[rpc(name = "getblocktemplate")]
        fn get_block_template(&self, BlockTemplateRequest) -> Result<BlockTemplate, Error>;

        /// Submit mined block.
        /// TODO: @curl-example: curl --data-binary '{"jsonrpc": "2.0", "method": "submitblock", "params": [{"data": "010000003d86e3dfab8149f072e31eedb1ef645da7f7970c8e7998d6f96995cdd09cd07bbfecac600500000020742ffeb4e26c7caf83a25783ba8524f5da9db026e586de0c1e3a1d2c14f9012a00000000fd000194cb44f8bcea06be63816d3ef71508c3a46d6d9c10a043f6e15fe57dde8f3defb43c424ed71fa6ea327b414b219afa063e2e27ac3e56838c5c4b896c71958cab053ecca89390530d6153931fec3ccaa5e857b6ca9790bb0fdfa2983e00218fff2727db27b0acaf49f70b74fedabf77a56708bf1c06ca45fb0f8153d1f2fe8d12c0c553087f69b15932aaf0c7871add7f7200f7939c94098eddfb1ef29a98c633d902e2bdd282527955abc0daa5d3671d08ed0cfdb827e04a0b49344b63cdcd326f1e364360e71dcd2f8fa12774b4832e0cd8986b7402d5225641bc7dc95d92482c9e7b03807cab6f2deb4bd8cf8ac47d89c64c47d0fd93c01f77efddc041407a00"}], "id":1 }' -H 'content-type: application/json' http://127.0.0.1:8332/
        #[rpc(name = "submitblock")]
        fn submit_block(&self, SubmitBlockRequest) -> Result<SubmitBlockResponse, Error>;
    }
}

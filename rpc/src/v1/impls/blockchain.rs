use hex::ToHex;
use jsonrpc_core::Error;
use jsonrpc_macros::Trailing;
use primitives::hash::H256 as GlobalH256;
use ser::serialize;
use storage;
use v1::helpers::errors::{block_at_height_not_found, block_not_found};
use v1::traits::BlockChain;
use v1::types::H256;
use v1::types::U256;
use v1::types::{GetBlockResponse, RawBlock, VerboseBlock};
use verification;

pub struct BlockChainClient<T: BlockChainClientCoreApi> {
    core: T,
}

pub trait BlockChainClientCoreApi: Send + Sync + 'static {
    fn best_block_hash(&self) -> GlobalH256;
    fn block_count(&self) -> u32;
    fn block_hash(&self, height: u32) -> Option<GlobalH256>;
    fn difficulty(&self) -> f64;
    fn raw_block(&self, hash: GlobalH256) -> Option<RawBlock>;
    fn verbose_block(&self, hash: GlobalH256) -> Option<VerboseBlock>;
}

pub struct BlockChainClientCore {
    storage: storage::SharedStore,
}

impl BlockChainClientCore {
    pub fn new(storage: storage::SharedStore) -> Self {
        BlockChainClientCore { storage: storage }
    }
}

impl BlockChainClientCoreApi for BlockChainClientCore {
    fn best_block_hash(&self) -> GlobalH256 {
        self.storage.best_block().hash
    }

    fn block_count(&self) -> u32 {
        self.storage.best_block().number
    }

    fn block_hash(&self, height: u32) -> Option<GlobalH256> {
        self.storage.block_hash(height)
    }

    fn difficulty(&self) -> f64 {
        self.storage.difficulty()
    }

    fn raw_block(&self, hash: GlobalH256) -> Option<RawBlock> {
        self.storage
            .block(hash.into())
            .map(|block| serialize(&block.to_raw_block()).into())
    }

    fn verbose_block(&self, hash: GlobalH256) -> Option<VerboseBlock> {
        self.storage.block(hash.into()).map(|block| {
            let height = self.storage.block_number(block.hash());
            let confirmations = match height {
                Some(block_number) => (self.storage.best_block().number - block_number + 1) as i64,
                None => -1,
            };
            let block_size = block.size();

            VerboseBlock {
                confirmations: confirmations,
                size: block_size as u32,
                height: height,
                difficulty: block.header.raw.bits.to_f64(),
                chainwork: U256::default(), // TODO: read from storage
                previousblockhash: Some(block.header.raw.previous_header_hash.clone().into()),
                nextblockhash: height
                    .and_then(|h| self.storage.block_hash(h + 1).map(|h| h.into())),
                bits: block.header.raw.bits.into(),
                hash: block.hash().clone().into(),
                pubkey_hex: block.header.raw.pubkey.to_bytes().to_hex(),
                randomness_hex: block.header.raw.randomness.to_string_radix(16),
                iterations: block.header.raw.iterations,
                version: block.header.raw.version,
                version_hex: format!("{:x}", &block.header.raw.version),
            }
        })
    }
}

impl<T> BlockChainClient<T>
where
    T: BlockChainClientCoreApi,
{
    pub fn new(core: T) -> Self {
        BlockChainClient { core: core }
    }
}

impl<T> BlockChain for BlockChainClient<T>
where
    T: BlockChainClientCoreApi,
{
    fn best_block_hash(&self) -> Result<H256, Error> {
        Ok(self.core.best_block_hash().reversed().into())
    }

    fn block_count(&self) -> Result<u32, Error> {
        Ok(self.core.block_count())
    }

    fn block_hash(&self, height: u32) -> Result<H256, Error> {
        self.core
            .block_hash(height)
            .map(|h| h.reversed().into())
            .ok_or(block_at_height_not_found(height))
    }

    fn difficulty(&self) -> Result<f64, Error> {
        Ok(self.core.difficulty())
    }

    fn block(&self, hash: H256, verbose: Trailing<bool>) -> Result<GetBlockResponse, Error> {
        let global_hash: GlobalH256 = hash.clone().into();
        if verbose.unwrap_or_default() {
            let verbose_block = self.core.verbose_block(global_hash.reversed());
            if let Some(mut verbose_block) = verbose_block {
                verbose_block.previousblockhash =
                    verbose_block.previousblockhash.map(|h| h.reversed());
                verbose_block.nextblockhash = verbose_block.nextblockhash.map(|h| h.reversed());
                verbose_block.hash = verbose_block.hash.reversed();
                verbose_block.randomness_hex = verbose_block.randomness_hex;
                Some(GetBlockResponse::Verbose(verbose_block))
            } else {
                None
            }
        } else {
            self.core
                .raw_block(global_hash.reversed())
                .map(|block| GetBlockResponse::Raw(block))
        }
        .ok_or(block_not_found(hash))
    }
}

#[cfg(test)]
pub mod tests {
    extern crate test_data;

    use super::*;
    use db::BlockChainDatabase;
    use jsonrpc_core::IoHandler;
    use primitives::hash::H256 as GlobalH256;
    use std::sync::Arc;
    use v1::traits::BlockChain;
    use v1::types::{RawBlock, VerboseBlock};

    #[derive(Default)]
    struct SuccessBlockChainClientCore;
    #[derive(Default)]
    struct ErrorBlockChainClientCore;

    impl BlockChainClientCoreApi for SuccessBlockChainClientCore {
        fn best_block_hash(&self) -> GlobalH256 {
            test_data::genesis().hash()
        }

        fn block_count(&self) -> u32 {
            1
        }

        fn block_hash(&self, _height: u32) -> Option<GlobalH256> {
            Some(test_data::genesis().hash())
        }

        fn difficulty(&self) -> f64 {
            1f64
        }

        fn raw_block(&self, _hash: GlobalH256) -> Option<RawBlock> {
            let b2_bytes = serialize(&test_data::block_h2());
            Some(RawBlock::from(b2_bytes))
        }

        fn verbose_block(&self, _hash: GlobalH256) -> Option<VerboseBlock> {
            Some(VerboseBlock {
                hash: test_data::block_h2().hash().into(),
                confirmations: 1, // h2
                size: serialize(&test_data::block_h2()).len() as u32,
                height: Some(2),
                version: 1,
                version_hex: "1".to_owned(),
                pubkey_hex: test_data::block_h2().header().pubkey.to_bytes().to_hex(),
                randomness_hex: test_data::block_h2()
                    .header()
                    .randomness
                    .to_string_radix(16),
                iterations: test_data::block_h2().header().iterations,
                bits: test_data::block_h2().header().bits.into(),
                difficulty: 1.0,
                chainwork: 0.into(),
                previousblockhash: Some(test_data::block_h1().hash().into()),
                nextblockhash: None,
            })
        }
    }

    impl BlockChainClientCoreApi for ErrorBlockChainClientCore {
        fn best_block_hash(&self) -> GlobalH256 {
            test_data::genesis().hash()
        }

        fn block_count(&self) -> u32 {
            1
        }

        fn block_hash(&self, _height: u32) -> Option<GlobalH256> {
            None
        }

        fn difficulty(&self) -> f64 {
            1f64
        }

        fn raw_block(&self, _hash: GlobalH256) -> Option<RawBlock> {
            None
        }

        fn verbose_block(&self, _hash: GlobalH256) -> Option<VerboseBlock> {
            None
        }
    }

    #[test]
    fn best_block_hash_success() {
        let client = BlockChainClient::new(SuccessBlockChainClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler
            .handle_request_sync(
                &(r#"
			{
				"jsonrpc": "2.0",
				"method": "getbestblockhash",
				"params": [],
				"id": 1
			}"#),
            )
            .unwrap();

        // direct hash is b2cdad40a2ff16c92abcb99732ad36431a72fc5c414c7649809907252cccf3f2
        // but client expects reverse hash
        assert_eq!(
            &sample,
            r#"{"jsonrpc":"2.0","result":"f2f3cc2c2507998049764c415cfc721a4336ad3297b9bc2ac916ffa240adcdb2","id":1}"#
        );
    }

    #[test]
    fn block_count_success() {
        let client = BlockChainClient::new(SuccessBlockChainClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler
            .handle_request_sync(
                &(r#"
                    {
                    	"jsonrpc": "2.0",
                    	"method": "getblockcount",
                    	"params": [],
                    	"id": 1
                    }"#),
            )
            .unwrap();

        assert_eq!(&sample, r#"{"jsonrpc":"2.0","result":1,"id":1}"#);
    }

    #[test]
    fn block_hash_success() {
        let client = BlockChainClient::new(SuccessBlockChainClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler
            .handle_request_sync(
                &(r#"
                    {
                    	"jsonrpc": "2.0",
                    	"method": "getblockhash",
                    	"params": [0],
                    	"id": 1
                    }"#),
            )
            .unwrap();

        // direct hash is b2cdad40a2ff16c92abcb99732ad36431a72fc5c414c7649809907252cccf3f2
        // but client expects reverse hash
        assert_eq!(
            &sample,
            r#"{"jsonrpc":"2.0","result":"f2f3cc2c2507998049764c415cfc721a4336ad3297b9bc2ac916ffa240adcdb2","id":1}"#
        );
    }

    #[test]
    fn block_hash_error() {
        let client = BlockChainClient::new(ErrorBlockChainClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler
            .handle_request_sync(
                &(r#"
                    {
                    	"jsonrpc": "2.0",
                    	"method": "getblockhash",
                    	"params": [0],
                    	"id": 1
                    }"#),
            )
            .unwrap();

        assert_eq!(
            &sample,
            r#"{"jsonrpc":"2.0","error":{"code":-32099,"message":"Block at given height is not found","data":"0"},"id":1}"#
        );
    }

    #[test]
    fn difficulty_success() {
        let client = BlockChainClient::new(SuccessBlockChainClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler
            .handle_request_sync(
                &(r#"
                    {
                    	"jsonrpc": "2.0",
                    	"method": "getdifficulty",
                    	"params": [],
                    	"id": 1
                    }"#),
            )
            .unwrap();

        assert_eq!(&sample, r#"{"jsonrpc":"2.0","result":1.0,"id":1}"#);
    }

    // TODO update tests as we changed block format
    #[test]
    fn verbose_block_contents() {
        let storage = Arc::new(BlockChainDatabase::init_test_chain(vec![
            test_data::genesis().into(),
            test_data::block_h1().into(),
            test_data::block_h2().into(),
        ]));

        let core = BlockChainClientCore::new(storage);

        // get info on block #1:
        let verbose_block = core.verbose_block(test_data::block_h1().hash().into());
        assert_eq!(
            verbose_block,
            Some(VerboseBlock {
                hash: test_data::block_h1().hash().into(),
                confirmations: 2, // h1 + h2
                size: 341,
                height: Some(1),
                version: 1,
                version_hex: "1".to_owned(),
                pubkey_hex: "0000000000000000000000000000000000000000000000000000000000000000"
                    .to_owned(),
                randomness_hex: "9fa3e85750af0157b3d08c44a2cef54cb0ccab50ea9d63ea016a8cc99c9a600a2a4c7ba9b4adc427148b6d3967c2a279e784d2514c7f3f6bcea6fe5d72fb78369f9026bebbdaffebf226d6a0de8e55356e90ac019b09318093c1b5d16b19b335d08713dfd05a251ebba267310c55b3093be6b0649f9a6318d9e61bfc6ba8b612690c7117018cef475972db62e79fbde36b6cd20b1b0ec946e78b916211de6963acf2cf4a72892f906b5ecc589d6f3fff4739028f47e377e592379fcfb63bcbd9810eddc0d7f8e0f9069e8dc9fb22f93f6f5685ad9e1779cbd03aecd689ad679db0364f44fa921056e583f4c71018cb2954216775ee09a6021548cf44b9c05e64".to_owned(),
                iterations: 1,
                bits: 553713663,
                difficulty: 0.00000000023283064365386963,
                chainwork: 0.into(),
                previousblockhash: Some(
                    test_data::genesis().hash().into()
                ),
                nextblockhash: Some(
                    test_data::block_h2().hash().into()
                ),
            })
        );

        // get info on block #2:
        let verbose_block = core.verbose_block(test_data::block_h2().hash().into());
        assert_eq!(
            verbose_block,
            Some(VerboseBlock {
                hash: test_data::block_h2().hash().into(),
                confirmations: 1, // h2
                size: 341,
                height: Some(2),
                version: 1,
                version_hex: "1".to_owned(),
                pubkey_hex: "0000000000000000000000000000000000000000000000000000000000000000"
                    .to_owned(),
                randomness_hex: "59c4420c8bd35716412451248f521db0fe76eb6a25c8a42127ceea885485d549e7215bf8535c3a651bf65a858df7c19b647dd571cce6cfc81981c801824a424b744e584ce01edb73c080e8181175838b89df08a629e579d87e258ebd0e3f6dda75c8e4e1cd1534506f700be8973335a95ade2235ad4e1bbda4aa14bd3b1e30b9110d7914652a528a07b85c06810651820baa186b435bea9884b2562ac4898a876a3015072be36ba7a29d15e49479c6d5a376d69c78b68d10dbea2107187be17719c066dd117e746f09a29e17fc4b72fdc9dfaa07fc0c8786970a6a6266659a4a038ec422160484fc6a4eac82a8079065bd4a4de416762237ddf208cc632af5d6".to_owned(),
                iterations: 1,
                bits: 553713663,
                difficulty: 0.00000000023283064365386963,
                chainwork: 0.into(),
                previousblockhash: Some(
                    test_data::block_h1().hash().into()
                ),
                nextblockhash: None,
            })
        );
    }

    #[test]
    fn raw_block_success() {
        let client = BlockChainClient::new(SuccessBlockChainClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let expected = r#"{"jsonrpc":"2.0","result":"010000008aa6954ae7b7fc8056227fe59bb8d2fe34a93e9d47d07acee0213d629066c78fea030000ffff002120000000000000000000000000000000000000000000000000000000000000000001000000fd000159c4420c8bd35716412451248f521db0fe76eb6a25c8a42127ceea885485d549e7215bf8535c3a651bf65a858df7c19b647dd571cce6cfc81981c801824a424b744e584ce01edb73c080e8181175838b89df08a629e579d87e258ebd0e3f6dda75c8e4e1cd1534506f700be8973335a95ade2235ad4e1bbda4aa14bd3b1e30b9110d7914652a528a07b85c06810651820baa186b435bea9884b2562ac4898a876a3015072be36ba7a29d15e49479c6d5a376d69c78b68d10dbea2107187be17719c066dd117e746f09a29e17fc4b72fdc9dfaa07fc0c8786970a6a6266659a4a038ec422160484fc6a4eac82a8079065bd4a4de416762237ddf208cc632af5d600","id":1}"#;

        let sample = handler
             .handle_request_sync(
                 &(r#"
                    {
                    	"jsonrpc": "2.0",
                    	"method": "getblock",
                    	"params": ["c5a1de8ad5d4fdb816cd9cd36b870ddaef07f0b383a4462d0fd9153d30374ea8", false],
                    	"id": 1
                    }"#),
             )
             .unwrap();
        assert_eq!(&sample, expected);

        // try without optional parameter
        let sample = handler
            .handle_request_sync(
                &(r#"
                    {
                    	"jsonrpc": "2.0",
                    	"method": "getblock",
                    	"params": ["c5a1de8ad5d4fdb816cd9cd36b870ddaef07f0b383a4462d0fd9153d30374ea8"],
                    	"id": 1
                    }"#),
            )
            .unwrap();
        assert_eq!(&sample, expected);
    }

    #[test]
    fn raw_block_error() {
        let client = BlockChainClient::new(ErrorBlockChainClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler
            .handle_request_sync(
                &(r#"
                    {
                    	"jsonrpc": "2.0",
                    	"method": "getblock",
                    	"params": ["000000006a625f06636b8bb6ac7b960a8d03705d1ace08b1a19da3fdcc99ddbd", false],
                    	"id": 1
                    }"#),
            )
            .unwrap();

        assert_eq!(
            &sample,
            r#"{"jsonrpc":"2.0","error":{"code":-32099,"message":"Block with given hash is not found","data":"000000006a625f06636b8bb6ac7b960a8d03705d1ace08b1a19da3fdcc99ddbd"},"id":1}"#
        );
    }

    #[test]
    fn verbose_block_success() {
        let client = BlockChainClient::new(SuccessBlockChainClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler
             .handle_request_sync(
                 &(r#"
                    {
                    	"jsonrpc": "2.0",
                    	"method": "getblock",
                    	"params": ["c5a1de8ad5d4fdb816cd9cd36b870ddaef07f0b383a4462d0fd9153d30374ea8",true],
                    	"id": 1
                    }"#),
             )
             .unwrap();

        assert_eq!(
            &sample,
            r#"{"jsonrpc":"2.0","result":{"bits":553713663,"chainwork":"0","confirmations":1,"difficulty":1.0,"hash":"a84e37303d15d90f2d46a483b3f007efda0d876bd39ccd16b8fdd4d58adea1c5","height":2,"iterations":1,"nextblockhash":null,"previousblockhash":"8fc76690623d21e0ce7ad0479d3ea934fed2b89be57f225680fcb7e74a95a68a","pubkeyHex":"0000000000000000000000000000000000000000000000000000000000000000","randomnessHex":"59c4420c8bd35716412451248f521db0fe76eb6a25c8a42127ceea885485d549e7215bf8535c3a651bf65a858df7c19b647dd571cce6cfc81981c801824a424b744e584ce01edb73c080e8181175838b89df08a629e579d87e258ebd0e3f6dda75c8e4e1cd1534506f700be8973335a95ade2235ad4e1bbda4aa14bd3b1e30b9110d7914652a528a07b85c06810651820baa186b435bea9884b2562ac4898a876a3015072be36ba7a29d15e49479c6d5a376d69c78b68d10dbea2107187be17719c066dd117e746f09a29e17fc4b72fdc9dfaa07fc0c8786970a6a6266659a4a038ec422160484fc6a4eac82a8079065bd4a4de416762237ddf208cc632af5d6","size":341,"version":1,"versionHex":"1"},"id":1}"#
        );
    }

    #[test]
    fn verbose_block_error() {
        let client = BlockChainClient::new(ErrorBlockChainClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler
             .handle_request_sync(
                 &(r#"
                    {
                    	"jsonrpc": "2.0",
                    	"method": "getblock",
                    	"params": ["000000006a625f06636b8bb6ac7b960a8d03705d1ace08b1a19da3fdcc99ddbd", true],
                    	"id": 1
                    }"#),
             )
             .unwrap();

        assert_eq!(
            &sample,
            r#"{"jsonrpc":"2.0","error":{"code":-32099,"message":"Block with given hash is not found","data":"000000006a625f06636b8bb6ac7b960a8d03705d1ace08b1a19da3fdcc99ddbd"},"id":1}"#
        );
    }
}

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
            let median_time = verification::median_timestamp(
                &block.header.raw,
                self.storage.as_block_header_provider(),
            );

            VerboseBlock {
                confirmations: confirmations,
                size: block_size as u32,
                height: height,
                mediantime: Some(median_time),
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
                time: block.header.raw.time,
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
    use primitives::bytes::Bytes as GlobalBytes;
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
            let b2_bytes: GlobalBytes = "010000004860eb18bf1b1620e37e9490fc8a427514416fd75159ab86688e9a8300000000d5fdcc541e25de1c7a5addedf24858b8bb665c9f36ef744ee42c316022c90f9bb0bc6649ffff001d08d2bd610101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff0704ffff001d010bffffffff0100f2052a010000004341047211a824f55b505228e4c3d5194c1fcfaa15a456abdf37f9b9d97a4040afc073dee6c89064984f03385237d92167c13e236446b417ab79a0fcae412ae3316b77ac00000000".into();
            Some(RawBlock::from(b2_bytes))
        }

        fn verbose_block(&self, _hash: GlobalH256) -> Option<VerboseBlock> {
            // https://blockexplorer.com/block/000000006a625f06636b8bb6ac7b960a8d03705d1ace08b1a19da3fdcc99ddbd
            // https://blockchain.info/ru/block/000000006a625f06636b8bb6ac7b960a8d03705d1ace08b1a19da3fdcc99ddbd
            // https://webbtc.com/block/000000006a625f06636b8bb6ac7b960a8d03705d1ace08b1a19da3fdcc99ddbd.json
            Some(VerboseBlock {
                hash: "bddd99ccfda39da1b108ce1a5d70038d0a967bacb68b6b63065f626a00000000".into(),
                randomness_hash: "bddd99ccfda39da1b108ce1a5d70038d0a967bacb68b6b63065f626a00000000"
                    .into(),
                confirmations: 1, // h2
                size: 215,
                height: Some(2),
                version: 1,
                version_hex: "1".to_owned(),
                pubkey_hex: "6969696969696969696969696969696969696969696969696969696969696969"
                    .to_owned(),
                randomness_hex: "7788".to_owned(),
                time: 1231469744,
                mediantime: None,
                iterations: 1639830024,
                bits: 486604799,
                difficulty: 1.0,
                chainwork: 0.into(),
                previousblockhash: Some(
                    "4860eb18bf1b1620e37e9490fc8a427514416fd75159ab86688e9a8300000000".into(),
                ),
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

        // direct hash is ....
        // but client expects reverse hash
        assert_eq!(
            &sample,
            r#"{"jsonrpc":"2.0","result":"a1ba7dca74f77a4b0053b0f6b8eea1268ec44337d59d5acfafa0d94b7bd18404","id":1}"#
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

        // direct hash is ...
        // but client expects reverse hash
        assert_eq!(
            &sample,
            r#"{"jsonrpc":"2.0","result":"a1ba7dca74f77a4b0053b0f6b8eea1268ec44337d59d5acfafa0d94b7bd18404","id":1}"#
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

    #[test]
    fn verbose_block_contents() {
        let storage = Arc::new(BlockChainDatabase::init_test_chain(vec![
            test_data::genesis().into(),
            test_data::block_h1().into(),
            test_data::block_h2().into(),
        ]));

        let core = BlockChainClientCore::new(storage);

        // get info on block #1:
        let verbose_block = core.verbose_block(
            "c6235208c895dbfd487d3c760194b77b5e0633835a0482fe6df049fc35b28277".into(),
        );
        assert_eq!(
            verbose_block,
            Some(VerboseBlock {
                hash: "c6235208c895dbfd487d3c760194b77b5e0633835a0482fe6df049fc35b28277".into(),
                randomness_hash: "c6235208c895dbfd487d3c760194b77b5e0633835a0482fe6df049fc35b28277"
                    .into(),
                confirmations: 2, // h1 + h2
                size: 55,
                height: Some(1),
                version: 1,
                version_hex: "1".to_owned(),
                // TODO:
                pubkey_hex: "6969696969696969696969696969696969696969696969696969696969696969"
                    .to_owned(),
                randomness_hex: "7788".to_owned(),
                time: 1231469665,
                mediantime: Some(1231006505),
                iterations: 2573394689,
                bits: 486604799,
                difficulty: 1.0,
                chainwork: 0.into(),
                previousblockhash: Some(
                    "0484d17b4bd9a0afcf5a9dd53743c48e26a1eeb8f6b053004b7af774ca7dbaa1".into()
                ),
                nextblockhash: Some(
                    "b6d94e340f618ec8f11682fe8eef6fdf19cbfdd0a67aad15907d88294cc961ae".into()
                ),
            })
        );

        // get info on block #2:
        let verbose_block = core.verbose_block(
            "b6d94e340f618ec8f11682fe8eef6fdf19cbfdd0a67aad15907d88294cc961ae".into(),
        );
        assert_eq!(
            verbose_block,
            Some(VerboseBlock {
                hash: "b6d94e340f618ec8f11682fe8eef6fdf19cbfdd0a67aad15907d88294cc961ae".into(),
                randomness_hash: "b6d94e340f618ec8f11682fe8eef6fdf19cbfdd0a67aad15907d88294cc961ae"
                    .into(),
                confirmations: 1, // h2
                size: 215,
                height: Some(2),
                version: 1,
                version_hex: "1".to_owned(),
                pubkey_hex: "6969696969696969696969696969696969696969696969696969696969696969"
                    .to_owned(),
                randomness_hex: "7788".to_owned(),
                time: 1231469744,
                mediantime: Some(1231469665),
                iterations: 1639830024,
                bits: 486604799,
                difficulty: 1.0,
                chainwork: 0.into(),
                previousblockhash: Some(
                    "c6235208c895dbfd487d3c760194b77b5e0633835a0482fe6df049fc35b28277".into()
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

        let expected = r#"{"jsonrpc":"2.0","result":"010000004860eb18bf1b1620e37e9490fc8a427514416fd75159ab86688e9a8300000000d5fdcc541e25de1c7a5addedf24858b8bb665c9f36ef744ee42c316022c90f9bb0bc6649ffff001d08d2bd610101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff0704ffff001d010bffffffff0100f2052a010000004341047211a824f55b505228e4c3d5194c1fcfaa15a456abdf37f9b9d97a4040afc073dee6c89064984f03385237d92167c13e236446b417ab79a0fcae412ae3316b77ac00000000","id":1}"#;

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
        assert_eq!(&sample, expected);

        // try without optional parameter
        let sample = handler
            .handle_request_sync(
                &(r#"
			{
				"jsonrpc": "2.0",
				"method": "getblock",
				"params": ["000000006a625f06636b8bb6ac7b960a8d03705d1ace08b1a19da3fdcc99ddbd"],
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
				"params": ["000000006a625f06636b8bb6ac7b960a8d03705d1ace08b1a19da3fdcc99ddbd",true],
				"id": 1
			}"#),
            )
            .unwrap();

        assert_eq!(
            &sample,
            r#"{"jsonrpc":"2.0","result":{"bits":486604799,"chainwork":"0","confirmations":1,"difficulty":1.0,"hash":"000000006a625f06636b8bb6ac7b960a8d03705d1ace08b1a19da3fdcc99ddbd","height":2,"mediantime":null,"nextblockhash":null,"iterations":1639830024,"previousblockhash":"00000000839a8e6886ab5951d76f411475428afc90947ee320161bbf18eb6048","randomnessHex":"7788","size":215,"time":1231469744,"version":1,"versionHex":"1"},"id":1}"#
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

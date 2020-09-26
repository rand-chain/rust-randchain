use network::Network;
use storage;
use sync;
use v1::traits::Raw;

pub struct RawClient<T: RawClientCoreApi> {
    core: T,
}

// TODO:
pub trait RawClientCoreApi: Send + Sync + 'static {}

pub struct RawClientCore {
    network: Network,
    local_sync_node: sync::LocalNodeRef,
    storage: storage::SharedStore,
}

// TODO:
impl RawClientCore {
    pub fn new(
        network: Network,
        local_sync_node: sync::LocalNodeRef,
        storage: storage::SharedStore,
    ) -> Self {
        RawClientCore {
            network,
            local_sync_node,
            storage,
        }
    }
}

// TODO:
impl RawClientCoreApi for RawClientCore {}

impl<T> RawClient<T>
where
    T: RawClientCoreApi,
{
    pub fn new(core: T) -> Self {
        RawClient { core: core }
    }
}

// TODO:
impl<T> Raw for RawClient<T> where T: RawClientCoreApi {}

#[cfg(test)]
pub mod tests {
    use super::*;
    use chain::Transaction as GlobalTransaction;
    use jsonrpc_core::IoHandler;
    use jsonrpc_macros::Trailing;
    use keys::Address;
    use primitives::hash::H256 as GlobalH256;
    use v1::traits::Raw;
    use v1::types::{
        Bytes, ScriptType, SignedTransactionInput, Transaction, TransactionInput,
        TransactionInputScript, TransactionOutputs,
    };

    #[derive(Default)]
    struct SuccessRawClientCore;

    #[derive(Default)]
    struct ErrorRawClientCore;

    impl RawClientCoreApi for SuccessRawClientCore {
        fn accept_transaction(&self, transaction: GlobalTransaction) -> Result<GlobalH256, String> {
            Ok(transaction.hash())
        }

        fn create_raw_transaction(
            &self,
            _inputs: Vec<TransactionInput>,
            _outputs: TransactionOutputs,
            _lock_time: Trailing<u32>,
        ) -> Result<GlobalTransaction, String> {
            Ok("0100000001ad9d38823d95f31dc6c0cb0724c11a3cf5a466ca4147254a10cd94aade6eb5b3230000006b483045022100b7683165c3ecd57b0c44bf6a0fb258dc08c328458321c8fadc2b9348d4e66bd502204fd164c58d1a949a4d39bb380f8f05c9f6b3e9417f06bf72e5c068428ca3578601210391c35ac5ee7cf82c5015229dcff89507f83f9b8c952b8fecfa469066c1cb44ccffffffff0170f30500000000001976a914801da3cb2ed9e44540f4b982bde07cd3fbae264288ac00000000".into())
        }

        fn get_raw_transaction(
            &self,
            _hash: GlobalH256,
            verbose: bool,
        ) -> Result<GetRawTransactionResponse, Error> {
            if !verbose {
                Ok(GetRawTransactionResponse::Raw(Bytes::from("0100000001273d7b971b6788f911038f917dfa9ba85980b018a80b2e8caa4fca85475afdaf010000008b48304502205eb82fbb78f3467269c64ebb48c66567b11b1ebfa9cf4dd793d1482e46d3851c022100d18e2091becaea279f6f896825e7ca669ee0607b30007ca88b43d1de91359ba9014104a208236447f5c93972a739105abb8292613eef741cab36a1b98fa4fcc2989add0e5dc6cda9127a2bf0b18357210ba0119ad700e1fa495143262720067f4fbf83ffffffff02003b5808000000001976a9147793078b2ebc6ab7b7fd213789912f1deb03a97088ac404b4c00000000001976a914ffc2838f7aeed00857dbbfc70d9830c6968aca5688ac00000000")))
            } else {
                Ok(GetRawTransactionResponse::Verbose(
					Transaction {
						hex: Some(Bytes::from("0100000001273d7b971b6788f911038f917dfa9ba85980b018a80b2e8caa4fca85475afdaf010000008b48304502205eb82fbb78f3467269c64ebb48c66567b11b1ebfa9cf4dd793d1482e46d3851c022100d18e2091becaea279f6f896825e7ca669ee0607b30007ca88b43d1de91359ba9014104a208236447f5c93972a739105abb8292613eef741cab36a1b98fa4fcc2989add0e5dc6cda9127a2bf0b18357210ba0119ad700e1fa495143262720067f4fbf83ffffffff02003b5808000000001976a9147793078b2ebc6ab7b7fd213789912f1deb03a97088ac404b4c00000000001976a914ffc2838f7aeed00857dbbfc70d9830c6968aca5688ac00000000")),
						txid: "635f07dc4acdfb9bc305261169f82836949df462876fab9017bb9faf4d5fdadb".into(),
						hash: "635f07dc4acdfb9bc305261169f82836949df462876fab9017bb9faf4d5fdadb".into(),
						size: 258,
						vsize: 258,
						version: 1,
						locktime: 0,
						vin: vec![
							SignedTransactionInput {
								txid: "affd5a4785ca4faa8c2e0ba818b08059a89bfa7d918f0311f988671b977b3d27".into(),
								vout: 1,
								script_sig: TransactionInputScript {
									asm: "OP_PUSHBYTES_72 0x304502205eb82fbb78f3467269c64ebb48c66567b11b1ebfa9cf4dd793d1482e46d3851c022100d18e2091becaea279f6f896825e7ca669ee0607b30007ca88b43d1de91359ba901\nOP_PUSHBYTES_65 0x04a208236447f5c93972a739105abb8292613eef741cab36a1b98fa4fcc2989add0e5dc6cda9127a2bf0b18357210ba0119ad700e1fa495143262720067f4fbf83\n".to_string(),
									hex: Bytes::from("48304502205eb82fbb78f3467269c64ebb48c66567b11b1ebfa9cf4dd793d1482e46d3851c022100d18e2091becaea279f6f896825e7ca669ee0607b30007ca88b43d1de91359ba9014104a208236447f5c93972a739105abb8292613eef741cab36a1b98fa4fcc2989add0e5dc6cda9127a2bf0b18357210ba0119ad700e1fa495143262720067f4fbf83"),
								},
								sequence: 4294967295,
								txinwitness: None,
							},
						],
						vout: vec![
							SignedTransactionOutput {
								value: 1.4000000000000001,
								n: 0,
								script: TransactionOutputScript {
									asm: "OP_DUP\nOP_HASH160\nOP_PUSHBYTES_20 0x7793078b2ebc6ab7b7fd213789912f1deb03a970\nOP_EQUALVERIFY\nOP_CHECKSIG\n".to_string(),
									hex: Bytes::from("76a9147793078b2ebc6ab7b7fd213789912f1deb03a97088ac"),
									req_sigs: 1,
									script_type: ScriptType::PubKeyHash,
									addresses: vec![
										Address::from("1BuFYcdZeBmpr5mycksjU7vQiD5VcT9hws"),
									],
								},
							},
							SignedTransactionOutput {
								value: 0.05,
								n: 1,
								script: TransactionOutputScript {
									asm: "OP_DUP\nOP_HASH160\nOP_PUSHBYTES_20 0xffc2838f7aeed00857dbbfc70d9830c6968aca56\nOP_EQUALVERIFY\nOP_CHECKSIG\n".to_string(),
									hex: Bytes::from("76a914ffc2838f7aeed00857dbbfc70d9830c6968aca5688ac"),
									req_sigs: 1,
									script_type: ScriptType::PubKeyHash,
									addresses: vec![
										Address::from("1QKLKyqomc3x9cuyh8tSSvboMoRQVtYhW6"),
									],
								},
							},
						],
						blockhash: Some("00000000000a948a5e6cbe8abde649c6fc353cb2ee592cbf47f3850180ef0c0e".into()),
						confirmations: Some(197043),
						time: Some(1289842148),
						blocktime: Some(1289842148),
					}
				))
            }
        }

        fn transaction_to_verbose_transaction(
            &self,
            _transaction: GlobalIndexedTransaction,
        ) -> Transaction {
            Transaction {
				hex: None,
				txid: "c586389e5e4b3acb9d6c8be1c19ae8ab2795397633176f5a6442a261bbdefc3a".into(),
				hash: "b759d39a8596b70b3a46700b83e1edb247e17ba58df305421864fe7a9ac142ea".into(),
				size: 216,
				vsize: 134,
				version: 2,
				locktime: 0,
				vin: vec![
					SignedTransactionInput {
						txid: "42f7d0545ef45bd3b9cfee6b170cf6314a3bd8b3f09b610eeb436d92993ad440".into(),
						vout: 1,
						script_sig: TransactionInputScript {
							asm: "0014a4b4ca48de0b3fffc15404a1acdc8dbaae226955".to_string(),
							hex: Bytes::from("160014a4b4ca48de0b3fffc15404a1acdc8dbaae226955"),
						},
						sequence: 4294967295,
						txinwitness: Some(vec![
							Bytes::from("30450221008604ef8f6d8afa892dee0f31259b6ce02dd70c545cfcfed8148179971876c54a022076d771d6e91bed212783c9b06e0de600fab2d518fad6f15a2b191d7fbd262a3e01"),
							Bytes::from("039d25ab79f41f75ceaf882411fd41fa670a4c672c23ffaf0e361a969cde0692e8"),
						]),
					},
				],
				vout: vec![
					SignedTransactionOutput {
						value: 1.00000000,
						n: 0,
						script: TransactionOutputScript {
							asm: "OP_HASH160 4a1154d50b03292b3024370901711946cb7cccc3 OP_EQUAL".to_string(),
							hex: Bytes::from("a9144a1154d50b03292b3024370901711946cb7cccc387"),
							req_sigs: 1,
							script_type: ScriptType::ScriptHash,
							addresses: vec![
								Address::from("38Segwituno6sUoEkh57ycM6K7ej5gvJhM"),
							],
						},
					},
				],
				blockhash: None,
				confirmations: None,
				time: None,
				blocktime: None,
			}
        }
    }

    impl RawClientCoreApi for ErrorRawClientCore {
        fn accept_transaction(
            &self,
            _transaction: GlobalTransaction,
        ) -> Result<GlobalH256, String> {
            Err("error".to_owned())
        }

        fn create_raw_transaction(
            &self,
            _inputs: Vec<TransactionInput>,
            _outputs: TransactionOutputs,
            _lock_time: Trailing<u32>,
        ) -> Result<GlobalTransaction, String> {
            Err("error".to_owned())
        }

        fn get_raw_transaction(
            &self,
            hash: GlobalH256,
            _verbose: bool,
        ) -> Result<GetRawTransactionResponse, Error> {
            Err(transaction_not_found(hash))
        }

        fn transaction_to_verbose_transaction(
            &self,
            _transaction: GlobalIndexedTransaction,
        ) -> Transaction {
            Transaction {
				hex: None,
				txid: "635f07dc4acdfb9bc305261169f82836949df462876fab9017bb9faf4d5fdadb".into(),
				hash: "635f07dc4acdfb9bc305261169f82836949df462876fab9017bb9faf4d5fdadb".into(),
				size: 258,
				vsize: 258,
				version: 1,
				locktime: 0,
				vin: vec![
					SignedTransactionInput {
						txid: "affd5a4785ca4faa8c2e0ba818b08059a89bfa7d918f0311f988671b977b3d27".into(),
						vout: 1,
						script_sig: TransactionInputScript {
							asm: "OP_PUSHBYTES_72 0x304502205eb82fbb78f3467269c64ebb48c66567b11b1ebfa9cf4dd793d1482e46d3851c022100d18e2091becaea279f6f896825e7ca669ee0607b30007ca88b43d1de91359ba901\nOP_PUSHBYTES_65 0x04a208236447f5c93972a739105abb8292613eef741cab36a1b98fa4fcc2989add0e5dc6cda9127a2bf0b18357210ba0119ad700e1fa495143262720067f4fbf83\n".to_string(),
							hex: Bytes::from("48304502205eb82fbb78f3467269c64ebb48c66567b11b1ebfa9cf4dd793d1482e46d3851c022100d18e2091becaea279f6f896825e7ca669ee0607b30007ca88b43d1de91359ba9014104a208236447f5c93972a739105abb8292613eef741cab36a1b98fa4fcc2989add0e5dc6cda9127a2bf0b18357210ba0119ad700e1fa495143262720067f4fbf83"),
						},
						sequence: 4294967295,
						txinwitness: None,
					},
				],
				vout: vec![
					SignedTransactionOutput {
						value: 1.4000000000000001,
						n: 0,
						script: TransactionOutputScript {
							asm: "OP_DUP\nOP_HASH160\nOP_PUSHBYTES_20 0x7793078b2ebc6ab7b7fd213789912f1deb03a970\nOP_EQUALVERIFY\nOP_CHECKSIG\n".to_string(),
							hex: Bytes::from("76a9147793078b2ebc6ab7b7fd213789912f1deb03a97088ac"),
							req_sigs: 1,
							script_type: ScriptType::PubKeyHash,
							addresses: vec![
								Address::from("1BuFYcdZeBmpr5mycksjU7vQiD5VcT9hws"),
							],
						},
					},
					SignedTransactionOutput {
						value: 0.05,
						n: 1,
						script: TransactionOutputScript {
							asm: "OP_DUP\nOP_HASH160\nOP_PUSHBYTES_20 0xffc2838f7aeed00857dbbfc70d9830c6968aca56\nOP_EQUALVERIFY\nOP_CHECKSIG\n".to_string(),
							hex: Bytes::from("76a914ffc2838f7aeed00857dbbfc70d9830c6968aca5688ac"),
							req_sigs: 1,
							script_type: ScriptType::PubKeyHash,
							addresses: vec![
								Address::from("1QKLKyqomc3x9cuyh8tSSvboMoRQVtYhW6"),
							],
						},
					},
				],
				blockhash: None,
				confirmations: None,
				time: None,
				blocktime: None,
			}
        }
    }

    #[test]
    fn sendrawtransaction_accepted() {
        let client = RawClient::new(SuccessRawClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler.handle_request_sync(&(r#"
			{
				"jsonrpc": "2.0",
				"method": "sendrawtransaction",
				"params": ["00000000013ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a0000000000000000000101000000000000000000000000"],
				"id": 1
			}"#)
		).unwrap();

        // direct hash is 0791efccd035c5fe501023ff888106eba5eff533965de4a6e06400f623bcac34
        // but client expects reverse hash
        assert_eq!(
            r#"{"jsonrpc":"2.0","result":"34acbc23f60064e0a6e45d9633f5efa5eb068188ff231050fec535d0ccef9107","id":1}"#,
            &sample
        );
    }

    #[test]
    fn sendrawtransaction_rejected() {
        let client = RawClient::new(ErrorRawClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler.handle_request_sync(&(r#"
			{
				"jsonrpc": "2.0",
				"method": "sendrawtransaction",
				"params": ["00000000013ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a0000000000000000000101000000000000000000000000"],
				"id": 1
			}"#)
		).unwrap();

        assert_eq!(
            r#"{"jsonrpc":"2.0","error":{"code":-32015,"message":"Execution error.","data":"\"error\""},"id":1}"#,
            &sample
        );
    }

    #[test]
    fn createrawtransaction_success() {
        let client = RawClient::new(SuccessRawClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler.handle_request_sync(&(r#"
			{
				"jsonrpc": "2.0",
				"method": "createrawtransaction",
				"params": [[{"txid":"4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b","vout":0}],{"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa":0.01}],
				"id": 1
			}"#)
		).unwrap();

        assert_eq!(
            r#"{"jsonrpc":"2.0","result":"0100000001ad9d38823d95f31dc6c0cb0724c11a3cf5a466ca4147254a10cd94aade6eb5b3230000006b483045022100b7683165c3ecd57b0c44bf6a0fb258dc08c328458321c8fadc2b9348d4e66bd502204fd164c58d1a949a4d39bb380f8f05c9f6b3e9417f06bf72e5c068428ca3578601210391c35ac5ee7cf82c5015229dcff89507f83f9b8c952b8fecfa469066c1cb44ccffffffff0170f30500000000001976a914801da3cb2ed9e44540f4b982bde07cd3fbae264288ac00000000","id":1}"#,
            &sample
        );
    }

    #[test]
    fn createrawtransaction_error() {
        let client = RawClient::new(ErrorRawClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler.handle_request_sync(&(r#"
			{
				"jsonrpc": "2.0",
				"method": "createrawtransaction",
				"params": [[{"txid":"4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b","vout":0}],{"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa":0.01}],
				"id": 1
			}"#)
		).unwrap();

        assert_eq!(
            r#"{"jsonrpc":"2.0","error":{"code":-32015,"message":"Execution error.","data":"\"error\""},"id":1}"#,
            &sample
        );
    }

    #[test]
    fn getrawtransaction_success() {
        let client = RawClient::new(SuccessRawClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler
            .handle_request_sync(
                &(r#"
			{
				"jsonrpc": "2.0",
				"method": "getrawtransaction",
				"params": ["635f07dc4acdfb9bc305261169f82836949df462876fab9017bb9faf4d5fdadb"],
				"id": 1
			}"#),
            )
            .unwrap();

        assert_eq!(
            r#"{"jsonrpc":"2.0","result":"0100000001273d7b971b6788f911038f917dfa9ba85980b018a80b2e8caa4fca85475afdaf010000008b48304502205eb82fbb78f3467269c64ebb48c66567b11b1ebfa9cf4dd793d1482e46d3851c022100d18e2091becaea279f6f896825e7ca669ee0607b30007ca88b43d1de91359ba9014104a208236447f5c93972a739105abb8292613eef741cab36a1b98fa4fcc2989add0e5dc6cda9127a2bf0b18357210ba0119ad700e1fa495143262720067f4fbf83ffffffff02003b5808000000001976a9147793078b2ebc6ab7b7fd213789912f1deb03a97088ac404b4c00000000001976a914ffc2838f7aeed00857dbbfc70d9830c6968aca5688ac00000000","id":1}"#,
            &sample
        );
    }

    #[test]
    fn getrawtransaction_verbose_success() {
        let client = RawClient::new(SuccessRawClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler
            .handle_request_sync(
                &(r#"
			{
				"jsonrpc": "2.0",
				"method": "getrawtransaction",
				"params": ["635f07dc4acdfb9bc305261169f82836949df462876fab9017bb9faf4d5fdadb", true],
				"id": 1
			}"#),
            )
            .unwrap();

        assert_eq!(
            r#"{"jsonrpc":"2.0","result":{"blockhash":"00000000000a948a5e6cbe8abde649c6fc353cb2ee592cbf47f3850180ef0c0e","blocktime":1289842148,"confirmations":197043,"hash":"635f07dc4acdfb9bc305261169f82836949df462876fab9017bb9faf4d5fdadb","hex":"0100000001273d7b971b6788f911038f917dfa9ba85980b018a80b2e8caa4fca85475afdaf010000008b48304502205eb82fbb78f3467269c64ebb48c66567b11b1ebfa9cf4dd793d1482e46d3851c022100d18e2091becaea279f6f896825e7ca669ee0607b30007ca88b43d1de91359ba9014104a208236447f5c93972a739105abb8292613eef741cab36a1b98fa4fcc2989add0e5dc6cda9127a2bf0b18357210ba0119ad700e1fa495143262720067f4fbf83ffffffff02003b5808000000001976a9147793078b2ebc6ab7b7fd213789912f1deb03a97088ac404b4c00000000001976a914ffc2838f7aeed00857dbbfc70d9830c6968aca5688ac00000000","locktime":0,"size":258,"time":1289842148,"txid":"635f07dc4acdfb9bc305261169f82836949df462876fab9017bb9faf4d5fdadb","version":1,"vin":[{"script_sig":{"asm":"OP_PUSHBYTES_72 0x304502205eb82fbb78f3467269c64ebb48c66567b11b1ebfa9cf4dd793d1482e46d3851c022100d18e2091becaea279f6f896825e7ca669ee0607b30007ca88b43d1de91359ba901\nOP_PUSHBYTES_65 0x04a208236447f5c93972a739105abb8292613eef741cab36a1b98fa4fcc2989add0e5dc6cda9127a2bf0b18357210ba0119ad700e1fa495143262720067f4fbf83\n","hex":"48304502205eb82fbb78f3467269c64ebb48c66567b11b1ebfa9cf4dd793d1482e46d3851c022100d18e2091becaea279f6f896825e7ca669ee0607b30007ca88b43d1de91359ba9014104a208236447f5c93972a739105abb8292613eef741cab36a1b98fa4fcc2989add0e5dc6cda9127a2bf0b18357210ba0119ad700e1fa495143262720067f4fbf83"},"sequence":4294967295,"txid":"affd5a4785ca4faa8c2e0ba818b08059a89bfa7d918f0311f988671b977b3d27","vout":1}],"vout":[{"n":0,"scriptPubKey":{"addresses":["1BuFYcdZeBmpr5mycksjU7vQiD5VcT9hws"],"asm":"OP_DUP\nOP_HASH160\nOP_PUSHBYTES_20 0x7793078b2ebc6ab7b7fd213789912f1deb03a970\nOP_EQUALVERIFY\nOP_CHECKSIG\n","hex":"76a9147793078b2ebc6ab7b7fd213789912f1deb03a97088ac","reqSigs":1,"type":"pubkeyhash"},"value":1.4000000000000002},{"n":1,"scriptPubKey":{"addresses":["1QKLKyqomc3x9cuyh8tSSvboMoRQVtYhW6"],"asm":"OP_DUP\nOP_HASH160\nOP_PUSHBYTES_20 0xffc2838f7aeed00857dbbfc70d9830c6968aca56\nOP_EQUALVERIFY\nOP_CHECKSIG\n","hex":"76a914ffc2838f7aeed00857dbbfc70d9830c6968aca5688ac","reqSigs":1,"type":"pubkeyhash"},"value":0.05}],"vsize":258},"id":1}"#,
            &sample
        );
    }

    #[test]
    fn getrawtransaction_error() {
        let client = RawClient::new(ErrorRawClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler
            .handle_request_sync(
                &(r#"
			{
				"jsonrpc": "2.0",
				"method": "getrawtransaction",
				"params": ["4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b"],
				"id": 1
			}"#),
            )
            .unwrap();

        assert_eq!(
            r#"{"jsonrpc":"2.0","error":{"code":-32096,"message":"Transaction with given hash is not found","data":"3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a"},"id":1}"#,
            &sample
        );
    }

    #[test]
    fn decoderawtransaction_success() {
        let client = RawClient::new(SuccessRawClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler.handle_request_sync(&(r#"
			{
				"jsonrpc": "2.0",
				"method": "decoderawtransaction",
				"params": ["0200000000010140d43a99926d43eb0e619bf0b3d83b4a31f60c176beecfb9d35bf45e54d0f7420100000017160014a4b4ca48de0b3fffc15404a1acdc8dbaae226955ffffffff0100e1f5050000000017a9144a1154d50b03292b3024370901711946cb7cccc387024830450221008604ef8f6d8afa892dee0f31259b6ce02dd70c545cfcfed8148179971876c54a022076d771d6e91bed212783c9b06e0de600fab2d518fad6f15a2b191d7fbd262a3e0121039d25ab79f41f75ceaf882411fd41fa670a4c672c23ffaf0e361a969cde0692e800000000"],
				"id": 1
			}"#)
		).unwrap();

        assert_eq!(
            r#"{"jsonrpc":"2.0","result":{"hash":"b759d39a8596b70b3a46700b83e1edb247e17ba58df305421864fe7a9ac142ea","locktime":0,"size":216,"txid":"c586389e5e4b3acb9d6c8be1c19ae8ab2795397633176f5a6442a261bbdefc3a","version":2,"vin":[{"script_sig":{"asm":"0014a4b4ca48de0b3fffc15404a1acdc8dbaae226955","hex":"160014a4b4ca48de0b3fffc15404a1acdc8dbaae226955"},"sequence":4294967295,"txid":"42f7d0545ef45bd3b9cfee6b170cf6314a3bd8b3f09b610eeb436d92993ad440","txinwitness":["30450221008604ef8f6d8afa892dee0f31259b6ce02dd70c545cfcfed8148179971876c54a022076d771d6e91bed212783c9b06e0de600fab2d518fad6f15a2b191d7fbd262a3e01","039d25ab79f41f75ceaf882411fd41fa670a4c672c23ffaf0e361a969cde0692e8"],"vout":1}],"vout":[{"n":0,"scriptPubKey":{"addresses":["38Segwituno6sUoEkh57ycM6K7ej5gvJhM"],"asm":"OP_HASH160 4a1154d50b03292b3024370901711946cb7cccc3 OP_EQUAL","hex":"a9144a1154d50b03292b3024370901711946cb7cccc387","reqSigs":1,"type":"scripthash"},"value":1.0}],"vsize":134},"id":1}"#,
            &sample
        );
    }

    #[test]
    fn decoderawtransaction_error() {
        let client = RawClient::new(SuccessRawClientCore::default());
        let mut handler = IoHandler::new();
        handler.extend_with(client.to_delegate());

        let sample = handler
            .handle_request_sync(
                &(r#"
			{
				"jsonrpc": "2.0",
				"method": "decoderawtransaction",
				"params": ["shjksdfjklgsdghjlsdfjk"],
				"id": 1
			}"#),
            )
            .unwrap();

        assert_eq!(
            r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Invalid params: invalid hex."},"id":1}"#,
            &sample
        );
    }
}

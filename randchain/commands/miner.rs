use chain::{Block, BlockHeader, IndexedBlock};
use crypto::seqpow::{init, prove, solve};
use ethcore_rpc::v1::types::BlockTemplate as rpcBlockTemplate;
use jsonrpc_core::types::response::{Output, Response, Success};

use miner::BlockTemplate as minerBlockTemplate;
use primitives::bytes::Bytes;
use primitives::compact::Compact;
use primitives::hash::H256 as GlobalH256;
use raccount::keypair::KeyPair;
use rhex::{FromHex, ToHex};
use ser::{deserialize, serialize, Stream};
use std::io::prelude::*;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

#[derive(Debug, PartialEq, Clone)]
enum Error {
    SerError,
}

fn try_req(url: &str, req_id: u64) -> Result<minerBlockTemplate, Error> {
    let req = ureq::post(url)
        .set("X-My-Header", "Secret")
        .send_json(ureq::json!({
           "jsonrpc": "2.0",
           "method": "getblocktemplate",
           "params": [{}],
           "id": format!("\"{}\"", req_id)
        }));
    let resp = req.unwrap(); // TODO error handling
    log::debug!("receive response of getblocktemplate: {:?}", resp);
    let success_resp = resp.into_json::<Success>().unwrap();

    let template =
        serde_json::from_str::<rpcBlockTemplate>(&success_resp.result.to_string()).unwrap();

    let previous_header_global_hash: GlobalH256 = template.previousblockhash.into();
    Ok(minerBlockTemplate {
        version: template.version,
        previous_header_hash: previous_header_global_hash,
        height: template.height,
        bits: Compact::from(template.bits),
    })
}

pub fn start(matches: &clap::ArgMatches) -> Result<(), String> {
    ::std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    // obtain endpoint
    let endpoint = matches
        .value_of("endpoint")
        .unwrap_or("http://127.0.0.1:8332/");
    // get key path
    let key_path = match matches.value_of("key-path") {
        Some(s) => PathBuf::from(s),
        None => return Err("Please provide --key-path".to_owned()),
    };
    // parse key file
    let keypair = match KeyPair::load(&key_path) {
        Ok(s) => s,
        Err(err) => return Err(err),
    };
    let pk = keypair.pk;
    // request ID. starting from 1
    let mut req_id = 1u64;

    let mut tpl: minerBlockTemplate = match try_req(endpoint, req_id) {
        Err(err) => {
            return Err(format!("error upon getblocktemplate: {:?}", err));
        }
        Ok(template) => template,
    };

    // update block template thread
    // TODO: not sure why the below line is needed;
    // if the below line is removed, the code won't compile.
    let endpoint1 = endpoint.to_owned().clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        match try_req(&endpoint1, req_id) {
            Err(err) => {
                error!("error upon getblocktemplate: {:?}", err);
            }
            Ok(template) => {
                req_id += 1;
                if tpl.previous_header_hash != template.previous_header_hash {
                    info!(
                        "receive new template with previous block hash {:?} and height {} from {:?}",
                        template.previous_header_hash,
                        template.height,
                        endpoint1,
                    );
                    tpl = template;
                }
            }
        };
    });

    // TODO not sure why tpl.to_bytes() do not work here...
    let mut stream = Stream::default();
    stream
        .append(&tpl.version)
        .append(&tpl.previous_header_hash)
        .append(&tpl.bits)
        .append(&Bytes::from(pk.to_bytes().to_vec()));
    let data = stream.out();

    // mine thread
    let mut sol = init(&data, &pk);
    let mut previous_header_hash = tpl.previous_header_hash;
    loop {
        // when tpl is new,
        // re-init solution and update header hash
        if previous_header_hash != tpl.previous_header_hash {
            sol = init(&data, &pk);
            previous_header_hash = tpl.previous_header_hash;
        }

        // increment by SeqPoW:Solve()
        let (new_solution, valid) = solve(&pk, &sol, tpl.bits);

        // if valid,
        // 1. generate proof
        // 2. construct block
        // 3. send block to node
        if valid {
            sol = prove(&pk, &data, &new_solution);
            info!("find solution: {:?}", sol.element);
            // construct block
            let blk = Block {
                block_header: BlockHeader {
                    version: tpl.version,
                    previous_header_hash: tpl.previous_header_hash,
                    bits: tpl.bits,
                    pubkey: pk.clone(),
                    iterations: sol.iterations as u32,
                    solution: sol.element.clone(),
                },
                proof: sol.proof.clone(),
            };
            // serialise block
            let ser_block: String = serialize(&blk).to_hex();
            // make request and receive response
            let req = ureq::post(&endpoint)
                .set("X-My-Header", "Secret")
                .send_json(ureq::json!({
                    "jsonrpc": "2.0",
                    "method": "submitblock",
                    "params": [{"data": &ser_block}],
                    "id": format!("\"{}\"", req_id)
                }));
            match req {
                Ok(resp) => {
                    log::info!("received response of submitblock: {:?}", resp.into_string())
                }
                Err(err) => log::info!("error upon submitblock: {:?}", err),
            }
        }
    }
}

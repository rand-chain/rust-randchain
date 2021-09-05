use ethcore_rpc::{start_http, Compatibility, MetaIoHandler, Remote, Server};
use network::Network;
use p2p;
use std::collections::HashSet;
use std::io;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use storage;
use sync;

pub struct Dependencies {
    pub network: Network,
    pub local_sync_node: sync::LocalNodeRef,
    pub storage: storage::SharedStore,
    pub p2p_context: Arc<p2p::Context>,
    pub remote: Remote,
}

#[derive(Debug, PartialEq)]
pub struct HttpConfiguration {
    pub enabled: bool,
    pub interface: String,
    pub port: u16,
    pub apis: ApiSet,
    pub cors: Option<Vec<String>>,
    pub hosts: Option<Vec<String>>,
}

impl HttpConfiguration {
    pub fn with_port(port: u16) -> Self {
        HttpConfiguration {
            enabled: true,
            interface: "127.0.0.1".into(),
            port: port,
            apis: ApiSet::default(),
            cors: None,
            hosts: Some(Vec::new()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Api {
    /// Miner-related methods
    Miner,
    /// BlockChain-related methods
    BlockChain,
    /// Network
    Network,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ApiSet {
    List(HashSet<Api>),
}

impl Default for ApiSet {
    fn default() -> Self {
        ApiSet::List(
            vec![Api::Miner, Api::BlockChain, Api::Network]
                .into_iter()
                .collect(),
        )
    }
}

impl FromStr for Api {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "miner" => Ok(Api::Miner),
            "blockchain" => Ok(Api::BlockChain),
            "network" => Ok(Api::Network),
            api => Err(format!("Unknown api: {}", api)),
        }
    }
}

impl ApiSet {
    pub fn list_apis(&self) -> HashSet<Api> {
        match *self {
            ApiSet::List(ref apis) => apis.clone(),
        }
    }
}

fn setup_rpc_server(apis: ApiSet, deps: Dependencies) -> MetaIoHandler<()> {
    use ethcore_rpc::v1::*;

    let mut handler = MetaIoHandler::with_compatibility(Compatibility::Both);
    for api in apis.list_apis() {
        match api {
            Api::Miner => handler.extend_with(
                MinerClient::new(MinerClientCore::new(deps.local_sync_node.clone())).to_delegate(),
            ),
            Api::BlockChain => handler.extend_with(
                BlockChainClient::new(BlockChainClientCore::new(
                    deps.p2p_context.clone(),
                    deps.storage.clone(),
                ))
                .to_delegate(),
            ),
            Api::Network => handler.extend_with(
                NetworkClient::new(NetworkClientCore::new(deps.p2p_context.clone())).to_delegate(),
            ),
        }
    }

    handler
}

fn setup_http_rpc_server(
    url: &SocketAddr,
    cors_domains: Option<Vec<String>>,
    allowed_hosts: Option<Vec<String>>,
    apis: ApiSet,
    deps: Dependencies,
) -> Result<Server, String> {
    let server = setup_rpc_server(apis, deps);
    let start_result = start_http(url, cors_domains, allowed_hosts, server);
    match start_result {
		Err(ref err) if err.kind() == io::ErrorKind::AddrInUse => {
			Err(format!("RPC address {} is already in use, make sure that another instance of a RandChain node is not running or change the address using the --jsonrpc-port and --jsonrpc-interface options.", url))
		},
		Err(e) => Err(format!("RPC error: {:?}", e)),
		Ok(server) => Ok(server),
	}
}

pub fn new_http_rpc_server(
    conf: HttpConfiguration,
    deps: Dependencies,
) -> Result<Option<Server>, String> {
    if !conf.enabled {
        return Ok(None);
    }

    let url = format!("{}:{}", conf.interface, conf.port);
    let addr = url
        .parse()
        .map_err(|_| format!("Invalid JSONRPC listen host/port given: {}", url))?;
    Ok(Some(setup_http_rpc_server(
        &addr, conf.cors, conf.hosts, conf.apis, deps,
    )?))
}

use message::Services;

/// Information of the network
/// See https://github.com/bitcoin/bitcoin/blob/master/src/rpc/net.cpp#L575
#[derive(Default, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub version: u32,                            // the server version
    pub subversion: String,                      // the server subversion string
    pub protocolversion: u32,                    // the protocol version
    pub localservices: u64,                      // the services we offer to the network
    pub localservicesnames: Option<Vec<String>>, // the services we offer to the network, in human-readable form
    pub localrelay: Option<bool>, // true if transaction relay is requested from peers
    pub timeoffset: Option<u32>,  // the time offset
    pub connections: u32,         // the total number of connections
    pub connections_in: u32,      // the number of inbound connections
    pub connections_out: u32,     // the number of outbound connections
    pub networkactive: Option<bool>, // whether p2p networking is enabled
    pub networks: Vec<Network>,   // information per network
    pub relayfee: Option<u32>,    // minimum relay fee rate for transactions in CURRENCY_UNIT
    pub incrementalfee: Option<u32>, // minimum fee rate increment for mempool limiting or BIP 125 replacement in CURRENCY_UNIT
    pub localaddresses: Vec<Address>, // list of local addresses
    pub warnings: Option<String>,    // any network and blockchain warnings
}

#[derive(Default, Serialize, Deserialize)]
pub struct Address {
    pub address: String, // network address
    pub port: u32,       // network port
    pub score: u32,      // relative score
}

#[derive(Default, Serialize, Deserialize)]
pub struct Network {
    pub name: String,                              //
    pub limited: Option<bool>,                     // is the network limited using -onlynet?
    pub reachable: bool,                           // is the network reachable?
    pub proxy: String, // (host:port) the proxy that is used for this network, or empty if none
    pub proxy_randomize_credentials: Option<bool>, // Whether randomized credentials are used
}

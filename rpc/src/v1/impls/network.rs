use jsonrpc_core::Error;
use jsonrpc_macros::Trailing;
use p2p;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use v1::helpers::errors;
use v1::traits::Network as NetworkRpc;
use v1::types::Address as AddressType;
use v1::types::Network as NetworkType;
use v1::types::{AddNodeOperation, NetworkInfo, NodeInfo};

pub trait NetworkApi: Send + Sync + 'static {
    fn add_node(&self, socket_addr: SocketAddr) -> Result<(), p2p::NodeTableError>;
    fn remove_node(&self, socket_addr: SocketAddr) -> Result<(), p2p::NodeTableError>;
    fn connect(&self, socket_addr: SocketAddr);
    fn node_info(&self, node_addr: IpAddr) -> Result<NodeInfo, p2p::NodeTableError>;
    fn nodes_info(&self) -> Vec<NodeInfo>;
    fn connection_count(&self) -> usize;
    fn net_info(&self) -> NetworkInfo;
}

impl<T> NetworkRpc for NetworkClient<T>
where
    T: NetworkApi,
{
    fn add_node(&self, node: String, operation: AddNodeOperation) -> Result<(), Error> {
        let addr = node.parse().map_err(|_| {
            errors::invalid_params(
                "node",
                "Invalid socket address format, should be ip:port (127.0.0.1:8008)",
            )
        })?;
        match operation {
            AddNodeOperation::Add => self
                .api
                .add_node(addr)
                .map_err(|_| errors::node_already_added()),
            AddNodeOperation::Remove => self
                .api
                .remove_node(addr)
                .map_err(|_| errors::node_not_added()),
            AddNodeOperation::OneTry => {
                self.api.connect(addr);
                Ok(())
            }
        }
    }

    fn node_info(&self, _dns: bool, node_addr: Trailing<String>) -> Result<Vec<NodeInfo>, Error> {
        let node_addr: Option<String> = node_addr.into();
        Ok(match node_addr {
            None => self.api.nodes_info(),
            Some(node_addr) => {
                let addr = node_addr.parse().map_err(|_| {
                    errors::invalid_params(
                        "node",
                        "Invalid ip address format, should be ip address (127.0.0.1)",
                    )
                })?;
                let node_info = self
                    .api
                    .node_info(addr)
                    .map_err(|_| errors::node_not_added())?;
                vec![node_info]
            }
        })
    }

    fn connection_count(&self) -> Result<usize, Error> {
        Ok(self.api.connection_count())
    }

    fn net_info(&self) -> Result<NetworkInfo, Error> {
        Ok(self.api.net_info())
    }
}

pub struct NetworkClient<T: NetworkApi> {
    api: T,
}

impl<T> NetworkClient<T>
where
    T: NetworkApi,
{
    pub fn new(api: T) -> Self {
        NetworkClient { api: api }
    }
}

pub struct NetworkClientCore {
    p2p: Arc<p2p::Context>,
}

impl NetworkClientCore {
    pub fn new(p2p: Arc<p2p::Context>) -> Self {
        NetworkClientCore { p2p: p2p }
    }
}

impl NetworkApi for NetworkClientCore {
    fn add_node(&self, socket_addr: SocketAddr) -> Result<(), p2p::NodeTableError> {
        self.p2p.add_node(socket_addr)
    }

    fn remove_node(&self, socket_addr: SocketAddr) -> Result<(), p2p::NodeTableError> {
        self.p2p.remove_node(socket_addr)
    }

    fn connect(&self, socket_addr: SocketAddr) {
        p2p::Context::connect_normal(self.p2p.clone(), socket_addr);
    }

    fn node_info(&self, node_addr: IpAddr) -> Result<NodeInfo, p2p::NodeTableError> {
        let exact_node = self
            .p2p
            .nodes()
            .iter()
            .find(|n| n.address().ip() == node_addr)
            .cloned()
            .ok_or(p2p::NodeTableError::NoAddressInTable)?;

        let peers: Vec<p2p::PeerInfo> = self
            .p2p
            .connections()
            .info()
            .into_iter()
            .filter(|p| p.address == exact_node.address())
            .collect();

        Ok(NodeInfo {
            addednode: format!("{}", exact_node.address()),
            connected: !peers.is_empty(),
            addresses: peers.into_iter().map(|p| p.into()).collect(),
        })
    }

    fn nodes_info(&self) -> Vec<NodeInfo> {
        let peers: Vec<p2p::PeerInfo> = self.p2p.connections().info();

        self.p2p
            .nodes()
            .iter()
            .map(|n| {
                let node_peers: Vec<p2p::PeerInfo> = peers
                    .iter()
                    .filter(|p| p.address == n.address())
                    .cloned()
                    .collect();
                NodeInfo {
                    addednode: format!("{}", n.address()),
                    connected: !node_peers.is_empty(),
                    addresses: node_peers.into_iter().map(|p| p.into()).collect(),
                }
            })
            .collect()
    }

    fn connection_count(&self) -> usize {
        self.p2p.connections().count()
    }

    fn net_info(&self) -> NetworkInfo {
        let cfg = self.p2p.config();
        NetworkInfo {
            version: 1,
            subversion: "/Satoshi:0.12.1/".to_owned(),
            protocolversion: cfg.connection.protocol_version,
            localservices: cfg.preferable_services.into(),
            localservicesnames: None,
            localrelay: None,
            timeoffset: None,
            connections: cfg.inbound_connections + cfg.outbound_connections,
            connections_in: cfg.inbound_connections,
            connections_out: cfg.outbound_connections,
            networkactive: None,
            networks: vec![NetworkType {
                name: cfg.connection.network.name(),
                limited: None,
                reachable: true,
                proxy: "".to_string(),
                proxy_randomize_credentials: None,
            }],
            relayfee: None,
            incrementalfee: None,
            localaddresses: vec![],
            warnings: None,
        }
    }
}

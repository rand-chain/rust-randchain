use chain::IndexedBlock;
use message::Services;
use p2p::OutboundSyncConnectionRef;
use parking_lot::RwLock;
use primitives::hash::H256;
use std::collections::HashMap;
use types::PeerIndex;
use utils::{ConnectionFilter, KnownHashType};

/// Block announcement type
#[derive(Debug, Clone, Copy)]
pub enum BlockAnnouncementType {
    /// Send inventory message with block hash [default behavior]
    SendInventory,
    /// Send headers message with block header
    SendHeaders,
    /// Do not announce blocks at all
    DoNotAnnounce,
}

/// Transaction announcement type
#[derive(Debug, Clone, Copy)]
pub enum TransactionAnnouncementType {
    /// Send inventory message with transaction hash [default behavior]
    SendInventory,
    /// Do not announce transactions at all
    DoNotAnnounce,
}

/// Connected peers
pub trait Peers: Send + Sync + PeersContainer + PeersFilters + PeersOptions {
    /// Require peers services.
    fn require_peer_services(&self, services: Services);
    /// Get peer connection
    fn connection(&self, peer_index: PeerIndex) -> Option<OutboundSyncConnectionRef>;
}

/// Connected peers container
pub trait PeersContainer {
    /// Enumerate all known peers (TODO: iterator + separate entity 'Peer')
    fn enumerate(&self) -> Vec<PeerIndex>;
    /// Insert new peer connection
    fn insert(
        &self,
        peer_index: PeerIndex,
        services: Services,
        connection: OutboundSyncConnectionRef,
    );
    /// Remove peer connection
    fn remove(&self, peer_index: PeerIndex);
    /// Close and remove peer connection due to misbehaving
    fn misbehaving(&self, peer_index: PeerIndex, reason: &str);
    /// Close and remove peer connection due to detected DOS attempt
    fn dos(&self, peer_index: PeerIndex, reason: &str);
}

/// Filters for peers connections
pub trait PeersFilters {
    /// Is block passing filters for the connection
    fn filter_block(&self, peer_index: PeerIndex, block: &IndexedBlock) -> BlockAnnouncementType;
    /// Remember known hash
    fn hash_known_as(&self, peer_index: PeerIndex, hash: H256, hash_type: KnownHashType);
    /// Is given hash known by peer as hash of given type
    fn is_hash_known_as(
        &self,
        peer_index: PeerIndex,
        hash: &H256,
        hash_type: KnownHashType,
    ) -> bool;
}

/// Options for peers connections
pub trait PeersOptions {
    /// Set up new block announcement type for the connection
    fn set_block_announcement_type(
        &self,
        peer_index: PeerIndex,
        announcement_type: BlockAnnouncementType,
    );
    /// Set up new transaction announcement type for the connection
    fn set_transaction_announcement_type(
        &self,
        peer_index: PeerIndex,
        announcement_type: TransactionAnnouncementType,
    );
}

/// Single connected peer data
struct Peer {
    /// Connection to this peer
    pub connection: OutboundSyncConnectionRef,
    /// Peer services
    pub services: Services,
    /// Connection filter
    pub filter: ConnectionFilter,
    /// Block announcement type
    pub block_announcement_type: BlockAnnouncementType,
    /// Transaction announcement type
    pub transaction_announcement_type: TransactionAnnouncementType,
}

/// Default implementation of connectd peers container
#[derive(Default)]
pub struct PeersImpl {
    /// All connected peers. Most of times this field is accessed, it is accessed in read mode.
    /// So this lock shouldn't be a performance problem.
    peers: RwLock<HashMap<PeerIndex, Peer>>,
}

impl Peer {
    pub fn new(services: Services, connection: OutboundSyncConnectionRef) -> Self {
        Peer {
            connection: connection,
            services: services,
            filter: ConnectionFilter::default(),
            block_announcement_type: BlockAnnouncementType::SendInventory,
            transaction_announcement_type: TransactionAnnouncementType::SendInventory,
        }
    }
}

impl Peers for PeersImpl {
    fn require_peer_services(&self, services: Services) {
        // possible optimization: force p2p level to establish connections to SegWit-nodes only
        // without it, all other nodes will be eventually banned (this could take some time, though)
        let mut peers = self.peers.write();
        for peer_index in peers
            .iter()
            .filter(|&(_, p)| p.services.includes(&services))
            .map(|(p, _)| *p)
            .collect::<Vec<_>>()
        {
            let peer = peers
                .remove(&peer_index)
                .expect("iterating peers keys; qed");
            let expected_services: u64 = services.into();
            let actual_services: u64 = peer.services.into();
            warn!(target: "sync", "Disconnecting from peer#{} because of insufficient services. Expected {:x}, actual: {:x}", peer_index, expected_services, actual_services);
            peer.connection.close();
        }
    }

    fn connection(&self, peer_index: PeerIndex) -> Option<OutboundSyncConnectionRef> {
        self.peers
            .read()
            .get(&peer_index)
            .map(|peer| peer.connection.clone())
    }
}

impl PeersContainer for PeersImpl {
    fn enumerate(&self) -> Vec<PeerIndex> {
        self.peers.read().keys().cloned().collect()
    }

    fn insert(
        &self,
        peer_index: PeerIndex,
        services: Services,
        connection: OutboundSyncConnectionRef,
    ) {
        trace!(target: "sync", "Connected to peer#{}", peer_index);
        assert!(self
            .peers
            .write()
            .insert(peer_index, Peer::new(services, connection))
            .is_none());
    }

    fn remove(&self, peer_index: PeerIndex) {
        if self.peers.write().remove(&peer_index).is_some() {
            trace!(target: "sync", "Disconnected from peer#{}", peer_index);
        }
    }

    fn misbehaving(&self, peer_index: PeerIndex, reason: &str) {
        if let Some(peer) = self.peers.write().remove(&peer_index) {
            warn!(target: "sync", "Disconnecting from peer#{} due to misbehavior: {}", peer_index, reason);
            peer.connection.close();
        }
    }

    fn dos(&self, peer_index: PeerIndex, reason: &str) {
        if let Some(peer) = self.peers.write().remove(&peer_index) {
            warn!(target: "sync", "Disconnecting from peer#{} due to DoS: {}", peer_index, reason);
            peer.connection.close();
        }
    }
}

impl PeersFilters for PeersImpl {
    fn filter_block(&self, peer_index: PeerIndex, block: &IndexedBlock) -> BlockAnnouncementType {
        if let Some(peer) = self.peers.read().get(&peer_index) {
            if peer.filter.filter_block(&block.header.hash) {
                return peer.block_announcement_type;
            }
        }

        BlockAnnouncementType::DoNotAnnounce
    }

    fn hash_known_as(&self, peer_index: PeerIndex, hash: H256, hash_type: KnownHashType) {
        if let Some(peer) = self.peers.write().get_mut(&peer_index) {
            peer.filter.hash_known_as(hash, hash_type)
        }
    }

    fn is_hash_known_as(
        &self,
        peer_index: PeerIndex,
        hash: &H256,
        hash_type: KnownHashType,
    ) -> bool {
        self.peers
            .read()
            .get(&peer_index)
            .map(|peer| peer.filter.is_hash_known_as(hash, hash_type))
            .unwrap_or(false)
    }
}

impl PeersOptions for PeersImpl {
    fn set_block_announcement_type(
        &self,
        peer_index: PeerIndex,
        announcement_type: BlockAnnouncementType,
    ) {
        if let Some(peer) = self.peers.write().get_mut(&peer_index) {
            peer.block_announcement_type = announcement_type;
        }
    }

    fn set_transaction_announcement_type(
        &self,
        peer_index: PeerIndex,
        announcement_type: TransactionAnnouncementType,
    ) {
        if let Some(peer) = self.peers.write().get_mut(&peer_index) {
            peer.transaction_announcement_type = announcement_type;
        }
    }
}

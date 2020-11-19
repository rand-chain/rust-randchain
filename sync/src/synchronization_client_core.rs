use chain::{IndexedBlock, IndexedBlockHeader};
use futures::Future;
use message::common::{InventoryType, InventoryVector};
use message::types;
use parking_lot::Mutex;
use primitives::hash::H256;
use std::cmp::{max, min};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
#[cfg(test)]
use synchronization_chain::Information as ChainInformation;
use synchronization_chain::{BlockInsertionResult, BlockState, Chain};
use synchronization_executor::{Task, TaskExecutor};
use synchronization_manager::ManagementWorker;
#[cfg(test)]
use synchronization_peers_tasks::Information as PeersTasksInformation;
use synchronization_peers_tasks::PeersTasks;
use synchronization_verifier::{BlockVerificationSink, VerificationSink, VerificationTask};
use time::precise_time_s;
use types::{
    BlockHeight, ClientCoreRef, EmptyBoxFuture, PeerIndex, PeersRef, SyncListenerRef,
    SynchronizationStateRef,
};
use utils::{AverageSpeedMeter, HashPosition, MessageBlockHeadersProvider, OrphanBlocksPool};
use verification::BackwardsCompatibleChainVerifier as ChainVerifier;

/// Approximate maximal number of blocks hashes in scheduled queue.
const MAX_SCHEDULED_HASHES: BlockHeight = 4 * 1024;
/// Approximate maximal number of blocks hashes in requested queue.
const MAX_REQUESTED_BLOCKS: BlockHeight = 256;
/// Approximate maximal number of blocks in verifying queue.
const MAX_VERIFYING_BLOCKS: BlockHeight = 256;
/// Minimum number of blocks to request from peer
const MIN_BLOCKS_IN_REQUEST: BlockHeight = 32;
/// Maximum number of blocks to request from peer
const MAX_BLOCKS_IN_REQUEST: BlockHeight = 128;
/// Number of blocks to receive since synchronization start to begin duplicating blocks requests
const NEAR_EMPTY_VERIFICATION_QUEUE_THRESHOLD_BLOCKS: usize = 20;
/// Number of seconds left before verification queue will be empty to count it as 'near empty queue'
const NEAR_EMPTY_VERIFICATION_QUEUE_THRESHOLD_S: f64 = 20_f64;
/// Number of blocks to inspect when calculating average sync speed
const SYNC_SPEED_BLOCKS_TO_INSPECT: usize = 512;
/// Number of blocks to inspect when calculating average blocks speed
const BLOCKS_SPEED_BLOCKS_TO_INSPECT: usize = 512;
/// Minimal time between duplicated blocks requests.
const MIN_BLOCK_DUPLICATION_INTERVAL_S: f64 = 10_f64;
/// Maximal number of blocks in duplicate requests.
const MAX_BLOCKS_IN_DUPLICATE_REQUEST: BlockHeight = 4;
/// Minimal number of blocks in duplicate requests.
const MIN_BLOCKS_IN_DUPLICATE_REQUEST: BlockHeight = 8;

/// Information on current synchronization state.
#[cfg(test)]
#[derive(Debug)]
pub struct Information {
    /// Current synchronization state.
    pub state: State,
    /// Information on synchronization peers.
    pub peers_tasks: PeersTasksInformation,
    /// Current synchronization chain inormation.
    pub chain: ChainInformation,
    /// Number of currently orphaned blocks.
    pub orphaned_blocks: usize,
}

/// Synchronization client trait
pub trait ClientCore {
    fn on_connect(&mut self, peer_index: PeerIndex);
    fn on_disconnect(&mut self, peer_index: PeerIndex);
    fn on_inventory(&self, peer_index: PeerIndex, message: types::Inv);
    fn on_headers(&mut self, peer_index: PeerIndex, message: Vec<IndexedBlockHeader>);
    fn on_block(
        &mut self,
        peer_index: PeerIndex,
        block: IndexedBlock,
    ) -> Option<VecDeque<IndexedBlock>>;
    fn on_notfound(&mut self, peer_index: PeerIndex, message: types::NotFound);
    fn after_peer_nearly_blocks_verified(&mut self, peer_index: PeerIndex, future: EmptyBoxFuture);
    fn install_sync_listener(&mut self, listener: SyncListenerRef);
    fn execute_synchronization_tasks(
        &mut self,
        forced_blocks_requests: Option<Vec<H256>>,
        final_blocks_requests: Option<Vec<H256>>,
    );
    fn try_switch_to_saturated_state(&mut self) -> bool;
}

/// Synchronization client configuration options.
#[derive(Debug)]
pub struct Config {
    /// If true, connection to peer who has provided us with bad block is closed
    pub close_connection_on_bad_block: bool,
}

/// Synchronization client.
pub struct SynchronizationClientCore<T: TaskExecutor> {
    /// Shared synchronization client state.
    shared_state: SynchronizationStateRef,
    /// Synchronization state.
    state: State,
    /// Sync management worker.
    management_worker: Option<ManagementWorker>,
    /// Synchronization peers
    peers: PeersRef,
    /// Synchronization peers tasks.
    peers_tasks: PeersTasks,
    /// Task executor.
    executor: Arc<T>,
    /// Chain reference.
    chain: Chain,
    /// Orphaned blocks pool.
    orphaned_blocks_pool: OrphanBlocksPool,
    /// Chain verifier
    chain_verifier: Arc<ChainVerifier>,
    /// Verify block headers?
    verify_headers: bool,
    /// Verifying blocks by peer
    verifying_blocks_by_peer: HashMap<H256, PeerIndex>,
    /// Verifying blocks futures
    verifying_blocks_futures: HashMap<PeerIndex, (HashSet<H256>, Vec<EmptyBoxFuture>)>,
    /// Hashes of items we do not want to relay after verification is completed
    do_not_relay: HashSet<H256>,
    /// Block processing speed meter
    block_speed_meter: AverageSpeedMeter,
    /// Block synchronization speed meter
    sync_speed_meter: AverageSpeedMeter,
    /// Configuration
    config: Config,
    /// Synchronization events listener
    listener: Option<SyncListenerRef>,
    /// Time of last duplicated blocks request.
    last_dup_time: f64,
}

/// Verification sink for synchronization client core
pub struct CoreVerificationSink<T: TaskExecutor> {
    /// Client core reference
    core: ClientCoreRef<SynchronizationClientCore<T>>,
}

/// Synchronization state
#[derive(Debug, Clone, Copy)]
pub enum State {
    /// We know that there are > 1 unknown blocks, unknown to us in the blockchain
    Synchronizing(f64, BlockHeight),
    /// There is only one unknown block in the blockchain
    NearlySaturated,
    /// We have downloaded all blocks of the blockchain of which we have ever heard
    Saturated,
}

/// Blocks request limits.
pub struct BlocksRequestLimits {
    /// Approximate maximal number of blocks hashes in scheduled queue.
    pub max_scheduled_hashes: BlockHeight,
    /// Approximate maximal number of blocks hashes in requested queue.
    pub max_requested_blocks: BlockHeight,
    /// Approximate maximal number of blocks in verifying queue.
    pub max_verifying_blocks: BlockHeight,
    /// Minimum number of blocks to request from peer
    pub min_blocks_in_request: BlockHeight,
    /// Maximum number of blocks to request from peer
    pub max_blocks_in_request: BlockHeight,
}

/// Blocks headers verification result
enum BlocksHeadersVerificationResult {
    /// Skip these blocks headers
    Skip,
    /// Error during verification of header with given index
    Error(usize),
    /// Successful verification
    Success,
}

impl State {
    pub fn is_saturated(&self) -> bool {
        match *self {
            State::Saturated => true,
            _ => false,
        }
    }

    pub fn is_synchronizing(&self) -> bool {
        match *self {
            State::Synchronizing(_, _) => true,
            _ => false,
        }
    }

    pub fn is_nearly_saturated(&self) -> bool {
        match *self {
            State::NearlySaturated => true,
            _ => false,
        }
    }
}

impl<T> ClientCore for SynchronizationClientCore<T>
where
    T: TaskExecutor,
{
    fn on_connect(&mut self, peer_index: PeerIndex) {
        // ask peer for its block headers to find our best common block
        let block_locator_hashes = self.chain.block_locator_hashes();
        self.executor.execute(Task::GetHeaders(
            peer_index,
            types::GetHeaders::with_block_locator_hashes(block_locator_hashes),
        ));
        // unuseful until respond with headers message
        self.peers_tasks.unuseful_peer(peer_index);
        self.peers_tasks.on_headers_requested(peer_index);
    }

    fn on_disconnect(&mut self, peer_index: PeerIndex) {
        // sync tasks from this peers must be executed by other peers
        let peer_tasks = self.peers_tasks.reset_blocks_tasks(peer_index);
        self.peers_tasks.disconnect(peer_index);
        self.execute_synchronization_tasks(Some(peer_tasks), None);
    }

    fn on_inventory(&self, peer_index: PeerIndex, message: types::Inv) {
        // else ask for all unknown transactions and blocks
        let unknown_inventory: Vec<_> = message
            .inventory
            .into_iter()
            .filter(|item| {
                match item.inv_type {
                    // check that block is unknown to us
                    InventoryType::MessageBlock => match self.chain.block_state(&item.hash) {
                        BlockState::Unknown => {
                            !self.orphaned_blocks_pool.contains_unknown_block(&item.hash)
                        }
                        BlockState::DeadEnd if !self.config.close_connection_on_bad_block => true,
                        BlockState::DeadEnd if self.config.close_connection_on_bad_block => {
                            self.peers.misbehaving(
                                peer_index,
                                &format!(
                                    "Provided dead-end block {:?}",
                                    item.hash.to_reversed_str()
                                ),
                            );
                            false
                        }
                        _ => false,
                    },
                    // unknown inventory type
                    InventoryType::Error => {
                        self.peers.misbehaving(
                            peer_index,
                            &format!(
                                "Provided unknown inventory type {:?}",
                                item.hash.to_reversed_str()
                            ),
                        );
                        false
                    }
                }
            })
            .collect();

        // if everything is known => ignore this message
        if unknown_inventory.is_empty() {
            trace!(target: "sync", "Ignoring inventory message from peer#{} as all items are known", peer_index);
            return;
        }

        // ask for unknown items
        let message = types::GetData::with_inventory(unknown_inventory);
        self.executor.execute(Task::GetData(peer_index, message));
    }

    /// Try to queue synchronization of unknown blocks when blocks headers are received.
    fn on_headers(&mut self, peer_index: PeerIndex, mut headers: Vec<IndexedBlockHeader>) {
        assert!(
            !headers.is_empty(),
            "This must be checked in incoming connection"
        );

        // update peers to select next tasks
        self.peers_tasks.on_headers_received(peer_index);

        // headers are ordered
        // => if we know nothing about headers[0].parent
        // => all headers are also unknown to us
        let header0 = headers[0].clone();
        if self.chain.block_state(&header0.raw.previous_header_hash) == BlockState::Unknown {
            warn!(
                target: "sync",
                "Previous header of the first header from peer#{} `headers` message is unknown. First: {}. Previous: {}",
                peer_index,
                header0.hash.to_reversed_str(),
                header0.raw.previous_header_hash.to_reversed_str(),
            );

            // there could be competing chains that are running the network with the same magic (like Zcash vs ZelCash)
            // => providing unknown headers. Penalize node so that it'll disconnect
            if self.peers_tasks.penalize(peer_index) {
                self.peers.misbehaving(peer_index, "Too many failures.");
            }

            return;
        }

        // find first unknown header position
        // optimization: normally, the first header will be unknown
        let num_headers = headers.len();
        let first_unknown_index = match self.chain.block_state(&header0.hash) {
            BlockState::Unknown => 0,
            _ => {
                // optimization: if last header is known, then all headers are also known
                let header_last = &headers[num_headers - 1];
                match self.chain.block_state(&header_last.hash) {
					BlockState::Unknown => 1 + headers.iter().skip(1)
						.position(|header| self.chain.block_state(&header.hash) == BlockState::Unknown)
						.expect("last header has UnknownState; we are searching for first unknown header; qed"),
					// else all headers are known
					_ => {
						trace!(target: "sync", "Ignoring {} known headers from peer#{}", headers.len(), peer_index);
						// but this peer is still useful for synchronization
						self.peers_tasks.useful_peer(peer_index);
						return;
					},
				}
            }
        };

        // validate blocks headers before scheduling
        let last_known_hash = if first_unknown_index > 0 {
            headers[first_unknown_index - 1].hash.clone()
        } else {
            header0.raw.previous_header_hash.clone()
        };
        if self.config.close_connection_on_bad_block
            && self.chain.block_state(&last_known_hash) == BlockState::DeadEnd
        {
            self.peers.misbehaving(
                peer_index,
                &format!(
                    "Provided after dead-end block {}",
                    last_known_hash.to_reversed_str()
                ),
            );
            return;
        }
        match self.verify_headers(
            peer_index,
            last_known_hash,
            &headers[first_unknown_index..num_headers],
        ) {
            BlocksHeadersVerificationResult::Error(error_index) => self
                .chain
                .mark_dead_end_block(&headers[first_unknown_index + error_index].hash),
            BlocksHeadersVerificationResult::Skip => (),
            BlocksHeadersVerificationResult::Success => {
                // report progress
                let num_new_headers = num_headers - first_unknown_index;
                trace!(target: "sync", "New {} headers from peer#{}. First {:?}, last: {:?}",
                    num_new_headers,
                    peer_index,
                    headers[first_unknown_index].hash.to_reversed_str(),
                    headers[num_headers - 1].hash.to_reversed_str()
                );

                // prepare new headers array
                let new_headers = headers.split_off(first_unknown_index);
                self.chain.schedule_blocks_headers(new_headers);

                // switch to synchronization state
                if !self.state.is_synchronizing() {
                    if self.chain.length_of_blocks_state(BlockState::Scheduled)
                        + self.chain.length_of_blocks_state(BlockState::Requested)
                        == 1
                    {
                        self.switch_to_nearly_saturated_state();
                    } else {
                        self.switch_to_synchronization_state();
                    }
                }

                // this peers has supplied us with new headers => useful indeed
                self.peers_tasks.useful_peer(peer_index);
                // and execute tasks
                self.execute_synchronization_tasks(None, None);
            }
        }
    }

    fn on_block(
        &mut self,
        peer_index: PeerIndex,
        block: IndexedBlock,
    ) -> Option<VecDeque<IndexedBlock>> {
        // update peers to select next tasks
        self.peers_tasks
            .on_block_received(peer_index, &block.header.hash);

        // prepare list of blocks to verify + make all required changes to the chain
        let mut result: Option<VecDeque<IndexedBlock>> = None;
        let block_state = self.chain.block_state(&block.header.hash);
        match block_state {
            BlockState::Verifying | BlockState::Stored => {
                // remember peer as useful
                // and do nothing else, because we have already processed this block before
                self.peers_tasks.useful_peer(peer_index);
            }
            BlockState::Unknown
            | BlockState::Scheduled
            | BlockState::Requested
            | BlockState::DeadEnd => {
                if block_state == BlockState::DeadEnd {
                    if self.config.close_connection_on_bad_block {
                        self.peers.misbehaving(
                            peer_index,
                            &format!(
                                "Provided dead-end block {}",
                                block.header.hash.to_reversed_str()
                            ),
                        );
                        return None;
                    }
                    warn!(target: "sync", "Peer#{} has provided dead-end block {}", peer_index, block.header.hash.to_reversed_str());
                }

                // check parent block state
                let parent_block_state = self
                    .chain
                    .block_state(&block.header.raw.previous_header_hash);
                match parent_block_state {
                    BlockState::Unknown | BlockState::DeadEnd => {
                        if parent_block_state == BlockState::DeadEnd {
                            if self.config.close_connection_on_bad_block {
                                self.peers.misbehaving(
                                    peer_index,
                                    &format!(
                                        "Provided dead-end block {}",
                                        block.header.hash.to_reversed_str()
                                    ),
                                );
                                return None;
                            }
                            warn!(target: "sync", "Peer#{} has provided dead-end block {}", peer_index, block.header.hash.to_reversed_str());
                        }

                        if self.state.is_synchronizing() {
                            // when synchronizing, we tend to receive all blocks in-order
                            trace!(
                                target: "sync",
                                "Ignoring block {} from peer#{}, because its parent is unknown and we are synchronizing",
                                block.header.hash.to_reversed_str(),
                                peer_index
                            );
                            // remove block from current queue
                            self.chain.forget_block(&block.header.hash);
                            // remove orphaned blocks
                            let removed_blocks_hashes: Vec<_> = self
                                .orphaned_blocks_pool
                                .remove_blocks_for_parent(block.hash())
                                .into_iter()
                                .map(|b| b.header.hash)
                                .collect();
                            self.chain
                                .forget_blocks_leave_header(&removed_blocks_hashes);
                        } else {
                            // remove this block from the queue
                            self.chain.forget_block_leave_header(&block.header.hash);
                            // remember this block as unknown
                            if !self
                                .orphaned_blocks_pool
                                .contains_unknown_block(&block.header.hash)
                            {
                                self.orphaned_blocks_pool.insert_unknown_block(block);
                            }
                        }
                    }
                    BlockState::Verifying | BlockState::Stored => {
                        // update synchronization speed
                        self.sync_speed_meter.checkpoint();
                        // remember peer as useful
                        self.peers_tasks.useful_peer(peer_index);
                        // schedule verification
                        let mut blocks_to_verify: VecDeque<IndexedBlock> = VecDeque::new();
                        blocks_to_verify.extend(
                            self.orphaned_blocks_pool
                                .remove_blocks_for_parent(&block.header.hash),
                        );
                        blocks_to_verify.push_front(block);
                        // forget blocks we are going to process
                        let blocks_hashes_to_forget: Vec<_> =
                            blocks_to_verify.iter().map(|b| b.hash().clone()).collect();
                        self.chain
                            .forget_blocks_leave_header(&blocks_hashes_to_forget);
                        // remember that we are verifying these blocks
                        let blocks_headers_to_verify: Vec<_> =
                            blocks_to_verify.iter().map(|b| b.header.clone()).collect();
                        self.chain.verify_blocks(blocks_headers_to_verify);
                        // remember that we are verifying block from this peer
                        for verifying_block_hash in
                            blocks_to_verify.iter().map(|b| b.hash().clone())
                        {
                            self.verifying_blocks_by_peer
                                .insert(verifying_block_hash, peer_index);
                        }
                        match self.verifying_blocks_futures.entry(peer_index) {
                            Entry::Occupied(mut entry) => {
                                entry
                                    .get_mut()
                                    .0
                                    .extend(blocks_to_verify.iter().map(|b| b.hash().clone()));
                            }
                            Entry::Vacant(entry) => {
                                let block_hashes: HashSet<_> =
                                    blocks_to_verify.iter().map(|b| b.hash().clone()).collect();
                                entry.insert((block_hashes, Vec::new()));
                            }
                        }
                        result = Some(blocks_to_verify);
                    }
                    BlockState::Requested | BlockState::Scheduled => {
                        // remember peer as useful
                        self.peers_tasks.useful_peer(peer_index);
                        // remember as orphan block
                        self.orphaned_blocks_pool.insert_orphaned_block(block);
                    }
                }
            }
        }

        result
    }

    /// When peer has no blocks
    fn on_notfound(&mut self, peer_index: PeerIndex, message: types::NotFound) {
        let notfound_blocks: HashSet<_> = message
            .inventory
            .into_iter()
            .filter(|item| item.inv_type == InventoryType::MessageBlock)
            .map(|item| item.hash)
            .collect();

        // we only interested in notfound blocks
        if notfound_blocks.is_empty() {
            return;
        }

        // we only interested in blocks, which we were asking before
        let is_requested_block =
            if let Some(requested_blocks) = self.peers_tasks.get_blocks_tasks(peer_index) {
                // check if peer has responded with notfound to requested blocks
                // if notfound some other blocks => just ignore the message
                requested_blocks
                    .intersection(&notfound_blocks)
                    .nth(0)
                    .is_some()
            } else {
                false
            };

        if is_requested_block {
            // for now, let's exclude peer from synchronization - we are relying on full nodes for synchronization
            let removed_tasks = self.peers_tasks.reset_blocks_tasks(peer_index);
            self.peers_tasks.unuseful_peer(peer_index);
            if self.state.is_synchronizing() {
                self.peers.misbehaving(
                    peer_index,
                    &format!("Responded with NotFound(unrequested_block)"),
                );
            }

            // if peer has had some blocks tasks, rerequest these blocks
            self.execute_synchronization_tasks(Some(removed_tasks), None);
        }
    }

    /// Execute after last block from this peer in NearlySaturated state is verified.
    /// If there are no verifying blocks from this peer or we are not in the NearlySaturated state => execute immediately.
    fn after_peer_nearly_blocks_verified(&mut self, peer_index: PeerIndex, future: EmptyBoxFuture) {
        // if we are currently synchronizing => no need to wait
        if self.state.is_synchronizing() {
            future.wait().expect("no-error future");
            return;
        }

        // we have to wait until all previous peer requests are server
        match self.verifying_blocks_futures.entry(peer_index) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().1.push(future);
            }
            _ => future.wait().expect("no-error future"),
        }
    }

    fn install_sync_listener(&mut self, listener: SyncListenerRef) {
        // currently single, single-setup listener is supported
        assert!(self.listener.is_none());
        self.listener = Some(listener);
    }

    /// Schedule new synchronization tasks, if any.
    fn execute_synchronization_tasks(
        &mut self,
        forced_blocks_requests: Option<Vec<H256>>,
        final_blocks_requests: Option<Vec<H256>>,
    ) {
        let mut tasks: Vec<Task> = Vec::new();

        // display information if processed many blocks || enough time has passed since sync start
        self.print_synchronization_information();

        // prepare limits. TODO: must be updated using current retrieval && verification speed && blocks size
        let mut limits = BlocksRequestLimits::default();
        if self.chain.length_of_blocks_state(BlockState::Stored) > 150_000 {
            limits.min_blocks_in_request = 8;
            limits.max_blocks_in_request = 16;
        }

        // if some blocks requests are forced => we should ask peers even if there are no idle peers
        let verifying_hashes_len = self.chain.length_of_blocks_state(BlockState::Verifying);
        if let Some(forced_blocks_requests) = forced_blocks_requests {
            let useful_peers = self.peers_tasks.useful_peers();
            // if we have to request blocks && there are no useful peers at all => switch to saturated state
            if useful_peers.is_empty() {
                warn!(target: "sync", "Last peer was marked as non-useful. Moving to saturated state.");
                self.switch_to_saturated_state();
                return;
            }

            let forced_tasks =
                self.prepare_blocks_requests_tasks(&limits, useful_peers, forced_blocks_requests);
            tasks.extend(forced_tasks);
        }

        // if some blocks requests are marked as last [i.e. blocks are potentialy wrong] => ask peers anyway
        if let Some(final_blocks_requests) = final_blocks_requests {
            let useful_peers = self.peers_tasks.useful_peers();
            if !useful_peers.is_empty() {
                // if empty => not a problem, just forget these blocks
                let forced_tasks = self.prepare_blocks_requests_tasks(
                    &limits,
                    useful_peers,
                    final_blocks_requests,
                );
                tasks.extend(forced_tasks);
            }
        }

        let mut blocks_requests: Option<Vec<H256>> = None;
        let blocks_idle_peers: Vec<_> = self
            .peers_tasks
            .idle_peers_for_blocks()
            .iter()
            .cloned()
            .collect();
        {
            // check if we can query some blocks headers
            let headers_idle_peers: Vec<_> = self
                .peers_tasks
                .idle_peers_for_headers()
                .iter()
                .cloned()
                .collect();
            if !headers_idle_peers.is_empty() {
                let scheduled_hashes_len = self.chain.length_of_blocks_state(BlockState::Scheduled);
                if scheduled_hashes_len < MAX_SCHEDULED_HASHES {
                    for header_peer in &headers_idle_peers {
                        self.peers_tasks.on_headers_requested(*header_peer);
                    }

                    let block_locator_hashes = self.chain.block_locator_hashes();
                    let headers_tasks = headers_idle_peers.iter().map(move |peer_index| {
                        Task::GetHeaders(
                            *peer_index,
                            types::GetHeaders::with_block_locator_hashes(
                                block_locator_hashes.clone(),
                            ),
                        )
                    });
                    tasks.extend(headers_tasks);
                }
            }

            let blocks_idle_peers_len = blocks_idle_peers.len() as BlockHeight;
            if blocks_idle_peers_len != 0 {
                // check if verification queue is empty/almost empty
                // && there are pending blocks requests
                // && there are idle block peers
                // => we may need to duplicate pending blocks requests to idle peers
                // this will result in additional network load, but verification queue will be filled up earlier
                // it is very useful when dealing with large blocks + some peer is responding, but with very low speed:
                //    requested: [B1, B2, B3, B4] from peer1
                //    orphans: [B5, B6, B7, B8, ... B1024] ===> 1GB of RAM
                //    verifying: None <=== we are waiting for B1 to come
                //    idle: [peer2]
                //    peer1 responds with single block in ~20 seconds
                //    => we could ask idle peer2 about [B1, B2, B3, B4]
                // these requests has priority over new blocks requests below
                let requested_hashes_len = self.chain.length_of_blocks_state(BlockState::Requested);
                if requested_hashes_len != 0 {
                    let verification_speed: f64 = self.block_speed_meter.speed();
                    let synchronization_speed: f64 = self.sync_speed_meter.speed();
                    // estimate time when verification queue will be empty
                    let verification_queue_will_be_empty_in = if verifying_hashes_len == 0 {
                        // verification queue is already empty
                        if self.block_speed_meter.inspected_items_len()
                            < NEAR_EMPTY_VERIFICATION_QUEUE_THRESHOLD_BLOCKS
                        {
                            // the very beginning of synchronization
                            // => peers have not yet responded with a single requested blocks
                            60_f64
                        } else {
                            // blocks were are already received
                            // => bad situation
                            0_f64
                        }
                    } else if verification_speed < 0.01_f64 {
                        // verification speed is too slow
                        60_f64
                    } else {
                        // blocks / (blocks / second) -> second
                        verifying_hashes_len as f64 / verification_speed
                    };
                    // estimate time when all synchronization requests will complete
                    let synchronization_queue_will_be_full_in = if synchronization_speed < 0.01_f64
                    {
                        // synchronization speed is too slow
                        60_f64
                    } else {
                        // blocks / (blocks / second) -> second
                        requested_hashes_len as f64 / synchronization_speed
                    };
                    // if verification queue will be empty before all synchronization requests will be completed
                    // + do not spam with duplicated blocks requests if blocks are too big && there are still blocks left for NEAR_EMPTY_VERIFICATION_QUEUE_THRESHOLD_S
                    // => duplicate blocks requests
                    let now = precise_time_s();
                    if synchronization_queue_will_be_full_in > verification_queue_will_be_empty_in
                        && verification_queue_will_be_empty_in
                            < NEAR_EMPTY_VERIFICATION_QUEUE_THRESHOLD_S
                        && now - self.last_dup_time > MIN_BLOCK_DUPLICATION_INTERVAL_S
                    {
                        // do not duplicate too often
                        self.last_dup_time = now;
                        // blocks / second * second -> blocks
                        let hashes_requests_to_duplicate_len = (synchronization_speed
                            * (synchronization_queue_will_be_full_in
                                - verification_queue_will_be_empty_in))
                            as BlockHeight;
                        // do not ask for too many blocks
                        let hashes_requests_to_duplicate_len = min(
                            MAX_BLOCKS_IN_DUPLICATE_REQUEST,
                            hashes_requests_to_duplicate_len,
                        );
                        // ask for at least 1 block
                        let hashes_requests_to_duplicate_len = max(
                            MIN_BLOCKS_IN_DUPLICATE_REQUEST,
                            min(requested_hashes_len, hashes_requests_to_duplicate_len),
                        );
                        blocks_requests = Some(self.chain.best_n_of_blocks_state(
                            BlockState::Requested,
                            hashes_requests_to_duplicate_len as BlockHeight,
                        ));

                        trace!(target: "sync", "Duplicating {} blocks requests. Sync speed: {} * {}, blocks speed: {} * {}.", hashes_requests_to_duplicate_len, synchronization_speed, requested_hashes_len, verification_speed, verifying_hashes_len);
                    }
                }

                // check if we can move some blocks from scheduled to requested queue
                {
                    // TODO: only request minimal number of blocks, if other urgent blocks are requested
                    let scheduled_hashes_len =
                        self.chain.length_of_blocks_state(BlockState::Scheduled);
                    if requested_hashes_len + verifying_hashes_len
                        < MAX_REQUESTED_BLOCKS + MAX_VERIFYING_BLOCKS
                        && scheduled_hashes_len != 0
                    {
                        let chunk_size = min(
                            limits.max_blocks_in_request,
                            max(
                                scheduled_hashes_len / blocks_idle_peers_len,
                                limits.min_blocks_in_request,
                            ),
                        );
                        let hashes_to_request_len = chunk_size * blocks_idle_peers_len;
                        let hashes_to_request =
                            self.chain.request_blocks_hashes(hashes_to_request_len);
                        match blocks_requests {
                            Some(ref mut blocks_requests) => {
                                blocks_requests.extend(hashes_to_request)
                            }
                            None => blocks_requests = Some(hashes_to_request),
                        }
                    }
                }
            }
        }

        // append blocks requests tasks
        if let Some(blocks_requests) = blocks_requests {
            tasks.extend(self.prepare_blocks_requests_tasks(
                &limits,
                blocks_idle_peers,
                blocks_requests,
            ));
        }

        // execute synchronization tasks
        for task in tasks {
            self.executor.execute(task);
        }
    }

    fn try_switch_to_saturated_state(&mut self) -> bool {
        let switch_to_saturated = {
            // requested block is received => move to saturated state if there are no more blocks
            self.chain.length_of_blocks_state(BlockState::Scheduled) == 0
                && self.chain.length_of_blocks_state(BlockState::Requested) == 0
        };

        if switch_to_saturated {
            self.switch_to_saturated_state();
        }

        switch_to_saturated
    }
}

impl<T> CoreVerificationSink<T>
where
    T: TaskExecutor,
{
    pub fn new(core: ClientCoreRef<SynchronizationClientCore<T>>) -> Self {
        CoreVerificationSink { core: core }
    }
}

impl<T> VerificationSink for CoreVerificationSink<T> where T: TaskExecutor {}

impl<T> BlockVerificationSink for CoreVerificationSink<T>
where
    T: TaskExecutor,
{
    /// Process successful block verification
    fn on_block_verification_success(&self, block: IndexedBlock) -> Option<Vec<VerificationTask>> {
        self.core.lock().on_block_verification_success(block)
    }

    /// Process failed block verification
    fn on_block_verification_error(&self, err: &str, hash: &H256) {
        self.core.lock().on_block_verification_error(err, hash)
    }
}

impl<T> SynchronizationClientCore<T>
where
    T: TaskExecutor,
{
    /// Create new synchronization client core
    pub fn new(
        config: Config,
        shared_state: SynchronizationStateRef,
        peers: PeersRef,
        executor: Arc<T>,
        chain: Chain,
        chain_verifier: Arc<ChainVerifier>,
    ) -> ClientCoreRef<Self> {
        let sync = Arc::new(Mutex::new(SynchronizationClientCore {
            shared_state: shared_state,
            state: State::Saturated,
            peers: peers,
            peers_tasks: PeersTasks::default(),
            management_worker: None,
            executor: executor,
            chain: chain,
            orphaned_blocks_pool: OrphanBlocksPool::new(),
            chain_verifier: chain_verifier,
            verify_headers: true,
            verifying_blocks_by_peer: HashMap::new(),
            verifying_blocks_futures: HashMap::new(),
            do_not_relay: HashSet::new(),
            block_speed_meter: AverageSpeedMeter::with_inspect_items(SYNC_SPEED_BLOCKS_TO_INSPECT),
            sync_speed_meter: AverageSpeedMeter::with_inspect_items(BLOCKS_SPEED_BLOCKS_TO_INSPECT),
            config: config,
            listener: None,
            last_dup_time: 0f64,
        }));

        {
            let csync = Arc::downgrade(&sync);
            let mut lsync = sync.lock();
            lsync.management_worker = Some(ManagementWorker::new(csync));
        }

        sync
    }

    /// Get information on current synchronization state.
    #[cfg(test)]
    pub fn information(&self) -> Information {
        Information {
            state: self.state,
            peers_tasks: self.peers_tasks.information(),
            chain: self.chain.information(),
            orphaned_blocks: self.orphaned_blocks_pool.len(),
        }
    }

    /// Get synchronization state
    pub fn state(&self) -> State {
        self.state
    }

    /// Return chain reference
    pub fn chain(&mut self) -> &mut Chain {
        &mut self.chain
    }

    /// Return peers reference
    pub fn peers(&self) -> PeersRef {
        self.peers.clone()
    }

    /// Return peers tasks reference
    pub fn peers_tasks(&mut self) -> &mut PeersTasks {
        &mut self.peers_tasks
    }

    /// Get orphaned blocks pool reference
    pub fn orphaned_blocks_pool(&mut self) -> &mut OrphanBlocksPool {
        &mut self.orphaned_blocks_pool
    }

    /// Verify block headers or not?
    #[cfg(test)]
    pub fn set_verify_headers(&mut self, verify: bool) {
        self.verify_headers = verify;
    }

    /// Print synchronization information
    pub fn print_synchronization_information(&mut self) {
        if let State::Synchronizing(timestamp, num_of_blocks) = self.state {
            let new_timestamp = precise_time_s();
            let timestamp_diff = new_timestamp - timestamp;
            let new_num_of_blocks = self.chain.best_storage_block().number;
            let blocks_diff = if new_num_of_blocks > num_of_blocks {
                new_num_of_blocks - num_of_blocks
            } else {
                0
            };
            if timestamp_diff >= 60.0 || blocks_diff >= 1000 {
                self.state = State::Synchronizing(precise_time_s(), new_num_of_blocks);
                let blocks_speed = blocks_diff as f64 / timestamp_diff;
                info!(target: "sync", "Processed {} blocks in {:.2} seconds ({:.2} blk/s).\tPeers: {:?}.\tChain: {:?}"
					, blocks_diff
					, timestamp_diff
					, blocks_speed
					, self.peers_tasks.information()
					, self.chain.information());
            }
        }
    }

    /// Forget blocks, which have been requested several times, but no one has responded
    pub fn forget_failed_blocks(&mut self, blocks_to_forget: &[H256]) {
        if blocks_to_forget.is_empty() {
            return;
        }

        for block_to_forget in blocks_to_forget {
            self.chain.forget_block_with_children(block_to_forget);
        }
    }

    /// Verify and select unknown headers for scheduling
    fn verify_headers(
        &mut self,
        peer_index: PeerIndex,
        last_known_hash: H256,
        headers: &[IndexedBlockHeader],
    ) -> BlocksHeadersVerificationResult {
        // validate blocks headers before scheduling
        let mut last_known_hash = &last_known_hash;
        let mut headers_provider =
            MessageBlockHeadersProvider::new(&self.chain, self.chain.best_block_header().number);
        for (header_index, header) in headers.iter().enumerate() {
            // check that this header is direct child of previous header
            if &header.raw.previous_header_hash != last_known_hash {
                self.peers.misbehaving(peer_index, &format!("Neighbour headers in `headers` message are unlinked: Prev: {}, PrevLink: {}, Curr: {}",
					last_known_hash.to_reversed_str(), header.raw.previous_header_hash.to_reversed_str(), header.hash.to_reversed_str()));
                return BlocksHeadersVerificationResult::Skip;
            }

            // check that we do not know all blocks in range [first_unknown_index..]
            // if we know some block => there has been verification error => all headers should be ignored
            // see when_previous_block_verification_failed_fork_is_not_requested for details
            match self.chain.block_state(&header.hash) {
                BlockState::Unknown => (),
                BlockState::DeadEnd if self.config.close_connection_on_bad_block => {
                    self.peers.misbehaving(
                        peer_index,
                        &format!(
                            "Provided dead-end block {:?}",
                            header.hash.to_reversed_str()
                        ),
                    );
                    return BlocksHeadersVerificationResult::Skip;
                }
                block_state => {
                    trace!(target: "sync", "Ignoring {} headers from peer#{} - known ({:?}) header {} at the {}/{} ({}...{})",
						headers.len(), peer_index, block_state, header.hash.to_reversed_str(), header_index, headers.len(),
						headers[0].hash.to_reversed_str(), headers[headers.len() - 1].hash.to_reversed_str());
                    self.peers_tasks.useful_peer(peer_index);
                    return BlocksHeadersVerificationResult::Skip;
                }
            }

            // verify header
            if self.verify_headers {
                if let Err(error) = self.chain_verifier.verify_block_header(
                    &headers_provider,
                    &header.hash,
                    &header.raw,
                ) {
                    if self.config.close_connection_on_bad_block {
                        self.peers.misbehaving(
                            peer_index,
                            &format!(
                                "Error verifying header {} from `headers`: {:?}",
                                header.hash.to_reversed_str(),
                                error
                            ),
                        );
                    } else {
                        warn!(target: "sync", "Error verifying header {} from `headers` message: {:?}", header.hash.to_reversed_str(), error);
                    }
                    return BlocksHeadersVerificationResult::Error(header_index);
                }
            }

            last_known_hash = &header.hash;
            headers_provider.append_header(header.hash.clone(), header.clone());
        }

        BlocksHeadersVerificationResult::Success
    }

    fn prepare_blocks_requests_tasks(
        &mut self,
        limits: &BlocksRequestLimits,
        mut peers: Vec<PeerIndex>,
        mut hashes: Vec<H256>,
    ) -> Vec<Task> {
        use std::mem::swap;

        // ask fastest peers for hashes at the beginning of `hashes`
        self.peers_tasks.sort_peers_for_blocks(&mut peers);

        let chunk_size = min(
            limits.max_blocks_in_request,
            max(hashes.len() as BlockHeight, limits.min_blocks_in_request),
        );
        let last_peer_index = peers.len() - 1;
        let mut tasks: Vec<Task> = Vec::new();
        let inv_type = InventoryType::MessageBlock;
        for (peer_index, peer) in peers.into_iter().enumerate() {
            // we have to request all blocks => we will request last peer for all remaining blocks
            let peer_chunk_size = if peer_index == last_peer_index {
                hashes.len()
            } else {
                min(hashes.len(), chunk_size as usize)
            };
            if peer_chunk_size == 0 {
                break;
            }

            let mut chunk_hashes = hashes.split_off(peer_chunk_size);
            swap(&mut chunk_hashes, &mut hashes);

            // remember that peer is asked for these blocks
            self.peers_tasks.on_blocks_requested(peer, &chunk_hashes);

            // request blocks. If block is believed to have witness - ask for witness
            let getdata = types::GetData {
                inventory: chunk_hashes
                    .into_iter()
                    .map(|h| InventoryVector {
                        inv_type: inv_type,
                        hash: h,
                    })
                    .collect(),
            };
            tasks.push(Task::GetData(peer, getdata));
        }

        assert_eq!(hashes.len(), 0);
        tasks
    }

    /// Switch to synchronization state
    fn switch_to_synchronization_state(&mut self) {
        if self.state.is_synchronizing() {
            return;
        }

        if let Some(ref listener) = self.listener {
            listener.synchronization_state_switched(true);
        }

        self.shared_state.update_synchronizing(true);
        self.state = State::Synchronizing(precise_time_s(), self.chain.best_storage_block().number);
    }

    /// Switch to nearly saturated state
    fn switch_to_nearly_saturated_state(&mut self) {
        if self.state.is_nearly_saturated() {
            return;
        }

        if let Some(ref listener) = self.listener {
            listener.synchronization_state_switched(false);
        }

        self.shared_state.update_synchronizing(false);
        self.state = State::NearlySaturated;
    }

    /// Switch to saturated state
    fn switch_to_saturated_state(&mut self) {
        if self.state.is_saturated() {
            return;
        }

        if let Some(ref listener) = self.listener {
            listener.synchronization_state_switched(false);
        }

        self.shared_state.update_synchronizing(false);
        self.state = State::Saturated;
        self.peers_tasks.reset();

        // remove sync orphans, but leave unknown orphans until they'll be removed by management thread
        let removed_orphans = self.orphaned_blocks_pool.remove_known_blocks();

        // leave currently verifying blocks
        {
            self.chain.forget_blocks(&removed_orphans);
            self.chain
                .forget_all_blocks_with_state(BlockState::Requested);
            self.chain
                .forget_all_blocks_with_state(BlockState::Scheduled);

            info!(target: "sync", "Switched to saturated state.\tChain: {:?}",
				self.chain.information());
        }

        // finally - ask all known peers for their best blocks inventory, in case if some peer
        // has lead us to the fork
        {
            let block_locator_hashes: Vec<H256> = self.chain.block_locator_hashes();
            for peer in self.peers_tasks.all_peers() {
                self.executor.execute(Task::GetHeaders(
                    *peer,
                    types::GetHeaders::with_block_locator_hashes(block_locator_hashes.clone()),
                ));
            }
        }
    }

    fn on_block_verification_success(
        &mut self,
        block: IndexedBlock,
    ) -> Option<Vec<VerificationTask>> {
        // update block processing speed
        self.block_speed_meter.checkpoint();

        // remove flags
        let needs_relay = !self.do_not_relay.remove(block.hash());

        let block_hash = block.hash().clone();
        // insert block to the storage
        match {
            // remove block from verification queue
            // header is removed in `insert_best_block` call
            // or it is removed earlier, when block was removed from the verifying queue
            if self
                .chain
                .forget_block_with_state_leave_header(block.hash(), BlockState::Verifying)
                != HashPosition::Missing
            {
                // block was in verification queue => insert to storage
                self.chain.insert_best_block(block)
            } else {
                Ok(BlockInsertionResult::default())
            }
        } {
            Ok(insert_result) => {
                // update shared state
                self.shared_state
                    .update_best_storage_block_height(self.chain.best_storage_block().number);

                // notify listener
                if let Some(best_block_hash) = insert_result.canonized_blocks_hashes.last() {
                    if let Some(ref listener) = self.listener {
                        listener.best_storage_block_inserted(best_block_hash);
                    }
                }

                // awake threads, waiting for this block insertion
                self.awake_waiting_threads(&block_hash);

                // continue with synchronization
                self.execute_synchronization_tasks(None, None);

                // relay block to our peers
                if needs_relay && (self.state.is_saturated() || self.state.is_nearly_saturated()) {
                    for block_hash in insert_result.canonized_blocks_hashes {
                        if let Some(block) = self.chain.storage().block(block_hash.into()) {
                            self.executor.execute(Task::RelayNewBlock(block));
                        }
                    }
                }

                // deal with block transactions
                let verification_tasks: Vec<VerificationTask> = Vec::with_capacity(0);
                // Vec::with_capacity(insert_result.transactions_to_reverify.len());
                Some(verification_tasks)
            }
            Err(e) => {
                // process as irrecoverable failure
                panic!(
                    "Block {} insertion failed with error {:?}",
                    block_hash.to_reversed_str(),
                    e
                );
            }
        }
    }

    fn on_block_verification_error(&mut self, err: &str, hash: &H256) {
        warn!(target: "sync", "Block {:?} verification failed with error {:?}", hash.to_reversed_str(), err);

        // remove flags
        self.do_not_relay.remove(hash);

        // close connection with this peer
        if let Some(peer_index) = self.verifying_blocks_by_peer.get(hash) {
            if self.config.close_connection_on_bad_block {
                self.peers.dos(
                    *peer_index,
                    &format!("Provided wrong block {}", hash.to_reversed_str()),
                )
            } else {
                warn!(target: "sync", "Peer#{} has provided wrong block {:?}", peer_index, hash.to_reversed_str());
            }
        }

        // forget for this block and all its children
        // headers are also removed as they all are invalid
        self.chain.forget_block_with_children(hash);

        // mark failed block as dead end (this branch won't be synchronized)
        self.chain.mark_dead_end_block(hash);

        // awake threads, waiting for this block insertion
        self.awake_waiting_threads(hash);

        // start new tasks
        self.execute_synchronization_tasks(None, None);
    }

    /// Execute futures, which were waiting for this block verification
    fn awake_waiting_threads(&mut self, hash: &H256) {
        // find a peer, which has supplied us with this block
        if let Entry::Occupied(block_entry) = self.verifying_blocks_by_peer.entry(hash.clone()) {
            let peer_index = *block_entry.get();
            // find a # of blocks, which this thread has supplied
            if let Entry::Occupied(mut entry) = self.verifying_blocks_futures.entry(peer_index) {
                let is_last_block = {
                    let &mut (ref mut waiting, ref mut futures) = entry.get_mut();
                    waiting.remove(hash);
                    // if this is the last block => awake waiting threads
                    let is_last_block = waiting.is_empty();
                    if is_last_block {
                        for future in futures.drain(..) {
                            future.wait().expect("no-error future");
                        }
                    }
                    is_last_block
                };

                if is_last_block {
                    entry.remove_entry();
                }
            }
            block_entry.remove_entry();
        }
    }
}

impl Default for BlocksRequestLimits {
    fn default() -> Self {
        BlocksRequestLimits {
            max_scheduled_hashes: MAX_SCHEDULED_HASHES,
            max_requested_blocks: MAX_REQUESTED_BLOCKS,
            max_verifying_blocks: MAX_VERIFYING_BLOCKS,
            min_blocks_in_request: MIN_BLOCKS_IN_REQUEST,
            max_blocks_in_request: MAX_BLOCKS_IN_REQUEST,
        }
    }
}

#[cfg(test)]
pub mod tests {
    extern crate test_data;

    use super::super::SyncListener;
    use super::{ClientCore, Config, CoreVerificationSink, SynchronizationClientCore};
    use chain::Block;
    use db::BlockChainDatabase;
    use inbound_connection::tests::DummyOutboundSyncConnection;
    use message::common::InventoryVector;
    use message::{types, Services};
    use network::Network;
    use parking_lot::Mutex;
    use primitives::hash::H256;
    use std::sync::Arc;
    use synchronization_chain::Chain;
    use synchronization_client::{Client, SynchronizationClient};
    use synchronization_executor::tests::DummyTaskExecutor;
    use synchronization_executor::Task;
    use synchronization_peers::PeersImpl;
    use synchronization_verifier::tests::DummyVerifier;
    use types::{ClientCoreRef, PeerIndex, StorageRef, SynchronizationStateRef};
    use utils::SynchronizationState;
    use verification::BackwardsCompatibleChainVerifier as ChainVerifier;

    #[derive(Default)]
    struct DummySyncListenerData {
        pub is_synchronizing: bool,
        pub best_blocks: Vec<H256>,
    }

    struct DummySyncListener {
        data: Arc<Mutex<DummySyncListenerData>>,
    }

    impl DummySyncListener {
        pub fn new(data: Arc<Mutex<DummySyncListenerData>>) -> Self {
            DummySyncListener { data: data }
        }
    }

    impl SyncListener for DummySyncListener {
        fn synchronization_state_switched(&self, is_synchronizing: bool) {
            self.data.lock().is_synchronizing = is_synchronizing;
        }

        fn best_storage_block_inserted(&self, block_hash: &H256) {
            self.data.lock().best_blocks.push(block_hash.clone());
        }
    }

    fn create_sync(
        storage: Option<StorageRef>,
        verifier: Option<DummyVerifier>,
    ) -> (
        Arc<DummyTaskExecutor>,
        ClientCoreRef<SynchronizationClientCore<DummyTaskExecutor>>,
        Arc<SynchronizationClient<DummyTaskExecutor, DummyVerifier>>,
    ) {
        let sync_peers = Arc::new(PeersImpl::default());
        let storage = match storage {
            Some(storage) => storage,
            None => Arc::new(BlockChainDatabase::init_test_chain(vec![
                test_data::genesis().into(),
            ])),
        };
        let sync_state =
            SynchronizationStateRef::new(SynchronizationState::with_storage(storage.clone()));
        let chain = Chain::new(storage.clone());
        let executor = DummyTaskExecutor::new();
        let config = Config {
            close_connection_on_bad_block: true,
        };

        let chain_verifier = Arc::new(ChainVerifier::new(storage.clone(), Network::Unitest));
        let client_core = SynchronizationClientCore::new(
            config,
            sync_state.clone(),
            sync_peers.clone(),
            executor.clone(),
            chain,
            chain_verifier.clone(),
        );
        {
            client_core.lock().set_verify_headers(false);
        }
        let mut verifier = verifier.unwrap_or_default();
        verifier.set_sink(Arc::new(CoreVerificationSink::new(client_core.clone())));
        verifier.set_storage(storage);
        verifier.set_verifier(chain_verifier);

        let client = SynchronizationClient::new(client_core.clone(), verifier);
        (executor, client_core, client)
    }

    fn request_block_headers_genesis(peer_index: PeerIndex) -> Task {
        Task::GetHeaders(
            peer_index,
            types::GetHeaders::with_block_locator_hashes(vec![test_data::genesis().hash()]),
        )
    }

    fn request_block_headers_genesis_and(peer_index: PeerIndex, mut hashes: Vec<H256>) -> Task {
        hashes.push(test_data::genesis().hash());
        Task::GetHeaders(
            peer_index,
            types::GetHeaders::with_block_locator_hashes(hashes),
        )
    }

    fn request_blocks(peer_index: PeerIndex, hashes: Vec<H256>) -> Task {
        Task::GetData(
            peer_index,
            types::GetData {
                inventory: hashes.into_iter().map(InventoryVector::block).collect(),
            },
        )
    }

    #[test]
    fn synchronization_request_inventory_on_sync_start() {
        let (executor, _, sync) = create_sync(None, None);
        // start sync session
        sync.on_connect(0);
        // => ask for inventory
        let tasks = executor.take_tasks();
        assert_eq!(tasks, vec![request_block_headers_genesis(0)]);
    }

    #[test]
    fn synchronization_saturated_on_start() {
        let (_, core, _) = create_sync(None, None);
        let info = core.lock().information();
        assert!(!info.state.is_synchronizing());
        assert_eq!(info.orphaned_blocks, 0);
    }

    #[test]
    fn synchronization_in_order_block_path_nearly_saturated() {
        let (executor, core, sync) = create_sync(None, None);

        let block1: Block = test_data::block_h1();
        let block2: Block = test_data::block_h2();

        sync.on_headers(5, vec![block1.block_header.clone().into()]);
        let tasks = executor.take_tasks();
        assert_eq!(
            tasks,
            vec![
                request_block_headers_genesis_and(5, vec![block1.hash()]),
                request_blocks(5, vec![block1.hash()])
            ]
        );
        assert!(core.lock().information().state.is_nearly_saturated());
        assert_eq!(core.lock().information().orphaned_blocks, 0);
        assert_eq!(core.lock().information().chain.scheduled, 0);
        assert_eq!(core.lock().information().chain.requested, 1);
        assert_eq!(core.lock().information().chain.stored, 1);
        assert_eq!(core.lock().information().peers_tasks.idle, 0);
        assert_eq!(core.lock().information().peers_tasks.active, 1);

        // push unknown block => will be queued as orphan
        sync.on_block(5, block2.into());
        assert!(core.lock().information().state.is_nearly_saturated());
        assert_eq!(core.lock().information().orphaned_blocks, 1);
        assert_eq!(core.lock().information().chain.scheduled, 0);
        assert_eq!(core.lock().information().chain.requested, 1);
        assert_eq!(core.lock().information().chain.stored, 1);
        assert_eq!(core.lock().information().peers_tasks.idle, 0);
        assert_eq!(core.lock().information().peers_tasks.active, 1);

        // push requested block => should be moved to the test storage && orphan should be moved
        sync.on_block(5, block1.into());
        assert!(core.lock().information().state.is_saturated());
        assert_eq!(core.lock().information().orphaned_blocks, 0);
        assert_eq!(core.lock().information().chain.scheduled, 0);
        assert_eq!(core.lock().information().chain.requested, 0);
        assert_eq!(core.lock().information().chain.stored, 3);
        // we have just requested new `inventory` from the peer => peer is forgotten
        assert_eq!(core.lock().information().peers_tasks.idle, 0);
        assert_eq!(core.lock().information().peers_tasks.active, 0);
    }

    #[test]
    fn synchronization_out_of_order_block_path() {
        let (_, core, sync) = create_sync(None, None);

        sync.on_headers(
            5,
            vec![
                test_data::block_h1().block_header.into(),
                test_data::block_h2().block_header.into(),
            ],
        );
        sync.on_block(5, test_data::block_h169().into());

        // out-of-order block was presented by the peer
        assert!(core.lock().information().state.is_synchronizing());
        assert_eq!(core.lock().information().orphaned_blocks, 0);
        assert_eq!(core.lock().information().chain.scheduled, 0);
        assert_eq!(core.lock().information().chain.requested, 2);
        assert_eq!(core.lock().information().chain.stored, 1);
        // we have just requested new `inventory` from the peer => peer is forgotten
        assert_eq!(core.lock().information().peers_tasks.idle, 0);
        assert_eq!(core.lock().information().peers_tasks.active, 1);
        // TODO: check that peer is penalized
    }

    #[test]
    fn synchronization_parallel_peers() {
        let (executor, core, sync) = create_sync(None, None);

        let block1: Block = test_data::block_h1();
        let block2: Block = test_data::block_h2();

        {
            // not synchronizing after start
            assert!(core.lock().information().state.is_saturated());
            // receive inventory from new peer#1
            sync.on_headers(1, vec![block1.block_header.clone().into()]);
            assert_eq!(core.lock().information().chain.requested, 1);
            // synchronization has started && new blocks have been requested
            let tasks = executor.take_tasks();
            assert!(core.lock().information().state.is_nearly_saturated());
            assert_eq!(
                tasks,
                vec![
                    request_block_headers_genesis_and(1, vec![block1.hash()]),
                    request_blocks(1, vec![block1.hash()])
                ]
            );
        }

        {
            // receive inventory from new peer#2
            sync.on_headers(
                2,
                vec![
                    block1.block_header.clone().into(),
                    block2.block_header.clone().into(),
                ],
            );
            assert_eq!(core.lock().information().chain.requested, 2);
            // synchronization has started && new blocks have been requested
            let tasks = executor.take_tasks();
            assert!(core.lock().information().state.is_synchronizing());
            assert_eq!(
                tasks,
                vec![
                    request_block_headers_genesis_and(2, vec![block2.hash(), block1.hash()]),
                    request_blocks(2, vec![block2.hash()])
                ]
            );
        }

        {
            // receive block from peer#2
            sync.on_block(2, block2.into());
            let information = core.lock().information();
            assert!(information.chain.requested == 2 && information.orphaned_blocks == 1);
            // receive block from peer#1
            sync.on_block(1, block1.into());

            let information = core.lock().information();
            assert!(
                information.chain.requested == 0
                    && information.orphaned_blocks == 0
                    && information.chain.stored == 3
            );
        }
    }

    #[test]
    fn synchronization_reset_when_peer_is_disconnected() {
        let (_, core, sync) = create_sync(None, None);

        // request new blocks
        {
            sync.on_headers(1, vec![test_data::block_h1().block_header.into()]);
            assert!(core.lock().information().state.is_nearly_saturated());
        }

        // lost connection to peer => synchronization state lost
        {
            sync.on_disconnect(1);
            assert!(core.lock().information().state.is_saturated());
        }
    }

    #[test]
    fn synchronization_not_starting_when_receiving_known_blocks() {
        let (executor, core, sync) = create_sync(None, None);
        // saturated => receive inventory with known blocks only
        sync.on_headers(1, vec![test_data::genesis().block_header.into()]);
        // => no need to start synchronization
        assert!(!core.lock().information().state.is_nearly_saturated());
        // => no synchronization tasks are scheduled
        let tasks = executor.take_tasks();
        assert_eq!(tasks, vec![]);
    }

    #[test]
    fn synchronization_asks_for_inventory_after_saturating() {
        let (executor, _, sync) = create_sync(None, None);
        let block = test_data::block_h1();
        sync.on_headers(1, vec![block.block_header.clone().into()]);
        sync.on_headers(2, vec![block.block_header.clone().into()]);
        executor.take_tasks();
        sync.on_block(2, block.clone().into());

        let tasks = executor.take_tasks();
        assert_eq!(tasks.len(), 4);
        // TODO: when saturating, RequestBlocksHeaders is sent twice to the peer who has supplied last block:
        // 1) from on_block_verification_success
        // 2) from switch_to_saturated_state
        assert!(tasks
            .iter()
            .any(|t| t == &request_block_headers_genesis_and(1, vec![block.hash()])));
        assert!(tasks
            .iter()
            .any(|t| t == &request_block_headers_genesis_and(2, vec![block.hash()])));
        assert!(tasks
            .iter()
            .any(|t| t == &Task::RelayNewBlock(block.clone().into())));
    }

    #[test]
    fn synchronization_remembers_correct_block_headers_in_order() {
        let (executor, core, sync) = create_sync(None, None);

        let b1 = test_data::block_h1();
        let b2 = test_data::block_h2();
        sync.on_headers(
            1,
            vec![
                b1.block_header.clone().into(),
                b2.block_header.clone().into(),
            ],
        );

        let tasks = executor.take_tasks();
        assert_eq!(tasks.len(), 2);
        assert!(tasks
            .iter()
            .any(|t| t == &request_block_headers_genesis_and(1, vec![b2.hash(), b1.hash()])));
        assert!(tasks
            .iter()
            .any(|t| t == &request_blocks(1, vec![b1.hash(), b2.hash()])));

        {
            let mut core = core.lock();
            let chain = core.chain();
            assert_eq!(chain.information().headers.best, 2);
            assert_eq!(chain.information().headers.total, 2);
        }

        sync.on_block(1, b1.clone().into());

        let tasks = executor.take_tasks();
        assert_eq!(tasks, vec![]);

        {
            let mut core = core.lock();
            let chain = core.chain();
            assert_eq!(chain.information().headers.best, 1);
            assert_eq!(chain.information().headers.total, 1);
        }

        sync.on_block(1, b2.clone().into());

        let tasks = executor.take_tasks();
        assert_eq!(
            tasks,
            vec![request_block_headers_genesis_and(
                1,
                vec![b2.hash(), b1.hash()]
            )]
        );

        {
            let mut core = core.lock();
            let chain = core.chain();
            assert_eq!(chain.information().headers.best, 0);
            assert_eq!(chain.information().headers.total, 0);
        }
    }

    #[test]
    fn synchronization_remembers_correct_block_headers_out_of_order() {
        let (executor, core, sync) = create_sync(None, None);

        let b1 = test_data::block_h1();
        let b2 = test_data::block_h2();
        sync.on_headers(
            1,
            vec![
                b1.block_header.clone().into(),
                b2.block_header.clone().into(),
            ],
        );

        let tasks = executor.take_tasks();
        assert_eq!(tasks.len(), 2);
        assert!(tasks
            .iter()
            .any(|t| t == &request_block_headers_genesis_and(1, vec![b2.hash(), b1.hash()])));
        assert!(tasks
            .iter()
            .any(|t| t == &request_blocks(1, vec![b1.hash(), b2.hash()])));

        {
            let mut core = core.lock();
            let chain = core.chain();
            assert_eq!(chain.information().headers.best, 2);
            assert_eq!(chain.information().headers.total, 2);
        }

        sync.on_block(1, b2.clone().into());

        let tasks = executor.take_tasks();
        assert_eq!(tasks, vec![]);

        {
            let mut core = core.lock();
            let chain = core.chain();
            assert_eq!(chain.information().headers.best, 2);
            assert_eq!(chain.information().headers.total, 2);
        }

        sync.on_block(1, b1.clone().into());

        let tasks = executor.take_tasks();
        assert_eq!(
            tasks,
            vec![request_block_headers_genesis_and(
                1,
                vec![b2.hash(), b1.hash()]
            ),]
        );

        {
            let mut core = core.lock();
            let chain = core.chain();
            assert_eq!(chain.information().headers.best, 0);
            assert_eq!(chain.information().headers.total, 0);
        }
    }

    #[test]
    fn synchronization_ignores_unknown_block_headers() {
        let (executor, core, sync) = create_sync(None, None);

        let b169 = test_data::block_h169();
        sync.on_headers(1, vec![b169.block_header.into()]);

        let tasks = executor.take_tasks();
        assert_eq!(tasks, vec![]);

        let mut core = core.lock();
        let chain = core.chain();
        assert_eq!(chain.information().headers.best, 0);
        assert_eq!(chain.information().headers.total, 0);
    }

    #[test]
    fn synchronization_works_for_forks_from_db_best_block() {
        let genesis = test_data::genesis();
        let storage = Arc::new(BlockChainDatabase::init_test_chain(vec![
            test_data::genesis().into(),
        ]));

        let (executor, core, sync) = create_sync(Some(storage), None);
        let genesis_header = &genesis.block_header;
        let fork1 = test_data::build_n_empty_blocks_from(2, 100, &genesis_header);
        let fork2 = test_data::build_n_empty_blocks_from(3, 200, &genesis_header);

        sync.on_headers(
            1,
            vec![
                fork1[0].block_header.clone().into(),
                fork1[1].block_header.clone().into(),
            ],
        );
        sync.on_headers(
            2,
            vec![
                fork2[0].block_header.clone().into(),
                fork2[1].block_header.clone().into(),
                fork2[2].block_header.clone().into(),
            ],
        );
        let tasks = { executor.take_tasks() };
        assert_eq!(
            tasks,
            vec![
                request_block_headers_genesis_and(1, vec![fork1[1].hash(), fork1[0].hash()]),
                request_blocks(1, vec![fork1[0].hash(), fork1[1].hash()]),
                // this is possibly wrong, because we have mixed two forks, but this works because we ask for headers on saturating
                request_block_headers_genesis_and(
                    2,
                    vec![
                        fork2[2].hash(),
                        fork2[1].hash(),
                        fork2[0].hash(),
                        fork1[1].hash(),
                        fork1[0].hash()
                    ]
                ),
                request_blocks(2, vec![fork2[0].hash(), fork2[1].hash(), fork2[2].hash()]),
            ]
        );

        sync.on_block(2, fork2[0].clone().into());
        {
            let mut core = core.lock();
            let chain = core.chain();
            assert_eq!(chain.best_storage_block().hash, fork2[0].hash());
            assert_eq!(chain.best_storage_block().number, 1);
        }

        sync.on_block(1, fork1[0].clone().into());
        {
            let mut core = core.lock();
            let chain = core.chain();
            assert_eq!(chain.best_storage_block().hash, fork2[0].hash());
            assert_eq!(chain.best_storage_block().number, 1);
        }

        sync.on_block(1, fork1[1].clone().into());
        {
            let mut core = core.lock();
            let chain = core.chain();
            assert_eq!(chain.best_storage_block().hash, fork1[1].hash());
            assert_eq!(chain.best_storage_block().number, 2);
        }

        sync.on_block(2, fork2[1].clone().into());
        {
            let mut core = core.lock();
            let chain = core.chain();
            assert_eq!(chain.best_storage_block().hash, fork1[1].hash());
            assert_eq!(chain.best_storage_block().number, 2);
        }

        sync.on_block(2, fork2[2].clone().into());
        {
            let mut core = core.lock();
            let chain = core.chain();
            assert_eq!(chain.best_storage_block().hash, fork2[2].hash());
            assert_eq!(chain.best_storage_block().number, 3);
        }
    }

    #[test]
    fn synchronization_works_for_forks_long_after_short() {
        let genesis = test_data::genesis();
        let storage = Arc::new(BlockChainDatabase::init_test_chain(vec![
            test_data::genesis().into(),
        ]));

        let (executor, core, sync) = create_sync(Some(storage), None);
        let common_block = test_data::block_builder()
            .header()
            .parent(genesis.hash())
            .build()
            .build();
        let fork1 = test_data::build_n_empty_blocks_from(2, 100, &common_block.block_header);
        let fork2 = test_data::build_n_empty_blocks_from(3, 200, &common_block.block_header);

        sync.on_headers(
            1,
            vec![
                common_block.block_header.clone().into(),
                fork1[0].block_header.clone().into(),
                fork1[1].block_header.clone().into(),
            ],
        );
        sync.on_headers(
            2,
            vec![
                common_block.block_header.clone().into(),
                fork2[0].block_header.clone().into(),
                fork2[1].block_header.clone().into(),
                fork2[2].block_header.clone().into(),
            ],
        );

        let tasks = { executor.take_tasks() };
        assert_eq!(
            tasks,
            vec![
                request_block_headers_genesis_and(
                    1,
                    vec![fork1[1].hash(), fork1[0].hash(), common_block.hash()]
                ),
                request_blocks(
                    1,
                    vec![common_block.hash(), fork1[0].hash(), fork1[1].hash()]
                ),
                request_block_headers_genesis_and(
                    2,
                    vec![
                        fork2[2].hash(),
                        fork2[1].hash(),
                        fork2[0].hash(),
                        fork1[1].hash(),
                        fork1[0].hash(),
                        common_block.hash()
                    ]
                ),
                request_blocks(2, vec![fork2[0].hash(), fork2[1].hash(), fork2[2].hash()]),
            ]
        );

        // TODO: this will change from 3 to 4 after longest fork will be stored in the BestHeadersChain
        // however id doesn't affect sync process, as it is shown below
        {
            let mut core = core.lock();
            let chain = core.chain();
            assert_eq!(chain.information().headers.best, 3);
            assert_eq!(chain.information().headers.total, 3);
        }

        sync.on_block(1, common_block.clone().into());
        sync.on_block(1, fork1[0].clone().into());
        sync.on_block(1, fork1[1].clone().into());
        sync.on_block(2, fork2[0].clone().into());
        sync.on_block(2, fork2[1].clone().into());
        sync.on_block(2, fork2[2].clone().into());

        {
            let mut core = core.lock();
            let chain = core.chain();
            assert_eq!(chain.best_storage_block().hash, fork2[2].hash());
            assert_eq!(chain.best_storage_block().number, 4);
        }
    }

    #[test]
    fn accept_out_of_order_blocks_when_saturated() {
        let (_, core, sync) = create_sync(None, None);

        sync.on_block(1, test_data::block_h2().into());
        assert_eq!(core.lock().information().orphaned_blocks, 1);

        {
            let mut core = core.lock();
            let chain = core.chain();
            assert_eq!(chain.best_storage_block().number, 0);
        }

        sync.on_block(1, test_data::block_h1().into());
        assert_eq!(core.lock().information().orphaned_blocks, 0);

        {
            let mut core = core.lock();
            let chain = core.chain();
            assert_eq!(chain.best_storage_block().number, 2);
        }
    }

    #[test]
    fn do_not_rerequest_unknown_block_in_inventory() {
        let (executor, _, sync) = create_sync(None, None);

        sync.on_block(1, test_data::block_h2().into());
        sync.on_inventory(
            1,
            types::Inv::with_inventory(vec![
                InventoryVector::block(test_data::block_h1().hash()),
                InventoryVector::block(test_data::block_h2().hash()),
            ]),
        );

        let tasks = executor.take_tasks();
        assert_eq!(
            tasks,
            vec![Task::GetData(
                1,
                types::GetData::with_inventory(vec![InventoryVector::block(
                    test_data::block_h1().hash()
                )])
            )]
        );
    }

    #[test]
    fn blocks_rerequested_on_peer_disconnect() {
        let (executor, _, sync) = create_sync(None, None);

        let block1: Block = test_data::block_h1();
        let block2: Block = test_data::block_h2();

        {
            // receive inventory from new peer#1
            sync.on_headers(1, vec![block1.block_header.clone().into()]);
            // synchronization has started && new blocks have been requested
            let tasks = executor.take_tasks();
            assert_eq!(
                tasks,
                vec![
                    request_block_headers_genesis_and(1, vec![block1.hash().clone()]),
                    request_blocks(1, vec![block1.hash()])
                ]
            );
        }

        {
            // receive inventory from new peer#2
            sync.on_headers(
                2,
                vec![
                    block1.block_header.clone().into(),
                    block2.block_header.clone().into(),
                ],
            );
            // synchronization has started && new blocks have been requested
            let tasks = executor.take_tasks();
            assert_eq!(
                tasks,
                vec![
                    request_block_headers_genesis_and(
                        2,
                        vec![block2.hash().clone(), block1.hash().clone()]
                    ),
                    request_blocks(2, vec![block2.hash()])
                ]
            );
        }

        {
            // peer#1 is disconnected && it has pending blocks requests => ask peer#2
            sync.on_disconnect(1);
            // blocks have been requested
            let tasks = executor.take_tasks();
            assert_eq!(tasks, vec![request_blocks(2, vec![block1.hash()])]);
        }
    }

    #[test]
    fn sync_after_db_insert_nonfatal_fail() {
        let block = test_data::block_h2();
        let storage = BlockChainDatabase::init_test_chain(vec![test_data::genesis().into()]);
        assert!(storage.insert(test_data::block_h2().into()).is_err());
        let best_genesis = storage.best_block();

        let (_, core, sync) = create_sync(Some(Arc::new(storage)), None);

        sync.on_block(1, block.into());

        let mut core = core.lock();
        let chain = core.chain();
        assert_eq!(chain.best_block(), best_genesis);
    }

    #[test]
    fn peer_removed_from_sync_after_responding_with_requested_block_notfound() {
        let (executor, core, sync) = create_sync(None, None);

        let b1 = test_data::block_h1();
        let b2 = test_data::block_h2();
        sync.on_headers(
            1,
            vec![
                b1.block_header.clone().into(),
                b2.block_header.clone().into(),
            ],
        );

        let tasks = executor.take_tasks();
        assert_eq!(
            tasks,
            vec![
                request_block_headers_genesis_and(1, vec![b2.hash().clone(), b1.hash().clone()]),
                request_blocks(1, vec![b1.hash(), b2.hash()])
            ]
        );

        assert_eq!(core.lock().information().peers_tasks.idle, 0);
        assert_eq!(core.lock().information().peers_tasks.unuseful, 0);
        assert_eq!(core.lock().information().peers_tasks.active, 1);

        sync.on_notfound(
            1,
            types::NotFound::with_inventory(vec![InventoryVector::block(b1.hash())]),
        );

        let tasks = executor.take_tasks();
        assert_eq!(tasks, vec![request_block_headers_genesis(1)]);

        assert_eq!(core.lock().information().peers_tasks.idle, 0);
        assert_eq!(core.lock().information().peers_tasks.unuseful, 1);
        assert_eq!(core.lock().information().peers_tasks.active, 0);
    }

    #[test]
    fn peer_not_removed_from_sync_after_responding_with_non_requested_block_notfound() {
        let (executor, core, sync) = create_sync(None, None);

        let b1 = test_data::block_h1();
        let b2 = test_data::block_h2();
        sync.on_headers(
            1,
            vec![
                b1.block_header.clone().into(),
                b2.block_header.clone().into(),
            ],
        );

        let tasks = executor.take_tasks();
        assert_eq!(
            tasks,
            vec![
                request_block_headers_genesis_and(1, vec![b2.hash().clone(), b1.hash().clone()]),
                request_blocks(1, vec![b1.hash(), b2.hash()])
            ]
        );

        assert_eq!(core.lock().information().peers_tasks.idle, 0);
        assert_eq!(core.lock().information().peers_tasks.unuseful, 0);
        assert_eq!(core.lock().information().peers_tasks.active, 1);

        sync.on_notfound(
            1,
            types::NotFound::with_inventory(vec![InventoryVector::block(
                test_data::block_h170().hash(),
            )]),
        );

        let tasks = executor.take_tasks();
        assert_eq!(tasks, vec![]);

        assert_eq!(core.lock().information().peers_tasks.idle, 0);
        assert_eq!(core.lock().information().peers_tasks.unuseful, 0);
        assert_eq!(core.lock().information().peers_tasks.active, 1);
    }

    #[test]
    // https://github.com/ethcore/parity-bitcoin/issues/121
    fn when_previous_block_verification_failed_fork_is_not_requested() {
        // got headers [b10, b11, b12] - some fork
        // got headers [b10, b21, b22] - main branch
        // got b10, b11, b12, b21. b22 is requested
        //
        // verifying: [b10, b11, b12, b21]
        // headers_chain: [b10, b11, b12]
        //
        // b21 verification failed => b22 is not removed (since it is not in headers_chain)
        // got new headers [b10, b21, b22, b23] => intersection point is b10 => scheduling [b21, b22, b23]
        //
        // block queue is empty => new tasks => requesting [b21, b22] => panic in hash_queue
        //
        // => do not trust first intersection point - check each hash when scheduling hashes.
        // If at least one hash is known => previous verification failed => drop all headers.

        let genesis = test_data::genesis();
        let b10 = test_data::block_builder()
            .header()
            .parent(genesis.hash())
            .build()
            .build();

        let b11 = test_data::block_builder()
            .header()
            .iterations(1)
            .parent(b10.hash())
            .build()
            .build();
        let b12 = test_data::block_builder()
            .header()
            .parent(b11.hash())
            .build()
            .build();

        let b21 = test_data::block_builder()
            .header()
            .iterations(2)
            .parent(b10.hash())
            .build()
            .build();
        let b22 = test_data::block_builder()
            .header()
            .parent(b21.hash())
            .build()
            .build();
        let b23 = test_data::block_builder()
            .header()
            .parent(b22.hash())
            .build()
            .build();

        // simulate verification during b21 verification
        let mut dummy_verifier = DummyVerifier::default();
        dummy_verifier.error_when_verifying(b21.hash(), "simulated");

        let (_, _, sync) = create_sync(None, Some(dummy_verifier));

        sync.on_headers(
            1,
            vec![
                b10.block_header.clone().into(),
                b11.block_header.clone().into(),
                b12.block_header.clone().into(),
            ],
        );
        sync.on_headers(
            2,
            vec![
                b10.block_header.clone().into(),
                b21.block_header.clone().into(),
                b22.block_header.clone().into(),
            ],
        );

        sync.on_block(1, b10.clone().into());
        sync.on_block(1, b11.into());
        sync.on_block(1, b12.into());

        sync.on_block(2, b21.clone().into());

        // should not panic here
        sync.on_headers(
            2,
            vec![
                b10.block_header.into(),
                b21.block_header.into(),
                b22.block_header.into(),
                b23.block_header.into(),
            ],
        );
    }

    #[test]
    fn relay_new_block_when_in_saturated_state() {
        let (executor, _, sync) = create_sync(None, None);
        let genesis = test_data::genesis();
        let b0 = test_data::block_builder()
            .header()
            .parent(genesis.hash())
            .build()
            .build();
        let b1 = test_data::block_builder()
            .header()
            .parent(b0.hash())
            .build()
            .build();
        let b2 = test_data::block_builder()
            .header()
            .parent(b1.hash())
            .build()
            .build();
        let b3 = test_data::block_builder()
            .header()
            .parent(b2.hash())
            .build()
            .build();

        sync.on_headers(
            1,
            vec![
                b0.block_header.clone().into(),
                b1.block_header.clone().into(),
            ],
        );
        sync.on_block(1, b0.clone().into());
        sync.on_block(1, b1.clone().into());

        // we were in synchronization state => block is not relayed
        {
            let tasks = executor.take_tasks();
            assert_eq!(
                tasks,
                vec![
                    request_block_headers_genesis_and(1, vec![b1.hash(), b0.hash()]),
                    request_blocks(1, vec![b0.hash(), b1.hash()]),
                    request_block_headers_genesis_and(1, vec![b1.hash(), b0.hash()]),
                ]
            );
        }

        sync.on_block(2, b2.clone().into());

        // we were in saturated state => block is relayed
        {
            let tasks = executor.take_tasks();
            assert_eq!(
                tasks,
                vec![
                    request_block_headers_genesis_and(2, vec![b2.hash(), b1.hash(), b0.hash()]),
                    Task::RelayNewBlock(b2.clone().into())
                ]
            );
        }

        sync.on_headers(1, vec![b3.block_header.clone().into()]);
        sync.on_block(1, b3.clone().into());

        // we were in nearly saturated state => block is relayed
        {
            let tasks = executor.take_tasks();
            assert!(tasks
                .iter()
                .any(|t| t == &Task::RelayNewBlock(b3.clone().into())));
        }
    }

    #[test]
    fn receive_same_unknown_block_twice() {
        let (_, _, sync) = create_sync(None, None);

        sync.on_block(1, test_data::block_h2().into());
        // should not panic here
        sync.on_block(2, test_data::block_h2().into());
    }

    #[test]
    fn collection_closed_on_block_verification_error() {
        let genesis = test_data::genesis();
        let b0 = test_data::block_builder()
            .header()
            .parent(genesis.hash())
            .build()
            .build();

        // simulate verification error during b0 verification
        let mut dummy_verifier = DummyVerifier::default();
        dummy_verifier.error_when_verifying(b0.hash(), "simulated");

        let (_, core, sync) = create_sync(None, Some(dummy_verifier));

        core.lock()
            .peers
            .insert(0, Services::default(), DummyOutboundSyncConnection::new());
        assert!(core.lock().peers.enumerate().contains(&0));

        sync.on_block(0, b0.into());

        assert!(!core.lock().peers.enumerate().contains(&0));
    }

    #[test]
    fn collection_closed_on_begin_dead_end_block_header() {
        let genesis = test_data::genesis();
        let b0 = test_data::block_builder()
            .header()
            .parent(genesis.hash())
            .build()
            .build();
        let b1 = test_data::block_builder()
            .header()
            .parent(b0.hash())
            .build()
            .build();
        let b2 = test_data::block_builder()
            .header()
            .parent(b1.hash())
            .build()
            .build();

        let (_, core, sync) = create_sync(None, None);
        {
            let mut core = core.lock();
            let chain = core.chain();
            chain.mark_dead_end_block(&b0.hash());
        }

        core.lock()
            .peers
            .insert(0, Services::default(), DummyOutboundSyncConnection::new());
        assert!(core.lock().peers.enumerate().contains(&0));

        sync.on_headers(
            0,
            vec![
                b0.block_header.into(),
                b1.block_header.into(),
                b2.block_header.into(),
            ],
        );

        assert!(!core.lock().peers.enumerate().contains(&0));
    }

    #[test]
    fn collection_closed_on_in_middle_dead_end_block_header() {
        let genesis = test_data::genesis();
        let b0 = test_data::block_builder()
            .header()
            .parent(genesis.hash())
            .build()
            .build();
        let b1 = test_data::block_builder()
            .header()
            .parent(b0.hash())
            .build()
            .build();
        let b2 = test_data::block_builder()
            .header()
            .parent(b1.hash())
            .build()
            .build();

        let (_, core, sync) = create_sync(None, None);
        {
            let mut core = core.lock();
            let chain = core.chain();
            chain.mark_dead_end_block(&b1.hash());
        }

        core.lock().set_verify_headers(true);
        core.lock()
            .peers
            .insert(0, Services::default(), DummyOutboundSyncConnection::new());
        assert!(core.lock().peers.enumerate().contains(&0));

        sync.on_headers(
            0,
            vec![
                b0.block_header.into(),
                b1.block_header.into(),
                b2.block_header.into(),
            ],
        );

        assert!(!core.lock().peers.enumerate().contains(&0));
    }

    #[test]
    fn collection_closed_on_providing_dead_end_block() {
        let genesis = test_data::genesis();
        let b0 = test_data::block_builder()
            .header()
            .parent(genesis.hash())
            .build()
            .build();

        let (_, core, sync) = create_sync(None, None);
        {
            let mut core = core.lock();
            let chain = core.chain();
            chain.mark_dead_end_block(&b0.hash());
        }

        core.lock()
            .peers
            .insert(0, Services::default(), DummyOutboundSyncConnection::new());
        assert!(core.lock().peers.enumerate().contains(&0));

        sync.on_block(0, b0.into());

        assert!(!core.lock().peers.enumerate().contains(&0));
    }

    #[test]
    fn collection_closed_on_providing_child_dead_end_block() {
        let genesis = test_data::genesis();
        let b0 = test_data::block_builder()
            .header()
            .parent(genesis.hash())
            .build()
            .build();
        let b1 = test_data::block_builder()
            .header()
            .parent(b0.hash())
            .build()
            .build();

        let (_, core, sync) = create_sync(None, None);
        {
            let mut core = core.lock();
            let chain = core.chain();
            chain.mark_dead_end_block(&b0.hash());
        }

        core.lock()
            .peers
            .insert(0, Services::default(), DummyOutboundSyncConnection::new());
        assert!(core.lock().peers.enumerate().contains(&0));

        sync.on_block(0, b1.into());

        assert!(!core.lock().peers.enumerate().contains(&0));
    }

    #[test]
    fn when_peer_does_not_respond_to_block_requests() {
        let genesis = test_data::genesis();
        let b0 = test_data::block_builder()
            .header()
            .parent(genesis.hash())
            .build()
            .build(); // block we will stuck with
        let b1 = test_data::block_builder()
            .header()
            .parent(genesis.hash())
            .build()
            .build(); // another branch
        let b2 = test_data::block_builder()
            .header()
            .parent(b1.hash())
            .build()
            .build();

        let (executor, core, sync) = create_sync(None, None);

        // when peer1 announces 'false' b0
        sync.on_headers(1, vec![b0.block_header.clone().into()]);
        // and peer2 announces 'true' b1
        sync.on_headers(
            2,
            vec![
                b1.block_header.clone().into(),
                b2.block_header.clone().into(),
            ],
        );

        // check that all blocks are requested
        assert_eq!(core.lock().information().chain.requested, 3);

        // forget tasks
        {
            executor.take_tasks();
        }

        // and then peer2 responds with with b1 while b0 is still left in queue
        sync.on_block(2, b1.into());

        // now simulate some time has passed && number of b0 failures is @max level
        {
            let mut core = core.lock();
            core.forget_failed_blocks(&vec![b0.hash()]);
            core.execute_synchronization_tasks(None, Some(vec![b0.hash()]));
        }

        // check that only one block (b2) is requested
        assert_eq!(core.lock().information().chain.requested, 1);
    }

    #[test]
    fn sync_listener_calls() {
        let (_, _, sync) = create_sync(None, None);

        // install sync listener
        let data = Arc::new(Mutex::new(DummySyncListenerData::default()));
        sync.install_sync_listener(Box::new(DummySyncListener::new(data.clone())));

        // at the beginning, is_synchronizing must be equal to false
        assert_eq!(data.lock().is_synchronizing, false);
        assert_eq!(data.lock().best_blocks.len(), 0);

        // supply with new block header => is_synchronizing is still false
        sync.on_headers(0, vec![test_data::block_h1().block_header.into()]);
        assert_eq!(data.lock().is_synchronizing, false);
        assert_eq!(data.lock().best_blocks.len(), 0);

        // supply with 2 new blocks headers => is_synchronizing is true
        sync.on_headers(
            0,
            vec![
                test_data::block_h2().block_header.into(),
                test_data::block_h3().block_header.into(),
            ],
        );
        assert_eq!(data.lock().is_synchronizing, true);
        assert_eq!(data.lock().best_blocks.len(), 0);

        // supply with block 3 => no new best block is informed
        sync.on_block(0, test_data::block_h3().into());
        assert_eq!(data.lock().is_synchronizing, true);
        assert_eq!(data.lock().best_blocks.len(), 0);

        // supply with block 1 => new best block is informed
        sync.on_block(0, test_data::block_h1().into());
        assert_eq!(data.lock().is_synchronizing, true);
        assert_eq!(data.lock().best_blocks.len(), 1);

        // supply with block 2 => 2 new best block is informed
        sync.on_block(0, test_data::block_h2().into());
        assert_eq!(data.lock().is_synchronizing, false);
        assert_eq!(data.lock().best_blocks.len(), 3);
    }
}

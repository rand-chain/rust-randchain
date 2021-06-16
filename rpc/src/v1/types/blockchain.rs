/// Information of the blockchain
/// See https://github.com/bitcoin/bitcoin/blob/master/src/rpc/blockchain.cpp#L1411-L1518
#[derive(Default, Serialize, Deserialize)]
pub struct BlockchainInfo {
    pub chain: String,             // current network name (main, test, signet, regtest)
    pub blocks: u32, // the height of the most-work fully-validated chain. The genesis block has height 0
    pub headers: u32, // the current number of headers we have validated
    pub bestblockhash: String, // the hash of the currently best block
    pub difficulty: f64, // the current difficulty
    pub mediantime: Option<u32>, // median time for the current best block
    pub verificationprogress: u32, // estimate of verification progress [0, 1]
    pub initialblockdownload: u32, // (debug information) estimate of whether this node is in Initial Block Download mode
    pub chainwork: String,         // total amount of work in active chain, in hexadecimal
    pub size_on_disk: Option<u32>, // the estimated size of the block and undo files on disk
    pub pruned: bool,              // if the blocks are subject to pruning
    pub pruneheight: Option<u32>, // lowest-height complete block stored (only present if pruning is enabled)
    pub automatic_pruning: Option<bool>, // whether automatic pruning is enabled (only present if pruning is enabled)
    pub prune_target_size: Option<u32>, // the target size used by pruning (only present if automatic pruning is enabled)
    pub softforks: Option<u32>,         // status of softforks TODO
    pub warnings: Option<String>,       // any network and blockchain warnings
}

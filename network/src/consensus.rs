use hash::H256;
use {Magic, Network};

#[derive(Debug, Clone)]
/// Parameters that influence chain consensus.
pub struct ConsensusParams {
    /// Network.
    pub network: Network,
}

// TODO: should extract this
#[derive(Debug, Clone)]
/// Concurrent consensus rule forks.
pub enum ConsensusFork {
    /// No fork.
    BitcoinCore,
}

impl ConsensusParams {
    // TODO: remove "_fork: ConsensusFork"
    pub fn new(network: Network, _fork: ConsensusFork) -> Self {
        match network {
            Network::Mainnet | Network::Other(_) => ConsensusParams { network: network },
            Network::Testnet => ConsensusParams { network: network },
            Network::Regtest | Network::Unitest => ConsensusParams { network: network },
        }
    }

    pub fn magic(&self) -> Magic {
        self.network.magic()
    }

    pub fn is_bip30_exception(&self, hash: &H256, height: u32) -> bool {
        (height == 91842
            && hash
                == &H256::from_reversed_str(
                    "00000000000a4d0a398161ffc163c503763b1f4360639393e0e4c8e300e0caec",
                ))
            || (height == 91880
                && hash
                    == &H256::from_reversed_str(
                        "00000000000743f190a18c5577a3c2d2a1f610ae9601ac046a38084ccb7cd721",
                    ))
    }

    // TODO: remove this
    /// Returns true if SegWit is possible on this chain.
    pub fn is_segwit_possible(&self) -> bool {
        self.network != Network::Regtest
    }
}

impl ConsensusFork {
    /// Absolute (across all forks) maximum block size. Currently is 8MB for post-HF BitcoinCash
    pub fn absolute_maximum_block_size() -> usize {
        32_000_000
    }

    /// Absolute (across all forks) maximum number of sigops in single block. Currently is max(sigops) for 8MB post-HF BitcoinCash block
    pub fn absolute_maximum_block_sigops() -> usize {
        160_000
    }

    /// Witness scale factor (equal among all forks)
    pub fn witness_scale_factor() -> usize {
        4
    }

    pub fn activation_height(&self) -> u32 {
        0
    }

    // TODO: remove this
    pub fn min_transaction_size(&self, _median_time_past: u32) -> usize {
        0
    }

    // TODO: remove this
    pub fn max_transaction_size(&self) -> usize {
        // BitcoinCash: according to REQ-5: max size of tx is still 1_000_000
        // SegWit: size * 4 <= 4_000_000 ===> max size of tx is still 1_000_000
        1_000_000
    }

    // TODO: remove this
    pub fn min_block_size(&self, _height: u32) -> usize {
        0
    }

    // TODO: remove this
    pub fn max_block_size(&self, _height: u32, _median_time_past: u32) -> usize {
        1_000_000
    }

    // TODO: remove this
    pub fn max_block_sigops(&self, _height: u32, _block_size: usize) -> usize {
        20_000
    }

    // TODO: remove this
    pub fn max_block_sigops_cost(&self, _height: u32, _block_size: usize) -> usize {
        80_000
    }

    // TODO: remove this
    pub fn max_block_weight(&self, _height: u32) -> usize {
        4_000_000
    }
}

#[cfg(test)]
mod tests {
    use super::super::Network;
    use super::{BitcoinCashConsensusParams, ConsensusFork, ConsensusParams};

    #[test]
    fn test_consensus_params_bip34_height() {
        assert_eq!(
            ConsensusParams::new(Network::Mainnet, ConsensusFork::BitcoinCore).bip34_height,
            227931
        );
        assert_eq!(
            ConsensusParams::new(Network::Testnet, ConsensusFork::BitcoinCore).bip34_height,
            21111
        );
        assert_eq!(
            ConsensusParams::new(Network::Regtest, ConsensusFork::BitcoinCore).bip34_height,
            100000000
        );
    }

    #[test]
    fn test_consensus_params_bip65_height() {
        assert_eq!(
            ConsensusParams::new(Network::Mainnet, ConsensusFork::BitcoinCore).bip65_height,
            388381
        );
        assert_eq!(
            ConsensusParams::new(Network::Testnet, ConsensusFork::BitcoinCore).bip65_height,
            581885
        );
        assert_eq!(
            ConsensusParams::new(Network::Regtest, ConsensusFork::BitcoinCore).bip65_height,
            1351
        );
    }

    #[test]
    fn test_consensus_params_bip66_height() {
        assert_eq!(
            ConsensusParams::new(Network::Mainnet, ConsensusFork::BitcoinCore).bip66_height,
            363725
        );
        assert_eq!(
            ConsensusParams::new(Network::Testnet, ConsensusFork::BitcoinCore).bip66_height,
            330776
        );
        assert_eq!(
            ConsensusParams::new(Network::Regtest, ConsensusFork::BitcoinCore).bip66_height,
            1251
        );
    }

    #[test]
    fn test_consensus_activation_threshold() {
        assert_eq!(
            ConsensusParams::new(Network::Mainnet, ConsensusFork::BitcoinCore)
                .rule_change_activation_threshold,
            1916
        );
        assert_eq!(
            ConsensusParams::new(Network::Testnet, ConsensusFork::BitcoinCore)
                .rule_change_activation_threshold,
            1512
        );
        assert_eq!(
            ConsensusParams::new(Network::Regtest, ConsensusFork::BitcoinCore)
                .rule_change_activation_threshold,
            108
        );
    }

    #[test]
    fn test_consensus_miner_confirmation_window() {
        assert_eq!(
            ConsensusParams::new(Network::Mainnet, ConsensusFork::BitcoinCore)
                .miner_confirmation_window,
            2016
        );
        assert_eq!(
            ConsensusParams::new(Network::Testnet, ConsensusFork::BitcoinCore)
                .miner_confirmation_window,
            2016
        );
        assert_eq!(
            ConsensusParams::new(Network::Regtest, ConsensusFork::BitcoinCore)
                .miner_confirmation_window,
            144
        );
    }

    #[test]
    fn test_consensus_fork_min_block_size() {
        assert_eq!(ConsensusFork::BitcoinCore.min_block_size(0), 0);
        let fork = ConsensusFork::BitcoinCash(BitcoinCashConsensusParams::new(Network::Mainnet));
        assert_eq!(fork.min_block_size(0), 0);
        assert_eq!(fork.min_block_size(fork.activation_height()), 1_000_001);
    }

    #[test]
    fn test_consensus_fork_max_transaction_size() {
        assert_eq!(ConsensusFork::BitcoinCore.max_transaction_size(), 1_000_000);
        assert_eq!(
            ConsensusFork::BitcoinCash(BitcoinCashConsensusParams::new(Network::Mainnet))
                .max_transaction_size(),
            1_000_000
        );
    }

    #[test]
    fn test_consensus_fork_min_transaction_size() {
        assert_eq!(ConsensusFork::BitcoinCore.min_transaction_size(0), 0);
        assert_eq!(
            ConsensusFork::BitcoinCore.min_transaction_size(2000000000),
            0
        );
        assert_eq!(
            ConsensusFork::BitcoinCash(BitcoinCashConsensusParams::new(Network::Mainnet))
                .min_transaction_size(0),
            0
        );
        assert_eq!(
            ConsensusFork::BitcoinCash(BitcoinCashConsensusParams::new(Network::Mainnet))
                .min_transaction_size(2000000000),
            100
        );
    }

    #[test]
    fn test_consensus_fork_max_block_sigops() {
        assert_eq!(
            ConsensusFork::BitcoinCore.max_block_sigops(0, 1_000_000),
            20_000
        );
        let fork = ConsensusFork::BitcoinCash(BitcoinCashConsensusParams::new(Network::Mainnet));
        assert_eq!(fork.max_block_sigops(0, 1_000_000), 20_000);
        assert_eq!(
            fork.max_block_sigops(fork.activation_height(), 2_000_000),
            40_000
        );
        assert_eq!(
            fork.max_block_sigops(fork.activation_height() + 100, 3_000_000),
            60_000
        );
    }
}

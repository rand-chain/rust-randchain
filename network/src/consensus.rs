use {Magic, Network};

#[derive(Debug, Clone)]
/// Parameters that influence chain consensus.
pub struct ConsensusParams {
    /// Network.
    pub network: Network,
}

impl ConsensusParams {
    pub fn new(network: Network) -> Self {
        ConsensusParams { network: network }
    }

    pub fn magic(&self) -> Magic {
        self.network.magic()
    }

    // TODO: remove this
    /// Returns true if SegWit is possible on this chain.
    pub fn is_segwit_possible(&self) -> bool {
        self.network != Network::Regtest
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

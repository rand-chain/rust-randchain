use chain::IndexedBlock;
use message::types;
use primitives::bytes::Bytes;
use primitives::hash::H256;
use synchronization_peers::MerkleBlockArtefacts;
use utils::{BloomFilter, KnownHashFilter, KnownHashType};

/// Filter, which controls data relayed over connection.
#[derive(Debug, Default)]
pub struct ConnectionFilter {
    /// Known hashes filter
    known_hash_filter: KnownHashFilter,
    /// Bloom filter
    bloom_filter: BloomFilter,
}

impl ConnectionFilter {
    /// Add known item hash
    pub fn hash_known_as(&mut self, hash: H256, hash_type: KnownHashType) {
        self.known_hash_filter.insert(hash, hash_type);
    }

    /// Is item with given hash && type is known by peer
    pub fn is_hash_known_as(&self, hash: &H256, hash_type: KnownHashType) -> bool {
        self.known_hash_filter.contains(hash, hash_type)
    }

    /// Check if block should be sent to this connection
    pub fn filter_block(&self, block_hash: &H256) -> bool {
        self.known_hash_filter.filter_block(block_hash)
    }

    /// Load filter
    pub fn load(&mut self, message: types::FilterLoad) {
        self.bloom_filter.set_bloom_filter(message);
    }

    /// Add filter
    pub fn add(&mut self, message: types::FilterAdd) {
        self.bloom_filter.update_bloom_filter(message);
    }

    /// Clear filter
    pub fn clear(&mut self) {
        self.bloom_filter.remove_bloom_filter();
    }

    /// Convert `Block` to `MerkleBlock` using this filter
    pub fn build_merkle_block(&self, block: &IndexedBlock) -> Option<MerkleBlockArtefacts> {
        if !self.bloom_filter.is_set() {
            // only respond when bloom filter is set
            return None;
        }

        // prepare result
        let result = MerkleBlockArtefacts {
            // let mut result = MerkleBlockArtefacts {
            merkleblock: types::MerkleBlock {
                block_header: block.header.raw.clone(),
                hashes: Vec::default(),
                flags: Bytes::default(),
            },
        };

        // TODO:
        // // build partial merkle tree
        // let partial_merkle_tree = build_partial_merkle_tree(all_hashes, all_flags);
        // result.merkleblock.hashes.extend(partial_merkle_tree.hashes);

        // // to_bytes() converts [true, false, true] to 0b10100000
        // // while protocol requires [true, false, true] to be serialized as 0x00000101
        // result.merkleblock.flags = partial_merkle_tree
        //     .flags
        //     .to_bytes()
        //     .into_iter()
        //     .map(|b| {
        //         ((b & 0b10000000) >> 7)
        //             | ((b & 0b01000000) >> 5)
        //             | ((b & 0b00100000) >> 3)
        //             | ((b & 0b00010000) >> 1)
        //             | ((b & 0b00001000) << 1)
        //             | ((b & 0b00000100) << 3)
        //             | ((b & 0b00000010) << 5)
        //             | ((b & 0b00000001) << 7)
        //     })
        //     .collect::<Vec<u8>>()
        //     .into();

        Some(result)
    }
}

#[cfg(test)]
pub mod tests {
    extern crate test_data;

    use super::ConnectionFilter;
    use chain::IndexedTransaction;
    use message::types;
    use primitives::bytes::Bytes;
    use std::iter::repeat;
    use utils::KnownHashType;

    #[test]
    fn filter_default_accepts_block() {
        assert!(ConnectionFilter::default().filter_block(&test_data::genesis().hash()));
    }

    #[test]
    fn filter_default_accepts_transaction() {
        assert!(ConnectionFilter::default().filter_transaction(
            &test_data::genesis().transactions[0].clone().into(),
            Some(0)
        ));
    }

    #[test]
    fn filter_rejects_block_known() {
        let mut filter = ConnectionFilter::default();
        filter.hash_known_as(test_data::block_h1().hash(), KnownHashType::Block);
        filter.hash_known_as(test_data::block_h2().hash(), KnownHashType::CompactBlock);
        assert!(!filter.filter_block(&test_data::block_h1().hash()));
        assert!(!filter.filter_block(&test_data::block_h2().hash()));
        assert!(filter.filter_block(&test_data::genesis().hash()));
    }

    #[test]
    fn filter_rejects_transaction_known() {
        let mut filter = ConnectionFilter::default();
        filter.hash_known_as(
            test_data::block_h1().transactions[0].hash(),
            KnownHashType::Transaction,
        );
        assert!(
            !filter.filter_transaction(&test_data::block_h1().transactions[0].clone().into(), None)
        );
        assert!(
            filter.filter_transaction(&test_data::block_h2().transactions[0].clone().into(), None)
        );
    }

    #[test]
    fn filter_rejects_transaction_feerate() {
        let mut filter = ConnectionFilter::default();
        filter.set_fee_rate(types::FeeFilter::with_fee_rate(1000));
        assert!(
            filter.filter_transaction(&test_data::block_h1().transactions[0].clone().into(), None)
        );
        assert!(filter.filter_transaction(
            &test_data::block_h1().transactions[0].clone().into(),
            Some(1500)
        ));
        assert!(!filter.filter_transaction(
            &test_data::block_h1().transactions[0].clone().into(),
            Some(500)
        ));
    }

    #[test]
    fn filter_rejects_transaction_bloomfilter() {
        let mut filter = ConnectionFilter::default();
        let tx: IndexedTransaction = test_data::block_h1().transactions[0].clone().into();
        filter.load(types::FilterLoad {
            filter: Bytes::from(repeat(0u8).take(1024).collect::<Vec<_>>()),
            hash_functions: 10,
            tweak: 5,
            flags: types::FilterFlags::None,
        });
        assert!(!filter.filter_transaction(&tx, None));
        filter.add(types::FilterAdd {
            data: (&*tx.hash as &[u8]).into(),
        });
        assert!(filter.filter_transaction(&tx, None));
        filter.clear();
        assert!(filter.filter_transaction(&tx, None));
    }
}

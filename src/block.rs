use crate::hash::{ternary_hash, TritHash, HASH_LEN};
use crate::merkle::TernaryMerkle;
use crate::transaction::{transactions_balanced, TernaryTransaction};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct TernaryBlock {
    pub index: u64,
    pub prev_hash: TritHash,
    pub transactions: Vec<TernaryTransaction>,
    pub nonce: u64,
    pub timestamp: u64,
    pub merkle_root: TritHash,
}

impl TernaryBlock {
    pub fn new(
        index: u64,
        prev_hash: TritHash,
        transactions: Vec<TernaryTransaction>,
        nonce: u64,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let merkle_root = TernaryMerkle::new(&transactions).root();
        TernaryBlock { index, prev_hash, transactions, nonce, timestamp, merkle_root }
    }

    /// Serialize block header to bytes for hashing.
    pub fn header_bytes(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&self.index.to_le_bytes());
        for &t in &self.prev_hash {
            v.push((t + 1) as u8);
        }
        for &t in &self.merkle_root {
            v.push((t + 1) as u8);
        }
        v.extend_from_slice(&self.nonce.to_le_bytes());
        v.extend_from_slice(&self.timestamp.to_le_bytes());
        v
    }

    pub fn hash(&self) -> TritHash {
        ternary_hash(&self.header_bytes())
    }

    /// Validate block: check prev_hash linkage and transaction balance.
    pub fn is_valid(&self, expected_prev_hash: &TritHash) -> bool {
        if &self.prev_hash != expected_prev_hash {
            return false;
        }
        transactions_balanced(&self.transactions)
    }

    /// Genesis block with empty prev_hash.
    pub fn genesis(transactions: Vec<TernaryTransaction>) -> Self {
        TernaryBlock::new(0, [0i8; HASH_LEN], transactions, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{TernaryTransaction, TxType};

    fn balanced_txs() -> Vec<TernaryTransaction> {
        vec![
            TernaryTransaction::new(10, TxType::Send, [1u8; 8], [2u8; 8]),
            TernaryTransaction::new(10, TxType::Receive, [2u8; 8], [1u8; 8]),
        ]
    }

    #[test]
    fn block_hash_is_deterministic() {
        let b = TernaryBlock::genesis(balanced_txs());
        assert_eq!(b.hash(), b.hash());
    }

    #[test]
    fn block_hash_changes_with_nonce() {
        let b1 = TernaryBlock::new(0, [0i8; HASH_LEN], balanced_txs(), 0);
        let b2 = TernaryBlock::new(0, [0i8; HASH_LEN], balanced_txs(), 1);
        assert_ne!(b1.hash(), b2.hash());
    }

    #[test]
    fn block_validity_checks_prev_hash() {
        let genesis = TernaryBlock::genesis(balanced_txs());
        let genesis_hash = genesis.hash();
        let b2 = TernaryBlock::new(1, genesis_hash, balanced_txs(), 0);
        assert!(b2.is_valid(&genesis_hash));
        assert!(!b2.is_valid(&[1i8; HASH_LEN]));
    }
}

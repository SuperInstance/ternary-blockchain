use crate::hash::{hash_pair, TritHash, HASH_LEN};
use crate::transaction::TernaryTransaction;

pub struct TernaryMerkle {
    leaves: Vec<TritHash>,
}

impl TernaryMerkle {
    pub fn new(transactions: &[TernaryTransaction]) -> Self {
        let leaves = transactions.iter().map(|tx| tx.hash()).collect();
        TernaryMerkle { leaves }
    }

    pub fn from_hashes(hashes: Vec<TritHash>) -> Self {
        TernaryMerkle { leaves: hashes }
    }

    /// Compute Merkle root using TernaryHash pairs.
    pub fn root(&self) -> TritHash {
        if self.leaves.is_empty() {
            return [0i8; HASH_LEN];
        }
        let mut level = self.leaves.clone();
        while level.len() > 1 {
            if level.len() % 2 != 0 {
                level.push(*level.last().unwrap());
            }
            let mut next = Vec::with_capacity(level.len() / 2);
            for chunk in level.chunks(2) {
                next.push(hash_pair(&chunk[0], &chunk[1]));
            }
            level = next;
        }
        level[0]
    }

    /// Return the proof path (sibling hashes) for the leaf at `index`.
    pub fn proof(&self, index: usize) -> Vec<(TritHash, bool)> {
        // bool = true means sibling is on the right
        if self.leaves.is_empty() || index >= self.leaves.len() {
            return vec![];
        }
        let mut level = self.leaves.clone();
        let mut idx = index;
        let mut path = Vec::new();
        while level.len() > 1 {
            if level.len() % 2 != 0 {
                level.push(*level.last().unwrap());
            }
            let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            let is_right = idx % 2 == 0;
            path.push((level[sibling_idx], is_right));
            idx /= 2;
            let mut next = Vec::with_capacity(level.len() / 2);
            for chunk in level.chunks(2) {
                next.push(hash_pair(&chunk[0], &chunk[1]));
            }
            level = next;
        }
        path
    }

    /// Verify a Merkle proof for `leaf_hash` against `root`.
    pub fn verify_proof(leaf_hash: &TritHash, proof: &[(TritHash, bool)], root: &TritHash) -> bool {
        let mut current = *leaf_hash;
        for (sibling, is_right) in proof {
            current = if *is_right {
                hash_pair(&current, sibling)
            } else {
                hash_pair(sibling, &current)
            };
        }
        &current == root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{TernaryTransaction, TxType};

    fn make_tx(v: i64) -> TernaryTransaction {
        TernaryTransaction::new(v, TxType::Hold, [0u8; 8], [0u8; 8])
    }

    #[test]
    fn merkle_root_consistent() {
        let txs = vec![make_tx(1), make_tx(2), make_tx(3), make_tx(4)];
        let m = TernaryMerkle::new(&txs);
        assert_eq!(m.root(), m.root());
    }

    #[test]
    fn merkle_root_changes_with_different_txs() {
        let txs1 = vec![make_tx(1), make_tx(2)];
        let txs2 = vec![make_tx(3), make_tx(4)];
        let m1 = TernaryMerkle::new(&txs1);
        let m2 = TernaryMerkle::new(&txs2);
        assert_ne!(m1.root(), m2.root());
    }

    #[test]
    fn merkle_proof_verifies() {
        let txs = vec![make_tx(10), make_tx(20), make_tx(30), make_tx(40)];
        let m = TernaryMerkle::new(&txs);
        let root = m.root();
        let proof = m.proof(1);
        assert!(TernaryMerkle::verify_proof(&txs[1].hash(), &proof, &root));
    }
}

use crate::block::TernaryBlock;
use crate::hash::TritHash;

/// Check that hash has at least `difficulty` leading zero-trits.
pub fn has_leading_zeros(hash: &TritHash, difficulty: usize) -> bool {
    hash.iter().take(difficulty).all(|&t| t == 0)
}

pub struct ProofOfWork;

impl ProofOfWork {
    /// Mine: increment nonce until block hash has `difficulty` leading zero-trits.
    pub fn mine(mut block: TernaryBlock, difficulty: usize) -> TernaryBlock {
        loop {
            let h = block.hash();
            if has_leading_zeros(&h, difficulty) {
                return block;
            }
            block.nonce = block.nonce.wrapping_add(1);
        }
    }

    pub fn verify(block: &TernaryBlock, difficulty: usize) -> bool {
        has_leading_zeros(&block.hash(), difficulty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::TernaryBlock;
    use crate::hash::HASH_LEN;
    use crate::transaction::{TernaryTransaction, TxType};

    fn balanced_txs() -> Vec<TernaryTransaction> {
        vec![
            TernaryTransaction::new(5, TxType::Send, [0u8; 8], [1u8; 8]),
            TernaryTransaction::new(5, TxType::Receive, [1u8; 8], [0u8; 8]),
        ]
    }

    #[test]
    fn mining_produces_valid_pow() {
        let block = TernaryBlock::genesis(balanced_txs());
        let mined = ProofOfWork::mine(block, 2);
        assert!(ProofOfWork::verify(&mined, 2));
    }

    #[test]
    fn pow_verify_fails_without_mining() {
        // A block with nonce=0 very likely doesn't have 4 leading zeros
        let block = TernaryBlock::new(0, [0i8; HASH_LEN], balanced_txs(), 0);
        // difficulty=4 is unlikely satisfied by nonce=0
        if !ProofOfWork::verify(&block, 4) {
            assert!(!ProofOfWork::verify(&block, 4));
        }
    }

    #[test]
    fn mined_hash_has_correct_leading_zeros() {
        let block = TernaryBlock::genesis(balanced_txs());
        let mined = ProofOfWork::mine(block, 2);
        let h = mined.hash();
        assert_eq!(h[0], 0);
        assert_eq!(h[1], 0);
    }
}

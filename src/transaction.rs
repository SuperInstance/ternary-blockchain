use crate::hash::{ternary_hash, TritHash};

#[derive(Debug, Clone, PartialEq)]
pub enum TxType {
    Send = -1,
    Hold = 0,
    Receive = 1,
}

impl TxType {
    pub fn as_trit(&self) -> i8 {
        match self {
            TxType::Send => -1,
            TxType::Hold => 0,
            TxType::Receive => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TernaryTransaction {
    pub value: i64,
    pub tx_type: TxType,
    pub from: [u8; 8],
    pub to: [u8; 8],
}

impl TernaryTransaction {
    pub fn new(value: i64, tx_type: TxType, from: [u8; 8], to: [u8; 8]) -> Self {
        TernaryTransaction { value, tx_type, from, to }
    }

    /// Serialize to bytes for hashing.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&self.value.to_le_bytes());
        v.push((self.tx_type.as_trit() + 1) as u8);
        v.extend_from_slice(&self.from);
        v.extend_from_slice(&self.to);
        v
    }

    pub fn hash(&self) -> TritHash {
        ternary_hash(&self.to_bytes())
    }
}

/// Check that the sum of tx_type trits across a set of transactions is 0 (balanced).
pub fn transactions_balanced(txs: &[TernaryTransaction]) -> bool {
    let sum: i64 = txs.iter().map(|t| t.tx_type.as_trit() as i64).sum();
    sum == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_serializes_consistently() {
        let tx = TernaryTransaction::new(100, TxType::Send, [1u8; 8], [2u8; 8]);
        assert_eq!(tx.to_bytes(), tx.to_bytes());
    }

    #[test]
    fn transaction_hash_is_trit_valued() {
        let tx = TernaryTransaction::new(50, TxType::Receive, [0u8; 8], [1u8; 8]);
        let h = tx.hash();
        for &t in &h {
            assert!(t == -1 || t == 0 || t == 1);
        }
    }

    #[test]
    fn balanced_transactions() {
        let txs = vec![
            TernaryTransaction::new(100, TxType::Send, [1u8; 8], [2u8; 8]),
            TernaryTransaction::new(100, TxType::Receive, [2u8; 8], [1u8; 8]),
        ];
        assert!(transactions_balanced(&txs));
    }

    #[test]
    fn unbalanced_transactions_detected() {
        let txs = vec![
            TernaryTransaction::new(100, TxType::Send, [1u8; 8], [2u8; 8]),
            TernaryTransaction::new(100, TxType::Send, [2u8; 8], [3u8; 8]),
        ];
        assert!(!transactions_balanced(&txs));
    }
}

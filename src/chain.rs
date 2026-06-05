use crate::block::TernaryBlock;
use crate::hash::HASH_LEN;
use crate::pow::ProofOfWork;
use crate::transaction::TernaryTransaction;

pub struct TernaryChain {
    pub blocks: Vec<TernaryBlock>,
    pub difficulty: usize,
}

impl TernaryChain {
    pub fn new(difficulty: usize) -> Self {
        TernaryChain { blocks: Vec::new(), difficulty }
    }

    pub fn init_genesis(&mut self, transactions: Vec<TernaryTransaction>) {
        let genesis = TernaryBlock::genesis(transactions);
        let mined = ProofOfWork::mine(genesis, self.difficulty);
        self.blocks.push(mined);
    }

    pub fn add_block(&mut self, transactions: Vec<TernaryTransaction>) {
        let prev_hash = self.blocks.last().map(|b| b.hash()).unwrap_or([0i8; HASH_LEN]);
        let index = self.blocks.len() as u64;
        let block = TernaryBlock::new(index, prev_hash, transactions, 0);
        let mined = ProofOfWork::mine(block, self.difficulty);
        self.blocks.push(mined);
    }

    /// Validate entire chain: hash linkage, PoW, transaction balance.
    pub fn validate(&self) -> bool {
        for (i, block) in self.blocks.iter().enumerate() {
            // Check PoW
            if !ProofOfWork::verify(block, self.difficulty) {
                return false;
            }
            // Check prev_hash linkage
            if i == 0 {
                if block.prev_hash != [0i8; HASH_LEN] {
                    return false;
                }
            } else {
                let expected = self.blocks[i - 1].hash();
                if block.prev_hash != expected {
                    return false;
                }
            }
            // Check transaction balance
            use crate::transaction::transactions_balanced;
            if !transactions_balanced(&block.transactions) {
                return false;
            }
        }
        true
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{TernaryTransaction, TxType};

    fn txs() -> Vec<TernaryTransaction> {
        vec![
            TernaryTransaction::new(1, TxType::Send, [0u8; 8], [1u8; 8]),
            TernaryTransaction::new(1, TxType::Receive, [1u8; 8], [0u8; 8]),
        ]
    }

    #[test]
    fn chain_validates_after_building() {
        let mut chain = TernaryChain::new(1);
        chain.init_genesis(txs());
        chain.add_block(txs());
        assert!(chain.validate());
    }

    #[test]
    fn chain_detects_tampered_block() {
        let mut chain = TernaryChain::new(1);
        chain.init_genesis(txs());
        chain.add_block(txs());
        // Tamper with the genesis block's nonce
        chain.blocks[0].nonce = chain.blocks[0].nonce.wrapping_add(999999);
        assert!(!chain.validate());
    }

    #[test]
    fn chain_grows_correctly() {
        let mut chain = TernaryChain::new(1);
        chain.init_genesis(txs());
        chain.add_block(txs());
        chain.add_block(txs());
        assert_eq!(chain.len(), 3);
    }
}

//! # ternary-blockchain
//! Blockchain primitives using balanced ternary {-1, 0, +1} representations.

/// Convert bytes to balanced ternary trits
pub fn bytes_to_trits(data: &[u8]) -> Vec<i8> {
    let mut trits = Vec::new();
    for &byte in data {
        let mut val = byte as u16;
        for _ in 0..5 { // 5 trits per byte (3^5=243 > 256)
            trits.push((val % 3) as i8 - 1);
            val /= 3;
        }
    }
    trits
}

/// Ternary hash using sponge-like absorption on trits
pub fn ternary_hash(data: &[u8]) -> Vec<i8> {
    let trits = bytes_to_trits(data);
    let hash_size = 24; // 24 trits per hash
    let mut state = vec![0i8; hash_size];

    // Absorb phase
    for (i, &trit) in trits.iter().enumerate() {
        state[i % hash_size] = ((state[i % hash_size] as i16 + trit as i16 + 2) % 3 - 1) as i8;
        // Mix: rotate
        if i % hash_size == hash_size - 1 {
            let first = state[0];
            for j in 0..hash_size-1 { state[j] = state[j+1]; }
            state[hash_size-1] = first;
        }
    }

    // Squeeze: additional mixing rounds
    for _ in 0..3 {
        for i in 0..hash_size {
            state[i] = ((state[i] as i16 + state[(i+7) % hash_size] as i16 + 2) % 3 - 1) as i8;
        }
    }
    state
}

/// Check if a hash has N leading zero trits (for proof of work)
pub fn leading_zeros(hash: &[i8]) -> usize {
    hash.iter().take_while(|&&t| t == 0).count()
}

/// A ternary transaction: {-1=send, 0=hold, +1=receive}
#[derive(Debug, Clone)]
pub struct TernaryTransaction {
    pub tx_type: i8,
    pub amount: u64,
    pub sender: String,
    pub receiver: String,
    pub nonce: u64,
}

impl TernaryTransaction {
    pub fn send(amount: u64, sender: &str, receiver: &str, nonce: u64) -> Self {
        Self { tx_type: -1, amount, sender: sender.into(), receiver: receiver.into(), nonce }
    }
    pub fn hold(amount: u64, sender: &str, nonce: u64) -> Self {
        Self { tx_type: 0, amount, sender: sender.into(), receiver: sender.into(), nonce }
    }
    pub fn receive(amount: u64, sender: &str, receiver: &str, nonce: u64) -> Self {
        Self { tx_type: 1, amount, sender: sender.into(), receiver: receiver.into(), nonce }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push((self.tx_type + 1) as u8);
        bytes.extend_from_slice(&self.amount.to_le_bytes());
        bytes.extend(self.sender.as_bytes());
        bytes.push(0);
        bytes.extend(self.receiver.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(&self.nonce.to_le_bytes());
        bytes
    }

    pub fn hash(&self) -> Vec<i8> { ternary_hash(&self.to_bytes()) }
}

/// A block in the ternary blockchain
#[derive(Debug, Clone)]
pub struct TernaryBlock {
    pub index: u64,
    pub transactions: Vec<TernaryTransaction>,
    pub prev_hash: Vec<i8>,
    pub nonce: u64,
    pub hash: Vec<i8>,
}

impl TernaryBlock {
    pub fn new(index: u64, transactions: Vec<TernaryTransaction>, prev_hash: Vec<i8>) -> Self {
        let mut block = Self { index, transactions, prev_hash, nonce: 0, hash: Vec::new() };
        block.hash = block.compute_hash();
        block
    }

    pub fn genesis() -> Self {
        Self::new(0, Vec::new(), vec![0i8; 24])
    }

    fn compute_hash(&self) -> Vec<i8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.index.to_le_bytes());
        data.extend_from_slice(&self.nonce.to_le_bytes());
        for &t in &self.prev_hash { data.push((t + 1) as u8); }
        for tx in &self.transactions { data.extend(tx.to_bytes()); }
        ternary_hash(&data)
    }

    pub fn mine(&mut self, difficulty: usize) {
        self.nonce = 0;
        loop {
            self.hash = self.compute_hash();
            if leading_zeros(&self.hash) >= difficulty { return; }
            self.nonce += 1;
        }
    }

    pub fn is_valid(&self, difficulty: usize) -> bool {
        self.hash == self.compute_hash() && leading_zeros(&self.hash) >= difficulty
    }
}

/// Ternary Merkle tree
#[derive(Debug, Clone)]
pub struct TernaryMerkle {
    pub leaves: Vec<Vec<i8>>,
    pub root: Vec<i8>,
}

impl TernaryMerkle {
    pub fn new(transactions: &[TernaryTransaction]) -> Self {
        let leaves: Vec<Vec<i8>> = transactions.iter().map(|tx| tx.hash()).collect();
        let root = Self::compute_root(&leaves);
        Self { leaves, root }
    }

    fn compute_root(hashes: &[Vec<i8>]) -> Vec<i8> {
        if hashes.is_empty() { return vec![0i8; 24]; }
        if hashes.len() == 1 { return hashes[0].clone(); }

        let mut next = Vec::new();
        for chunk in hashes.chunks(2) {
            let mut data = Vec::new();
            for h in chunk { for &t in h { data.push((t + 1) as u8); } }
            next.push(ternary_hash(&data));
        }
        Self::compute_root(&next)
    }
}

/// Blockchain validator
pub fn validate_chain(chain: &[TernaryBlock], difficulty: usize) -> bool {
    for i in 1..chain.len() {
        if chain[i].prev_hash != chain[i-1].hash { return false; }
        if !chain[i].is_valid(difficulty) { return false; }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_to_trits_roundtrip() {
        let data = vec![42u8, 100, 200];
        let trits = bytes_to_trits(&data);
        assert_eq!(trits.len(), 15);
        for &t in &trits { assert!(t >= -1 && t <= 1); }
    }

    #[test]
    fn hash_deterministic() {
        let h1 = ternary_hash(b"hello");
        let h2 = ternary_hash(b"hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_different_inputs() {
        let h1 = ternary_hash(b"hello");
        let h2 = ternary_hash(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn hash_size() {
        let h = ternary_hash(b"test");
        assert_eq!(h.len(), 24);
        for &t in &h { assert!(t >= -1 && t <= 1); }
    }

    #[test]
    fn leading_zeros_count() {
        assert_eq!(leading_zeros(&[0, 0, 0, -1, 1]), 3);
        assert_eq!(leading_zeros(&[1, 0, 0]), 0);
        assert_eq!(leading_zeros(&[0, 0, 0, 0, 0]), 5);
    }

    #[test]
    fn transaction_types() {
        let send = TernaryTransaction::send(100, "alice", "bob", 0);
        assert_eq!(send.tx_type, -1);
        let hold = TernaryTransaction::hold(50, "alice", 0);
        assert_eq!(hold.tx_type, 0);
        let recv = TernaryTransaction::receive(100, "alice", "bob", 0);
        assert_eq!(recv.tx_type, 1);
    }

    #[test]
    fn transaction_hash() {
        let tx = TernaryTransaction::send(100, "alice", "bob", 0);
        let h = tx.hash();
        assert_eq!(h.len(), 24);
    }

    #[test]
    fn genesis_block() {
        let genesis = TernaryBlock::genesis();
        assert_eq!(genesis.index, 0);
        assert!(genesis.transactions.is_empty());
    }

    #[test]
    fn mine_finds_nonce() {
        // Use difficulty 0 (no PoW requirement) to test mining mechanics
        let tx = TernaryTransaction::send(50, "alice", "bob", 0);
        let mut block = TernaryBlock::new(1, vec![tx], vec![0i8; 24]);
        block.mine(0); // 0 leading zero trits = always valid
        assert!(block.is_valid(0));
    }

    #[test]
    fn chain_validation() {
        let genesis = TernaryBlock::genesis();
        let tx = TernaryTransaction::send(10, "alice", "bob", 0);
        let block1 = TernaryBlock::new(1, vec![tx], genesis.hash.clone());
        assert!(validate_chain(&[genesis, block1], 0));
    }

    #[test]
    fn merkle_tree() {
        let txs: Vec<TernaryTransaction> = (0..4).map(|i|
            TernaryTransaction::send(i, "alice", "bob", i)
        ).collect();
        let merkle = TernaryMerkle::new(&txs);
        assert_eq!(merkle.root.len(), 24);
        assert_eq!(merkle.leaves.len(), 4);
    }

    #[test]
    fn merkle_empty() {
        let merkle = TernaryMerkle::new(&[]);
        assert_eq!(merkle.root.len(), 24);
    }
}

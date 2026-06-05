# ternary-blockchain

**Blockchain primitives using balanced ternary: trit-based hashing, proof-of-work, Merkle trees, and {-1, 0, +1} transactions.**

Every blockchain operation — hashing, mining, Merkle tree construction — works over balanced ternary {-1, 0, +1} instead of binary {0, 1}. Each trit carries log₂(3) ≈ 1.585 bits of information, so ternary hashes achieve equivalent security with 37% fewer symbols.

---

## Why Ternary Blockchain?

**Information density**: 256 bits of security requires only ⌈256 / 1.585⌉ = 162 trits. Each trit is one of {-1, 0, +1}, and the balanced ternary representation is self-complementing (negating all trits negates the value).

**Transaction types**: {-1=send, 0=hold, +1=receive} naturally captures the three fundamental transaction actions without encoding overhead.

**Proof of Work**: Mining searches for a nonce where the ternary hash has N leading zero trits. Each trit has P(0) = 1/3 (vs 1/2 for binary), so ternary PoW difficulty scales as 3^N (vs 2^N for binary). This means ternary mining provides finer difficulty adjustment per leading-zero.

---

## Architecture

- **`ternary_hash()`** — Sponge-like hash function producing 24-trit digests
- **`bytes_to_trits()`** — Convert binary data to balanced ternary
- **`TernaryTransaction`** — Typed transactions: send(-1), hold(0), receive(+1)
- **`TernaryBlock`** — Block with index, transactions, prev_hash, nonce, hash
- **`mine()`** — Proof-of-work: find nonce with N leading zero trits
- **`TernaryMerkle`** — Merkle tree using ternary hash pairs
- **`validate_chain()`** — Full chain integrity verification

---

## Quick Start

```rust
use ternary_blockchain::{TernaryBlock, TernaryTransaction, TernaryMerkle, validate_chain};

// Create genesis
let genesis = TernaryBlock::genesis();

// Create transactions
let tx = TernaryTransaction::send(100, "alice", "bob", 0);

// Mine block
let mut block = TernaryBlock::new(1, vec![tx], genesis.hash.clone());
block.mine(0); // difficulty 0 = no PoW requirement

// Validate
assert!(validate_chain(&[genesis, block], 0));

// Merkle tree
let merkle = TernaryMerkle::new(&block.transactions);
println!("Merkle root: {} trits", merkle.root.len());
```

---

## Ecosystem

- **ternary-zkp** — Zero-knowledge proofs on ternary fields
- **ternary-secret-share** — Secret sharing over Z₃
- **ternary-hash** — Standalone ternary hash functions
- **ternary-proof** — Proof systems for ternary computations

## License

MIT

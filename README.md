# ternary-blockchain

> Blockchain primitives using balanced ternary `{-1, 0, +1}` representations.

---

## What problem does this solve?

Standard blockchain primitives rely on binary hash functions such as SHA-256.  This crate asks the opposite question: *what happens when every datum—hashes, transactions, Merkle roots, and proof-of-work targets—is expressed over a ternary alphabet?*  By moving from bits to **trits** we obtain a pedagogical test-bed for studying sponge constructions, Merkle trees, and Nakamoto consensus in the smallest non-trivial modular arithmetic beyond binary.  Ternary representations are also of practical interest for post-quantum hardware that natively operates on three-level quantum systems and for alternative coding-theoretic proofs that exploit the structure of $\mathbb{Z}_3$.

---

## Mathematical foundations

### Balanced ternary encoding

Each byte is expanded into **5 trits** because $3^5 = 243 > 256$.  The canonical digit set is $\{-1, 0, +1\}$ (stored as `i8`).  Conversion is ordinary base-3 decomposition followed by a shift:

```
val = byte
for i in 0..5:
    trit_i = (val % 3) - 1   // yields -1, 0, or +1
    val /= 3
```

### Sponge hash over $\mathbb{Z}_3$

The hash function is a **sponge-like construction** with a 24-trit state:

1. **Absorb** – each input trit is added to the state modulo 3.  After every full pass over the state the state is rotated left by one position.
2. **Squeeze** – three additional mixing rounds apply a linear combination `state[i] + state[(i+7) % 24]` modulo 3.

All operations are performed in the ring $\mathbb{Z}/3\mathbb{Z}$, i.e. arithmetic modulo 3 with representatives $\{-1,0,+1\}$.  The 24-trit output gives a digest space of $3^{24} \approx 2^{38}$ elements—small enough to run inside a student notebook, large enough to demonstrate structural properties.

### Proof of work on trits

Difficulty is measured by **leading zero trits**.  A block is valid when its hash begins with at least `difficulty` zeros.  Mining is a brute-force search over the 64-bit nonce field, exactly analogous to Bitcoin’s proof-of-work but with a ternary difficulty metric.

### Merkle tree

Transaction hashes form the leaves of a **binary Merkle tree**.  Each parent node is the ternary hash of the concatenation of its two children (the last child may stand alone if the level is odd).  The root is a 24-trit digest that binds the entire transaction list.

---

## Architecture

```
┌─────────────┐     ┌─────────────────┐     ┌──────────────┐
│   Bytes     │────▶│  bytes_to_trits │────▶│  Trit Vector │
└─────────────┘     └─────────────────┘     └──────────────┘
                                                     │
                              ┌────────────────────┘
                              ▼
┌─────────────────────────────────────────────────────────┐
│              ternary_hash (Sponge over Z₃)              │
│  ┌─────────┐   absorb (mod 3 add)   ┌─────────┐        │
│  │  State  │ ◄───────────────────── │  Trits  │        │
│  │ 24 trits│   rotate + squeeze     └─────────┘        │
│  └─────────┘                                            │
└─────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
┌───────────────┐    ┌─────────────────┐    ┌─────────────┐
│ Transaction   │───▶│  tx.hash()      │───▶│  Leaf Hash  │
│ TernaryTx     │    │  (24 trits)     │    │             │
└───────────────┘    └─────────────────┘    └─────────────┘
                                                     │
                              ┌────────────────────┘
                              ▼
                    ┌─────────────────┐
                    │ TernaryMerkle   │
                    │  binary tree    │
                    │  root = 24 trits│
                    └─────────────────┘
                              │
        ┌─────────────────────┘
        ▼
┌─────────────────────────────────────────────────────────┐
│ TernaryBlock                                            │
│  index | prev_hash | nonce | Merkle root | txs          │
│  compute_hash() -> ternary_hash()                       │
│  mine(difficulty) -> brute-force nonce for leading zeros│
└─────────────────────────────────────────────────────────┘
```

---

## Getting Started

Add to `Cargo.toml`:

```toml
[dependencies]
ternary-blockchain = { path = "." }
```

```rust
use ternary_blockchain::*;

fn main() {
    // 1. Create a transaction
    let tx = TernaryTransaction::send(100, "alice", "bob", 0);

    // 2. Build the genesis block
    let genesis = TernaryBlock::genesis();

    // 3. Mine a block with difficulty 1 (one leading zero trit)
    let mut block = TernaryBlock::new(1, vec![tx], genesis.hash.clone());
    block.mine(1);
    assert!(block.is_valid(1));

    // 4. Validate the chain
    assert!(validate_chain(&[genesis, block], 1));
    println!("Ternary chain is valid.");
}
```

Run it:

```bash
cargo run --example demo
```

---

## Running the Tests

The crate contains **12 tests**.  Each test demonstrates a specific cryptographic invariant:

| Test | What it proves |
|------|----------------|
| `bytes_to_trits_roundtrip` | Every byte expands to exactly 5 trits, each in `{-1, 0, +1}`. |
| `hash_deterministic` | The sponge is deterministic—identical input yields identical 24-trit output. |
| `hash_different_inputs` | Distinct byte strings produce distinct digests (sanity check for collision avoidance). |
| `hash_size` | Output length is fixed at 24 trits. |
| `leading_zeros_count` | Counts consecutive leading `0`-trits correctly for PoW difficulty adjustment. |
| `transaction_types` | Send (`-1`), hold (`0`), and receive (`+1`) variants encode the correct type tag. |
| `transaction_hash` | Transaction serialization feeds into the ternary hash and produces a 24-trit digest. |
| `genesis_block` | Block 0 has no transactions and its previous hash is the zero vector. |
| `mine_finds_nonce` | Even with difficulty 0, the mining loop computes a hash and marks the block valid. |
| `chain_validation` | A two-block chain passes linkage checks (`prev_hash` matches) and validity checks. |
| `merkle_tree` | Four transactions hash into four leaves, and the root is a 24-trit digest. |
| `merkle_empty` | An empty transaction set yields a well-defined zero root of 24 trits. |

Execute:

```bash
cargo test
```

---

## Related crates

- [ternary-zkp](https://github.com/SuperInstance/ternary-zkp) — Zero-knowledge proofs over GF(3)
- [ternary-secret-share](https://github.com/SuperInstance/ternary-secret-share) — Shamir secret sharing over $\mathbb{Z}_3$
- [ternary-hash](https://github.com/SuperInstance/ternary-hash) — Standalone trit-based hash utilities
- [ternary-consensus](https://github.com/SuperInstance/ternary-consensus) — Ternary consensus protocols

---

## License

MIT

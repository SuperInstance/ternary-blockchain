# ternary-blockchain

**Blockchain primitives using balanced ternary {-1, 0, +1} representations — hashing, Merkle trees, proof-of-work, and chain validation.**

## Background

Blockchain technology relies on three core cryptographic primitives: hash functions for data integrity, Merkle trees for efficient inclusion proofs, and proof-of-work for Sybil resistance. All mainstream blockchains (Bitcoin, Ethereum) operate on binary data — bits and bytes. But the mathematical foundations of hashing and proof-of-work are representation-agnostic: what matters is collision resistance, preimage resistance, and adjustable difficulty.

`ternary-blockchain` reimplements the full blockchain stack using **balanced ternary trits** {-1, 0, +1} instead of bits {0, 1}:

- **Ternary hashing** — a sponge-like construction absorbing trits
- **Ternary Merkle trees** — binary tree of paired trit hashes
- **Proof-of-work** — leading zero-trit difficulty adjustment
- **Transaction model** — ternary-typed transactions (send/hold/receive)
- **Chain validation** — hash linkage, PoW verification, balance checking

This is structurally similar to IOTA's ternary transaction model (though IOTA uses ternary trytes, 3-trit groups), and reflects the broader SuperInstance ecosystem's commitment to balanced ternary as its fundamental data representation.

## How It Works

### Ternary Hashing (`hash.rs`)

The hash function uses a sponge construction on a 27-trit state:

1. **Domain separation** — inject the input length as trits
2. **Absorption** — each input byte is converted to 5 balanced trits (3⁵ = 243 > 256) and absorbed into the state
3. **Mixing** — three rounds of nonlinear mixing: each trit is updated using its neighbors (`trit_add(prev[i], trit_mul(left, right))`)
4. **Rotation** — state is rotated after each round for diffusion
5. **Finalization** — a padding sentinel is absorbed

The output is a 27-trit `TritHash` (`[i8; 27]`). Helper functions provide ternary addition, multiplication, and rotation in Z₃ (balanced ternary arithmetic).

### Transactions (`transaction.rs`)

`TernaryTransaction` carries:

- **Value** — amount in arbitrary units
- **Type** — `Send` (−1), `Hold` (0), `Receive` (+1)
- **From/To** — 8-byte addresses

The `transactions_balanced()` function verifies that the sum of transaction type trits equals zero — a conservation law ensuring every send is matched by a receive.

### Blocks (`block.rs`)

`TernaryBlock` contains:

- **Index** — block height
- **Previous hash** — 27-trit linkage
- **Transactions** — list of ternary transactions
- **Nonce** — proof-of-work counter
- **Timestamp** — unix epoch seconds
- **Merkle root** — 27-trit root of the transaction Merkle tree

The `hash()` method computes the block hash over the serialized header. `is_valid()` checks both hash linkage and transaction balance.

### Merkle Trees (`merkle.rs`)

`TernaryMerkle` builds a binary Merkle tree over transaction hashes:

- **Root computation** — pairwise hash combination up to a single root
- **Proof generation** — sibling hash path for any leaf index
- **Proof verification** — reconstruct root from leaf + proof path

If the number of leaves is odd, the last leaf is duplicated (standard Merkle tree technique).

### Proof-of-Work (`pow.rs`)

`ProofOfWork::mine(block, difficulty)` increments the nonce until the block hash has `difficulty` leading zero-trits. This is directly analogous to Bitcoin's leading-zero-bits requirement, but in ternary space. The probability of a random trit being zero is 1/3 (vs. 1/2 for bits), so difficulty adjustment scales differently.

### Chain (`chain.rs`)

`TernaryChain` manages the blockchain:

- **Genesis initialization** — mine the first block
- **Block addition** — mine and append subsequent blocks
- **Validation** — verify entire chain: PoW, hash linkage, transaction balance for every block

Tampering with any block invalidates the chain from that point forward — the standard blockchain immutability guarantee.

## Experimental Results

The test suite validates:

- **Hash determinism** — same input produces same output
- **Hash sensitivity** — different inputs produce different outputs
- **Block hash changes with nonce** — proof-of-work is effective
- **Block validity** — correct prev_hash linkage required
- **Mining** — produces valid PoW at difficulty levels 1-2
- **Chain validation** — valid chains pass, tampered chains fail
- **Chain growth** — blocks are added correctly
- **Merkle consistency** — root is stable, different transactions produce different roots
- **Transaction balance** — balanced transactions accepted, unbalanced rejected
- **All hashes are trit-valued** — output values are in {-1, 0, +1}

## Impact

`ternary-blockchain` demonstrates that the entire blockchain stack can be rebuilt on balanced ternary foundations. The key insight is that hashing, Merkle trees, and proof-of-work are mathematical primitives that don't depend on binary representation.

The ternary proof-of-work has different economic properties than binary PoW:

- **Per-trit zero probability**: 1/3 (vs. 1/2 for bits)
- **Difficulty n requires ~3ⁿ attempts** (vs. ~2ⁿ for bits)
- **Faster difficulty growth per unit** — each difficulty level is harder relative to binary

This means ternary PoW can achieve equivalent security with smaller difficulty numbers.

## Use Cases

1. **Ternary ledger for fleet transactions** — Rooms in a SuperInstance fleet exchange resources (compute, storage, bandwidth). `TernaryTransaction` records these exchanges with ternary types (send/receive), and the chain provides an immutable audit log.

2. **Tamper-evident configuration history** — Fleet configuration changes are recorded as transactions on the chain. Any unauthorized modification is detectable through chain validation, providing a cryptographic audit trail.

3. **Byzantine fault-tolerant consensus** — Combined with `ternary-voting`, the blockchain serves as the consensus layer: rooms mine blocks containing agreed-upon transactions, with PoW providing Sybil resistance.

4. **Merkle-based inclusion proofs** — A room can prove that a specific transaction occurred without revealing the entire block contents. `TernaryMerkle::proof()` generates a compact sibling path, and `verify_proof()` confirms inclusion against the known root.

5. **Ternary cryptocurrency research** — The crate provides a sandbox for experimenting with ternary-based cryptocurrencies, studying how ternary hashing and PoW affect mining economics, block propagation, and security.

## Open Questions

- **Cryptographic security:** The sponge hash construction is custom and hasn't been formally analyzed for collision/preimage resistance. For production use, should the crate adopt a standardized ternary hash (e.g., Keccak variants) or formally prove security properties?
- **Consensus protocol:** The crate provides PoW but no full consensus protocol (no mempool, no fork resolution, no finality). Should it integrate with `ternary-voting`'s Byzantine agreement for a complete consensus stack?
- ** ternary signature scheme:** Transactions are not signed. Should the crate include a ternary digital signature scheme (e.g., ternary lattice-based signatures) for transaction authentication?

## Connection to Oxide Stack

`ternary-blockchain` is the trust layer:

- **`ternary-hash`** — the hash function is the foundation for blocks and Merkle trees
- **`ternary-protocol`** — blocks and transactions are serialized for network transport
- **`ternary-voting`** — Byzantine agreement provides the consensus mechanism for block finality
- **`ternary-channel`** — block propagation uses channel abstractions
- **`ternary-zkp`** — zero-knowledge proofs could enable private transactions (prove validity without revealing amounts/parties)
- **`ternary-game-theory`** — mining is a game; strategic behavior (selfish mining, block withholding) can be analyzed with game-theoretic tools

The 27-trit hash length (3³ trits) and 5-trits-per-byte encoding ensure that all blockchain data remains in the ternary domain, preserving consistency with the ecosystem's core representation.

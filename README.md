# ternary-blockchain

Balanced ternary blockchain primitives for the SuperInstance ecosystem.

## Features

- **TernaryHash** — sponge-style 27-trit hash; 3-round nonlinear mixing with byte-to-trit conversion
- **TernaryTransaction** — `{-1=Send, 0=Hold, +1=Receive}` transaction types with trit hash
- **TernaryBlock** — header bytes, Merkle root, deterministic hash, validity check
- **TernaryMerkle** — binary Merkle tree using ternary hash pairs; inclusion proofs + verification
- **ProofOfWork** — mine until hash has N leading zero-trits; `verify` to check
- **TernaryChain** — full chain with genesis, block addition, and end-to-end validation

## Usage

```rust
use ternary_blockchain::{TernaryChain, TernaryTransaction, TxType};

let mut chain = TernaryChain::new(2); // difficulty=2
chain.init_genesis(vec![
    TernaryTransaction::new(100, TxType::Send, [1u8; 8], [2u8; 8]),
    TernaryTransaction::new(100, TxType::Receive, [2u8; 8], [1u8; 8]),
]);
assert!(chain.validate());
```

## Tests

19 tests across hash, transaction, block, Merkle, PoW, and chain modules.

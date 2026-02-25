# IC42N Anchor Program

**IC42N** is the on-chain program powering an epoch-based prediction game on Solana.

The system derives winning numbers from finalized Solana blockchain data, ensuring outcomes are **transparent, deterministic, and verifiable** without oracles.

---

## ✨ Overview

IC42N enables players to submit predictions and compete for a shared prize pool.  
Each game epoch resolves deterministically using public blockchain data.

### Key Principles

- 🔐 **Trustless** — outcomes derived from Solana finalized data
- 🔍 **Verifiable** — anyone can independently confirm results
- ⚖️ **Fair** — no off-chain randomness or oracle manipulation
- 🚀 **On-chain** — resolution logic lives in the program

---

## 🧠 How Winning Numbers Are Determined

Winning numbers are derived from:

- finalized epoch slot
- finalized blockhash

This data is:

✔ publicly accessible  
✔ immutable  
✔ reproducible

Anyone can recompute the result locally.

### Verifier

👉 https://verify.iseefortune.com  
👉 https://github.com/IC42N/verifier

---

## 🏗 Program Information

**Program ID (mainnet)**

ic429goRDdS7BXEDYr2nZeAYMxtT6FL3AsB3sneaSu7

**Cluster:** mainnet-beta

---

## 📦 Repository Structure

programs/          Anchor program source  
target/idl/        Generated IDL (included for integration)  
tests/             Program tests  
Anchor.toml        Anchor configuration

---

## ⚙️ Build & Test

### Requirements

- Rust
- Solana CLI
- Anchor Framework

### Build

anchor build

### Run tests

anchor test

---

## 🚀 Deployment

anchor deploy

Ensure your Solana CLI is configured for the desired cluster.

---

## 🔍 Transparency & Verification

IC42N emphasizes auditability and public verification:

- deterministic result generation
- reproducible calculations
- public verifier tool
- on-chain resolution records

This approach ensures outcomes can be independently verified without trust assumptions.

---

## 🔐 Security Considerations

- No private randomness sources
- No oracle dependencies
- Deterministic outcome generation
- On-chain state validation

---

## 🧪 Development Notes

Local development uses `localnet` and `devnet` configurations defined in `Anchor.toml`.

IDL is included to support client integrations and review.

---

## 🏆 Hackathon Submission Notes

IC42N demonstrates:

- transparent on-chain game mechanics
- verifiable randomness derived from blockchain state
- trustless resolution design
- public verification tooling

---

## 📄 License

MIT

---

## 🤝 Contributing

Contributions and audits are welcome.

---

## 🌌 About IC42N

IC42N explores transparent, verifiable game mechanics built entirely on public blockchain data.

The project prioritizes fairness, auditability, and trustless design.

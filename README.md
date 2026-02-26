# ISeeFortune Anchor Program

**ISeeFortune** is the on-chain program powering an epoch-based
prediction game on Solana.

The system derives winning numbers from finalized Solana blockchain
data, ensuring outcomes are **transparent, deterministic, and
verifiable** without oracles.

------------------------------------------------------------------------

## ✨ Overview

ISeeFortune enables players to submit predictions and compete for a
shared prize pool.\
Each game epoch resolves deterministically using public blockchain data.

### Key Principles

-   🔐 **Trustless** --- outcomes derived from Solana finalized data
-   🔍 **Verifiable** --- anyone can independently confirm results
-   ⚖️ **Fair** --- no off-chain randomness or oracle manipulation
-   🚀 **On-chain** --- resolution logic lives entirely in the program

------------------------------------------------------------------------

## 🧠 How Winning Numbers Are Determined

Winning numbers are derived from:

-   Finalized epoch slot
-   Finalized blockhash

This data is:

✔ Publicly accessible\
✔ Immutable\
✔ Reproducible

Anyone can recompute the result locally.

------------------------------------------------------------------------

## 🔎 Public Verifier

-   🌐 https://verify.iseefortune.com\
-   🧾 https://github.com/IC42N/verifier

The verifier independently recomputes winning numbers using publicly
available Solana data.

------------------------------------------------------------------------

## 🏗 Program Information

**Program ID (mainnet-beta)**

    ic429goRDdS7BXEDYr2nZeAYMxtT6FL3AsB3sneaSu7

**Cluster:** mainnet-beta\
**Solana Version:** 2.3.0\
**Anchor Version:** 0.32.1

------------------------------------------------------------------------

## 🔍 Deterministic Verification

This program is fully verifiable using `solana-verify`.

### Verify From Source

On Apple Silicon:

    DOCKER_DEFAULT_PLATFORM=linux/amd64 solana-verify verify-from-repo   --library-name ic42n   -u https://api.mainnet-beta.solana.com   --program-id ic429goRDdS7BXEDYr2nZeAYMxtT6FL3AsB3sneaSu7   https://github.com/IC42N/iseefortune-anchor

On x86 systems:

    solana-verify verify-from-repo   --library-name ic42n   -u https://api.mainnet-beta.solana.com   --program-id ic429goRDdS7BXEDYr2nZeAYMxtT6FL3AsB3sneaSu7   https://github.com/IC42N/iseefortune-anchor

The command:

1.  Clones the repository
2.  Builds deterministically inside the official Solana verifiable
    Docker image
3.  Compares the produced `.so` hash against the deployed on-chain
    program
4.  Confirms a byte-for-byte match

If verification succeeds, the deployed program is cryptographically
confirmed to match this repository.

------------------------------------------------------------------------

## 📦 Repository Structure

    programs/           Anchor program source
    verifier-deps/      Dependency pinning for deterministic verification
    target/idl/         Generated IDL (included for integrations)
    tests/              Program tests
    Anchor.toml         Anchor configuration
    Cargo.lock          Deterministic dependency resolution

------------------------------------------------------------------------

## ⚙️ Build & Test

### Requirements

-   Rust (stable ≥ 1.85)
-   Solana CLI
-   Anchor Framework 0.32.1
-   Docker (for verifiable builds)

### Build

    anchor build

### Run Tests

    anchor test

------------------------------------------------------------------------

## 🚀 Deployment

    anchor deploy

Ensure your Solana CLI is configured for the desired cluster.

------------------------------------------------------------------------

## 🔐 Security

ISeeFortune emphasizes:

-   Deterministic result generation
-   No oracle dependencies
-   No private randomness
-   Strict account validation
-   Deterministic build verification
-   Upgrade authority controls

Responsible disclosure policy is defined in:

👉 `SECURITY.md`

On-chain security metadata is embedded using the Solana security.txt
standard.

------------------------------------------------------------------------

## 🏆 Hackathon Notes

ISeeFortune demonstrates:

-   Transparent on-chain game mechanics
-   Verifiable randomness derived from blockchain state
-   Deterministic build verification
-   Trustless resolution design
-   Public verification tooling

------------------------------------------------------------------------

## 📄 License

MIT

------------------------------------------------------------------------

## 🌌 About

ISeeFortune explores transparent, verifiable game mechanics built
entirely on public blockchain data.

The project prioritizes fairness, auditability, and trustless design.

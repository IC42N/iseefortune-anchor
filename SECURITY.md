# Security Policy

## Supported Versions

The current mainnet deployment of **ISeeFortune** is actively maintained and
monitored.

Security updates and patches apply to: - The latest deployed mainnet
program - The `main` branch of this repository

Older deployments or forks are not guaranteed to receive updates.

------------------------------------------------------------------------

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly.

**Do NOT create a public GitHub issue for security vulnerabilities.**

Instead, report privately via:

-   📧 Email: contact@iseefortune.com

Please include:

-   A clear description of the vulnerability
-   Steps to reproduce (if applicable)
-   Affected program accounts or instructions
-   Potential impact assessment
-   Suggested remediation (if known)

------------------------------------------------------------------------

## Response Timeline

-   Initial acknowledgment within **72 hours**
-   Status update within **7 days**
-   Resolution timeline depends on severity and complexity

Critical vulnerabilities affecting user funds or core program invariants
will be prioritized immediately.

------------------------------------------------------------------------

## Scope

This policy covers:

-   The ISeeFortune on-chain Anchor program
-   Deterministic resolution logic
-   PDA account validation
-   Epoch result derivation logic
-   Prize pool accounting logic

This policy does not cover:

-   Third-party wallets
-   RPC providers
-   Frontend hosting infrastructure
-   External integrations

------------------------------------------------------------------------

## Disclosure Policy

We follow responsible disclosure principles:

-   Reported vulnerabilities will be investigated confidentially
-   Coordinated disclosure may occur after remediation
-   Public acknowledgment may be offered at the reporter's request

------------------------------------------------------------------------

## On-Chain Security Model

ISeeFortune is designed with the following principles:

-   Deterministic result generation from finalized Solana data
-   No oracle dependencies
-   No private randomness
-   Strict account validation
-   Deterministic build verification via solana-verify
-   Upgrade authority controls enforced on-chain

------------------------------------------------------------------------

## Program Information

Program ID (mainnet-beta):

`ic429goRDdS7BXEDYr2nZeAYMxtT6FL3AsB3sneaSu7`

------------------------------------------------------------------------

Thank you for helping improve the security of ISeeFortune.

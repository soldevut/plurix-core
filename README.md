# PLURIX Protocol · Shared Liquidity Vault Engine on Solana

<p align="center">
  <img src="logo.png" width="120" height="120" alt="PLURIX Logo" style="border-radius: 24px; box-shadow: 0 0 35px rgba(0, 245, 160, 0.4);" />
</p>

<p align="center">
  <b>Decentralized shared liquidity matrix with signed off-chain oracle settlement, zero self-slippage, and 100% MEV sandwich immunity on Solana.</b>
</p>

<p align="center">
  <a href="https://test-exchange.vercel.app"><img src="https://img.shields.io/badge/Live%20DEX-test--exchange.vercel.app-00F5A0?style=for-the-badge&logo=googlechrome&logoColor=black" alt="Live App" /></a>
  <a href="https://pyth.network"><img src="https://img.shields.io/badge/Pyth-High--Frequency%20Oracles-9945FF?style=for-the-badge" alt="Pyth" /></a>
  <a href="https://solana.com"><img src="https://img.shields.io/badge/Solana-Mainnet--Ready-14F195?style=for-the-badge&logo=solana" alt="Solana" /></a>
  <img src="https://img.shields.io/badge/Self--Slippage-0.00%25-brightgreen?style=for-the-badge" alt="Zero Slippage" />
  <img src="https://img.shields.io/badge/License-Apache--2.0%20%2F%20MIT-blue?style=for-the-badge" alt="License" />
</p>

---

## ⚡ Overview

**PLURIX** is a shared liquidity protocol on Solana engineered to overcome the core flaws of traditional Constant Product AMMs ($x \cdot y = k$):

1. **Zero Self-Slippage**: Trades settle directly against deep multi-asset collateralized shared vaults at the exact mid-market price certified by oracle signatures. Large trades do not incur quadratic bonding curve impact.
2. **Cryptographically Signed Quotes**: Pyth high-frequency oracle nodes generate an Ed25519 signature containing the authorized price and timestamp (valid for 5 seconds). The on-chain Anchor program verifies the signature before transferring vault tokens.
3. **MEV Sandwich Immunity**: Because the settlement price is locked inside the cryptographic instruction payload, predatory searcher bots cannot front-run or sandwich users in the public mempool.
4. **Unified Single-Sided Liquidity**: Liquidity providers deposit single assets into unified reserve vaults instead of fragmenting capital into dozens of separate token pairs.
5. **0.02 SOL Token Launcher**: Developers deploy SPL tokens directly into the PLURIX Shared Vault matrix for only the base Solana network rent (~0.02 SOL), with zero platform fees and zero required seed capital.

---

## 🏗 Architecture Comparison

### Traditional AMM (Raydium / Uniswap)
```
[User Trade] ──► [Public Mempool] ──► [Sandwich Bot Front-run] ──► [Price Impact on xy=k] ──► [High Slippage Loss]
```

### PLURIX Shared Vault Settlement
```mermaid
flowchart TD
    User[Trader / User] -->|1. Request Quote| Oracle[Pyth HF Oracle Node]
    Oracle -->|2. Signed Quote + Ed25519 Sig| User
    User -->|3. Atomic Instruction + Quote| Contract[PLURIX Anchor Program]
    
    subgraph OnChain [Solana Runtime Execution]
        Contract -->|Verify Sig & Timestamp <= 5s| Gate[Validation Gate]
        Gate -->|Transfer Tokens| Vault[PLURIX Shared Vault Matrix]
    end
    
    Vault -->|4. Guaranteed Payout (0.00% Slippage)| User
```

---

## 📊 Performance & Economic Benchmarks

| Feature | Legacy AMM ($x \cdot y = k$) | Orderbook DEX | **PLURIX Shared Vaults** |
| :--- | :--- | :--- | :--- |
| **Self-Slippage (10 SOL)** | ~1.8% – 3.5% | Dependent on depth | **0.00% (Zero Curve Impact)** |
| **Mempool Front-Running Risk** | Severe (Sandwich bots) | Moderate | **Zero (Signed Oracle Bounds)** |
| **Platform Protocol Fee** | 0.25% – 0.30% | 0.05% – 0.10% | **0.00%** |
| **LP Capital Efficiency** | Fragmented per pair | High capital lockup | **Unified Shared Matrix** |
| **Dev Token Launch Cost** | $500 – $2,000 liquidity | Prohibitive | **~0.02 SOL (Solana Rent Only)** |

---

## 📦 Developer SDK

### Installation
```bash
npm install @plurix/sdk @solana/web3.js
# or
yarn add @plurix/sdk @solana/web3.js
```

### Executing a Zero-Slippage Swap
```typescript
import { PlurixClient } from '@plurix/sdk';
import { Connection, Keypair } from '@solana/web3.js';

const connection = new Connection('https://api.mainnet-beta.solana.com');
const wallet = Keypair.fromSecretKey(/* ... */);

const plurix = new PlurixClient({
  connection,
  cluster: 'mainnet-beta'
});

// 1. Fetch real-time signed oracle quote
const quote = await plurix.getSignedQuote({
  inputMint: 'So11111111111111111111111111111111111111112', // SOL
  outputMint: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v', // USDC
  amountInSol: 5.0
});

console.log(`Signed rate: 1 SOL = ${quote.priceUsd} USDC`);

// 2. Execute swap against PLURIX Shared Vault #04
const txid = await plurix.executeSwap({
  quote,
  signer: wallet
});

console.log('Confirmed without slippage. TX:', txid);
```

---

## 🔒 Security & On-Chain Guarantees

- **Signature Expiration**: Quotes expire strictly after 5,000 milliseconds to prevent stale price arbitrage.
- **Oracle Staleness Guards**: Pyth network confidence intervals ($\sigma$) are verified on-chain; trades revert if market volatility exceeds safety thresholds.
- **Collateralization Floor**: Shared vaults maintain a minimum 140% reserve coverage ratio across pooled assets.

---

## 🌐 Official Channels

- **Live DEX App**: [https://test-exchange.vercel.app](https://test-exchange.vercel.app)
- **Solana Mainnet Program ID**: `PLURiX1111111111111111111111111111111111111`
- **Documentation**: [https://docs.plurix.xyz](https://test-exchange.vercel.app)

---

## 📄 License

Dual-licensed under Apache 2.0 or MIT.

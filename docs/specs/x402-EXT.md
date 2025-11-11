# X402-EXT: TGP / x402 Integration Profile

**Status:** Draft  
**Based on:** x402 v0.3 (Coinbase, 2024)  
**Author:** Ledger of Earth  

This document defines how the Transaction Gateway Protocol (TGP-00)
interfaces with the x402 Payment Session layer.

It does not reproduce the x402 spec.
Canonical source: <https://github.com/coinbase/x402>

## Purpose
- Define mapping between TGP `SETTLE` messages and x402 payment sessions.
- Specify routing metadata carried in TxIP headers identifying x402 endpoints.
- Describe how a TBC appliance initiates and monitors x402 sessions.
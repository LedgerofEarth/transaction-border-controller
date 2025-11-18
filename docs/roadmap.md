===========================================================
                TRANSACTION BORDER CONTROLLER
                       ROADMAP (ASCII)
===========================================================

M0 — FOUNDATION & BOOTSTRAP  (✓ Completed)
————————————————————
• Repo + project structure established
• Health endpoint, config loader, logging
• Local dev harness + diagnostics
• Initial TGP scaffolding

M1 — TGP LAYER-8 SIGNALING CORE  (✓ Completed)
————————————————————
• TGP-00 v3.1 implemented
• TGP message parsing + envelope validation
• HTTP 402 + Direct Pay flow foundations
• Extension → TBC interface defined
• Economic Envelope schema established

M2 — CONSUMER SECURITY GATEWAY  (In Progress)
————————————————————
• Full Onion Security Model (Layers 1–5)
• Merchant registry + signature checks
• Multi-RPC contract bytecode verification
• Optional ZK attestation layer
• Fail-closed rule engine
• Economic Envelope generator
• TBC error schema + error codes
• Wallet handoff boundary enforcement

M3 — MULTI-CHAIN & X402 INTEGRATION
————————————————————
• Multi-chain EVM support (PLS, Base, ETH testnets)
• RPC quorum verification engine
• Chain-aware envelope generation
• Coinbase x402 compatibility layer
• Preflight settlement routing

M4 — MERCHANT ECOSYSTEM INTEGRATION
————————————————————
• CBS backend integration (contract factory)
• Immutable contract template registry
• Payment Profile metadata signing
• Merchant Portal integration
• QR generation + integration snippets
• Verified Merchant registry hooks

M5 — PRODUCTION-GRADE APPLIANCE (Telecom-Grade)
————————————————————
• Hardened Docker/Kubernetes deployment
• High-availability verification cluster
• Rate limiting + anomaly detection
• Enterprise policy engine
• Observability (metrics/logs/traces)
• Configurable verification profiles
• Contract-template registry update system

===========================================================
      “VERIFY FIRST. ROUTE ALWAYS.” — THE COREPROVE WAY
===========================================================
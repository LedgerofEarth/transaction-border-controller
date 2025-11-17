📚 Documentation Overview — TBC / TGP Project

Version: 0.4
Repository Component: /docs
Author: Ledger of Earth

This directory contains the complete documentation set for the Transaction Border Controller (TBC) and the Transaction Gateway Protocol (TGP). It serves as the canonical entry point for anyone seeking to understand the architecture, specifications, topology, and development roadmap of the system.

This folder intentionally complements—but does not duplicate—the detailed technical specifications in /specs.

⸻

🧭 Documentation Structure

The documentation is organized into the following major categories:

/docs
   /architecture     # System topology, architecture diagrams, high-level models
   /analysis         # Engineering reviews, notes, test scaffolding insights
   /roadmap          # Rebuild plans, implementation tracks, timelines
   context_summary.md
   README.md          <-- (this file)

Each section is designed to give contributors and partners a progressively deeper understanding of the system.

⸻

🔷 1. Architecture Documentation

The architecture folder contains high-level documents describing the conceptual structure of the TBC system and its relation to the broader blockchain and agentic ecosystem.

Key files:

architecture.md

Defines the overall TBC/TGP architecture, including:
	•	Layer-8 control plane model
	•	Separation of Client, TBC, Wallet, and Settlement Contract
	•	Design goals
	•	Responsibilities of each subsystem
	•	Relationship to wallets and agent frameworks

system_topology.md

End-to-end topology for the full transaction pipeline:

Applications (x402)
  → TGP Client (extension)
  → TBC Gateway
  → Wallet
  → Settlement Contract
  → Blockchain

Includes diagrams, trust boundaries, and chain interactions.

(future) diag/

Optional folder for diagrams, sequence charts, and architecture visuals.

⸻

🔷 2. Specifications (Located in /specs)

While not stored in the /docs folder, the /specs directory is core to the architecture.
This documentation hub links to the spec hierarchy:

Core Specifications
	•	TxIP-00 — Signaling primitive
	•	TGP-00 — Query/Ack protocol
	•	TGP-CP-00 — Client runtime
	•	TGP-EX-00 — Browser extension
	•	TBC-00 — TBC server API
	•	x402-EXT — Integration with x402 agents

Appendices
	•	TGP-01: Economic Envelope
	•	TGP-POS-00: Proof of Settlement
	•	CoreProver Settlement Model

Additional API
	•	TBC-MGMT-API-00

The architecture docs explain these, while the /specs folder defines them.

⸻

🔶 3. Analysis & Engineering Notes

/docs/analysis

Contains deep technical reviews, investigations, and engineering notes used during protocol development.

Examples include:
	•	Rust analysis of early TxIP implementations
	•	Contract verification notes
	•	ABI and session handling breakdowns

These are non-normative (not binding specs), but important for understanding design choices and implementation challenges.

⸻

🔶 4. Roadmap & Rebuild Plans

/docs/roadmap

This directory tracks the real engineering plan for implementing TGP/TBC.

Key documents:

Rebuild.md

Defines the current implementation path:
	•	TGP Client runtime
	•	Browser Extension
	•	TBC Gateway
	•	Payment Profile Contract
	•	End-to-end demo plan
	•	Agent integration path

It reflects the most recent architecture consolidation.

Implementation Phasing Documents

Deep dives into:
	•	core milestones
	•	demo sequencing
	•	integration order
	•	dependency tracking

This folder is the engineering control center for delivering TBC/TGP.

⸻

🔷 5. Context Summary

context_summary.md

Provides a quick “You Are Here” view for new contributors:
	•	Architecture status
	•	Short-term goals
	•	Long-term vision
	•	Implementation checkpoints

Useful for onboarding and syncing contributors.

⸻

🎯 Intended Audience

This documentation hub is for:
	•	protocol engineers
	•	agent framework developers
	•	wallet developers
	•	network operators
	•	researchers
	•	early adopters and partners
	•	YC / investor technical reviewers

It ties the specifications to real implementation plans.

⸻

🧱 Documentation Philosophy
	1.	Architecture-first — high-level docs define the system before specs.
	2.	Spec-driven — normative specs in /specs set precise expectations.
	3.	Separation of concerns — Clients, TBC, Wallets, Contracts remain cleanly partitioned.
	4.	Extensibility — appendices and analysis documents evolve as needed.
	5.	Transparency — all decisions and design rationale live in this folder.

⸻

📌 Next Steps (Documentation)

Future additions planned:
	•	Architecture diagrams (mermaid or SVG)
	•	A full “How Payments Flow” guide
	•	Merchant Integration Guide
	•	Wallet Integration Guide (Presence API)
	•	TBC Deployment Guide
	•	ZK-enabled settlement receipt documentation

⸻

📎 Where to Start

New contributors should begin here:
	1.	/docs/architecture/architecture.md
	2.	/docs/architecture/system_topology.md
	3.	/specs/TGP-00.md
	4.	/specs/TBC-00.md
	5.	/docs/context_summary.md
	6.	/docs/roadmap/Rebuild.md

This path gives a complete understanding of the project in fewer than 30 minutes.

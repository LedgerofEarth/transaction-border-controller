# TGP-00 v3.1 Patch Application Summary

**Date:** November 18, 2025  
**Base Version:** 3.0  
**Target Version:** 3.1  
**Method:** Patch-only update (no wholesale rewrite)

——

## Patches Applied

### ✅ Patch Category 1: Structure & Narrative Cleanup

**Changes:**

- Moved discount/coupon mechanics to Appendix B
- Restructured Section 1 with hierarchy: actors → interaction patterns → responsibilities matrix → topology
- Added Section 1.2 (Interaction Patterns) with comparative table for HTTP 402 vs Direct Pay
- Added Section 1.3 (System Responsibilities Matrix) showing deployment/payment/settlement phases
- Restructured Section 2 with overview → interaction model approach
- Added Section 2.1 (Overview) with shared payment initiation semantics
- Reduced redundancy between HTTP 402 and Direct Pay descriptions
- Added domain separation taxonomy (profile/transaction/verification domains)
- Renumbered sections to accommodate new structure

**Result:** Clearer narrative flow from abstract → actors → patterns → technical details

### ✅ Patch Category 2: CBS vs TBC Responsibility Clarification

**Changes:**

- Added explicit “CBS is merchant-side contract lifecycle system only” statement
- Added Section 1.3 (System Responsibilities Matrix) with phase separation
- Added pre-deployment vs post-deployment architecture diagrams
- Strengthened CBS boundaries: “NO consumer-facing interface, NO participation in TBC verification, NO settlement execution”
- Added TBC boundary: “Consumer-side security gateway with no merchant-facing interface”
- Added normative rule: “CBS MUST NOT have programmatic access to TBC verification endpoints”
- Added verification clarification: “TBC MUST independently verify; CBS metadata is advisory only”
- Updated Section 1.5 (Key Architectural Boundaries) with stronger isolation language

**Result:** Absolute separation of CBS (merchant/pre-deployment) and TBC (consumer/runtime) domains

### ✅ Patch Category 3: TBC Onion Security Model Refinement

**Changes:**

- Added REQUIRED vs OPTIONAL layer taxonomy at section header
- Added single-sentence purpose statements for each layer emphasizing fail-closed behavior
- Added Section 3.1 (Layer Evaluation Semantics) with normative sequencing rule
- Added explicit rejection cases for ambiguous/partial RPC results
- Strengthened Layer 1: Added fail-closed rule for registry unavailability
- Strengthened Layer 2: Clarified merchant-signed descriptors as advisory (not authoritative for contract verification)
- Strengthened Layer 3:
  - Added canonical algorithm with MUST requirements
  - Added deterministic multi-RPC consistency requirements
  - Added quorum M-of-N consensus specification
  - Added rejection cases for RPC disagreement
- Updated Layer 4: Marked as OPTIONAL, added configuration thresholds
- Updated Layer 5: Marked as REQUIRED, added all transaction MUST pass policy validation
- Added Section 3.3 (Normative Verification Sequencing) with mandatory sequence and fail-closed semantics
- Added implementation notes referencing TGP-TBC-SEC-00 for details
- Removed redundant descriptions across layers

**Result:** Clear normative specification with sequential evaluation, explicit failure conditions, and separation of semantics from implementation

### ✅ Patch Category 4: Economic Envelope & Messaging Cleanup

**Changes:**

- Replaced Section 4 with structured messaging section
- Added Section 4.1 (Overview) with normative rule on envelope authority
- Added Section 4.2 (TGP Message Type Registry) with all core message types
- Added Section 4.3 (QUERY Message Schema) with REQUIRED/OPTIONAL field taxonomy
- Added domain separation: profile/transaction/verification domains
- Added Section 4.4 (Economic Envelope Schema) with REQUIRED/OPTIONAL fields separated
- Added domain separation for envelope fields
- Added Consumer Safety Requirements (5 mandatory checks)
- Added Section 4.5 (Error Response Schema) with mandatory error schema
- Added error type registry with retry-allowed flags
- Added extension behavior specifications
- Added Section 4.6 (SETTLE Message Schema) reference
- Clarified wallet handoff is outside TGP scope
- Removed CBS participation references for envelope formation
- Added normative: “CBS does NOT participate in envelope formation”

**Result:** Complete message specification with field taxonomies, error handling, and clear scope boundaries

### ✅ Patch Category 5: Payment Profile & Contract Layer Clarification

**Changes:**

- Replaced Section 5 with structured payment profiles section
- Added Section 5.1 (Definition) with normative rule: “merchant-authored, CBS-deployed, TBC-verified immutable rule-set”
- Added Section 5.2 (Minimal Canonical Schema) with REQUIRED/OPTIONAL field separation
- Added 4 normative rules for profiles (immutable, adminless, TBC independent verification, metadata advisory)
- Added TBC Rejection Criteria (5 specific conditions)
- Added Section 5.3 (Contract Lifecycle Diagram) showing CBS deployment → TBC verification separation
- Added “Separation of Concerns” list with explicit no-overlap rule
- Updated Section 5.4 (Profile Deployment via CBS) with “CBS Does NOT” list
- Clarified CBS participation ends at deployment
- Moved example profiles to integration guide (referenced in appendix)
- Added separation between profile metadata, contract template, and runtime state

**Result:** Minimal, implementation-agnostic profile specification with clear CBS/TBC separation

### ✅ Patch Category 6: State Machine Extraction (TGP-ENGINE-00)

**Changes:**

- Replaced Sections 6-7 (State Machine & Receipt System) with Section 6 (Engine Overview)
- Added Section 6.1 (Signaling vs Settlement Separation) with normative demarcation
- Added Section 6.2 (High-Level Settlement Flow) with state summary
- Added Section 6.3 (Event Messages) with reference to TGP-ENGINE-00
- Removed detailed state transitions, timing windows, withdrawal logic
- Removed detailed receipt metadata and ZK proof schemas
- Removed detailed discount mechanics (moved to Appendix B)
- Added “For Complete Details: See TGP-ENGINE-00” references throughout
- Kept high-level overview only (6-step flow, 4 key states, alternative paths)

**Result:** TGP-00 focuses on signaling; TGP-ENGINE-00 handles settlement details

——

## Document Structure Changes

### Section Renumbering

**v3.0 Structure:**

1. Architecture Overview
1. Consumer Payment Flows
1. TBC Security Model
1. Message Types
1. Payment Profiles
1. State Machine
1. Receipt System
1. Security Considerations
1. Examples
1. Implementation Checklist
1. Migration & Versioning
1. Glossary
1. References

**v3.1 Structure:**

1. Architecture Overview (expanded with actors, patterns, matrix)
1. Payment Initiation (restructured with unified semantics)
1. TBC Security Model (refined with normative rules)
1. Economic Envelope & Messaging (new structured section)
1. Payment Profiles (minimal canonical schema)
1. Engine Overview & Settlement Lifecycle (extracted, high-level only)
1. Security Considerations (preserved)
1. Examples (moved to Appendix A reference)
1. Implementation Checklist (preserved, renumbered)
1. Migration & Versioning (updated with v3.1 changes)
1. Glossary (preserved)
1. References (preserved)
   Appendix A: Integration Examples (reference)
   Appendix B: Discount Coupon Mechanics (extracted from main body)

——

## Content Preserved

The following content was **preserved unchanged**:

- All glossary entries
- All references and external standards
- Implementation repositories list
- Website & resources section
- Motto
- Migration notes (updated with v3.1 additions)
- Security attack vectors and mitigations (reorganized but content intact)
- Privacy considerations
- Implementation checklists (structure preserved)

——

## Specifications Referenced

**New References Added:**

- TGP-ENGINE-00: Settlement engine state machine and lifecycle
- TGP-TBC-SEC-00: Detailed TBC implementation and RPC quorum algorithms
- Integration Guide: Detailed examples and merchant/consumer integration

**Existing References:**

- TGP-MGMT-00: Management protocol
- TGP-CP-EXT-00: Browser extension interface
- TGP-02: ZK proof circuits
- TGP-03: Receipt vault implementation

——

## Key Improvements

1. **Clarity:** Hierarchical structure from abstract → actors → patterns → technical
1. **Separation:** Absolute CBS/TBC domain separation with no overlap
1. **Normative:** Explicit MUST/OPTIONAL rules, fail-closed semantics, sequential evaluation
1. **Modularity:** Extracted state machine to TGP-ENGINE-00, discount mechanics to Appendix B
1. **Completeness:** Added error schemas, field taxonomies, rejection criteria
1. **Implementation-agnostic:** Separated semantics from implementation details

——

## Validation

**Patch Application Method:** Sequential string replacement (no wholesale rewrite)  
**Content Loss:** None (all v3.0 content preserved or extracted to appendices/subspecs)  
**Structural Integrity:** Maintained (section numbering, cross-references updated)  
**Tone:** Preserved (formal specification language throughout)

——

END OF SUMMARY
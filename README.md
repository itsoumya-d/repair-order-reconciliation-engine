# repair-order-reconciliation-engine

[![CI](https://github.com/itsoumya-d/repair-order-reconciliation-engine/actions/workflows/ci.yml/badge.svg)](https://github.com/itsoumya-d/repair-order-reconciliation-engine/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Deterministic collision repair order parts reconciliation and supervised human-in-the-loop (HITL) delegation engine written in Rust.

## Problem Statement

In automotive collision repair and workshop estimating, repair lines must be resolved from damaged assembly descriptions to exact purchasable part numbers and verified interchange alternatives. 

Manual reconciliation of dozens of line items per repair order creates estimating bottlenecks and purchasing delays. However, fully automated purchasing poses severe financial and safety risks if uncertified or incorrect structural components are ordered.

This engine provides a high-reliability, confidence-scored reconciliation pipeline with automated policy guardrails:
1. **Confidence-Scored Match Resolution**: Evaluates candidate replacement parts with fitment compatibility, condition, and interchange equivalence scoring.
2. **Supervised Delegation Policy**: Automatically qualifies repair lines with high confidence (>= 95%) for straight-through purchasing while routing ambiguous, safety-critical, or low-confidence matches to human estimator review.
3. **Audited Resolution State Machine**: Tracks reviewer decisions (Approve, Override, Reject) with explicit audit rationale and timestamps.
4. **Order Batch Manifest Generation**: Compiles validated line items into structured purchase order manifests with cost totals, unit quantities, and confidence breakdowns.

## State Machine Architecture

```text
Repair Line Item
       │
       ▼
[Score Candidates] ── (Condition, Interchange, OEM Equivalence)
       │
       ├─ Score >= 0.95 && Non-Safety-Critical ──► [AutoApproved] ──────────┐
       │                                                                    │
       └─ Score < 0.95 || OEM Safety-Critical  ──► [PendingReview]          │
                                                         │                  │
                                            ┌────────────┼────────────┐     │
                                            ▼            ▼            ▼     │
                                       [Approved]   [Overridden]  [Rejected]│
                                            │            │                  │
                                            └────────────┴──────────────────┤
                                                                            ▼
                                                                [Purchase Order Manifest]
```

## Quick Start

### Prerequisites
- Rust 1.75+ (Cargo)

### Build & Run Tests

```bash
# Clone the repository
git clone https://github.com/itsoumya-d/repair-order-reconciliation-engine.git
cd repair-order-reconciliation-engine

# Run test suite
cargo test --verbose

# Build optimized release binary
cargo build --release
```

## Test Suite Summary

- `tests::test_auto_approval_high_confidence`: Validates straight-through automated purchasing for high-confidence non-critical components.
- `tests::test_safety_critical_requires_review`: Verifies that OEM safety-critical parts (sensors, structural beams, airbags) are never auto-purchased without human review.
- `tests::test_supervisor_override`: Validates human estimator override workflows with required audit logging.
- `tests::test_manifest_aggregation`: Ensures correct line-item consolidation, quantity arithmetic, and total cost computation.

## License

MIT License. Copyright (c) 2026 Soumya Debnath.

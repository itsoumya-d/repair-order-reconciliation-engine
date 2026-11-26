# repair-order-reconciliation-engine

Deterministic collision repair order parts reconciliation and supervised human-in-the-loop (HITL) delegation engine.

## Overview

In automotive collision repair and workshop estimating, repair lines must be resolved from damaged assembly descriptions to exact purchasable part numbers and verified interchange alternatives. 

This engine implements:
1. **Confidence-Scored Match Resolution**: Evaluates candidate replacement parts with fitment compatibility, condition, and interchange equivalence scoring.
2. **Supervised Delegation Policy**: Automatically qualifies repair lines with high confidence (>= 95%) for straight-through purchasing while routing ambiguous, safety-critical, or low-confidence matches to human estimator review.
3. **Audited Resolution State Machine**: Tracks reviewer decisions (`Approve`, `Override`, `Reject`) with explicit rationale and timestamps.
4. **Order Batch Manifest Generation**: Compiles validated line items into structured purchase order manifests with cost totals and confidence breakdowns.

## Usage

```bash
cargo test
cargo build --release
```

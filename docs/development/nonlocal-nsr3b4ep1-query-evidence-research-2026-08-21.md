# NSR3-B4EP1 query-evidence separation research -- 2026-08-21

Status: `COMPLETE / WORK_ONLY_TRANSACTION_POLICY_SELECTED / NO_PHYSICS_CHANGE`

## Problem

B4EP0 attributes 41.36% sampled self time to SHA-256. The hot path always
constructs `joint_workspace_hash`, which serializes every fluid/support
position plus pair and pressure-tape hashes. This happens for all 227 B4E1M
workspaces even though inner hashes are research provenance, not inputs to the
energy, gradient, HVP, KKT decision or canonical publication.

The final frame/trajectory/ledger hashes must remain. The B4E0 parent
preflight must also retain its exact full workspace root. The separable cost is
only the transient transaction's per-current/per-trial state evidence.

## Selected policy

Add an internal-only workspace evidence policy to `JointQueryTrace`:

- `FULL_STATE` is the default and preserves every existing byte and test;
- `WORK_ONLY` skips `joint_workspace_hash`, `joint_pair_hash` and
  `joint_pressure_tape_hash` for transient query state;
- work-only still records kind, pair/directed/active counts, cell tests, tape
  payload and lifecycle into a separately domain-tagged deterministic chain;
- counters expose full hashes computed and full hashes skipped.

The policy does not skip neighborhood, evaluation, tape, HVP, publication,
ledger or final output hashes. It creates no runtime/public option; only a
dedicated research command selects work-only for the B4E1M transaction. The
parent preflight remains full-state.

## Correspondence gate

The candidate must reproduce the frozen B4E1M solver decisions and durable
physics exactly:

- levels 14/28, selected level 1, 42 attempted and 28 accepted substeps;
- 221 outer trials, zero rejects, 411 nonlinear and 48 spectral HVPs;
- exact frame, aggregate, trajectory and both ledger roots;
- exact strain, energy, penetration, canonical aggregate and ownership facts;
- 226 transaction workspace hashes skipped, zero computed; one full parent
  workspace hash computed separately;
- zero all-pairs work and zero final ownership.

B4E1M itself must remain byte-identical. Candidate reports repeat byte-exactly
twice; their work-chain root is new evidence and is not compared to the
full-state query-chain root.

## Performance experiment

Use one Release binary and six fresh timed processes in fixed alternating
order: `FULL/WORK_ONLY`, `WORK_ONLY/FULL`, `FULL/WORK_ONLY`. Keep reports and
timing separate. Require work-only to win all three positional pairs and its
median process speedup to be at least `1.10x`.

This threshold validates a material engineering win but cannot close the
corpus gap. From B4EP0, deleting the entire SHA share has only a `~1.70x`
ceiling. HVP and neighborhood/topology work remain mandatory next research
regardless of B4EP1 PASS.

## Decision

Freeze B4EP1 as a query-evidence-policy ablation with no mathematical or
temporal changes. PASS retains work-only as the nominal research hot-path
candidate and authorizes residual-cost profiling/B4EP2 design only. B4E2,
runtime and production remain blocked.

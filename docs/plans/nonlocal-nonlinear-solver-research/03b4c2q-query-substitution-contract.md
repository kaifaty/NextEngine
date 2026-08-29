# NSR3-B4C2Q -- one-substep joint pressure query substitution

Status: `PASS / JOINT_PRESSURE_KKT_QUERY_CANDIDATE / B4C2T_DESIGN_AUTHORIZED`

Parent B4C1 selects `JOINT_PRESSURE_RADIUS_TAPE_CANDIDATE`; semantic SHA-256
is `b7b05aa735f2b8153013cb02f7eab220e575df187061a5fd6663ec63d136c1d7`
and JSON-without-final-LF SHA-256 must equal
`8a2c27833953297e1bdffabe480e05d849a39271e6e147d0a01d79e831218182`.

## Identity

```text
joint-pressure-kkt-query-substitution-r0
```

Every candidate forecast, current and projected-trial pressure query builds a
B4C0R one-pass neighborhood and one B4C1 tape at that query's exact binary64
position. Candidate HVPs use only that tape. All-pairs evaluation/HVP calls
are forbidden except in separately counted audit-oracle comparisons.

## Query and transaction gates

For every candidate workspace require:

- joint density, energy, complete fluid/support gradient, active count and
  pair counts equal the all-pairs oracle bit-for-bit at the same position;
- all taped HVP outputs equal the audit oracle bit-for-bit;
- the state digest covers binary64 positions, pair digest and tape digest;
- one neighborhood and one tape build occur per evaluation query;
- every projected trial has a distinct owned workspace and rebuild counter;
- acceptance promotes position, KKT state, neighborhood, tape and digest
  together;
- rejection publishes none of the trial state and leaves current arrays and
  digest bit-for-bit unchanged.

The P1 active macro forecast must perform exactly 48 taped Lanczos calls and
match the all-pairs spectral estimate bit-for-bit. P2 detached forecast must
take the exact inactive zero-HVP path. Both forecast queries are read-only:
fixture positions, velocities and committed solver state remain unchanged.

## One-substep corpus

Compare legacy all-pairs and joint-tape KKT solves on:

| Case | Start state | Timestep |
|---|---|---:|
| P1 initial | frozen supported column | `frame/21` |
| P1 forecast active | clamped macro forecast | `frame/42` |
| P2 detached | frozen released block | `frame` |
| P1 compressed | fluid positions scaled `0.99` about their mean | `frame/48` |

Require exact pass/failure, position, velocity, displacement, full KKT state,
active axes/faces, impulses/reactions, objective, all iteration/accept/reject/
HVP counters and failure string. At least one case must be pressure-active and
execute taped HVPs; at least one must take `INACTIVE_EXACT` behavior.

A separate forced-reject control builds a finite, distinct projected trial at
the P1 forecast state, audits it, then rejects it without entering the solve.
Current position, Evaluation, pair list, CSR/tape arrays and digest must remain
exact; the trial digest must differ.

## Work and capacity gates

Report candidate and audit-oracle queries separately. Candidate all-pairs
evaluation and HVP counters must both be zero. For each joint workspace report
cell distance tests, all-pairs candidate checks, pairs, directed records and
tape bytes. Every neighborhood must remain within B4C0R capacity and perform
fewer distance tests than its corresponding all-pairs candidate checks.

Report tape builds, taped HVP calls, accepted workspace promotions, rejected
workspace destructions and maximum simultaneously live workspaces. The maximum
must be two: current plus one trial. No elapsed-time or production-memory claim
is permitted.

## Repeatability and exit

Two reports must be byte-identical. B4C1, B4C0R and B4C0 raw reports remain
exact.

PASS selects `JOINT_PRESSURE_KKT_QUERY_CANDIDATE` and authorizes only B4C2T
full B4B2 controller substitution contract design. FAIL blocks it and
preserves B4C1.

No full trajectory, canonical continuation, nominal run, CUDA, runtime,
schema or production authority is granted.

Execution passes all frozen gates; see the
[dated evidence](../../development/nonlocal-nsr3b4c2q-query-substitution-evidence-2026-08-21.md).

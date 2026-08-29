# NSR3-B4C3MAR -- complete adaptive macro recovery replay

Status: `FROZEN / PASS / ADAPTIVE_FIXED_COMPARISON_DESIGN_AUTHORIZED`

Parent B4C3MAG passes with JSON-without-final-LF SHA-256
`5823054bdf6ee4a9f3624f68f9577cba5b5acf18824b542787551a315f45fe28`
and semantic SHA-256
`052f7a3f481c7f2b09781bdbc5991b1640cdbd3616c6cc38eb22301e54fc5e08`.

## Policy identity

```text
sha256  3b7f50281b356e48952989d2f85e2464a3813b38d238e1a8002712f01cb4938e
text    nextengine.nonlocal.macro-adaptive-replay|v1|frames=8,16|topology=position-um|publication=accepted-macro-only|recovery=exact-reject-limit|ledger=macro-kkt-sum
```

Reuse B4C3MAG transaction/topology, B4C3MA recovery/work, B4C3PE1 admission,
B4C3P representation/ledger, P1/P2 fixtures, spectral estimator, embedded gate
and all solver tolerances exactly.

## Complete lanes

Run P1 for eight and P2 for sixteen macro frames. Each frame begins from the
last decoded commit and owns:

- one selected private fine endpoint and its physical diagnostics;
- one balanced canonical frame at step `frame_index+1`;
- one aggregate macro ledger entry;
- one mixed position and velocity admission;
- one exact canonical topology certificate.

All other levels are private. Record each attempt's planned/completed/attempted
substeps, exact failure/recovery class, nonlinear work and adjacent gate.
Require exact total attempted accounting, accepted `<=192`, attempted level
`<=768`, four-level cap, zero live workspaces and no all-pair candidate path.

## Schedule and physical gates

- P1 frame zero is `FORECAST_ACTIVE`; every frame selects and commits.
- P2 frames 0--13 are `INACTIVE_EXACT`, frame 14 `FORECAST_ACTIVE`, frame 15
  `START_ACTIVE`; pressure is inactive before onset.
- Require at least one contact event per complete lane, exact canonical
  feasibility/topology per commit, maximum penetration `<=1e-6`, KKT ledger
  residual `<=1e-9`, support-reaction closure `<=1e-10` and bounded pairs.
- P1 final center-y drift is `<=0.05dx`, maximum positive density strain
  `<=1e-3` and maximum speed `<=0.01c`.
- P2 precontact support reaction is `<=1e-12`, position/velocity analytical
  error is `<1e-6 + 64 epsilon`, velocity spread is
  `<=2e-6*precontact_steps + 64 epsilon`, and pressure violations are zero.

For each lane use
`E_scale=max(abs(initial mechanical),N*m*abs(g_y)*dx,1e-12)`. Cumulative
absolute pressure and mechanical publication deltas must each be `<=1%` of
this scale. Cumulative publication impulse remains within the sum of balanced
half-unit bounds plus its forward accumulation bound.

## Roots and rollback

Require exactly 8/16 committed frames and ledger entries, contiguous macro
steps, final state equal to last frame decode, and exact recomputation of
trajectory, legacy ledger and macro-policy ledger roots.

After one accepted P1 frame, run the next adaptive selection and inject failure
before publication. State, committed frame/ledger vectors, all roots, counts
and cumulative publication totals must remain bit-exact.

Reuse exact recovery/topology negative controls. Two complete reports must be
byte-identical and reproduce B4C3MAG at its exact parent hash.

## Decision boundary

PASS selects `COMPLETE_CANONICAL_TOPOLOGY_ADAPTIVE_MACRO_CONTROLLER_CANDIDATE`
and authorizes only adaptive-versus-fixed macro comparison design. FAIL
preserves B4C3MAG. Fixed comparison execution, nominal corpus, B4C4/B4D, CUDA,
runtime/schema and production remain blocked.

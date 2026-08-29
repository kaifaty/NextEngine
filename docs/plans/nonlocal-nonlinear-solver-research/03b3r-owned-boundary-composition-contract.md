# NSR3-B3R -- owned-residual boundary-composition retry contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / PHYSICAL_CORPUS_BLOCKED`

Parent D5 selects `OWNED_RESIDUAL_TRAJECTORY_CANDIDATE`, semantic SHA-256
`c716675fd75c4c7ecaa2410eb2f26154d7c31f36264a1d41bd9ad5b3f0aee40c`.
Original B3 remains exact FAIL and is not amended.

## Retry identity

```text
post-solve-swept-contact-composition-r1-owned-residual
```

Reuse the complete B3 contract without changing fixture geometry, spectral
target, fine-state controller, reference ladder, contact order, ledger,
capacity or accuracy thresholds. The only remediation is the exact D5
numerical candidate in every adaptive and fixed-reference substep:

- owned prediction/correction/contact displacement;
- inertia energy and gradient from `delta-delta*`;
- unchanged analytic HVP;
- reaction-aware active stop;
- at inherited energy floor, at most four strictly decreasing residual-merit
  accepts, no line search;
- velocity from `delta/h`.

Fixed `96/192/384` references use the same remediation and independent fixed
scheduling. This isolates adaptive composition error rather than comparing two
different numerical solvers.

## Additional accounting

Besides every original B3 field, publish for accepted and discarded paths:

- floor-merit trials/accepts and maximum accepts per smooth solve;
- ordinary outer/reject/HVP work;
- active and inactive substep counts;
- maximum active stationarity ratio and inactive reconstruction-bound ratio;
- cumulative reconstruction bound.

Every executed comparator path is charged. A failed candidate cannot mutate
the frame start. The original strict per-substep normalized ledger `<=1e-9`
is unchanged; D5's diagnostic decomposition cannot replace it.

## Parent, regression and exit

Require exact D5 JSON-without-newline SHA-256
`fd7627b22734b6be1183e0bbd53f03a99170bda35d2f37585c4da066516c2b91`.
Two B3R reports must be byte-identical. D5, D4, D3, D2, D1, original B3, B3D,
B2 and B1R1 remain byte-exact.

PASS selects `STATIC_BOUNDARY_SMOKE_CANDIDATE` and authorizes only B4
physical-corpus contract design. It does not authorize B4 execution,
hydrostatic/dam-break claims, moving solids, CUDA, performance, runtime or
production integration.

Failure preserves the exact first B3R gate; no threshold or contact-order
remediation is pre-authorized.

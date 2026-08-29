# NSR3-B4C3MC0 -- adaptive versus fixed macro diagnostic

Status: `FROZEN / MEASUREMENT_AUTHORIZED / ACCURACY_GATE_BLOCKED`

Parent B4C3MAR passes with JSON-without-final-LF SHA-256
`b1549cc6929f86256d81330a7a6d70cdb8df0e8db81f1d9d3e3fbf5d684b9dff`
and semantic SHA-256
`0da1f9eab67f0fd3288f06f2de1baf482a24067393675ef34b4abadbcb734cf5`.

## Identity

```text
sha256  6b233bc82ede0dca0a5585b9c6baf7c957ae6eb193c60e94c65a5ab41e326ac2
text    nextengine.nonlocal.adaptive-fixed-diagnostic|v1|adaptive=macro|fixed=48,96,192|comparison=frame-state-aggregate-contact|thresholds=none
```

## Inputs

Re-run B4C3MAR adaptive P1/P2 lanes and B4C3PE1-selected macro-publication
fixed `48/96/192` lanes from the same fixtures and scenario identities. Require
each adaptive lane PASS and each fixed lane's mixed admission, observed
first-order convergence, non-tube physics, roots and rollback selection PASS.
The obsolete B4C3P tube remains diagnostic and does not invalidate the selected
fixed reference.

## Per-frame fields

Require exact frame/sample alignment. Report adaptive RMS position/velocity
distance to each fixed level. For fixed `96/192`, report temporal difference,
computed binary64 floor, resolved flag and adaptive/fine ratio only when
resolved. Always report utilization of existing physical scales `0.05dx` and
`0.001c`.

For adaptive versus fixed-192 report center component distance in `dx`, q99
height/front distance in `dx`, kinetic absolute/relative difference with the
existing near-zero floor classification, and per-frame terminal contact-set
equality. Report global onset-time difference and final terminal contact
equality.

## Gate and decision

Gate only source PASS, exact alignment, finite metrics, valid resolved/floor
classification, unchanged roots and byte repeatability. There is no candidate
error threshold and no branch that selects accuracy.

Two complete reports must be byte-identical and reproduce B4C3MAR at its exact
parent hash. PASS authorizes only B4C3MC1 comparison-budget design. Adaptive
accuracy selection, nominal corpus, B4C4/B4D, CUDA, runtime/schema and
production remain blocked.

# NSR3-B4C3MC1 -- adaptive fixed-reference accuracy budget

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / NOMINAL_CORPUS_BLOCKED`

Parent B4C3MC0 passes with JSON-without-final-LF SHA-256
`923c09a8e86a473df8b5903ce170a10154c959a6ca8532136618bd05cf2d57c8`
and semantic SHA-256
`063a7e4b9955ef5c35f5fef743707728aa0373c2bd5c8747748aeff5d28532a6`.

## Identity

```text
sha256  40f5923fc02e696bf7c98d09d0964ec99c331c4ffda0e8fcef4f81b8ee8b6503
text    nextengine.nonlocal.adaptive-fixed-accuracy|v1|adaptive=macro|reference=fixed192|state=0.05dx,0.001c|aggregate=0.05dx,0.10dx,0.10dx,0.15ke|contact=adaptive-substep|temporal=classify-only
```

## Inputs and provenance

Re-run B4C3MC0 from the same P1/P2 fixtures. Require its PASS, exact parent
hash, exact frame/sample alignment and finite measurements. Preserve and report
the adaptive and every fixed-level trajectory, legacy-ledger and policy-ledger
root separately. The old B4C3P tube remains a negative diagnostic and is not a
selected reference.

## Per-frame accuracy gate

Compare adaptive durable state with fixed-192 at every macro frame. Reuse the
existing B4B thresholds without modification:

```text
RMS position error / dx <= 0.05
RMS velocity error / c  <= 0.001
center component / dx   <= 0.05
q99 height / dx         <= 0.10
q99 front / dx          <= 0.10
```

When maximum adaptive/fixed kinetic energy exceeds `1e-12 J`, require relative
kinetic difference `<=0.15`. Otherwise require absolute difference to lie
within the existing computed binary64 kinetic floor. Require exact terminal
contact-set identity on every aligned frame and at completion.

For a finite first-contact time, identify its owning adaptive macro frame and
require onset error no greater than
`SMOKE_FRAME_TIME / accepted_substeps + 64*epsilon`. If neither trajectory has
contact, require exact infinite agreement; one finite and one infinite event
is rejected.

## Temporal evidence

For each frame/field use the B4C3MC0 values `e`, `D` and floor `F` and assign
exactly one non-gating classification:

```text
RESOLVED_RATIO                D > F; report e/D
FLOOR_COINCIDENT              D <= F and e <= F
STABLE_REFERENCE_SEPARATION   D <= F and e > F
```

Report branch counts per case and field. Temporal class never changes the
physical accuracy decision and no observed ratio threshold exists.

## Negative controls

Synthetic controls must prove exact-boundary acceptance and next-representable
rejection for all five state/aggregate thresholds, relative kinetic rejection,
near-zero kinetic floor acceptance/rejection, onset acceptance/rejection,
contact-set identity, non-finite rejection and all three temporal branches.
Controls do not change solver state.

## Gate and decision

Two complete reports must be byte-identical and reproduce B4C3MC0 at its exact
parent hash. PASS requires all P1/P2 physical gates and all controls. PASS
selects `ADAPTIVE_MACRO_TINY_CORPUS_ACCURACY_CANDIDATE` and authorizes only
nominal-corpus contract design. Temporal equivalence, B4C4/B4D, CUDA,
performance, runtime/schema and production remain blocked.

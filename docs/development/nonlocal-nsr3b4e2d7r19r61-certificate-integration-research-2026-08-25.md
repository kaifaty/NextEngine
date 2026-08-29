# NSR3-B4E2D7R19R61 certificate-integration research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / TOPOLOGY-OWNED SHADOW INTEGRATION SELECTED`.

Parent: R60 `PASS / ROW_LOCAL_CERTIFICATE_VALIDATION_CANDIDATE`, stdout
SHA-256 `6c6c62b6efb196d97dc4210ad7bcb56732f0c2d107a2df6aa10e9de84005c7f7`,
semantic `d93dbdc95eb66cf45f074541ae44b621755f45f011d2f1589af834a45238aec9`.

## Question

What is the smallest safe integration boundary for the validated row-local
certificate without changing historical audit bytes or prematurely applying
the restoration witness?

## Ownership decision

The integration owner must not accept a caller-supplied degree vector. It owns
one exact `ALNormalizedWorkspace`, executes the directed JVP itself and derives
each degree from the same flat offsets that control row traversal:

```text
workspace
  flat_offsets + flat_directed_pair_indices
  constraint
  witness
       |
       v
directed JVP exactly once
       |
       +-- raw = constraint + image
       +-- legacy global upper (shadow)
       `-- row-local upper (candidate)
```

This makes topology and certificate work count one lifecycle-owned object.
There is no degree cache to become stale or undercount a row.

## Candidate/shadow rules

The owner publishes both paths in the same stable row order:

1. Candidate and shadow share the exact directed image, absolute sums and raw
   values.
2. Shadow uses unchanged `gamma(16*maximum_degree+66)` and must reproduce the
   R58 terminal upper vector, active root, positive count and maximum bitwise.
3. Candidate uses `gamma(16*row_degree+66)` and must reproduce R59 active root,
   zero positive count, maximum, degree histogram and comparison root.
4. Candidate upper must be no greater than shadow on every row; candidate
   active mask must be a subset of shadow. A shadow-certified state can never
   become candidate-uncertified.
5. Exact topology validation requires offsets start at zero, are monotone, end
   at the directed-slot count, contain valid pair indices and derive the owned
   declared maximum.

R61 retains the R60 dense corruption controls. It also adds a shadow/candidate
synthetic control where maximum-degree equality and lower-degree strict
improvement are both exercised.

## Alternatives

### Modify `al_r48_primal_audit` or `al_r50_audit` in place

Rejected. Those functions are transitive parents of R48--R60; changing them
would rewrite historical evidence and confound integration with regression.

### Pass a row-degree vector into the legacy audit

Rejected. The vector could be stale or undercounted. R60 proved rejection is
necessary; ownership is safer than repeatedly trusting callers.

### Remove the global bound after validation

Rejected. The shadow is cheap, provides rollout observability and proves the
candidate is a monotone tightening rather than a different residual.

### Carry binary128 into the integrated owner

Rejected. R60 used it as an offline validator. R61 must remain a binary64
candidate path suitable for a later runtime owner.

### New topology-owned dual-path audit

Selected. Historical functions remain byte-exact, while the new owner is
independently comparable to R58/R59.

## Selected R61 experiment

1. Passively reproduce exact R60 and retain exact R58 witness/workspace source.
2. Rebuild one moved workspace and verify topology/master/witness roots.
3. Run one new topology-owned audit with exactly one directed JVP.
4. Reproduce R58 global shadow and R59 local candidate roots/counts/maxima.
5. Prove all-row candidate `<=` shadow, active-subset relation and shared raw
   image identity.
6. Run retained degree controls and one dual-path dense control with forced
   topology corruption rejection.
7. Publish owner topology/image/raw/shadow/candidate/comparison roots and exact
   work/rollback.
8. Select topology-owned audit candidate, shadow mismatch, candidate mismatch,
   relation rejection or topology rejection by frozen precedence.

R61 is rollback-only integration research. It creates a reusable candidate
owner but does not replace legacy callers, apply a witness, authorize
restoration exit, commit public state, run timing or claim runtime/production.

## Expected consequence

If R61 passes, freeze R62 as a restoration promotion/transaction boundary:
decide how a validated compatible normal witness becomes owned by the next
TRQP/restoration state, with fresh nonlinear/contact checks and atomic rollback.
Performance work remains stopped until that semantic transaction is correct.

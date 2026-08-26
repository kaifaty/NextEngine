# NSR3-B4E2D7R20R63L wide-refinement frontier research

Status: `IMPLEMENTED / EXPORTED_FACTOR_WIDE_STANDARD_REFINEMENT_CANDIDATE`.

## Question

R63K rejects its frozen 16-step budget, but retained-wide standard refinement
is monotone and has only four unresolved signs at the boundary; export/wide is
also contracting late. Do either or both wide lanes reach the exact R60 sign
semantics by iteration 32 when continued without any algorithmic change?

This stage locates a convergence frontier. It does not amend or relabel the
closed R63K result.

## Frozen continuation

Replay R63K exactly, including all three 16-iteration lanes and its negative
route. Retain only the actual iteration-16 solutions of:

```text
retained-wide R / binary128 solve+update
exported R / binary128 solve+update
```

For each lane execute the unchanged standard transaction for iterations
17--32:

```text
r(k)       = b - H x(k)
delta(k)   = B^-1 r(k)  via the same R and permutation
x(k+1)     = x(k) + delta(k)
```

Residual, substitutions and update remain binary128. The strict-binary64 lane
is not continued; R63K already showed nonmonotone order-`1e15` errors.

## Every-iteration certificate

Independently certify all 16 new iterates, not a sparse or adaptive subset:

```text
17, 18, ..., 32
```

Use original `H,b` and the R60 left-inverse sign certificate. Record error,
residual bound, signs, separation and root at each iteration, plus the first
passing iteration. Continue through 32 after a pass so later stability remains
observable.

The lane succeeds only if some frozen iteration has all 102 componentwise
signs equal to R60. Monotonic error is reported but cannot replace the sign
certificate.

## Routes

- apparatus, parent, continuation, certificate, work or lifecycle failure:
  `WIDE_REFINEMENT_FRONTIER_APPARATUS_REJECTED`;
- retained-wide has no certified iterate by 32:
  `RETAINED_WIDE_REFINEMENT_FRONTIER_EXHAUSTED`;
- retained-wide passes but export/wide does not:
  `EXPORTED_FACTOR_WIDE_REFINEMENT_FRONTIER_EXHAUSTED`;
- both pass: `EXPORTED_FACTOR_WIDE_STANDARD_REFINEMENT_CANDIDATE`.

## Interpretation ceiling

A two-lane pass proves ordinary wide-arithmetic refinement eventually recovers
the captured RHS with binary64 factor storage/export. It does not prove that
17--32 dense correction passes are acceptable for production. The next
research stage should compare a preconditioned Krylov accelerator and a
targeted weak-direction/compensated correction under equal correctness work,
without shared-host timing.

A frontier exhaustion selects GMRES-IR/PCG/MINRES or a certified low-rank
correction directly. It does not authorize extending the depth again without a
new research hypothesis.

No state replacement, following transition, rank/row/regularization change,
trajectory, timing, runtime/GPU or production inference occurs. R64 and R65
remain blocked.

## Result

The frozen apparatus selects
`EXPORTED_FACTOR_WIDE_STANDARD_REFINEMENT_CANDIDATE` at semantic
`16633e07...7333`; stdout repeats at `bf59f1ed...6de5`. Retained-wide first
certifies at iteration 17 and export/wide at iteration 21; both remain
certified through 32. See the
[evidence record](nonlocal-nsr3b4e2d7r20r63l-wide-refinement-frontier-evidence-2026-08-26.md).

The stationary correctness frontier is closed. The next stage should compare
preconditioned CG against the exact 17/21 correction-work baselines before
researching GMRES-IR or explicit weak-direction correction.

# NSR3-B4E2D7R20R63L wide-refinement frontier evidence

Status: `PASS / EXPORTED_FACTOR_WIDE_STANDARD_REFINEMENT_CANDIDATE`.

## Reproducible result

Implementation commit: `3794114c`.

Command:

```text
/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v5-wide-refinement-frontier
```

The command repeated byte-identically twice:

```text
stdout sha256   bf59f1edd265db77fb993fbb6e366e93514085aa63dd7e864909e7bec5f96de5
semantic sha256 16633e078acd397d817ecfc0ec5639b41135258209fc355fea3eb9c58c557333
route           EXPORTED_FACTOR_WIDE_STANDARD_REFINEMENT_CANDIDATE
```

Platform, controls, exact R63K reconstruction, 32 added factor-refinement
iterations, 32 every-iteration certificates, work and lifecycle gates all
pass. Strict-binary64 continuation work is exactly zero.

## Exact convergence frontier

The retained-wide lane first certifies at iteration 17:

```text
iteration 16  error 144.035, 21+/77-/4 unresolved
iteration 17  error 19.6106, 24+/78-/0, minimum separation 12.8128
```

The export/wide lane first certifies at iteration 21:

```text
iteration 17  error 9793.73, 49 unresolved
iteration 18  error 1872.77, 22 unresolved
iteration 19  error 361.052, 8 unresolved
iteration 20  error 69.1978, 2 unresolved
iteration 21  error 13.3195, 24+/78-/0, minimum separation 19.1039
```

Both lanes remain componentwise certified at every subsequent frozen iterate
through 32.

## Arithmetic floor

After convergence the error is no longer monotone because residual/correction
rounding reaches a floor. The best observed frozen checkpoints are:

```text
retained-wide  iteration 29  error 2.28040e-5
export/wide    iteration 28  error 2.11013e-4
```

Later errors fluctuate around `1e-4..1e-3`, but the immutable sign margin is
about 32, so all remain far inside the certified region. Continuing stationary
refinement beyond first admissibility has no correctness value for this RHS.

## Meaning

This closes the stationary-method question:

- binary64 operator/factor export is recoverable using original residuals;
- retained-wide standard refinement needs 17 total factor corrections;
- exported-factor/wide-consumption needs 21 total corrections;
- strict binary64 standard refinement remains rejected by R63K;
- dense `X` is only the verifier, not the correction provider.

The exported factor is therefore a mathematically valid preconditioner/state
representation, but 21 sequential corrections plus software binary128 are not
yet a production recommendation. The next performance research should compare
an SPD preconditioned Krylov method against this exact 21-correction baseline
using deterministic work counts, not shared-host timing.

Because original `H` and factor Gram `B` are symmetric positive definite on the
represented rank-102 space, preconditioned conjugate gradients is the first
candidate. GMRES-IR remains the fallback if finite-precision PCG loses SPD or
does not reduce correction count materially.

## Regression evidence

All parent stdout hashes R63B--R63K remain byte-identical, including:

```text
R63J b4a2908e1cfb724354e16895221df688d27675de48de5da9717674d2fe99191d
R63K 7b13fa402665836d36f3442942c34c2c611df942df0fff582034f487224773c8
```

No timing was admitted. Shared-host duration is not performance evidence.

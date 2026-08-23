# NSR3-B4E2D7R19R8 tiered-grace model research

Date: `2026-08-23`

Status: `COMPLETE / REPLAY-ONLY DISCRIMINATOR SELECTED / CONTRACT FROZEN NEXT`

## Question

Can one online-computable bounded rule retain the original one-HVP guard while
admitting the later two-HVP recurrence, and does the residual-derived
`H(step)` image remain valid at the 34-HVP solution?

## Evidence available

The two exact boundaries differ:

```text
                         original        later
ratio after HVP 32       1.201853 eta     1.891245 eta
last-eight decrease      true             true
HVP 33 ratio             0.8830 eta       1.294362 eta
HVP 33 converged         true             false
HVP 34 ratio             not needed       0.933412 eta
HVP 34 converged         n/a              true
condition estimate       36.13            35.81
```

Both recurrences are finite, positive-curvature, interior and numerically
well behaved. The difference is the remaining Krylov distance, not an
observed instability or conditioning regime change.

## Candidate envelope

Preserve the existing one-shot lane exactly:

```text
tier 1
  first 32 finite, positive, interior
  ratio32 in (eta, 1.25 eta]
  last 8 ratios strictly decrease
  -> allow HVP 33
  -> HVP 33 must converge
```

Add only a separate replay candidate for states outside tier 1:

```text
tier 2 entry
  first 32 finite, positive, interior
  ratio32 in (1.25 eta, 2 eta]
  last 8 ratios strictly decrease
  -> allow diagnostic HVP 33

tier 2 continuation
  HVP 33 finite, positive, interior
  ratio33 in (eta, 1.5 eta]
  q33 = ratio33 / ratio32 <= 0.75
  updated last 8 ratios strictly decrease
  -> allow diagnostic HVP 34
  -> HVP 34 must converge
```

The bounds are deliberately coarse binary64 constants with material margin
from the captured values. They are not called a production policy: two exact
recurrences cannot establish general safety. R8 asks only whether this
falsifiable envelope classifies both known boundaries without changing tier 1.

The second continuation is decided only after observing HVP 33. It is not an
unconditional two-HVP grant.

## Model-image requirement

At the later HVP-34 solution compute:

```text
candidate image = r_final - g
candidate model = -g dot p - 0.5 p dot candidate_image
```

Build one exact sparse workspace at the captured later state and spend one
direct `H(step)` oracle HVP. Compare:

- image L2 relative error;
- maximum component error scaled by the direct absolute-term sum;
- `p dot H(p)` relative error;
- predicted-reduction relative error and sign;
- step and all input roots.

Use the frozen R4 bounds of `1e-10`. Prefer the residual-derived image only if
all bounds and positive-model signs pass. Do not form a trial.

## Rejected alternatives

### Set recurrence cap to 34

Rejected. It grants work without a progress certificate and changes the live
transaction before the second model image is validated.

### Replace the original `1.25 eta` guard with `2 eta`

Rejected. Tier 1 is already proven and must remain byte-exact. The wider entry
belongs to an explicit second lane.

### Predict convergence from condition number

Rejected. Ritz condition is diagnostic and does not provide an online
residual certificate for the next one or two finite-precision CG iterations.

### Form the later trial immediately after offline convergence

Rejected. Model correspondence precedes divided, precision and acceptance
work, as it did in R4/R5.

### Generalize from two boundaries to production

Rejected. R8 can at most authorize a later shadow-trial discriminator or a
new full-transaction contract; it cannot establish a universal cap policy.

## Smallest falsifiable experiment

Freeze D7R19R8 to:

1. reproduce exact R7/R6/R5 parent bytes;
2. retain the original R3/R4/R5 one-shot boundary and exact tier-1 decision;
3. bind the R7 later capture, live/offline prefix and 34-HVP convergence;
4. evaluate every tier-2 clause from only online-available recurrence values;
5. prove HVP 33 is admitted but insufficient and HVP 34 is separately
   admitted and converges;
6. compare `r_final-g` at HVP 34 with one direct `H(step)` oracle under all R4
   bounds and positive signs;
7. consume one new workspace and one oracle HVP, with zero model HVP credited
   to the candidate;
8. retain finite, static binding, lifecycle, zero all-pair work and rollback;
9. form no trial, acceptance or precision audit and run no transaction,
   substep, macro, trajectory or timing lane;
10. reproduce one report from each of two clean Release builds.

## Authority boundary

D7R19R8 may select or reject one replay-only tiered-grace/model-image
candidate over two frozen recurrences. It cannot change live guard/cap policy,
form a trial, continue a transaction, publish state, claim performance or
grant runtime/production authority.

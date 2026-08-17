# W0C — Hydro calibration reclosure

Status: `CLOSED / RESEARCH_ONLY / PROFILE_DECISION_REQUIRED`.

## Outcome

Replace the numerically rejected W0B water profile with the smallest
scientifically defensible revision that passes the serial hydro discriminator
without weakening deterministic authority, failure semantics or the existing
free-fall control. W0C is calibration/research, not production implementation.

This outcome was not achieved. The bounded candidate ladder is exhausted and
W0C is closed `RESEARCH_ONLY`; resumption requires a new explicit
architecture/profile decision rather than another in-place candidate.

The original W0B document and its roots stay unchanged as historical evidence.
Only a selected W0C candidate may define successor document/profile/corpus
roots and unblock W1.

## Entry evidence

- W1 clean-tree free-fall passes 97 frames and same-target repeat equality.
- `CW-HYDRO-001` fails its first density solve at iteration 20 with
  `74,482,699 ppb` against `100,000 ppb`.
- [W1-RC1](../../development/continuum-water-w1-rc1-audit-2026-08-17.md)
  independently reproduces all inputs, boundary volumes, the global residual
  curve and four representative row traces bit-for-bit.

## Current W0C result

The first bounded cycle is recorded in the
[W0C hydro-calibration report](../../development/continuum-water-w0c-hydro-calibration-2026-08-17.md).
Production and independent calculators exactly match the contribution
decomposition and the unchanged-profile 320-iteration curve. The original
boundary overfills corner, edge and face rows by different amounts; its
prospective state already escapes the analytical box at iteration 20 and still
misses the density threshold at iteration 320. A larger ceiling and a uniform
boundary multiplier are rejected.

`ghost-cell-shell-v1` restores the selected initial rows to the interior
partition error and passes the first hydro step plus exact free-fall control.
It nevertheless fails the 24-step soak on step 2. Counterfactual ceilings 100
and 160 only move failure to steps 4 and 5 while accepted-step iteration demand
and penetration grow. The candidate is `NOT_SELECTED`; no successor roots or
corpus credit exist.

`zero-velocity-settle-v1` then tests step 4 with a deterministic, independently
reproduced generator. Full damping does not contract the transient: density
iteration demand grows `2 → 33 → 40 → 47` while penetration grows
`171 → 340 → 521 → 710 µm`. The adverse-trend guard rejects the generator on
pass 4, and its last diagnostic state fails the first unchanged-ceiling hydro
step at `162,015 ppb`. It defines no canonical successor initial state.

Steps 1–5 have now rejected an unchanged-profile ceiling extension, one
particle-boundary replacement and one position-only initialization family.
Under the persistent-problem rule, the final adjacent-layer discriminator was
`volume-map-box-bender2019-ref-v1`. It duplicates the primary cubic extension,
degree-30 Gauss-Legendre quadrature, reference `0.8` factor and virtual-point
offset over an exact analytical box SDF. Production and independent
calculators match exactly, and free-fall remains exact, but face/corner density
reconstructs to `2.139 / 2.600`; the first step ends at `70,690,915 ppb`, and
the 24-step soak accepts zero steps. The candidate is `NOT_SELECTED` and the
predeclared stop rule is now active.

## Fixed constraints

- Keep CPU serial `f64`, exact target/toolchain flags, integer neighbor
  admission, canonical ties-to-even publication and no warm start.
- Keep the selected basin/product sample spacing and the W1 failure policy
  unless evidence explicitly proves that the product fixture itself must be
  revised.
- Do not change the original W0B file or claim corpus credit for a
  counterfactual run.
- Do not use tolerance relaxation, retries, retained float state, parallelism,
  PhysX or GPU execution to turn the current failure into a pass.

## Counterfactual order

All runs are typed `COUNTERFACTUAL / NO_CORPUS_CREDIT`, retain the free-fall
non-regression and emit bounded reports outside Git.

1. Decompose initial density into self, fluid and boundary contributions for
   the same corner/edge/face/interior rows. Partition boundary contributions by
   plane/edge/corner feature and check the discrete partition-of-unity error.
2. Extend the unchanged-profile density curve only as a diagnostic at fixed
   ceilings `40/80/160/320`. Record residual, maximum multiplier, velocity and
   clearance trends; no extended run may satisfy the current gate.
3. Evaluate boundary-calibration candidates derived from a documented
   discretization rule, not a fitted arbitrary multiplier. This is the
   recommended first candidate family because RC1 localizes the initial excess
   to boundary-adjacent rows.
4. Evaluate an initialization revision only if boundary calibration cannot
   produce a stable partition and preserve the selected physical clearance.
   Any relaxed/pre-equilibrated state must have a deterministic generator and
   a canonical root.
5. Consider a larger fixed iteration ceiling only after the reconstructed
   density field is justified and the extended curve shows stable convergence
   without penetration or unbounded velocity. Do not loosen the density
   tolerance.

Before selecting a formula, review current primary DFSPH and Akinci-boundary
sources and the SPlisHSPlasH reference behavior. External material informs the
candidate; it never replaces exact local evidence.

## Selection gate

One candidate may be selected only when:

- production and independent calculators agree exactly under the candidate;
- free-fall remains byte-identical to the W1 root;
- hydro passes the existing density/divergence/clearance/conservation gates;
- boundary contribution and convergence behavior have an explicit numerical
  rationale rather than a threshold-fitting rationale;
- a bounded SPlisHSPlasH aggregate comparison is available;
- the candidate declares every changed constant, operation, scenario field,
  capacity and expected consequence.

No candidate passed without weakening the selected product or authority
boundary. The water program is therefore `RESEARCH_ONLY`; all failed families
are recorded in the W0C evidence reports. Do not advance to W1 continuation or
W2.

## Reclosure output

No W0C selection or reclosure output exists. Do not produce successor roots
from these candidates. A future decision may authorize either a separately
specified density-map discriminator (recommended) or a revised
lattice-clearance/product profile. Only that new decision can define its own
closure and eventual W1 rerun. `CONTINUUM-WATER-REF-P1` remains `NOT_RUN`.

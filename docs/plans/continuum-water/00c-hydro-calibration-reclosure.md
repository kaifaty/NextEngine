# W0C — Hydro calibration reclosure

Status: `READY / NOT_STARTED`.

## Outcome

Replace the numerically rejected W0B water profile with the smallest
scientifically defensible revision that passes the serial hydro discriminator
without weakening deterministic authority, failure semantics or the existing
free-fall control. W0C is calibration/research, not production implementation.

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

If no candidate passes without weakening the selected product or authority
boundary, stop the water program as `RESEARCH_ONLY` and record the failed
families. Do not advance to W2.

## Reclosure output

After selection, produce one successor numeric document with new document,
float-profile, corpus and execution roots; update implementation preflight and
goldens in the same coherent change. Then rerun W1 from free-fall through the
full nominal/external corpus. `CONTINUUM-WATER-REF-P1` remains `NOT_RUN` until
that entire successor corpus passes.

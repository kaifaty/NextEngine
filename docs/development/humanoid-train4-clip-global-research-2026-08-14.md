# TRAIN-4 clip-global contact trajectory research

| Field | Value |
| --- | --- |
| Date | 2026-08-14 |
| Scope | Optimizer-free complete-clip research for `REQ-HUM-DATA-005/007` |
| Status | `DOMAIN_CLOSED / SOLVER_REJECTED` |
| Accepted result | One solve per selected clip; every selected case is an exact slice |
| Claim ceiling | Research infrastructure only; no V19, TRAIN-4 Advance, visual gate or optimization |

## Frozen constraints

The cycle retained active tangential motion `2000 µm/frame`, active normal
motion `1000 µm/frame`, active residual `5000 µm`, collider floor `-2 µm`,
joint-velocity reserve `2500` basis points and root vertical velocity
`200060 µm/s`. Fresh scenes remain the only acceptance authority under
ADR-070. Indexed partial reset remains report-only. Optimizer steps and
training runs remain zero.

## Falsifiable problem statement

Bounded V7/R49 proved that one 12-frame projection can be safe in a fresh
scene. It did not prove that independently solved windows assign the same
state to an overlapping source frame or that one complete trajectory can
satisfy contact, collider and velocity bounds simultaneously.

The research cycle tested three competing explanations:

1. centered finite differences alone create the complete-clip failures;
2. temporal smoothing crosses contact-authorization boundaries and is the
   primary pose defect;
3. a sequence of contact, smoothing, collider and velocity post-passes can
   converge without a coupled trajectory solve.

## Primary-source review

- MIT's [Planning and Control through Contact](https://underactuated.mit.edu/contact.html)
  models making and breaking contact as hybrid dynamics with an impact reset,
  rather than one smooth velocity field. This supports evaluating the
  post-impact side at contact entry and the pre-liftoff side at contact exit;
  it does not by itself select our discrete stencil.
- Kovar, Schreiner and Gleicher's
  [Footskate Cleanup for Motion Capture Editing](https://pages.cs.wisc.edu/~kovar/footskateCleanup.pdf)
  identifies constraint switches as a source of discontinuity and combines
  root filtering with exact limb constraints. This predicts that unconstrained
  smoothing across a footplant boundary can move the symptom rather than
  preserve the plant.
- Posa, Cantu and Tedrake's
  [direct trajectory method](https://groups.csail.mit.edu/robotics-center/public_papers/Posa13.pdf)
  treats contact and trajectory variables in one constrained problem. It
  supports the coupled-solve hypothesis after our alternating projections fail;
  it is not adopted as a production dependency or an optimizer authorization.

These sources informed discriminators only. Repository requirements and exact
local evidence remain authoritative.

## Evidence sequence

All artifacts are under the external
`humanoid-motor-rebuild-v1/evaluations/TRAIN-4` root.

| Evidence | Result | Interpretation |
| --- | --- | --- |
| R50 V7 complete `cmu16`, report SHA-256 `9e5fa9de04c6725753f406afd809c5a30d75b935ac062557342bdc2bc4d27cc0` | Collider, joint, root and normal bounds pass; analytic tangent is `7130/2000` | Complete-clip failure is smaller than the earlier V5 result and initially isolates velocity semantics |
| R51/R52 complete `cmu05`/`cmu139`, report SHA-256 `708ea97e202bfea1cdbc1392b30df62ea4ece077f462a187276284ce1534aebc` / `7134206ad671e8cdd296231a9178101fde0fd457a395b382f2ac2d2fed304336` | Both fail contact pose, analytic velocity and joint reserve; final reprojection drops `3` / `7` points | The `cmu16` explanation does not generalize |
| R53/R54 localization, report SHA-256 `d9385d433d7009cfc22703092241ff17d5cf93d2264eeb458bab749d377c837e` / `9f4752538c0be16a816f0a5e141443281a3863de73522e37c3f606a213467c4d` | Hybrid edge stencil reduces analytic tangent `18345 -> 1916` and `20433 -> 2328`; normal, joint and root failures remain | Contact-edge stencil is evidence-backed but insufficient |
| R55 contact-segment counterfactual, report SHA-256 `c83a491cfabd9975db69c1f1cd1dbf59da57362414182827442394e922c58705` | Drops become zero, tangent becomes `573/723` and joint reserve becomes exactly `2500`; root collider lift then creates `25747/33411 µm` contact residuals | Hard masking exchanges contact failure for collider/root failure |
| R56 alternating floor counterfactual, report SHA-256 `038583c3758832e09b3e718dede237b00a404932c666271930b4deeeb7899eb8` | Contact passes and joint reserve is `2500`, but after 30 iterations `27` flight deficits remain, maximum `30063 µm`; root velocity reaches `682560 µm/s` | Another sequential post-pass is rejected |
| R57 clip-global V7 all-17, canonical SHA-256 `3873ac81acb1e5da86abbb7a3321b51ee4a33be9899638e4a5088eed7aebe9e5`, file SHA-256 `59fb91c9e19e22dde5caa017724a26e643cd889239a789333e3ad7b25b866271` | Clean commit `cb3fcae`; one solve per clip, exact slices `17/17`, overlap disagreement `0`; complete clips `0/3`, selected slices `16/17` | Projection-domain identity is closed; V7 solver is rejected globally |

An independent NPZ audit compared all `221` non-metadata case arrays against
their complete-clip ranges and found zero disagreement. R57 records `30`
overlap pairs and `390` internal array comparisons with zero disagreement.

## R57 complete-clip result

| Clip | Contact | Collider | Dropped | Joint bp | Residual | Normal / analytic normal | Tangent / analytic tangent |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| `cmu05-walk-validation` | FAIL | FAIL | 3 | 3192 | 16767 | 4467 / 5261 | 1535 / 18345 |
| `cmu16-walk-nominal-b` | FAIL | PASS | 0 | 2500 | 2290 | 742 / 707 | 815 / 7130 |
| `cmu139-walk-heldout` | FAIL | FAIL | 7 | 3477 | 25678 | 6690 / 6640 | 1928 / 20433 |

Collider floor remains exactly `-2 µm` and root vertical velocity remains
`200040 µm/s` in all three unmodified V7 complete solves. The only failing
selected slice is `cmu139@626`: finite normal `1312 µm/frame` and analytic
normal `4003 µm/frame`. This is why the result is not eligible for a fresh
PhysX probe.

## Mechanism

The existing closure chooses eight smoothing passes whenever *any* frame in
the solve has active support. On a complete clip that condition is globally
true, so flight-leg correction is convolved through same-foot contact frames
and through unsupported intervals. R53/R54 measured non-zero correction on
`68/59` active left/right frames in `cmu05` and `89/118` in `cmu139`, with
active-frame magnitudes up to `112310` and `112580 µrad`.

Centered velocity then mixes flight motion into contact entry and exit. A
hybrid edge stencil removes most of that artificial tangential component, but
cannot repair the pose changed by cross-boundary smoothing. Conversely,
forcing the active correction to zero makes the joint trajectory and contact
valid but removes clearance needed by the opposite flight foot. Root lift
cannot restore that clearance without lifting the planted foot. R56 shows that
alternating those projections stagnates rather than closing the coupled
constraint set.

## Decision

1. Retain the clip-global builder and exact-slice evidence path implemented at
   clean commit `cb3fcae`.
2. Reject V7 as a complete-clip solver and reject an onset-stencil-only V8.
3. Reject hard segment masking, another smoothing-pass sweep and another
   ordered root/contact/collider post-pass.
4. Prototype one deterministic trajectory-level solve that treats active
   point anchors, flight collider inequalities, root/joint velocity envelopes,
   ROM and temporal regularity as one coupled constraint set. Contact entry
   and exit velocity semantics must be explicit in that identity.
5. Start with complete `cmu05`, the shorter clip that exposes the coupled
   conflict. Promote the solver to all three clips only after `cmu05` has zero
   dropped points and passes every unchanged complete-clip bound.

The smallest next proof is offline only. After all three complete clips and
their exact selected slices pass, rebuild the R57 matrix with the new solver,
then run the same fresh all-17 PhysX acceptance. Full V19, native, visual,
exhaustive, TRAIN-4 Advance and training remain blocked until those exact
results pass.

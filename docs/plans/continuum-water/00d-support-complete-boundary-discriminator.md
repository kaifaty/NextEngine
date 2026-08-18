# W0D — Support-complete boundary discriminator

Status: `CLOSED / CANDIDATE_REJECTED / RESEARCH_ONLY`.

## Outcome

The candidate is rejected. Production and the independent calculator match
exactly, all four selected initial rows retain the small `-27,534 ppb`
regular-lattice partition error, and the first hydro step converges in two
iterations at `95,755 ppb`. The unchanged-ceiling trajectory nevertheless
accepts only that step; step 2 ends at `192,430 ppb`. Diagnostic ceilings 100
and 160 accept only three and four steps before ending at `108,211 ppb` and
`126,883 ppb`, respectively, while penetration grows. The free-fall control
matches all 97 frames exactly.

The second exterior layer changes the failing residual by only `1 ppb`
relative to the rejected one-layer shell and does not change the number of
accepted steps. H3 is falsified: support truncation is real but is not the
cause of the observed instability. No third layer, fitted scale or further
particle-shell candidate is authorized.

Exact roots, terminal values, source audit and verification are recorded in
the [W0D evidence report](../../development/continuum-water-w0d-support-complete-boundary-2026-08-18.md).

## Decision

Authorize one successor research experiment after the closed W0C cycle:
`support-complete-lattice-complement-v1`. This is not an amendment to W0B or
W0C, not a selected water profile and not permission to resume W1. It may add
only report-only serial-oracle code and evidence; the original document,
float-profile, execution-profile, scenario and corpus roots remain unchanged.

The experiment tests whether the dynamic failure of `ghost-cell-shell-v1` is
caused by truncating the exterior lattice complement to one layer. It does not
change the DFSPH equations, lattice phase, wall clearance, density tolerance,
20-iteration ceiling, canonical integer authority or failure policy.

## Why this discriminator precedes a density map

The consistent rigid-fluid coupling equations in Bender, Westhofen and Jeske
(2023) place boundary volume-gradient terms in the density constraint and the
central constraint gradient, exclude individual boundary-gradient squares
from the diagonal, and apply the fluid row multiplier to boundary pressure
acceleration. The serial oracle follows those operations. A formula mismatch
in that operator is therefore not the leading local hypothesis.

The one-layer ghost shell gives the exact regular-lattice partition at the
initial `25,000 µm` clearance because the next exterior layer is exactly at
the `100,000 µm` support boundary, where the cubic kernel is zero. After a
particle moves toward a wall, that omitted layer enters the non-zero support.
The boundary field then ceases to be the discrete complement that was tested
at initialization. This explains the observed pattern without changing a
formula: exact initial partition, one accepted step, then rising density
demand and penetration.

A standard volume map is not an automatic repair. The W0C implementation of
the published reference construction strongly overfilled the selected face,
edge and corner rows, and a constant half-space integral requires different
post-hoc scales for those feature ranks. Before specifying a new convolved
density field, W0D first tests the exact discrete field that such a map would
need to reproduce for this lattice.

## Frozen candidate

`support-complete-lattice-complement-v1` is defined as follows:

- keep the hydro fluid lattice, box, `REST_VOLUME` and all solver operations
  unchanged;
- place `REST_VOLUME` samples at canonical lattice centres in exactly two
  exterior layers around every face of the analytical box;
- use the existing lattice origin `bounds.min + PARTICLE_RADIUS` and stable
  lexicographic `x-y-z` generation order;
- include face, edge and corner exterior cells once, with no fitted scale,
  pressure mirror, warm start, retained float state or repair pass;
- rebuild neighbours from canonical integer positions before every substep.

Two layers are support-complete for every state admitted by the unchanged
clearance gate. At the minimum centre clearance of `22,500 µm`, the first
and second exterior centres are `47,500 µm` and `97,500 µm` away; the
third is `147,500 µm` away and cannot enter the `100,000 µm` support.
The selected `1 m` hydro box therefore has
`24³ − 20³ = 5,824` boundary samples, below the unchanged `16,384`
capacity. The `1 × 2 × 1 m` free-fall box has `9,344` samples.

## Hypotheses and gates

| ID | Hypothesis | Discriminator | Consequence |
| --- | --- | --- | --- |
| H1 | The pressure operator is transcribed incorrectly | Equation-by-equation audit against the primary consistent-coupling paper and pinned upstream implementation | A match falsifies H1; do not alter the operator |
| H2 | The published reference volume map supplies the missing discrete partition | Existing exact production/independent W0C result | Large feature-dependent overfill falsifies H2 |
| H3 | One-layer support truncation causes the dynamic ghost-shell failure | Two-layer production and independently generated boundary, complete trace, normal step and 24-step soak | Survival keeps the discrete boundary family viable; failure rejects it |
| H4 | A direct density field is still required | Evaluate only if H3 fails or survives locally but cannot meet later bounded cost/corpus gates | Requires a new field/interpolation/profile specification |

The candidate survives the local discriminator only if all of the following
hold without changing the frozen solver profile:

1. Production and the independently generated two-layer boundary positions,
   volume bits, selected density rows and 320-iteration diagnostic trace match
   exactly.
2. The selected corner, edge, face and interior initial partition errors are
   at most `100,000 ppb` in absolute value.
3. The normal first hydro step converges within 20 iterations and satisfies
   the existing clearance/finiteness gates.
4. All 24 unchanged-ceiling hydro steps pass; no retry or higher ceiling can
   earn survival credit.
5. The 97-frame free-fall control remains byte-identical.

The existing `max100` and `max160` paths may be emitted only as rejected-path
diagnostics. They cannot select the candidate or modify this gate.

## Stop rule and possible outcomes

- If initial equality, partition, the normal step, the 24-step soak or
  free-fall control fails, reject the candidate and stop particle-layer
  variants. Do not add a third layer: it is outside support for every admitted
  state and cannot address the failure.
- If the candidate survives, record only
  `LOCAL_DISCRIMINATOR_SURVIVED / NO_CORPUS_CREDIT / NOT_SELECTED`. Before any
  profile selection, specify successor roots and run the existing full serial
  corpus plus an external aggregate comparison.
- If it fails despite complete support, the next architecture decision must
  choose between a directly convolved missing-density field and a revised
  lattice-clearance/product profile. Neither is authorized by this document.

`CONTINUUM-WATER-REF-P1` remains `NOT_RUN` in every W0D outcome.

The first stop-rule branch occurred. Further work must reclose the boundary
field, authored initial state and non-pressure/stabilization profile together.
A published density map is not a drop-in replacement for the current lattice:
its basis field deliberately extends into the fluid-side buffer and therefore
changes the equilibrium wall offset. It also requires its own predicted
boundary-density operation, gradient semantics and deterministic equilibrium
generator. A direct density-map implementation under the rejected W0B roots
is not authorized.

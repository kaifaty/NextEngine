# NSR3-B4E2D7R19R65 proportioning/filter discriminator — revision 1

Date: `2026-08-25`

Status: `FROZEN / MEASUREMENT-ONLY IMPLEMENTATION AUTHORIZED`.

## Contract

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R19R65-PROP-FILTER-R1` |
| Architecture snapshot | SPEC-38 `Proposed` v1.6; ADR-076 `Proposed` v1.4; ADR-081 `Accepted` v1.0; R64 bounded sparse equivalence; v6 semantic `bd568e0f...f0e2` |
| Engineering consumer | choose the next R65 phase controller: proportioning/face change versus filter globalization |
| Claim class | finite profile-bound numerical discriminator |
| Claim status target | `SUPPORTED_BOUNDED`, `REFUTED` or `INCONCLUSIVE`; never `PROVED` |
| Budget | one exact v6 replay; zero extra sparse actions, transposes or projections; no timing |

## Exact observation

v6 finds 239 dual-decreasing projected-path candidates. Only 14, all in outer
1, also improve the post-Hildreth composed baseline; 221 worsen it. Candidate
and committed projections agree, so the first unknown is phase suitability,
not recurrence correctness.

## Fixed mathematical model

At a post-Hildreth dual state `lambda>=0`, the fixed-correction QP gradient is

```text
g = -raw.
```

For each owned row define disjoint components:

```text
free_i    = g_i            when lambda_i > 0, else 0
chopped_i = min(g_i, 0)    when lambda_i = 0, else 0
```

The projected-gradient identity is

```text
||g_P||^2 = ||free||^2 + ||chopped||^2.
```

`chopped/free > 1` is frozen only as a scale-free diagnostic split for this
experiment. It is not a selected production `Gamma`, convergence theorem or
tolerance.

Units are those of the nondimensional R63/R64 dual QP. Topology, row order,
binary64 operation order, one Hildreth identification sweep, 15 face-PCG HVPs,
16 dyadic candidates and all joint/model gates remain v6-exact.

## Competing hypotheses

| ID | Causal hypothesis | Prediction | Falsifier | State |
|---|---|---|---|---|
| H1 | face-PCG is invoked before the active face is proportional | every no-composed-descent outer has `||chopped|| > ||free||` | any blocked outer has `||chopped|| <= ||free||` | `TEST` |
| H2 | the face is proportional but density descent conflicts with composed inertia | every blocked outer has `||chopped|| <= ||free||` while dual-decreasing candidates worsen composed inertia | any blocked outer has chopped dominance | `TEST` |
| H3 | maintained recurrence or candidate projection creates the apparent conflict | candidate/commit gap or reprojection disagreement appears | v5/v6 zero gap and exact reprojection | `FALSIFIED_BOUNDED` |
| H4 | the mechanism changes by phase | both H1 and H2 predictions occur across blocked outers | uniform classification of all blocked outers | `TEST` |

Mixed classifications resolve to H4, not to selective removal of inconvenient
outers. A zero free norm is chopped-dominant when chopped is positive; both
zero is a stationary diagnostic. Nonfinite values fail the probe.

## Evidence and controls

1. Record all 16 pre-PCG free, chopped and projected norms, their ratio and
   classification; checkpoints alone are insufficient.
2. Recompute the projected norm directly and require the squared decomposition
   under a frozen `gamma(64*N+256)` bound.
3. Successful scalar control: `lambda=[1,0,0]`, `raw=[2,3,-4]` must give
   free squared `4`, chopped squared `9`, projected squared `13`.
4. Negative control: replacing `min(g,0)` at the lower bound by `max(g,0)` on
   that control gives `20`, must disagree with the authoritative projected
   value `13` and be rejected.
5. Preserve v6 algorithm state/route/result SHA exactly. Put diagnostic fields
   under a separate semantic root so instrumentation cannot gain solver credit.

Primary-source basis is limited to the phase decomposition:

- [Moré and Toraldo, GPCG](https://doi.org/10.1137/0801008): CG explores the
  current face and gradient projection changes faces for strictly convex BQP.
- [Dostál, proportioning and projections](https://doi.org/10.1137/S1052623494266250):
  compares active-variable KKT violation with the free-gradient norm and ends
  face exploration at disproportional iterates.
- [di Serafino et al., two-phase proportionality method](https://arxiv.org/abs/1705.01797v4):
  separates identification and reduced-space minimization phases.

None of these sources covers our alternating Dykstra/contact/inertia
composition; applicability remains an engineering hypothesis.

## Predeclared resolution

```text
all blocked outers chopped-dominant -> PROPORTIONING_PHASE_CANDIDATE
all blocked outers free-dominant    -> FILTER_MERIT_CONFLICT_CANDIDATE
mixed blocked outers                -> PHASE_DEPENDENT_CONTROLLER_REQUIRED
stationary/nonfinite/identity fail  -> DIAGNOSTIC_REFERENCE_RETAINED
```

The strongest possible result is `SUPPORTED_BOUNDED` for selecting the next
experiment on this fixture. It cannot freeze R65, authorize a solver step,
change SPEC-38/ADR-076 status, run performance or mutate runtime state.

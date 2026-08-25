# NSR3-B4E2D7R19R65 projected filter-pair research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / MEASUREMENT CONTRACT FROZEN / IMPLEMENTATION NEXT`.

## Question left by the proportioning discriminator

The v7 discriminator is uniform: all 15 blocked post-Hildreth states are
free-dominant. The active face is therefore not the leading problem on the
frozen fixture. Face-PCG produces fixed-density dual descent, but 221 projected
candidates worsen strict post-Hildreth composed inertia and v6 rejects them.

The next question is narrower than selecting a filter algorithm:

> Do the already generated, normal-safe projected candidates trade a bounded
> increase in inertia for robust reduction of the full linearized density
> violation, or do they fail to improve either coordinate?

No result from this discriminator may accept a step. It only decides whether a
complete filter controller is worth designing or whether the search direction
itself must change.

## Primary-source boundary

Fletcher and Leyffer's filter construction treats objective and constraint
violation as separate coordinates instead of forcing both into one penalty
number. The trust-region SQP-filter analysis accepts a trial only if it clears
the margin against every filter entry and the current iterate, and it keeps
normal-step compatibility, model decrease, restoration and trust updates as
separate requirements:

- Fletcher and Leyffer,
  [*Nonlinear programming without a penalty function*](https://doi.org/10.1007/s101070100244);
- Fletcher, Gould, Leyffer, Toint and Wächter,
  [*Global Convergence of a Trust-Region SQP-Filter Algorithm*](https://doi.org/10.1137/S1052623499357258),
  with an [author-hosted PDF](https://www.numerical.rl.ac.uk/media/people/nick-gould/FletGoulLeyfToinWach02_siopt.pdf).

Dykstra's algorithm is a primal-dual row-action/dual-coordinate method for
best approximation, and Hildreth is its polyhedral special case. This supports
auditing a dual-improving block separately from intermediate primal distance:

- Bregman, Censor and Reich,
  [*Dykstra's Algorithm as the Nonlinear Extension of Bregman's Optimization Method*](https://math.haifa.ac.il/yair/Dykstra.jca99.pdf);
- Tibshirani,
  [*Dykstra's Algorithm, ADMM, and Coordinate Descent*](https://arxiv.org/abs/1705.04768).

These papers do **not** prove convergence of our accelerated density block
inside a joint box-ball Dykstra composition. Their role is to define a
falsifiable bicriteria measurement, not to transfer theorem authority.

## Frozen coordinates

For each post-Hildreth no-PCG joint projection `s_b` and each existing
dual-decreasing projected candidate `s_c`, measure

```text
f(s) = 0.5 * sum_i ||x_origin_i + SPACING*s_i - x_predicted_i||^2
h(s) = ||max(c + A*s, 0)||_2.
```

`f` is the same pure inertia owned by R63. `h` is computed from a fresh R61
directed JVP over all 6,000 rows, not from owned rows or a candidate cache. The
PHR/dual objective is excluded from `f`; it remains a mandatory independent
descent gate and must not be counted twice.

The local source is the no-PCG composed baseline from the same outer. This is
a one-step pair discriminator, not a persistent filter. A candidate remains
eligible for classification only when it already has:

1. strict fixed-density dual quadratic decrease;
2. strict positive inertia reduction relative to the immutable cached-normal
   reference;
3. finite exact projection and row audit.

## Envelope audit

Reuse R46's fixed dyadic margin ladder without choosing a production value:

```text
gamma_k = 2^-k, k=1..24

h_c <= (1-gamma_k)*h_b
    OR
f_c <= f_b - gamma_k*h_b.
```

Record all 16 outers and all 239 dual-decreasing candidates. A robust candidate
passes `gamma=1/2` by the feasibility branch with slack above

```text
1024*epsilon*max(h_b,h_c,min_normal).
```

A weak candidate passes only the `gamma=2^-24` envelope. The ladder is a
discriminator; no observed margin becomes policy.

## Competing hypotheses

| ID | Hypothesis | Prediction | Falsifier |
|---|---|---|---|
| F1 | strict composed inertia hides robust feasibility progress | every blocked outer has at least one normal-safe candidate passing `gamma=1/2` by `h` | any blocked outer lacks a strong candidate |
| F2 | the PCG path is not a useful bicriteria direction | no blocked outer has a normal-safe candidate passing even `gamma=2^-24` | any weak or strong candidate |
| F3 | merit conflict depends on phase or margin | only a strict subset of blocked outers is filter-acceptable, or all require weaker-than-`1/2` margins | uniform strong or uniform none |
| F4 | v6 rejection is an operator/measurement artifact | fresh directed residual disagrees with inherited state/provenance or old semantic changes | exact fresh audit and unchanged v6/v7 roots |

Predeclared routes:

```text
15/15 blocked outers strong -> UNIFORM_FILTER_FEASIBILITY_PATH_CANDIDATE
15/15 rescued, some weak     -> MARGIN_SENSITIVE_FILTER_PATH_CANDIDATE
0/15 rescued                 -> COMMON_DESCENT_DIRECTION_REQUIRED
otherwise                    -> PHASE_DEPENDENT_FILTER_REQUIRED
identity/control failure     -> FILTER_PAIR_REFERENCE_RETAINED
```

## Work and controls

One replay performs 16 fresh baseline and 239 fresh candidate directed JVP
audits: exactly 255 diagnostic pair passes, or 154,311,720 directed slots.
They are accounted separately from v6's unchanged 468,968,743 solver terms.
No new transpose, joint projection, candidate, PCG iteration or state update is
allowed.

Dense controls must prove:

- feasibility admission with worse objective;
- objective admission with unchanged violation;
- dominated rejection;
- positive-part violation (`[-3,4] -> h=4`) and rejection of an absolute-value
  substitute (`h=5`);
- source self-rejection at every positive margin;
- route precedence and solver/v7 semantic preservation.

The strongest possible outcome is `SUPPORTED_BOUNDED` for selecting the next
research contract. No runtime, production, performance or convergence claim is
available.

Frozen contract:
[projected filter-pair discriminator](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r65-filter-pair-contract.md).

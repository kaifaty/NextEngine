# NSR3-B4E2D7R19R1 topology-precision shell research

Date: `2026-08-23`

Status: `RESEARCH_COMPLETE / CONTRACT_FROZEN / IMPLEMENTATION_NEXT`

## Question

Do D7R19's long-double membership mismatches change the discrete objective or
accepted sign, or are they confined to the C2 compact-support zero shell where
binary64 and extended arithmetic choose different membership bits for a
physically null pair?

The distinction matters. The runtime candidate is a binary64 discrete solver,
so an extended-precision acceptance oracle must audit that same discrete
objective. It may not silently switch to a different pair topology. Conversely,
freezing binary64 membership is not justified if mismatched pairs carry
material kernel value or change the sign.

## Analytic boundary

For the selected cubic kernel, `q=2r/h`. At `r=h`, the outer branch gives:

```text
W(h)   = 0
W'(h)  = 0
W''(h) = 0
```

Thus a membership-bit disagreement exactly at the horizon is a representation
difference, not a force discontinuity. Near the horizon, value, gradient and
curvature decay cubically, quadratically and linearly. D7R19 observes minimum
horizon margin zero in all three mismatched trials, but it does not report the
maximum mismatch distance or the aggregate effect.

Use the conservative, formula-derived shell bound
`64 * epsilon_binary64 * h`. This is not fitted to the observed result. Any
mismatched pair outside it rejects the zero-shell hypothesis.

## Competing hypotheses

| Hypothesis | Evidence for | Evidence against | Discriminator |
|---|---|---|---|
| H1: C2 zero-shell representation | `7,315` unique-state mismatches, every target has minimum horizon margin zero, all three existing extended signs are resolved positive | minimum margin does not bound every mismatched pair | enumerate every mismatched pair; require shell bound and exact `W/W'/W''` horizon closure |
| H2: extended topology changes acceptance | the long-double evaluator currently executes its own membership branch | no resolved negative sign has appeared | compare live-extended, binary64-owned and horizon-canonicalized extended reductions in long double and binary128 |
| H3: sparse union/topology bug | mismatch count is large | exact pair-union construction and zero all-pair calls already pass | require exact union/order/target roots and classify any mismatch outside the shell as nonlocal |

## Selected experiment

Replay only D7R19 outer `0`, accepted trials `0`, `1`, and `2`. Reproduce the
parent report and exact long-double roots before diagnostics. For every
mismatched pair record endpoint identity, binary64 radius bits, extended
radius-minus-horizon, binary/extended membership and kernel value/first/second
derivative magnitude.

Evaluate three non-mutating precision lanes over the same sorted sparse union:

1. live extended membership, reproducing D7R19;
2. binary64-owned membership with long-double/binary128 arithmetic;
3. horizon-canonicalized mismatches, setting only the mismatched radius to
   exact `h` in the extended evaluator.

Naive and compensated long-double reductions must resolve at `1024` extended
ULPs. Naive and compensated binary128 reductions must resolve at `4096`
binary128 ULPs. The existing candidate divided reduction retains its `5%`
relative-error gate. No lane may accept a trial or update solver state.

## Route decision

Route in this order:

1. `NONLOCAL_TOPOLOGY_MISMATCH` if any mismatch is outside the derived shell,
   pair/union identity differs or C2 horizon closure fails;
2. `TOPOLOGY_MISMATCH_ALTERS_SIGN` if a resolved lane contradicts the
   candidate/live positive sign;
3. `TOPOLOGY_PRECISION_UNRESOLVED` if required signs do not resolve;
4. `RUNTIME_TOPOLOGY_PRECISION_CANDIDATE` only if all three lanes preserve the
   positive sign and every mismatch is a certified C2 shell event.

A candidate route authorizes only a separately frozen full precision-audit
reclosure. It does not waive D7R19, change the HVP cap, rerun the nominal
transaction, authorize runtime wider precision or publish state.

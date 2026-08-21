# NSR3-B4C3Q -- aggregate-balanced canonical quantization discriminator

Status: `PASS / CANONICAL_AGGREGATE_BALANCED_CANDIDATE / B4C3A1_DESIGN_AUTHORIZED`

Parent B4C3A selects `JOINT_PRESSURE_CANONICAL_STAGE_CANDIDATE`; semantic
SHA-256 is `fdaa0befd5538cdf76ff8601a655bf1f9726b5ced19db36943a665f8b06bb5fe`
and JSON-without-final-LF SHA-256 must equal
`45b4f8965be67f90878a6f4d703c978ed74914457cd7f64aaccbc84491dd8a48`.

## Candidate identity

```text
identity  canonical-aggregate-balanced-apportionment-r0
profile   f57d88c222a8334206a962a72a814bdd530696ed2767c94fa88910281265579c
algebra   bf3046b6fad7675a3f784d11864fd7c42ce5e63116e3103cfb1cdf639cfe265e
P1        d42eeb5753b654e5a13fe3d4db30010bc77118c6f216f9b9cb6ca404643124de
P2        5755be013a3704373b7cb2d21ea1608d6497dec980120af42d73fa7d9f802cd8
temporal  f90fed20f6ecd56c7b52fe57f7f0d90afc8b17957aa4e0c33c61d3d1e8b37aa3
```

Roots are SHA-256 of the exact ASCII domain strings recorded in the dated
research note. The candidate remains isolated from NPR1 and runtime schemas.

## Exact apportionment

Canonical-sort by ascending `SampleId`. For each position/velocity component:

1. Compute existing integer nearest-ties-to-even `r_i` at scale `1,000,000`.
2. Compute `T = round_even(sum_i(value_i * scale))` from the exact binary64
   rationals, without a floating reduction.
3. Let `D = T - sum_i r_i`.
4. For `D > 0`, add one to the `D` smallest exact residuals
   `r_i - value_i*scale`; for `D < 0`, subtract one from the `-D` largest.
   Equal residuals use ascending `SampleId`.
5. Validate final position/velocity integer ranges and publish with the existing
   frame/root byte encoding.

No sample may receive more than one correction. Failure is typed and atomic.
The implementation may use exact multiprecision integers in this research tool;
no runtime dependency or performance claim follows.

## Algebraic gates

Compare independent nearest-even and balanced publication on:

- positive and negative half ties;
- 48 equally biased `+0.49`-unit residuals;
- mixed signed/correlated residuals;
- exact-integer and near-range values;
- exhaustive small vectors up to eight samples against all feasible one-unit
  correction subsets.

Require for the balanced candidate:

- aggregate error `<=0.5` scaled unit per component;
- local error `<1` scaled unit per component;
- exact target sum and minimum squared error versus exhaustive oracle;
- exact repeat, reverse and coprime-affine input-order frames/roots;
- exact sign covariance and canonical-lattice translation covariance: add an
  integer number of microunits to the published integers, decode, then publish;
  do not add a non-representable decimal microunit to an arbitrary unquantized
  binary64 input and mistake binary addition error for a policy transform;
- exact equality to nearest-even whenever `D == 0`;
- the biased-48 aggregate error at least `16x` smaller than independent
  nearest-even.

Nonfinite, range, duplicate-ID, capacity and infeasible-apportionment paths must
be typed, commit nothing and preserve the pre-transaction root.

## Physical and temporal gates

Run P1 `21/42` and P2 `1/2` one-frame staged transactions with balanced
publication and decoded continuation. Preserve all B4C3A transaction/order/
failure gates. Against both the binary64 fine run and nearest-even B4C3A fine
run require:

- RMS position `<=100e-6 m`;
- RMS velocity `<=1e-3 m/s`;
- contact feature set exact and contact timing within one coarse substep;
- unchanged embedded coarse/fine gate PASS.

For every publication report the exact aggregate position and velocity error,
the implied momentum impulse, center shift and kinetic-energy change. Require
the aggregate component bounds implied by `0.5` unit and verify

```text
abs(delta K) <= m * (norm(v) * norm(delta v)
                     + 0.5 * norm(delta v)^2)
```

using flattened vectors and an explicit floating-point allowance.

Run a 48-sample, 1,024-step deterministic free-flight/residual-stress control.
Require balanced center-velocity and center-position drift within the summed
per-step aggregate bounds and strictly below independent nearest-even drift.
This is a representation discriminator, not a fluid-accuracy claim.

## Exit

Two reports must be byte-identical. B4C3A, NPR1-A and B4C2T raw reports remain
exact.

PASS selects `CANONICAL_AGGREGATE_BALANCED_CANDIDATE` and authorizes only
B4C3A1 selected-policy one-frame transaction revalidation under a new profile.
FAIL preserves independent nearest-even only as B4C3A transaction evidence and
requires choosing snapshot-only publication or another separately frozen
policy. No B4C3T, nominal, CUDA, runtime, schema or production authority is
granted.

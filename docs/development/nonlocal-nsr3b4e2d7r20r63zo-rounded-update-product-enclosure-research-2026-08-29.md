# NSR3-B4E2D7R20R63ZO rounded-update product-enclosure research

Status: `PREFLIGHT_NEGATIVE / TIGHT_PRODUCT_ENCLOSURE_REJECTED / CLOSED_BEFORE_PACKAGE / NO_ENDPOINT_AUTHORITY`.

## Decision

R63ZN is closed `INCONCLUSIVE`; it cannot be repaired, re-reviewed or consumed.
The next experiment therefore does not build another recurrence checker. It
asks whether the already reviewed R63ZM products for two adjacent states,
together with the exact set of real updates that round to the later state, can
rigorously enclose the missing first direction product.

The frozen boundary is the
[R63ZO contract](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r20r63zo-rounded-update-product-enclosure-contract.md).

## Competing hypotheses

| Hypothesis | Discriminator | Prior belief |
|---|---|---|
| H1: exact dense `|H|` keeps the update-rounding image narrow enough | derive the exact binary128 update-preimage intersection and propagate its error through `|H|` | plausible because the observed R63ZN `rho0` margin is wide, but untrusted author values cannot decide it |
| H2: only the cancellation-free rectangular envelope is too loose | compare `|H|u` with `sigma|T|(|T|^T u)` under identical scalar gates | plausible because the tangent Gram contains substantial cancellation |
| H3: even the tight envelope is destroyed by conditioning | test strict denominator positivity and step-subset containment | plausible because prior active-block condition estimates are order `1e15` and weak directions are tiny |
| H4: the cached transition is inconsistent with the independently derived direction | intersect all 102 exact rounding preimages and run a late direct-product containment oracle | low prior probability, but it is the apparatus-failure alternative |

The experiment is deliberately asymmetric: a positive result is only a
post-state enclosure certificate, while a negative tight-envelope result
eliminates this entire reconstruction path and justifies returning to a direct
direction-product representation.

## Why this differs from R63ZN

R63ZN required bit-exact causal consumption of `H*x0` before constructing any
future state. R63ZO instead starts from reviewed adjacent state/product pairs
and uses the later state only as an explicit witness. It does not admit the
R63ZN prefix, trust an author route, reconstruct `p0` by rounded subtraction or
claim it can generate `x1`.

The scalar identity is real-valued and interval-based:

```text
x1 = round(x0 + alpha0*p0)
delta = x1 - (x0 + alpha0*p0)
H*p0 = (H*x1 - H*x0 - H*delta) / alpha0.
```

The earlier one-scalar absorption counterexample refutes exact subtraction but
does not refute a sound enclosure of `delta`. R63ZO measures whether that
enclosure is useful on the frozen profile.

## External evidence

Bounded web research on `2026-08-29` used primary sources only:

- [Bagnara et al., Correct approximation of IEEE 754 floating-point arithmetic for program verification](https://doi.org/10.1007/s10601-021-09322-9)
  proves interval filtering rules for correctly rounded operations and exact
  round-to-nearest predecessor/successor error cells. This supports deriving
  `A0` from the actual binary format rather than assuming a uniform epsilon.
- [Rump and Ogita, Verified Error Bounds for Matrix Decompositions](https://doi.org/10.1137/24M165096X)
  documents rigorous entrywise inclusions using accurate dot products and
  outward bounds. This supports separating primaries from their error terms.
- [Carson, The Adaptive s-Step Conjugate Gradient Method](https://doi.org/10.1137/16M1107942)
  shows that block Krylov reformulations require an explicit computed basis
  whose conditioning controls attainable accuracy. It does not support
  treating two cached state products as a free Krylov basis.
- [Barrett et al., Templates for the Solution of Linear Systems](https://www.netlib.org/utk/papers/etemplates/node10.html)
  keeps the matrix-vector product and preconditioner as explicit PCG
  operations. This supports treating direct `H*p0` as a late falsifying oracle,
  not silently erasing it from the algorithm.

These sources motivate the method; none proves that the fixed NextEngine
profile will pass.

## Smallest next action

Implement one offset-only, whole-file-hash-closed preflight that independently
derives `p0`, constructs exact update-preimage endpoints, evaluates both
absolute-operator envelopes and runs the late direct-product oracle. Publish
the widths and first failing gate twice, then stop. The preflight may select a
future contract direction but cannot establish the R63ZO claim.

No R63ZN repair, checker package, complete recurrence, representation choice,
dynamic builder, corpus, timing, runtime, Rust, GPU or production work is
authorized.

## Implemented preflight

The new standalone target
`nonlocal-formula-r63zo-rounded-update-preflight-release` uses strict C++20,
round-to-nearest binary128, disabled contraction and the three whole-file
R63ZM identities. It independently performs:

- the two factor solves needed for `x0` and `p0`;
- two state products and their published bound-root comparisons;
- all `102` update-preimage constraints;
- `5,253` dense Gram dots plus `10,404` absolute-bound terms;
- both `32,130 + 32,130` rectangular passes; and
- one late `64,260`-term direct-product oracle and `102` compensated update
  checks.

The first diagnostic incorrectly checked the late update as a rounded
multiplication followed by a rounded addition and matched only `94/102`
components. Source inspection showed that the frozen recurrence calls the
two-term compensated `Dot2` schedule. A separate minimal discriminator found
`94/102` for the two-operation expression and `102/102` for both `fma` and the
actual `Dot2`, with all `102` exact/no-underflow flags set. The contracted
real-expression rounding model therefore remained valid; only the oracle
control implementation was corrected.

## Reproducible result

Source and Release identities:

```text
rounded_update_preflight.cpp  b4afd531637403d3a7874ffdb61effe3f397bea0c979122153af9e0aa0bb76b0
Release executable             5bcffaf790aaf3b3a0ede538ecf9daa0dc288fa252da846418023484ce226245
stdout run 1                   d3cfe83ecf064ce042cae7bbb17a9a8a6f2376f6bc548edab621ce082f2d5ce3
stdout run 2                   d3cfe83ecf064ce042cae7bbb17a9a8a6f2376f6bc548edab621ce082f2d5ce3
sanitized stdout               d3cfe83ecf064ce042cae7bbb17a9a8a6f2376f6bc548edab621ce082f2d5ce3
```

LeakSanitizer is unavailable under the desktop ptrace environment; ASan/UBSan
with leak detection disabled completed and reproduced the Release stdout
byte-for-byte.

The first-specific route is:

```text
TIGHT_PRODUCT_ENCLOSURE_REJECTED
```

Load-bearing observations are:

```text
x0/state products/compensated updates       102 / exact / 102
alpha outer-preimage width                  0x1p-112
late direct alpha                           0x1.d08d760f7ac79a531bbaa83cb76cp-1
late direct product contained               yes, both envelopes
tight denominator lower                     0x1.10f6ed39e7f2b399bffffffffffep+4
tight step                                  [0x1.cb8097...p-1, 0x1.d5b710...p-1]
rectangular denominator lower               0x1.0f4aa05c90acfcdebffffffffffep+4
rectangular step                            [0x1.c8c37b...p-1, 0x1.d89ca0...p-1]
```

The tight step width is about `0.01995`, or `2^106.35` times the published
outer update-consistency width. This is not caused by the reviewed state
products: their weighted denominator uncertainty is only about `2^-54`. The
tight absolute image of the update rounding contributes about `2^-3`; the
rectangular construction contributes about `2^-2`. Worst product components
are rows `50` and `65`, respectively.

## Conclusion and claim ceiling

H1 and H2 are rejected for this fixed profile; H3 is selected. The direct
oracle and positive-denominator gates pass, so H4 is also rejected after the
compensated-update correction. Exact rational midpoint/tie machinery and the
full receipt/control package were deliberately not built after the frozen
tight-envelope stop fired. Accordingly this is a negative feasibility
selection, not a positive mathematical enclosure claim.

R63ZO is closed before a checker package. Do not infer a direction product
from adjacent rounded state products, tighten by an observed oracle, fit a
tolerance or retry the same absolute-envelope construction. The smallest
materially different successor question is whether one directly computed
`H*p0` value-plus-error artifact can be independently admitted at the fixed
R63ZM boundary without recurrence, cached future states or any R63ZN/R63ZJ/K/L
authority.

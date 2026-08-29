# NSR3-B4E2D7R20R63ZO rounded-update product-enclosure research

Status: `CONTRACT_FROZEN / PREFLIGHT_PENDING / NO_ENDPOINT_AUTHORITY`.

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

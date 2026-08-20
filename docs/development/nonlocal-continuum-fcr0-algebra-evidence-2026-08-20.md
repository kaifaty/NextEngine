# Nonlocal FCR0 algebra evidence — 2026-08-20

Status: `FCR_ALGEBRA_CANDIDATE / PASS / REPORT_ONLY`

## Outcome

The separate `nuv-variational-fcr1` identity passes its strict binary64
formula controls. Every declared smooth-region analytical derivative agrees
with an independent central directional difference below the frozen `1e-7`
limit. Translation and equal/opposite closure pass below `1e-12`. Two fresh
reports are byte-identical.

This result proves only internal consistency of the discrete potentials and
forces in the [FCR0 contract](../plans/nonlocal-continuum-formula-reclosure/00-formula-contract.md).
It does not prove SISSM convergence, water behavior, CPU/GPU correspondence,
performance or production readiness.

## Implementation boundary

Commit `0ddf5797e76b6e7873d8c957ee3a92011ce58097` adds a separate
`nonlocal-formula-reclosure` executable. It does not call the stopped CPU
oracle, source-shaped term controls or CUDA kernels. It independently
implements:

- the cubic `W(r)` and physical-distance derivative `dW/dr`;
- compression-only density energy plus the literal two-sided comparator;
- time-integrated bulk/shear viscous potentials with `omega=-dW/dr`;
- the explicit physical-distance surface potential
  `C(r)=r0*C_hat(r/r0)`;
- analytical gradients, central differences, translation controls and
  equal/opposite pair closure.

The target compiles with contraction and fast math disabled.

## Results

| Control | Relative derivative error | Result |
|---|---:|---|
| cubic kernel, inner branch | `6.37439e-11` | PASS |
| cubic kernel, outer branch | `4.83663e-11` | PASS |
| compression-only, above rest | `1.23537e-9` | PASS |
| compression-only, below rest | `0` | PASS |
| two-sided below-rest comparator | `1.09602e-9` | PASS as comparator |
| bulk viscosity | `3.83647e-9` | PASS |
| shear viscosity | `2.55782e-9` | PASS |
| surface repulsive branch | `1.50050e-8` | PASS |
| surface attractive branch | `3.82429e-9` | PASS |
| surface outside support | `0` | PASS |

All pair-closure errors are exactly zero. Maximum translation error is
`1.72e-14`.

The pressure discriminator is decisive at the same underdense state:

```text
compression-only derivative = 0
two-sided derivative         = -3090.4671958606159
```

This establishes that the two semantics are observably different. FCR1 must
still test their bounded free-surface behavior before closing the selection.

## Exact artifacts

| Artifact | SHA-256 |
|---|---|
| FCR0 raw report, run 1 | `996eff3d61126491a3c1c92b6147d1c1f3eec0487d4b9dee45caadc6588b4345` |
| FCR0 raw report, run 2 | `996eff3d61126491a3c1c92b6147d1c1f3eec0487d4b9dee45caadc6588b4345` |
| FCR0 executable | `5f8f865f89c0fcdd48cd36a203c5ab9358f4a3c2d70139fb2da77252fb3f4156` |
| FCR0 result root in report | `ea5c4b423ce47b89699150dd17086ac63ccc4e4a59ec234a829a066022b54412` |
| implementation `.cpp` | `beb331df7b0321fc3235e6b085756cd5a10c5effe9854104203901203436fbe0` |
| frozen NPR1-A report | `464a55bc33741f18ddc4b3c1cda6b46b5248bb653789a5e31585482f871ee207` |
| frozen NPR1-B failing report | `e8ed6950e33fb9388887c40e304c74b517761698c0c7accbaff9a99899bf2c3a` |

The retained CUDA source-shaped self-test also returns `PASS`. Its raw report
contains runtime timings and is intentionally not treated as a stable hash.

Execution used Linux `7.0.0-29-generic`, GCC `15.2.0`, CUDA `13.3` and an
RTX 3080. FCR0 itself executes on CPU binary64.

## Decision

Select `FCR_ALGEBRA_CANDIDATE` and proceed to FCR1. Runtime, CUDA authority,
old-profile inheritance and product-scale execution remain blocked.

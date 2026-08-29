# NSR3-B4EP5 HVP coefficient-tape research -- 2026-08-22

Status: `COMPLETE / IMPLEMENTED_AND_PASS / RESEARCH_ONLY`

## Question

Which mechanical HVP change can remove a material part of B4EP4's 62.14%
inclusive cost without changing the Hessian formula, pair/reduction order,
trust-region schedule or existing command bytes?

## Exact hot-path audit

`apply_joint_pressure_tape` visits every active centre's directed adjacency
twice. The first pass forms the scalar `Jv`; the second scatters
`J^T J v + compression * H v`. Across 459 HVPs, the B4EP4 call graph records
971,831,424 calls to `kernel_scale`, split as three kernel-coefficient
evaluations per active directed visit:

```text
weight_gradient(radius)                       first pass
weight_gradient(radius), weight_second(radius) second pass
```

Radius and positions are fixed for the lifetime of one pressure tape, so both
scalar function results are invariant across every HVP using that workspace.
The current transaction visits 85,716,150 unique pairs across 226 workspaces.
Computing two scalars once per pair would perform 171,432,300 kernel
evaluations and replace 800,399,124 repeated evaluations, an 82.36% reduction
in this exact call class.

The wrappers also allocate joint direction, joint output and fluid output
vectors. Their conservative aggregate payload is about 311 MiB over the whole
process, but allocator/vector-growth leaves receive no gprof samples. Output
reuse is therefore not selected first.

## Candidate and exactness boundary

Extend only the optional candidate pressure tape with two arrays indexed by
the existing canonical pair index:

```text
gradient[p] = weight_gradient(radius[p])
second[p]   = weight_second(radius[p])
```

The HVP substitutes these stored binary64 values at the exact call sites. It
continues to recompute displacement, normal, `gradient/radius`, relative
direction, radial Hessian product and endpoint accumulation in the same order.
It does not cache normals, a matrix, scaled coefficients or pair contributions;
those alternatives can change operation grouping or opposite-centre rounding.

The coefficient arrays are transaction-only and opt-in through the internal
query trace. Existing commands build the unchanged radius/compression tape.
The full-state parent remains unchanged. Missing/partial coefficient arrays
fail closed; there is no per-call fallback.

Two doubles per admitted pair add at most 15,360,000 bytes to one workspace
under the existing 960,000-pair capacity. At most two workspaces may be live.
The candidate reports coefficient builds, pairs, kernel evaluations, HVP
lookups and maximum payload separately.

## Alternatives rejected for this stage

- Precomputed normals or Hessian matrices: larger memory and changed arithmetic
  grouping before an invariant-scalar result exists.
- One-pass pair reduction: `Jv` is required before the scatter pass and a
  different traversal would change reduction order.
- Reusable output buffers: no measured allocation dominance; it can follow
  only if scalar caching leaves allocation material.
- CPU threads/SIMD/GPU: worker partitions and merge order need separate exact
  contracts after the serial data path is minimized.
- Fewer HVPs or different trust tolerances: solver-policy changes, not a
  mechanical optimization.

## Decision

Freeze one B4EP5 optional coefficient-tape implementation and controlled
Release A/B against B4EP3I. Keep it only on exact B4EP1 physics, exact old
command bytes, the predeclared work counts, three timing wins and median
speedup at least `1.10x`. Otherwise preserve B4EP4 and research another HVP
mechanism without stacking changes.

The implementation passes all gates with median paired speedup `1.2482x`;
see the [dated evidence](nonlocal-nsr3b4ep5-hvp-coefficient-tape-evidence-2026-08-22.md).

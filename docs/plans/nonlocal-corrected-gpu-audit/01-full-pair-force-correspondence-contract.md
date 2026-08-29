# Nonlocal corrected CUDA full-pair force correspondence — revision 2

| Field | Value |
| --- | --- |
| Research ID | `NCGA0` revision 2 |
| Status | `FROZEN / REPAIR_IMPLEMENTATION_PENDING / REPORT_ONLY` |
| Supersedes | Revision 1 only for its ambiguous viscosity-output definition; revision 1 remains a recorded `NO-GO` |
| Architecture snapshot | `349f12e66be970276d91c96bda1f33fad1a9919e`; SPEC-38 and ADR-076 remain `Proposed`; ADR-081 remains `Accepted` |
| Engineering consumer | Decide whether a corrected Nonlocal CUDA lineage may proceed from full-pair scalar/force terms to a separately frozen neighborhood/linearization audit |
| Claim class | Finite profile-bound implementation correspondence |
| Review budget | The single repair and single re-review remaining from revision 1 |

## Reason for revision

Revision 1 used the phrase “directed-pair coefficient” while its observable was
the full equal/opposite endpoint force of one undirected pair. The independent
review correctly rejected that ambiguity: the directed-edge energy coefficients
and the derivative of the combined pair energy differ by a factor of two.

Revision 2 changes no fixture, profile, arithmetic mode or tolerance. It fixes
the observable and adds an independent energy-derivative discriminator that
must distinguish the two identities.

## Exact claim

On the current Linux x86-64 host with NVIDIA RTX 3080 (`sm_86`), CUDA 13.3,
strict binary32 device arithmetic (`--fmad=false --prec-div=true
--prec-sqrt=true --ftz=false`) and the immutable revision-1 profile, a
separately implemented CUDA evaluator matches an independent host `long double`
evaluator on the same nine scalar/full-pair-force fixtures.

For every nonzero scalar or force component,

```text
abs(candidate - reference) <= 2e-5 * max(1, abs(reference)).
```

Every reference zero requires candidate absolute value `<=2e-7`; all outputs
must be finite; endpoint forces must close within `2e-7`. Ten cold CUDA
allocations/executions must produce byte-identical binary32 payloads.

For the two viscosity fixtures, an additional independently implemented host
energy oracle evaluates a central directional derivative at
`epsilon=h*1e-8`, moving only the first endpoint along normalized
`(-0.27, 0.91, 0.31)`. The CUDA first-endpoint force projection must satisfy

```text
abs((-F_i dot d) - dE/depsilon)
    <= 2e-5 * max(1, abs(dE/depsilon)).
```

The historical source-shaped gradient must reject `kernel_inner`,
`kernel_outer` and `compression_above_rest`. A deliberate directed-edge
half-force variant must reject both `bulk_viscosity` and `shear_viscosity`
against the full-pair component and energy-derivative gates.

## Exact viscosity observable

Let

```text
delta = (y_i-y_j) - (x_i-x_j)
P_n = n*n^T
P_t = I-P_n
omega(r) = -dW/dr
```

and define the frozen undirected-pair energy

```text
E_pair = m/(rho0*dt) * omega(r)
       * [mu * |P_t delta|^2 + (lambda/2) * |P_n delta|^2].
```

The candidate output is the full endpoint force `F_i=-dE_pair/dy_i`, not one
directed-edge accumulator contribution:

```text
F_i = -m/(rho0*dt) * omega(r)
    * [2*mu*P_t delta + lambda*P_n delta]
F_j = -F_i.
```

Thus `mu` and `lambda/2` are the energy coefficients, while `2*mu` and
`lambda` are the full-force coefficients. The half-force control uses
`mu` and `lambda/2` as force coefficients and must fail.

## Frozen definitions, fixtures and identities

- All profile values, kernel, compression and surface definitions are exactly
  revision 1.
- Fixture order and every input value are exactly revision 1.
- Host term oracle remains an independent `long double` force implementation.
- The new energy oracle is a separate `long double` translation unit that
  implements energy plus central differencing, not either force formula.
- CUDA candidate remains binary32, one thread per fixture, without atomics,
  reductions, neighbor search, solver update or shared formula code.
- Revision-1 contract SHA-256:
  `71538590e6a6dfa8f2a7c55ffe02ccc91785be13e280d5ab6b997851a2e29bc2`.
- Revision-1 candidate:
  `349f12e66be970276d91c96bda1f33fad1a9919e`.
- Frozen parent FCR contract SHA-256:
  `8d693724d6b32d4aa899f57551b45248d645a1ecdcec3ec20cd66572b4a1c5ab`.

## Resolution and near-miss firewall

- Positive: all revision-1 corrected fixture/component/closure/repeat gates,
  both independent energy-derivative gates, all five named negative gates and
  all sanitizers/non-regression controls pass.
- Negative: retain the first failed gate; no input, tolerance or arithmetic
  change after observation.
- Does not count: CPU/CUDA agreement without the energy oracle; closure alone;
  a half-force interpretation; successful build; aggregate appearance; hidden
  doubles; relaxed bounds; or a different device.
- Claim ceiling: corrected scalar/full-pair-force term correspondence only. No
  neighborhood/indexing, local matrix, `3x3` solve, SISSM, recurrence,
  multi-step physics, performance, canonical authority, runtime or production
  claim.

## Evidence and re-review protocol

1. Build in a fresh directory outside Git and run the exact target twice.
2. Require ten cold corrected runs, the three source-gradient negatives, the
   two half-force negatives and both energy derivatives in every process.
3. Run `memcheck`, `initcheck`, `synccheck` and the same three old controls as
   revision 1.
4. Freeze repaired source, binary, stdout, fixture/result roots and exact
   revision-1-to-revision-2 diff.
5. Give the original reviewer this revision first, then only the repaired diff
   and candidate sources. The candidate stays read-only.

## Stop and reconsider

- Stop `INCONCLUSIVE` if the energy oracle shares force code, either negative
  identity passes, the device/toolchain cannot execute, or any load-bearing
  defect survives the one re-review.
- Do not patch `cuda_baseline.cu`, alter revision-1 files to erase the failed
  history, change fixtures/tolerances, or proceed to a full solver port.
- Even a positive re-review leaves GPU work correspondence-only under SPEC-38
  and ADR-076/081.

# Nonlocal corrected GPU objective assembly audit — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / NCGA2_CONTRACT_FROZEN / IMPLEMENTATION_PENDING` |
| Updated | `2026-08-30` |
| Task key | `nonlocal-corrected-gpu-assembly-audit` |
| Scope | Test tiny strict-f32 CUDA energy/gradient/Hessian assembly for the corrected Nonlocal objective, without reviving the stopped SISSM solver |
| Definition of done | NCGA2 receives one bounded independently reviewed result or stops at its first reproducible failing boundary |
| Authority | Working context only; SPEC-38, ADR-076/081, FCR0 and the frozen NCGA2 contract outrank this file |

## Resume in 60 seconds

- **Current state:** NCGA0 corrected terms and NCGA1 exact neighborhood/index
  construction are independently reviewed `GO / SUPPORTED_BOUNDED`.
- **Decision:** NCGA2 assembles objective energy, analytical gradient, exact
  dense Hessian/diagonal blocks and HVP on tiny immutable graphs. It does not
  port the stopped SISSM local matrix.
- **Why:** FCR3-B2 already rejected the pressure-bearing SISSM/Chebyshev
  recurrence. The nonlinear objective and its derivatives remain the valid
  mathematical boundary for a future separately selected solver.
- **Next action:** implement independent `long double` analytical and
  energy-only derivative oracles, then the owner-row strict-f32 CUDA candidate
  and six negative identities.
- **Current blocker:** none; RTX 3080 (`sm_86`) and CUDA 13.3 are available.
- **Claim ceiling:** tiny objective assembly correspondence only; no solve,
  trajectory, performance, runtime or product-water claim.

## Competing hypotheses

| Hypothesis | Prediction | Discriminator | Status |
| --- | --- | --- | --- |
| H1 corrected GPU assembly corresponds | graphs, density, energies, gradient and Hessian pass the frozen mixed bounds | independent long-double oracle plus energy-only derivatives | open |
| H2 historical mismatch was in accumulation/composition | isolated NCGA0 terms pass but active-pressure or combined assembly differs | active pressure and combined clusters | open |
| H3 a wrong matrix can hide behind matching forces | gradient passes while exact Hessian or second derivative fails | dense matrix, HVP and Gauss-Newton/SISSM negatives | open |
| H4 reference/current graph roles are mixed | support-crossing viscosity or current terms differ | `reference_current_support_crossing` | open |

## Decisions

### D-001 — Do not port the stopped SISSM split

- **Observation:** FCR3-B2 closed the existing SISSM/Chebyshev lineage after
  reproducible pressure non-descent/quality failures.
- **Evidence:** `docs/development/nonlocal-continuum-fcr3b2-chebyshev-evidence-2026-08-20.md`.
- **Conclusion:** CPU/GPU equality for that local split would not select a
  corrected solver.
- **Decision:** assemble derivatives of `nuv-variational-fcr1` directly and
  include the historical SISSM matrix only as a mandatory rejected identity.
- **Rejected alternatives:** port old `source/local_matrix`, or jump directly
  to a full GPU solve.
- **Consequences:** NCGA2 can feed a future matrix-free operator audit but
  cannot claim a solver step.
- **Remaining uncertainty:** whether strict-f32 active-pressure Hessian
  composition stays inside the frozen bound.
- **Reconsider when:** a separately reviewed solver chooses a different
  derivative/operator identity.

### D-002 — Keep product and research profiles separate

- **Observation:** SPEC-38 V1 selects CPU DFSPH with `0.1 m` support and no
  viscosity/surface model; NCGA0/FCR uses the Nonlocal `0.15 m` audit profile.
- **Decision:** NCGA2 stays on the latter and names it report-only. It does not
  inherit SPEC-38 product authority from NCGA1's exact integer cache test.
- **Consequence:** successful assembly is useful GPU-port evidence, not a claim
  that the V1 game water profile is implemented.

## Required context

1. `docs/architecture/agent-routing.md`, SPEC-38 and ADR-076/081.
2. FCR0 formula contract and FCR3-B2 stop evidence.
3. NCGA0 and NCGA1 contracts, task states and independent reviews.
4. `docs/plans/nonlocal-corrected-gpu-assembly-audit/00-objective-assembly-correspondence-contract.md`.

## Do not retry or infer

- do not use `variational_reference.cpp` or historical CPU/CUDA equality as
  the independent oracle;
- do not replace the exact Hessian with a clamped/Gauss-Newton/SISSM block;
- do not hide a host-built candidate graph or floating atomics;
- do not report NCGA2 work counts or the parallel NCGP0 neighborhood benchmark
  as full solver/game throughput.


# Nonlocal continuum NR1 baseline evidence — 2026-08-19

Status: `BASELINE_MISMATCH / WATER_AND_VISCOUS_CONTROLS_REPRODUCED / NR2_BLOCKED / REPORT_ONLY`

## Scope and identity

NR1 implemented the quarantined tool specified by the
[research contract](../plans/nonlocal-continuum/00-research-contract.md) and
[baseline plan](../plans/nonlocal-continuum/01-source-faithful-baseline-and-oracle.md).
It does not link PeriDyno or the Rust workspace, publish continuum state, change
SPEC-38/ADR-076, or receive W2/ProductCheck credit.

| Item | Exact identity |
|---|---|
| CPU oracle checkpoint | `5e77bbcfa5a7fa3ae967d08f1c0597fec3bf5384` |
| CUDA baseline checkpoint | `107839b8250964e6bae63a52fd01532ca089774b` |
| Standalone binary SHA-256 | `fdafb8ba9840a0ba21657179cf2c6086414fb5c37914535ba597d073f5ed425e` |
| Paper PDF SHA-256 | `610047ef32e895026c3661c57999f14ae550d8e2719ba2fcd95742371ad031c1` |
| Interpreted upstream | PeriDyno `1aa892bb296fe766d2f9249c881b8605af23a69b` |
| Host | Linux x86_64; NVIDIA GeForce RTX 3080 10 GB; compute capability `8.6`; driver `610.43.02` |
| Toolchain | CMake `4.2.3`; Ninja `1.13.2`; GCC `15.2.0`; CUDA compiler/runtime `13.3.73` / `13.3` |

The post-run available device query reported P8, `210/405 MHz` SM/memory,
`18/320 W` draw/limit and `38 C`; those are idle observations, not claimed
run-time clocks. No clocks or power limit were locked.

## Implementation boundary

The standalone executable contains:

- an independent direct-loop CPU `f64` oracle;
- a separately written CUDA `f32` implementation of density,
  incompressibility, bulk/shear viscosity, bidirectional surface tension and
  the unregularized local `3x3` SISSM update;
- GPU uniform-grid construction, CUB radix sort/scan and frozen CSR neighbors;
- distinct accumulation passes with symmetric reverse `atomicAdd`, a full
  per-particle matrix, fixed iteration count and explicit device-to-device
  position handoff;
- profile/input hashes, bounded self-test/check/benchmark commands and
  stage-separated CUDA-event timing.

CPU and CUDA share fixture records and report structures, not pair-contribution
functions. The profile-specific uniform grid is a Next Engine research choice;
the inspected upstream receives a framework neighbor list. Final energy
diagnostics and report copies occur after the measured solver interval.

Normalization was fixed in the executable as:

- total energy: `max(N*m*dx^2/dt^2, max_abs_cpu_energy)`;
- source: `max(dx, max_abs_cpu_source)`;
- matrix: `max(1, max_abs_cpu_matrix)`.

The absolute-plus-relative tolerances themselves remain exactly those frozen
in NR0.

## Oracle result

Both commands passed:

```text
nonlocal-feasibility --cpu-self-test
nonlocal-feasibility --self-test
```

The CPU report covers all eleven required named cases: isolated particle,
symmetric pair, collinear triplet, tetrahedron, uniform interior block, free
surface patch, viscosity shear pair and fixed-shell closed-box runs for one
through four iterations. The CUDA command independently repeats and compares
the same cases.

| Oracle field | Maximum observed CPU/CUDA discrepancy | Acceptance |
|---|---:|---|
| density absolute | `0.000610352 kg/m^3` | PASS (`<= 0.1` or relative bound) |
| normalized energy absolute | `2.11943e-6` | PASS |
| normalized source absolute | `4.65963e-7` | PASS |
| normalized matrix absolute | `4.66142e-7` | PASS |
| position absolute | `1.15484e-9 m` | PASS |
| velocity absolute | `9.31323e-7 m/s` | PASS |
| normalized pair-momentum residual | `4.10735e-8` | PASS (`<= 1e-5`) |

The CPU and CUDA report SHA-256 values are respectively
`5b007b5dc6aa8e9c18ba580779aae04694b9e54d92da7109a4f5d1ffeb597c91`
and `340550959015ec29808d75f82764708aada5a9a3628ee98d28db24c58a8e248a`.

## Fixed-work measurements

Each retained profile used five warm-ups and 50 measured runs. Values are CUDA
device-timeline milliseconds; process startup, report I/O and post-interval
diagnostic copies are excluded. Allocation is persistent and reported
separately. Neighbor construction is included in total.

| Profile | Correctness | Samples / directed pairs / max degree | median / p95 / p99 / mean ms | Device memory |
|---|---|---:|---:|---:|
| `nuv-water-16k.v0` | PASS | `16,000 / 1,699,688 / 123` | `5.923 / 5.944 / 5.972 / 5.709` | `10,914,054 B` |
| `nuv-water-48k.v0` | PASS | `48,000 / 5,190,588 / 123` | `13.835 / 13.875 / 13.901 / 13.828` | `32,725,766 B` |
| `nuv-viscous-16k.v0` | PASS | `16,000 / 1,699,688 / 123` | `20.019 / 20.039 / 20.046 / 20.019` | `10,914,054 B` |
| `nuv-surface-16k.v0` | **FAIL** | `16,000 / 1,699,688 / 123` | `13.969 / 14.005 / 14.140 / 13.975` | `10,914,054 B` |

Raw report SHA-256 values, in table order, are:

- `91dae4827cc6b1d2290c702afc7b4667103764fcff3dd2ac1ecf80aecc9a8d40`;
- `8cc9b16309f9f8edc821152da584d1bf2b910245de9d640da85ddef5cf88ab94`;
- `96aebba7861f9ff17fb302d0f88fc44a0787859c7be121273876e068f1b83b81`;
- `f95c55c48e83341779710a64b0bfe4419584d6008f7c0b0631df059ef368be0e`.

The water-48k p95 is `1.734x` the research cutoff before NR2. It is a baseline
denominator, not a final `HN-4` falsifier because the ordered fixed-work
optimization ladder did not begin. Water-16k fits `8 ms`; the viscous profile
does not. The surface time has no feasibility credit because correctness fails.

## Profiler attribution

One Nsight Systems `2026.1.3` capture of the repeated water-48k check is stored
outside Git:

| Artifact | SHA-256 |
|---|---|
| `/tmp/nuv-water-48k-nr1.nsys-rep` | `a7217943ec9c063d4f5c41a7bd9f03e884c3bffd4f74d1ca12b7cb1e540cfcf5` |
| exported CSV summary | `a4d410ec3cdad60fd7a0c5bc1c524932406dc8be6f188562bc9b91733684c214` |
| profiled check JSON | `fa574e3f7cbb9b4b5ae379f3f087a7977aedac6511e461cec4e9f4a51e0cd508` |

Across the two five-iteration controls, viscosity accumulation owns `60.2%`
of GPU kernel time, incompressibility `29.6%`, and density `5.4%`. Neighbor
count/fill together use `2.6%`; local updates use `0.2%`. This supports `HN-2`:
the baseline is dominated by repeated pair traversal/atomic accumulation, not
launch overhead or neighbor construction.

## Surface mismatch

The `gamma=1000`, 20-iteration surface profile is finite, stays inside capacity
and has normalized momentum residual below `1e-8`, but repeated identical runs
do not remain inside the frozen field tolerances. A current 20-iteration check
observed maximum repeated differences of:

- density `8.135 kg/m^3`;
- normalized source `0.01355` and matrix `0.02067`;
- position `6.346e-5 m`;
- velocity `0.06346 m/s`.

The checked CSR offsets and neighbor IDs are byte-identical between runs. An
iteration ladder localizes amplification to the second SISSM iteration: a
current two-iteration check reached `1.362e-4 m` and `0.1362 m/s` repeated
position/velocity differences while keeping the same neighbors. Its JSON
SHA-256 is `b3d40ef35166a24cabfe729973a799677753de2b11ccb0f8855432c0e5115de9`;
the 20-iteration check is
`dacfdf8a86ec1a7e36baa67d9a625d31c235a4ad00e297699aa8ebe33d144690`.

The evidence rejects a fixture hash, neighbor membership/order, capacity,
nonfinite local solve or one-iteration formula/sign mismatch. The surviving
explanation is inter-block `f32 atomicAdd` order perturbing the stiff surface
system and the nonlinear iterations amplifying that perturbation. This is an
inference from the controls, not a proof about every GPU or surface profile.

## Decision

NR1 records `BASELINE_MISMATCH`, not `BASELINE_REPRODUCED`. Tiny equations,
water and viscous fixed controls reproduce; the required stiff surface control
does not satisfy repeated-output correctness. Tolerances are not widened and
the failing timing is not promoted.

Under the current stage contract NR2 is blocked. The recommended reclosure is
to retain `source-atomic-v0` as the immutable failed denominator and specify a
separate deterministic gather/segmented pair-accumulation remediation identity.
That candidate must first pass the same tiny cases and the two-iteration
surface discriminator before any optimization timing. Implementing it now as
if NR1 had passed would erase the distinction between source reproduction and
the planned O2 accumulation change.

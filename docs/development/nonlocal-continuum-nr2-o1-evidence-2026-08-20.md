# Nonlocal continuum NR2-O1 pointer-swap evidence — 2026-08-20

Status: `O1_RETAINED_POINTER_SWAP / O2_NEXT / REPORT_ONLY`

## Scope and identity

This report executes the ordered gates in the
[NR2-O1 specification](../plans/nonlocal-continuum/04-nr2-o1-pointer-swap.md).
It compares the correctness-valid NR1-RC1 gather path with two explicit,
orthogonal handoff identities:

```text
baseline  = nuv-gather-directed-r0 + copy-v0
candidate = nuv-gather-directed-r0 + pointer-swap-o1
```

The candidate changes only the post-update handoff. `update` writes the other
already allocated position buffer and the host swaps the two private device
pointers. It removes the per-iteration device-to-device position copy without
changing equations, launches for physical terms, accumulation order, fixture,
iteration count, capacity or report ordering. Commands without `--handoff`
still select `copy-v0`.

| Item | Exact identity |
|---|---|
| O1 specification checkpoint | `30ec1675be6c86a8650b927cbd7a9b40b3affc90` |
| O1 implementation checkpoint | `b192f0a3d77f877d4e88895a7f0137418bc54ceb` |
| Legacy-gate/final implementation checkpoint | `d7ef06aca04767bc28d858efce19db6b50d05934` |
| Standalone binary SHA-256 | `a9a8b7ad47fbe1ed27e832e521a096d589b7b529e661889e0898e36f3ffe7d74` |
| Host | Linux x86-64; NVIDIA GeForce RTX 3080 10 GB; compute capability `8.6` |
| Device | 68 SMs; `10,351,214,592 B` global memory |
| Toolchain reported by binary | CUDA compiler/runtime `13.3.73` / `13.3`; CCCL/CUB `3.3.4`; driver API `13.3` |
| CUDA compile contract | target property `sm_86`; `-O3 --fmad=false --prec-div=true --prec-sqrt=true --ftz=false` |

Build was external to the repository. Generated binaries and raw reports were
not checked in.

## Ordered correctness gates

No timing command ran before all correctness gates passed.

The final audit found and corrected one harness-only scope regression: an
intermediate self-test applied the new exact reused-instance requirement to
the legacy source-atomic path. That path is intentionally tolerance-checked
because floating atomic association is not exact. Final checkpoint `d7ef06a`
restores one legacy execution while reporting and enforcing three reused
executions only for `pointer-swap-o1`; the candidate implementation itself is
unchanged. All gates and timings below were rerun with the final binary.

| Gate | Result |
|---|---|
| `O1-BUILD` | PASS; CMake/Ninja reported no pending work after the final rebuild |
| `O1-LEGACY` | default `source-atomic-v0 + copy-v0` tiny 11/11 PASS; explicit `nuv-gather-directed-r0 + copy-v0` tiny 11/11 PASS |
| `O1-TINY-RESET` | gather/swap tiny 11/11 PASS; three executions on each reused instance were exact; copy/swap output and CSR digests were exact |
| `O1-SURFACE-I2` | ten cold repeats exact; copy/swap exact; output `52a3d852c05b9cc7931a3133816e0ddb1445695b88ea25193d0981022080b65e`, CSR `a0304020eeb98889b47c0e179c85aaf4f93fb671bf58f15a99d3e034f1ad4698` |
| `O1-SURFACE-I20` | ten cold repeats exact; copy/swap exact; output `0f16c58f72fd3e31e2bd13bf12899359dbafdb90d06fdeef947371671ead7dbd`, CSR `a0304020eeb98889b47c0e179c85aaf4f93fb671bf58f15a99d3e034f1ad4698` |
| `O1-FULL` | water-16k/48k at five iterations and viscous-16k at twenty passed reused-instance, copy correspondence, topology, finite, local-solve, capacity and momentum controls |

The odd five-iteration water controls exercise pointer orientation changes
across repeated calls. Their exact reused-instance results, together with the
even twenty-iteration viscous and surface controls, reject the bounded stale
buffer/parity hypothesis on this implementation.

The surface profile SHA-256 was
`9ad78788bee7dc17dcc06877bd6d2fbe7ffbee77dfc76d001f8c6ceefea1c038`.
Its two- and twenty-iteration input SHA-256 values were respectively
`a97e0e5be388b3a124843fc386746e4e51ffe68c5dfd1273c8f3d2fb9bc0ed9a`
and `6a25eb25dcfcd1812f4d993d52667e0f87f023aad7d6faea928fdab92e0d5de5`.

### Full fixed-work controls

| Profile | Profile / input SHA-256 | Samples / pairs / max degree | Ordered output SHA-256 | Momentum residual | Memory | Result |
|---|---|---:|---|---:|---:|---|
| water-16k, i5 | `8445417063a80ac1626632de49d74f349f6690f7ed135bc20090106534f74dcb` / `f6b63269db1555ec4d4ad8be8041da088c7d9bd9380fa15e47504c4e0c4bb975` | `16,000 / 1,699,688 / 123` | `675efc7f729e0e0e496f8000b809858573a87315834a909c3035f2e2424d68eb` | `7.4952794e-8` | `10,914,054 B` | PASS |
| water-48k, i5 | `cb1868b86b4d9d40e996ebfa8e529982f73e647e358dd9f0a7c9269fce1211e8` / `1be1a98e586bae70a2ffa72eb56d4b79071e404a546616811eea1764c1e1ce51` | `48,000 / 5,190,588 / 123` | `03671a4fccf72958ff772dfde7870ad24e8a8678c669f2a1286a178676203d1b` | `5.3916989e-8` | `32,725,766 B` | PASS |
| viscous-16k, i20 | `c91b60682a8722085f6d244eeec3588812e0f9d55eb0cd29b27c14cf6432cc24` / `9b6297eb6e923d822588a1d900fb16b55e8bae2d0ac577ed9897bd77f7afd11c` | `16,000 / 1,699,688 / 123` | `6f8e97c8f4db11cd759628a078fefbc0177d34110b494c99f79fb03ca2d9fdb6` | `1.7105814e-7` | `10,914,054 B` | PASS |

Both executions on each reused candidate instance had the displayed digest.
The separately constructed `copy-v0` baseline had the same digest, CSR and
device-memory byte count. Density, energy, source, matrix, position and
velocity correspondence passed the unchanged tolerances.

## Adjacent timing

Each table is one adjacent `copy-v0` then `pointer-swap-o1` pair from the same
binary and device state. Each command used five warm-ups and 50 measured runs.
Values are CUDA device-timeline milliseconds in
`minimum / median / p95 / p99 / mean` order. Totals include neighbor
construction. Every timed command passed its tiny preflight and exact repeated
output check.

### Water 16k, five fixed iterations

| Stage | `copy-v0` | `pointer-swap-o1` |
|---|---:|---:|
| prediction | `0.006240 / 0.199104 / 0.416896 / 0.497504 / 0.169391` | `0.007872 / 0.267872 / 0.596448 / 0.700000 / 0.300862` |
| neighbor construction | `0.278528 / 0.279552 / 0.284672 / 0.333824 / 0.281994` | `0.243712 / 0.245760 / 0.253952 / 0.296960 / 0.247788` |
| density | `0.493568 / 0.847872 / 1.022976 / 1.252352 / 0.768400` | `0.474112 / 0.476160 / 0.479232 / 0.486400 / 0.476481` |
| buffer reset | `0.032768 / 0.034816 / 0.035840 / 0.036864 / 0.034806` | `0.028672 / 0.030912 / 0.032928 / 0.033792 / 0.031226` |
| incompressibility | `0.504960 / 0.533504 / 0.535552 / 0.536576 / 0.532892` | `0.480256 / 0.483328 / 0.487424 / 0.488448 / 0.483945` |
| viscosity | `0.577536 / 0.623712 / 0.626688 / 0.627712 / 0.623038` | `0.558080 / 0.561152 / 0.563200 / 0.563200 / 0.560767` |
| surface tension | `0 / 0 / 0 / 0 / 0` | `0 / 0 / 0 / 0 / 0` |
| local update | `0.038912 / 0.044032 / 0.045056 / 0.046080 / 0.043816` | `0.035840 / 0.039936 / 0.040960 / 0.041984 / 0.039557` |
| state handoff + velocity | `0.020608 / 0.024576 / 0.354304 / 0.539648 / 0.060735` | `0.010240 / 0.012288 / 0.014336 / 0.015360 / 0.012579` |
| total | `2.129760 / 2.602272 / 2.874016 / 2.962304 / 2.574449` | `1.920416 / 2.175296 / 2.502816 / 2.606368 / 2.210626` |

Handoff p95 decreased `95.954%`; total p95 decreased `12.916%`.

### Water 48k, five fixed iterations

| Stage | `copy-v0` | `pointer-swap-o1` |
|---|---:|---:|
| prediction | `0.164480 / 0.376128 / 0.627136 / 0.817120 / 0.393284` | `0.011136 / 0.262080 / 0.427680 / 0.495296 / 0.265318` |
| neighbor construction | `0.436224 / 0.490496 / 0.539648 / 0.551936 / 0.483333` | `0.440320 / 0.446464 / 0.485376 / 0.499712 / 0.453285` |
| density | `0.817152 / 0.832512 / 0.847872 / 0.888832 / 0.833213` | `0.818176 / 0.827392 / 0.834560 / 0.863232 / 0.828567` |
| buffer reset | `0.035840 / 0.039936 / 0.043008 / 0.062592 / 0.040129` | `0.033792 / 0.035840 / 0.037888 / 0.037920 / 0.035592` |
| incompressibility | `0.887808 / 0.915456 / 1.526784 / 1.810432 / 1.015555` | `0.882688 / 0.899072 / 0.920576 / 0.954368 / 0.899517` |
| viscosity | `1.283072 / 1.715200 / 2.359488 / 2.470016 / 1.751913` | `1.539264 / 1.653792 / 1.921120 / 1.976320 / 1.673036` |
| surface tension | `0 / 0 / 0 / 0 / 0` | `0 / 0 / 0 / 0 / 0` |
| local update | `0.051200 / 0.055296 / 0.058368 / 0.058368 / 0.055166` | `0.051200 / 0.053248 / 0.054432 / 0.055296 / 0.052865` |
| state handoff + velocity | `0.022656 / 0.026624 / 0.028832 / 0.029696 / 0.026458` | `0.011264 / 0.013376 / 0.015360 / 0.015552 / 0.013661` |
| total | `4.288832 / 4.613344 / 5.263680 / 5.576352 / 4.658137` | `4.003616 / 4.257440 / 4.626496 / 4.655424 / 4.278113` |

Handoff p95 decreased `46.726%`; total p95 decreased `12.105%`.

### Viscous 16k, twenty fixed iterations

| Stage | `copy-v0` | `pointer-swap-o1` |
|---|---:|---:|
| prediction | `0.012096 / 0.134624 / 0.380320 / 0.690240 / 0.176513` | `0.008768 / 0.130752 / 0.272256 / 0.310176 / 0.139112` |
| neighbor construction | `0.243712 / 0.245888 / 0.280576 / 0.289792 / 0.256577` | `0.245760 / 0.246784 / 0.279552 / 0.300032 / 0.250593` |
| density | `1.648928 / 1.656960 / 3.109888 / 3.475584 / 2.073604` | `1.652992 / 1.986560 / 2.253888 / 2.277568 / 1.973068` |
| buffer reset | `0.132096 / 0.139264 / 0.154816 / 0.559104 / 0.154852` | `0.122016 / 0.127232 / 0.131072 / 0.328096 / 0.131165` |
| incompressibility | `1.948672 / 2.187264 / 2.520256 / 2.598208 / 2.253750` | `1.958976 / 2.292768 / 2.614272 / 2.706432 / 2.335804` |
| viscosity | `2.256896 / 2.588864 / 3.022976 / 3.154944 / 2.682433` | `2.266112 / 2.596864 / 3.086464 / 3.256352 / 2.582175` |
| surface tension | `0 / 0 / 0 / 0 / 0` | `0 / 0 / 0 / 0 / 0` |
| local update | `0.150528 / 0.157696 / 0.453760 / 0.581632 / 0.205260` | `0.147584 / 0.152576 / 0.157696 / 0.456864 / 0.162687` |
| state handoff + velocity | `0.068608 / 0.072896 / 0.081920 / 0.647360 / 0.085948` | `0.032768 / 0.036864 / 0.039936 / 0.040960 / 0.036477` |
| total | `7.569888 / 7.902144 / 8.987584 / 9.343808 / 8.092054` | `7.427680 / 7.769920 / 8.313024 / 8.551904 / 7.818182` |

Handoff p95 decreased `51.250%`; total p95 decreased `7.505%`.

## Retention decision

| O1 retention condition | Observation | Result |
|---|---|---|
| all correctness gates pass | build, legacy, tiny/reset, surface i2/i20 and full controls pass | PASS |
| memory does not increase | exact byte equality at 16k and 48k | PASS |
| handoff p95 decreases on every profile | `95.954%`, `46.726%`, `51.250%` | PASS |
| total p95 improves on water-48k and viscous-16k | `12.105%`, `7.505%` | PASS |

O1 exits `O1_RETAINED_POINTER_SWAP`. The geometric-mean total-p95 speedup on
the two HN-3 denominator profiles is `1.1091x`. The candidate also remains
below the separate `8 ms` local feasibility cutoff on water-16k, but viscous
16k remains above it at `8.313024 ms`. Water-48k remains below its research
cutoff at `4.626496 ms`; neither observation is production or W2 credit.

Only the handoff-stage reduction is directly attributable to the removed copy.
Other stage percentile movement reflects adjacent-run variability and is
reported rather than credited. O1 percentages are not added to RC1 or future
non-adjacent results. In particular, the water-16k copy handoff p95 contains a
tail excursion relative to its `0.024576 ms` median; its `95.954%` reduction is
reported by the frozen rule but is not extrapolated as a typical copy cost.

`pointer-swap-o1` becomes the retained NR2 input to O2. `copy-v0`, the failed
source-atomic evidence and all RC1 identities stay explicitly reproducible.
No profiler capture, NR2 aggregate speedup, NR4 selection, runtime integration,
GPU authority, SPEC/ADR change or ProductCheck claim follows from O1.

## Commands

```text
cmake --build /tmp/nextengine-nonlocal-feasibility-build
nonlocal-feasibility --self-test --accumulation source-atomic-v0 --handoff copy-v0
nonlocal-feasibility --self-test --accumulation nuv-gather-directed-r0 --handoff copy-v0
nonlocal-feasibility --self-test --accumulation nuv-gather-directed-r0 --handoff pointer-swap-o1
nonlocal-feasibility --repeatability nuv-surface-16k.v0 --iterations 2 --runs 10 --accumulation nuv-gather-directed-r0 --handoff pointer-swap-o1
nonlocal-feasibility --repeatability nuv-surface-16k.v0 --iterations 20 --runs 10 --accumulation nuv-gather-directed-r0 --handoff pointer-swap-o1
nonlocal-feasibility --check nuv-water-16k.v0 --iterations 5 --accumulation nuv-gather-directed-r0 --handoff pointer-swap-o1
nonlocal-feasibility --check nuv-water-48k.v0 --iterations 5 --accumulation nuv-gather-directed-r0 --handoff pointer-swap-o1
nonlocal-feasibility --check nuv-viscous-16k.v0 --iterations 20 --accumulation nuv-gather-directed-r0 --handoff pointer-swap-o1
nonlocal-feasibility --benchmark <profile> --warmup 5 --runs 50 --accumulation nuv-gather-directed-r0 --handoff <copy-v0|pointer-swap-o1>
```

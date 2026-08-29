# Nonlocal continuum NR1-RC1 reclosure evidence — 2026-08-19

Status: `NR1_RECLOSED_GATHER_DIRECTED / NR2_UNBLOCKED / REPORT_ONLY`

## Scope and identity

This report executes the ordered gates in the
[NR1-RC1 specification](../plans/nonlocal-continuum/03-nr1-deterministic-accumulation-reclosure.md).
It tests one separately identified accumulation path,
`nuv-gather-directed-r0`, while preserving the failed `source-atomic-v0`
[baseline evidence](nonlocal-continuum-nr1-baseline-evidence-2026-08-19.md).
It neither relabels that baseline nor changes the Nonlocal equations, fixtures,
fixed iteration counts, tolerances, CSR construction, CUDA arithmetic or local
`3x3` SISSM solve.

| Item | Exact identity |
|---|---|
| RC1 specification checkpoint | `597605b20342efd774ac79b392e234a8eeeabf3e` |
| Independent CPU gather checkpoint | `23a7f7303ca1ad79ab589171083e680e6be0713c` |
| CUDA gather/report checkpoint | `c3ba0076f33d2bd45a703000c85bee4eb0950dc0` |
| Standalone binary SHA-256 | `adb663d755a0a6d5798d10895e2d2d25d0d716a491c7ed3318c733839901dc1a` |
| Host | Linux x86-64; NVIDIA GeForce RTX 3080 10 GB; compute capability `8.6` |
| Toolchain reported by binary | CUDA compiler/runtime `13.3.73` / `13.3`; CCCL/CUB `3.3.4`; driver API `13.3` |
| CUDA compile contract | `sm_86`; `-O3 --fmad=false --prec-div=true --prec-sqrt=true --ftz=false` |

The CUDA implementation has three new owner-only kernels. One thread owns
`source[i]` and `matrix[i]`, traverses the unchanged directed CSR order and
evaluates both `local(i,j)` and `reverse(j,i)`. The incompressibility incoming
term reads `density_ratio[j]`. Viscosity and surface tension keep the two named
endpoint additions rather than replacing them with a factor of two. The
candidate kernels contain no floating atomic or write to another particle.
The explicit source/matrix clears remain, and no device buffer was added.

## Ordered correctness gates

The gates were run in specification order. No timing campaign began before all
correctness and capacity controls passed.

### CPU algebra and CUDA tiny oracle

`RC1-CPU-ALGEBRA` compares the existing scatter implementation with an
independently transcribed CPU `f64` gather implementation over all eleven tiny
fixtures. It passed 11/11. Maximum discrepancies were:

| Field | Maximum observed discrepancy |
|---|---:|
| density absolute | `1.47793e-12` |
| normalized energy absolute | `1.00986e-15` |
| normalized source absolute | `6.64126e-16` |
| normalized matrix absolute | `2.86904e-16` |
| position absolute | `5.20417e-18 m` |
| velocity absolute | `5.20417e-15 m/s` |
| gather normalized momentum residual | `1.49387e-16` |

Large relative values on individual near-zero components did not weaken a
gate: the frozen absolute-or-relative rule passed in every case.

`RC1-CUDA-TINY` then compared the independently written CUDA `f32` gather path
with the CPU oracle and passed 11/11. Its maximum normalized/absolute
discrepancies were density `6.75123e-4`, energy `1.42513e-6`, source
`6.03188e-7`, matrix `7.07499e-7`, position `1.63913e-9 m` and velocity
`1.86265e-6 m/s`; maximum normalized momentum residual was `3.77590e-8`.
The frozen tiny profile hash was
`bb2d032169e4cddefc26f076b4f99a05c3545efe86ad430316458719a7ff1125`.
The legacy default `source-atomic-v0` tiny command also remained 11/11 PASS.

### Stiff-surface repeatability

Each repeat constructed a new `CudaBaseline`, reallocated its storage and
reset it from the immutable fixture. The exact digest covers ordered final
positions and velocities plus last density, source and matrix. Timing and
completion metadata are excluded.

| Gate | Runs | Profile / input SHA-256 | CSR SHA-256 | Unique output SHA-256 | Result |
|---|---:|---|---|---|---|
| `RC1-SURFACE-I2` | 10 | `9ad78788bee7dc17dcc06877bd6d2fbe7ffbee77dfc76d001f8c6ceefea1c038` / `a97e0e5be388b3a124843fc386746e4e51ffe68c5dfd1273c8f3d2fb9bc0ed9a` | `a0304020eeb98889b47c0e179c85aaf4f93fb671bf58f15a99d3e034f1ad4698` | `52a3d852c05b9cc7931a3133816e0ddb1445695b88ea25193d0981022080b65e` | PASS |
| `RC1-SURFACE-I20` | 10 | `9ad78788bee7dc17dcc06877bd6d2fbe7ffbee77dfc76d001f8c6ceefea1c038` / `6a25eb25dcfcd1812f4d993d52667e0f87f023aad7d6faea928fdab92e0d5de5` | `a0304020eeb98889b47c0e179c85aaf4f93fb671bf58f15a99d3e034f1ad4698` | `0f16c58f72fd3e31e2bd13bf12899359dbafdb90d06fdeef947371671ead7dbd` | PASS |

All ten output digests and all ten CSR digests were identical in each gate.
The normalized momentum residual was likewise exact across repeats:
`5.63853e-9` at two iterations and `1.19405e-9` at twenty. Every state was
finite, every local solve succeeded, topology/capacity passed and repeated
field correspondence was within the unchanged tolerances.

This supports the bounded hypothesis that run-dependent reverse-endpoint
`f32 atomicAdd` association caused the NR1 stiff-surface mismatch and that the
owner-only CSR order removes it on this frozen RTX 3080 environment. It is not
a cross-GPU determinism claim or proof about every surface configuration.

### Full fixed-work controls

| Profile | Profile / input SHA-256 | Samples / directed pairs / max degree | Exact repeated fields | Momentum residual | Result |
|---|---|---:|---|---:|---|
| `nuv-water-16k.v0`, 5 iterations | `8445417063a80ac1626632de49d74f349f6690f7ed135bc20090106534f74dcb` / `f6b63269db1555ec4d4ad8be8041da088c7d9bd9380fa15e47504c4e0c4bb975` | `16,000 / 1,699,688 / 123` | yes | `7.49528e-8` | PASS |
| `nuv-water-48k.v0`, 5 iterations | `cb1868b86b4d9d40e996ebfa8e529982f73e647e358dd9f0a7c9269fce1211e8` / `1be1a98e586bae70a2ffa72eb56d4b79071e404a546616811eea1764c1e1ce51` | `48,000 / 5,190,588 / 123` | yes | `5.39170e-8` | PASS |
| `nuv-viscous-16k.v0`, 20 iterations | `c91b60682a8722085f6d244eeec3588812e0f9d55eb0cd29b27c14cf6432cc24` / `9b6297eb6e923d822588a1d900fb16b55e8bae2d0ac577ed9897bd77f7afd11c` | `16,000 / 1,699,688 / 123` | yes | `1.71058e-7` | PASS |

All density, energy, source, matrix, position and velocity discrepancies
between the two check executions were exactly zero. Memory stayed at
`10,914,054 B` for 16k and `32,725,766 B` for 48k, exactly matching the atomic
binary's capacity report.

## Adjacent cost observation

After correctness passed, each valid atomic denominator was run immediately
before gather in the same binary with five warm-ups and 50 measurements.
Totals include neighbor construction and use the CUDA device timeline.

| Profile | Accumulation | min / median / p95 / p99 / mean ms | p95 speedup | Device memory |
|---|---|---:|---:|---:|
| water 16k, i5 | `source-atomic-v0` | `5.314 / 5.982 / 6.795 / 7.010 / 5.930` | — | `10,914,054 B` |
| water 16k, i5 | `nuv-gather-directed-r0` | `2.133 / 2.173 / 2.632 / 2.711 / 2.302` | `2.581x` | `10,914,054 B` |
| water 48k, i5 | `source-atomic-v0` | `13.856 / 15.053 / 16.960 / 17.188 / 15.265` | — | `32,725,766 B` |
| water 48k, i5 | `nuv-gather-directed-r0` | `3.550 / 3.862 / 4.630 / 4.820 / 3.950` | `3.663x` | `32,725,766 B` |
| viscous 16k, i20 | `source-atomic-v0` | `20.601 / 21.497 / 23.046 / 23.309 / 21.572` | — | `10,914,054 B` |
| viscous 16k, i20 | `nuv-gather-directed-r0` | `6.691 / 7.376 / 8.693 / 8.812 / 7.571` | `2.651x` | `10,914,054 B` |

The p95 geometric-mean speedup on the two declared HN-3 denominator profiles,
water 48k and viscous 16k, is `3.116x`. This is an RC1 adjacent observation,
not retained NR2 credit: the ordered O1–O6 ladder and its required profiler
captures have not run. The surface atomic path remains correctness-invalid and
was not used as a denominator.

## Commands

```text
nonlocal-feasibility --cpu-self-test
nonlocal-feasibility --cpu-gather-self-test
nonlocal-feasibility --self-test --accumulation source-atomic-v0
nonlocal-feasibility --self-test --accumulation nuv-gather-directed-r0
nonlocal-feasibility --repeatability nuv-surface-16k.v0 --iterations 2 --runs 10 --accumulation nuv-gather-directed-r0
nonlocal-feasibility --repeatability nuv-surface-16k.v0 --iterations 20 --runs 10 --accumulation nuv-gather-directed-r0
nonlocal-feasibility --check nuv-water-16k.v0 --iterations 5 --accumulation nuv-gather-directed-r0
nonlocal-feasibility --check nuv-water-48k.v0 --iterations 5 --accumulation nuv-gather-directed-r0
nonlocal-feasibility --check nuv-viscous-16k.v0 --iterations 20 --accumulation nuv-gather-directed-r0
nonlocal-feasibility --benchmark <passing-profile> --warmup 5 --runs 50 --accumulation <source-atomic-v0|nuv-gather-directed-r0>
```

Every CUDA JSON report binds the accumulation identity, binary hash, exact
command, profile/input hash and CUDA/CCCL/driver/device identity.

## Decision

NR1-RC1 exits `NR1_RECLOSED_GATHER_DIRECTED`. `nuv-gather-directed-r0` is the
correctness-valid NR1 successor, while `source-atomic-v0` and its original
failure hashes remain immutable evidence. NR2 is unblocked and must start at
O1 using gather as its correctness baseline. RC1 does not authorize production
integration, GPU authority, SPEC/ADR changes, W2 credit, a lower-precision
profile or an NR4 product interpretation.

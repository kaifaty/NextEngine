# Nonlocal continuum NR2-O4 cell-locality evidence — 2026-08-20

Status: `O4_RETAINED_STABLE_SAMPLE / NR2_FIXED_WORK_COMPLETE / NR4_NEXT / REPORT_ONLY`

## Scope and identity

This report executes the ordered gates in the
[NR2-O4 specification](../plans/nonlocal-continuum/07-nr2-o4-cell-sorted-locality.md).
It compares only the retained O2 stack with the separately selectable storage
candidate:

```text
retained  = nuv-gather-directed-r0
            + pointer-swap-o1
            + nuv-terms-specialized-o2
            + stable-sample-v0

candidate = nuv-gather-directed-r0
            + pointer-swap-o1
            + nuv-terms-specialized-o2
            + cell-sorted-o4
```

The candidate stores solver state in stable packed-cell order, builds CSR by
physical storage owner, maps neighbor entries to physical storage IDs, then
reconstructs all captured fields and logical CSR in stable sample-ID order.
For each logical owner, fixed 27-cell traversal and in-cell sample order remain
unchanged; unlike O3, O4 introduces no different floating association.

| Item | Exact identity |
|---|---|
| O4 specification checkpoint | `e3486c8` |
| O4 implementation checkpoint | `bba32c6` |
| Final aggregate-comparator checkpoints | `879b8b3`, `37e68f1` |
| Final standalone binary SHA-256 | `1cc2dc078c4ee5a38fba3ba73d66f722c2cace4be1cdc97c540cd6cf65b0c53c` |
| Final standalone binary size | `2,082,624 B` |
| Host | Linux x86-64; NVIDIA GeForce RTX 3080 10 GB; compute capability `8.6` |
| Toolchain | CUDA compiler/runtime `13.3.73` / `13.3`; CCCL/CUB `3.3.4`; driver API `13.3` |
| CUDA compile contract | target `sm_86`; `-O3 --fmad=false --prec-div=true --prec-sqrt=true --ftz=false` |

Build trees, binaries, Nsight reports and raw JSON remain outside Git.

## Ordered correctness result

| Gate | Result |
|---|---|
| `O4-BUILD` | PASS; clean external CMake/Ninja rebuild |
| `O4-LEGACY` | PASS; default atomic/copy/runtime and retained gather/swap/specialized each 11/11; O3 diagnostic remains selectable; invalid atomic or segmented + O4 combinations reject |
| `O4-TINY` | PASS 11/11 against CPU; three reused candidate executions exact; retained output/logical CSR bit-exact; maps and physical CSR valid |
| `O4-SURFACE-I2` | PASS; ten cold outputs and CSR exact; retained surface digest preserved |
| `O4-SURFACE-I20` | PASS; ten cold outputs and CSR exact; retained surface digest preserved |
| `O4-FULL` | PASS; water-16k/48k i5 and viscous-16k i20 finite/topology/capacity/momentum; two reused outputs and retained correspondence bit-exact |
| `O4-STORAGE` | PASS; packed-key order, stable equal-key sample IDs, bounded inverse maps, measured-map equality, maximum degree and memory exact |

The two stiff-surface identities remain exactly those of RC1/O2:

| Iterations / runs | Unique candidate output SHA-256 | Logical CSR SHA-256 |
|---:|---|---|
| 2 / 10 cold | `52a3d852c05b9cc7931a3133816e0ddb1445695b88ea25193d0981022080b65e` | `a0304020eeb98889b47c0e179c85aaf4f93fb671bf58f15a99d3e034f1ad4698` |
| 20 / 10 cold | `0f16c58f72fd3e31e2bd13bf12899359dbafdb90d06fdeef947371671ead7dbd` | `a0304020eeb98889b47c0e179c85aaf4f93fb671bf58f15a99d3e034f1ad4698` |

The full-control candidate output digests are
`675efc7f729e0e0e496f8000b809858573a87315834a909c3035f2e2424d68eb`
for water-16k,
`03671a4fccf72958ff772dfde7870ad24e8a8678c669f2a1286a178676203d1b`
for water-48k and
`6f8e97c8f4db11cd759628a078fefbc0177d34110b494c99f79fb03ca2d9fdb6`
for viscous-16k. Each pair of reused executions is identical and exactly
matches its retained stable-sample comparator after logical remapping.

## Storage and capacity proof

The implementation allocates only the specified two maps, sorted immutable
reference/velocity/fixed fields and one error flag:

| Scale | Required O4 bytes | Observed O4 bytes | Candidate total | Ceiling |
|---|---:|---:|---:|---:|
| 16k | `528,004` | `528,004` | `11,442,058` | `11,442,058` |
| 48k | `1,584,004` | `1,584,004` | `34,309,770` | `34,309,770` |

The 48k retained/candidate pair consumes `67,035,536 B`, below the
`70,000,000 B` co-resident ceiling. Directed counts remain `1,699,688` at
16k and `5,190,588` at 48k, with maximum degree `123`.

## Same-process O4 tournament

Each row is one final-binary process with both instances resident, 32 warm-up
rounds, 96 measured rounds and alternating order. All correctness and capacity
pre/post gates pass, so timings are admissible.

| Profile | Stable total p95 | Cell-sorted total p95 | O4 speedup | Stable / sorted pair-stage p95 sum | Profile rule |
|---|---:|---:|---:|---:|---|
| water-16k i5 | `2.168832 ms` | `2.456416 ms` | `0.882925x` | `1.659232 / 1.788928 ms` | FAIL |
| water-48k i5 | `3.938816 ms` | `4.216704 ms` | `0.934098x` | `3.147136 / 3.641472 ms` | FAIL |
| viscous-16k i20 | `7.068416 ms` | `7.495552 ms` | `0.943015x` | `6.309248 / 6.966912 ms` | FAIL |

The HN-3 denominator-profile geometric-mean O4 speedup is `0.938546x`, or a
`6.54779%` candidate regression. It misses the required `1.05x` floor, both
denominator total-improvement rules, the water-16k `<=2%` regression rule and
every pair-stage non-regression rule.

## Diagnosis

The frozen performance fixtures are generated in `z/y/x` loops with `x` as
the fastest stable sample dimension. That initial layout is already strongly
spatially coherent for the regular lattice. Re-sorting by packed horizon cells
does not create the usual gain expected from unordered particles:

| Scale | Stable mean / p95 storage distance | Cell-sorted mean / p95 |
|---|---:|---:|
| 16k | `910.484 / 1,679` | `941.264 / 2,647` |
| 48k | `3,575.627 / 6,559` | `3,387.929 / 10,275` |

At 16k both metrics worsen. At 48k the mean improves slightly, but the tail
worsens by about `56.7%`. Exact arithmetic also requires retaining each
logical owner's original neighbor sequence, so O4 cannot reorder the inner
reduction for a more favorable physical walk. The extra map validation and
indirection then add cost without enough cache benefit. This falsifies O4 for
the current frozen workloads; it does not claim cell sorting is generally
useless for dynamically disordered particles.

## Final retained NR2 comparison

O4 is not retained. A separate two-instance tournament compares the final
retained fixed-work stack against the correctness-valid HN-3 source-atomic
denominator on exactly the two admitted profiles, plus water-16k as a local
control. O3 is not instantiated or timed.

| Profile | Source atomic p95 | Final retained p95 | Accumulated speedup |
|---|---:|---:|---:|
| water-16k i5 | `5.724288 ms` | `2.075040 ms` | `2.758640x` |
| water-48k i5 | `14.580704 ms` | `4.019520 ms` | `3.627474x` |
| viscous-16k i20 | `20.971104 ms` | `7.084416 ms` | `2.960174x` |

The two HN-3 denominators give `3.27688x` geometric-mean speedup. Therefore
`NR2-SPEEDUP` passes, water-48k passes the research-only `8 ms` cutoff and the
water/viscous 16k controls pass the local cutoff. The viscous atomic/gather
cross-trajectory diagnostic can exceed their mutual field tolerance because
atomic endpoint association differs; this is not a denominator gate. Both
identities pass their frozen independent controls, and the retained output is
exact.

## Final profiler attribution

Nsight Systems `2026.1.3.425-261338342291v0` captured the final retained binary
with five warm-ups and 50 measured executions. Water-48k GPU kernel time is
`35.9%` specialized viscosity, `24.6%` incompressibility gather and `23.5%`
density; the two neighbor visitors add `11.8%`. Viscous-16k is `36.1%`,
`31.4%` and `26.8%` for the same three pair kernels, with `3.1%` in neighbor
visitors. Thus already-tested neighbor/pair work owns about `84.0%` and
`94.3%`; no untested O5 clear/handoff/launch stage owns `20%`.

| Raw record | SHA-256 |
|---|---|
| final water-48k `.nsys-rep` | `a3bea3ac32a08a959d1e68e8f634570b6f6a715a99dc96ff4a70663c65a773ab` |
| final water-48k CUDA kernel summary | `31ac237adbb84ab3314abed12f4688f02a841d13ae3f0576e35747fbdd683dac` |
| final viscous-16k `.nsys-rep` | `b1087d13aaaaaa36a4d932326fc3d9e0c92b38ba61bbc299c827547127069324` |
| final viscous-16k CUDA kernel summary | `2db4e4fbc874e311398cc5f7609ac282b21bbbf3db99cc6f46783283950acab7` |

Nsight Compute counters remain unavailable under the already recorded
`ERR_NVGPUCTRPERM` boundary; no occupancy or throughput value is inferred.
O5 is not admitted by its profiling precondition. O6 remains optional and is
not funded without a separately bounded packed-field hypothesis. NR2 fixed
work is complete and proceeds to NR4; NR3 algorithm-changing work is not
required to establish the fixed-work decision.

## Decision and rollback

O4 exits `O4_RETAINED_STABLE_SAMPLE`. `cell-sorted-o4` remains selectable to
reproduce exact maps, correctness and negative timing. The retained identity
for NR4 is:

```text
nuv-gather-directed-r0
+ pointer-swap-o1
+ nuv-terms-specialized-o2
+ stable-sample-v0
```

This is report-only evidence. It creates no runtime integration, GPU
authority, W2 credit, public contract or production claim.

## Final raw JSON hashes

| Report | SHA-256 |
|---|---|
| default tiny | `3ab0971ebec86c0d93e779361b01798578daca6a5692b9fc518588b4d9bd3179` |
| retained tiny | `fab68fb3138acc7159919506d87c77cacc214e98424047964ac484df608ebf4a` |
| O4 tiny | `5ac55cd2f81b0b5e3cdb1b5a59d63bfee3c659f8af8f5c1a6e7715a662dba0e3` |
| O4 surface i2 / i20 | `0aec296cdca21cf39bf74810c872b156faa5a5149f71a0c0542846e1d6ddcbff` / `de271aa5e037a73a9d0f8afb831d4e2cc2ebbf77eead2d107d6fb5c084a462f7` |
| O4 water-16k / water-48k / viscous-16k checks | `ab11ff40072751547740fa760d309d9cf2d051f7990a8b1559c6d541553e2a67` / `a54a5234e453fd1c60f431272c79e0aeaf8126629288d2749a2b5382567b54ee` / `b3affd9963b03a6f60cd18185b08a95322922b6873884aff04cbb11eabdf1cdd` |
| O4 water-16k / water-48k / viscous-16k tournaments | `cf395146113b9ee4a1f0f406e103ce9131c1fe8cf0693a688129bb5a2a5d6b85` / `8698ccb0b81f5b5c4876be8a39bbd9fa30df8f5dac494517ea349ee9ff068d8a` / `59bb345753f7ad3ad034cf09c9e5db3f0bd46c916ac0597f32200acc94f97354` |
| final NR2 water-16k / water-48k / viscous-16k tournaments | `41d0d3134febdbd4d8c355155c9e2dba64aea6ada5268ba88210b26517b1124b` / `45e7a6f1e4224b304c41349479303b064cb5a1ac9bba7b721a6841ec7363cafe` / `36dae10163a62772506d8ae4ba73b3688b89fd895a34ff9da465e6e5331117c6` |

## Commands

```text
cmake --build /tmp/nextengine-nonlocal-o4-build --clean-first --parallel
nonlocal-feasibility --self-test --accumulation nuv-gather-directed-r0 --handoff pointer-swap-o1 --term-kernels nuv-terms-specialized-o2 --storage cell-sorted-o4
nonlocal-feasibility --repeatability nuv-surface-16k.v0 --iterations <2|20> --runs 10 --accumulation nuv-gather-directed-r0 --handoff pointer-swap-o1 --term-kernels nuv-terms-specialized-o2 --storage cell-sorted-o4
nonlocal-feasibility --check <profile> --iterations <5|20> --accumulation nuv-gather-directed-r0 --handoff pointer-swap-o1 --term-kernels nuv-terms-specialized-o2 --storage cell-sorted-o4
nonlocal-feasibility --locality-tournament <profile> --warmup 32 --runs 96
nonlocal-feasibility --retained-tournament <profile> --warmup 32 --runs 96
```

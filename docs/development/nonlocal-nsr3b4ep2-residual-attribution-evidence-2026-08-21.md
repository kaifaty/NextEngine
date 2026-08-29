# NSR3-B4EP2 residual cost-attribution evidence -- 2026-08-21

Status: `PASS / WORKSPACE_PIPELINE_DOMINANT / B4EP3_TOPOLOGY_AUDIT_DESIGN_AUTHORIZED`

## Result

The exact-output work-only profile records 1,728 ten-millisecond CPU samples.
Complete workspace construction owns 58.33% inclusive sampled time. Its
topology/CSR component is 40.97%, while evaluation plus tape refresh is
16.84%. Exact HVP application is the other large path at 39.64% inclusive.
Hashing is now only 0.46%.

Topology and HVP are a near tie and the profiler is statistical. The next
stage therefore does not claim that topology is the final bottleneck. It
selects a bounded topology-reuse feasibility audit because the aggregate
workspace pipeline is clearly largest and reuse can be rejected cheaply before
changing the hot path. No solver-policy or physics change is selected.

## Build and correspondence

| Field | Value |
|---|---|
| B4EP2 identity | `3268d59c30c11f892e45c5d781989fb085d54b7d5924069285fa4b56e8d0196d` |
| source implementation commit | `5e40aa26e6ba0fcf2d0f3c4ca8d6f77857a660f1` |
| compiler | GCC 15.2.0 |
| compile flags | `-O3 -DNDEBUG -g -pg` |
| linker flags | `-pg` |
| gprof | GNU 2.46 |
| compile commands | 14,601 bytes; `bbe9d7c2f845d3e2da51e5e2cf59246468dbc2b5bf21496d8570db8900ab8c06` |
| instrumented executable | 36,274,032 bytes; `acf7c6cfb7518883ef72e7e0cc7a15e6736039e8581158cdf730a26bcf6271fd` |
| GNU Build ID | `d1eb18fd56b3500f6c39d62161f31bde1156b034` |

The instrumented stdout is exactly 5,780 bytes with SHA-256
`4d63f5f05811357b958b18380ec483cd97073ae02c3a0228098e255d73da8112`
and semantic result
`25b1f00c0c7477a03532dca2acb97314853e9b4184962d9695792273280f5e04`.
It matches the B4EP1 candidate byte-for-byte.

Instrumented wall/RSS are 24.21 s, 62,660 KiB and 99% CPU. They prove normal
completion only and are not Release throughput evidence.

## Profile artifacts

| Artifact | Size | SHA-256 |
|---|---:|---|
| `gmon.out` | 1,370,641 bytes | `953b61096c488555c2de55e0db5845ed5fea86959f07622a063df7bdc24700de` |
| flat profile | 47,039 bytes | `85e9e24c3a3b014c76a57354d3d1456f890fcc3cf1b3ddec4a04e164d28a2ea1` |
| call graph | 310,949 bytes | `13d381726ca5eb5de03ecb15b1ef8b786dbc7b0f9152fb9e473ba2996d09f63e` |

B4EP2 result attestation root:
`314bc306099a28f46d1856704145eddb90f9410fe94050cadc9dffcbf3557693`.

## Attribution

| Category | Sampled CPU | Share |
|---|---:|---:|
| topology/CSR construction inclusive | 7.08 s | 40.97% |
| exact HVP application inclusive | 6.85 s | 39.64% |
| evaluation inclusive | 2.44 s | 14.12% |
| pressure-tape refresh | 0.47 s | 2.72% |
| complete workspace construction inclusive | 10.08 s | 58.33% |
| SHA-256 self | 0.08 s | 0.46% |

The HVP total consists of 4.63 s direct work plus 2.22 s of the shared
`kernel_scale` leaf over 459 calls. Evaluation consists of 1.94 s direct work
plus 0.50 s of `kernel_scale` over 227 calls. This call-graph split accounts
for all 1,189,890,693 recorded `kernel_scale` calls.

As in B4EP0, gprof labels the optimized static-support topology builder as
`run_joint_pressure_tape_controls()`. The mapping is established by its sole
227-call arc from `build_joint_query_workspace`, its cell/adjacency children
and the exact workspace count; the displayed name is not taken literally.
Nonlinear-control bookkeeping outside workspace/HVP accounts for only the
small unclassified remainder and is not selected for solver-policy research.

## Prior Verlet negative and current distinction

The earlier NP1-P4 GPU/f32 candidate remains a correctness stop: cached
anchor-cell order differed after a cell crossing even with exact membership.
That evidence cannot be relabelled or ignored.

The current research root is structurally different. Its CPU/f64 neighborhood
builder explicitly sorts final `JointPair` values lexicographically after cell
discovery. A lexicographically sorted superset filtered by the exact current
horizon should therefore preserve current pair order independently of cell
crossings. This is a hypothesis, not transferred credit. B4EP3 must compare
every filtered pair list and flat CSR against the canonical full builder and
stop on the first mismatch before any timing candidate exists.

## Decision

Select `B4EP3_CANONICAL_SUPERSET_FEASIBILITY_AUDIT`. Capture the exact 227
nominal query states once, use a frozen conservative skin/certificate, and
measure coverage, exact order, capacity, reuse and candidate-pair overhead.
If the audit cannot prove exact current pairs/CSR or useful bounded reuse,
route directly to HVP design. B4E2, runtime, CUDA and production remain
blocked.

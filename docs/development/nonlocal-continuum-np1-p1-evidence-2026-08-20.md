# Nonlocal continuum NP1-P1 evidence — 2026-08-20

Status: `P1_RETAINED / P2_NEXT / EXACT_WORK / REPORT_ONLY / NO_W2_CREDIT`

## Decision

Retain `fused-owner-terms-p1` on top of the NP0 stack. It passes exact output,
CSR, capacity and momentum correspondence on the independent tiny corpus,
stiff `gamma=1000` i2 control, coherent/permuted 50k fixtures and every step of
all three 32-step advected traces.

Every adjacent profile passes both the targeted performance gate and the
no-regression gate:

| Profile | Total p95 retained -> P1 | Speedup | Pair p95 retained -> P1 | Speedup |
|---|---:|---:|---:|---:|
| coherent water 50k | `6.134048 -> 4.724736 ms` | `1.2983x` | `3.763200 -> 2.641024 ms` | `1.4249x` |
| permuted water 50k | `11.840800 -> 9.734752 ms` | `1.2163x` | `6.542432 -> 3.168320 ms` | `2.0650x` |
| advected water 50k | `5.749504 -> 4.588064 ms` | `1.2531x` | `3.443904 -> 2.407424 ms` | `1.4305x` |
| viscous 16k | `8.313536 -> 6.957184 ms` | `1.1950x` | `5.474368 -> 3.745120 ms` | `1.4617x` |
| surface 16k | `11.732064 -> 9.368256 ms` | `1.2523x` | `7.360640 -> 5.200288 ms` | `1.4154x` |

P1 alone exceeds HP-2's combined P1+P2 threshold on every profile: total p95
falls `16.3–23.0%` and pair-stage p95 falls `29.4–51.6%`. P2 remains useful
because the exact-50k P1 adjacent totals still exceed 4 ms and the permuted
profile remains much slower than coherent.

## Exact-work identity

The retained identity becomes:

```text
nuv-gather-directed-r0
+ pointer-swap-o1
+ nuv-terms-specialized-o2
+ stable-sample-v0
+ fused-owner-terms-p1
```

Density remains its own kernel/global barrier. P1 visits each post-density CSR
row once, keeps incompressibility, viscosity and surface source/matrix sums in
separate f32 accumulators, then commits them in the retained term order. It
adds no device allocation and changes no iteration, neighbor, storage,
coefficient, tolerance or accepted-state semantics.

The evidence binary SHA-256 is
`2c1d810bf306fcc26a3cd40761d7646b901befcd318dfd9e1afa426455aa54b9`.

## Correctness

- retained CUDA self-test: PASS;
- independent CPU f64 gather oracle for every tiny fixture: PASS;
- stiff surface `gamma=1000`, i2: exact retained output and CSR;
- fixed 50k coherent/permuted: exact retained output and CSR;
- water/viscous/surface advected traces: exact output, handoff state and CSR at
  all 32 steps; first mismatch `-1`;
- trace SHA-256 values remain the frozen NP0 roots:
  - water: `333e487207ffab664e1519705953f8530af1f5bb180c7f30d72d3193607aae46`;
  - viscous: `9d3f953923921e2b455c040023ec0e843b9a369ed2036976281c866681df03e9`;
  - surface: `bf3d4fb3c7484dcb574eea8a1165caa978b8361c3d8c2800eacebb257def3513`.

The full candidate output hashes equal the retained hashes recorded by each
tournament. No tolerance fallback or algorithm reclassification was used.

## Compile diagnostics

An independent `sm_86` build with `-Xptxas=-v` reports no stack frame, spill
store or spill load for any P1 specialization. The three used masks consume:

| Mask | Registers | Spill stores/loads |
|---|---:|---:|
| incompressibility + bulk | 59 | `0 / 0` |
| incompressibility + bulk/shear | 62 | `0 / 0` |
| incompressibility + surface | 48 | `0 / 0` |

The compile-log SHA-256 is
`59563d083d2d2d843774e8befa3b9ab632a61faef7267e79812a520a9d24f5a0`.
Thus the win is not hiding local-memory spill traffic.

## Measurement protocol

Each report co-resides the retained and candidate instances, performs 256
conditioning rounds, 32 formal warm-up rounds and 96 measured rounds, and
alternates execution order. Dynamic identities advance in lockstep and reset
only after complete 32-step epochs. Candidate device bytes exactly equal the
retained instance.

Raw JSON remains outside Git:

| Artifact | SHA-256 |
|---|---|
| coherent | `2d20646439ddf4c988091eeb6db6cea5254890cc382cf07d46b06007ea34987e` |
| permuted | `b909bb8d48c101b6db1d15f988af0915a8f121a205ec3a14d413d4fe185381c3` |
| advected | `ba8de9f87f5b28f004cb85127c3d0fb0d7176b4d5d4a89957f3eaa8e6571c430` |
| viscous | `fc05287213cb25e837abc07f778b8b1b021173a92c68a0c5e0877bb0e4ed1fb8` |
| surface | `62758ab6d04e25f2ad463987395db599b28561b8cb462355a96b192b02245b9c` |

## Consequence

P2 compact CSR is authorized and must use P1 as its adjacent denominator. P1
remains selectable separately as rollback. P2 may change neighbor ID encoding
and memory access only; it may not change the exact logical CSR or P1's term
association. P3/P4 and NP2 remain queued in the frozen order. No runtime,
production, W2 or architecture authority changed.

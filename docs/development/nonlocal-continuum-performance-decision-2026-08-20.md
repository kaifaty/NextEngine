# Nonlocal continuum performance reclosure decision — 2026-08-20

Status: `NONLOCAL_50K_FIXED_WORK_RECLOSURE_CANDIDATE / ROADMAP_COMPLETE / REPORT_ONLY / NO_W2_CREDIT`

## Decision

Select the frozen NP4 terminal state:

```text
NONLOCAL_50K_FIXED_WORK_RECLOSURE_CANDIDATE
```

The retained fixed-work identity passes the standalone Linux RTX 3080 exact
50k `<=4 ms` p95 and `<=6 ms` p99 gates on both coherent and sequentially
advected water. Two fresh processes per profile pass exact trace, capacity and
timing gates. This authorizes drafting a later Proposed fixed-work solver
reclosure and a broader independent corpus.

It does **not** close W2, replace DFSPH, grant GPU/runtime authority, amend
SPEC-38/ADR-076/081, prove Windows behavior or pass the integrated
`world-dynamics-step` budget.

## Retained identity

```text
nuv-gather-directed-r0
+ pointer-swap-o1
+ nuv-terms-specialized-o2
+ stable-sample-v0
+ fused-owner-terms-p1
+ compact-csr-u16-p2
```

- P1 fuses compatible owner-term CSR traversals while preserving independent
  accumulators and retained commit order.
- P2 directly constructs checked 16-bit neighbor IDs for eligible fixtures;
  offsets remain checked 32-bit and 100k falls back exactly to 32-bit IDs.
- P3 dynamic cell locality and P4 Verlet reuse remain diagnostic, not retained.

## Final artifact and protocol

- branch: `codex/nonlocal-continuum-n0`
- decision executable SHA-256:
  `a1c5c4cfe1e2fed741cc37c7189de8c58e843115168f2f721e322638dd50dc67`
- platform: Linux x86-64, NVIDIA GeForce RTX 3080 10 GB, `sm_86`
- compiler: CUDA strict f32 root, `-O3 --fmad=false --prec-div=true
  --prec-sqrt=true --ftz=false`
- each run: exact P1/P2 trace preflight, comparator release, one finalist
  instance, 256 conditioning, 64 warm-up, 512 measured executions
- advected state resets only at a complete 32-step epoch boundary
- raw totals, stage distributions, p95 and p99 are emitted from each process

## Decision results

All four reports have `status=PASS`, `trace_exact=true`,
`trace_memory_exact=true`, 512 valid samples and both decision gates true.

| Profile / process | p95 ms | p99 ms | mean ms | p95 headroom | p99 headroom |
|---|---:|---:|---:|---:|---:|
| coherent A | 3.210912 | 3.223072 | 3.052631 | 19.73% | 46.28% |
| coherent B | 3.211200 | 3.227296 | 3.072551 | 19.72% | 46.21% |
| advected A | 3.129184 | 3.181760 | 2.788862 | 21.77% | 46.97% |
| advected B | 3.149024 | 3.189504 | 2.798997 | 21.27% | 46.84% |

Fresh-process p95 spread is about `0.009%` coherent and `0.632%` advected.
The worst observed decision value is coherent B p95 `3.211200 ms`, leaving
`0.788800 ms` before the stop threshold.

Representative process-A stage p95:

| Profile | Neighbor | Density | Fused owner terms | Total |
|---|---:|---:|---:|---:|
| coherent 50k | 0.771072 | 0.821312 | 1.684512 | 3.210912 |
| advected 50k | 0.778240 | 0.790528 | 1.577984 | 3.129184 |

Stage percentiles are marginal distributions and are not summed to reconstruct
the total percentile.

## Correctness and capacity receipts

| Profile | Trace SHA-256 | Seed output SHA-256 | Seed logical CSR SHA-256 | Device bytes |
|---|---|---|---|---:|
| coherent 50k | `8de7dfcc1de15788ff932bc7e47338efca4bc23ec9767568ad21fd1297507af3` | `6b377e876439cbc72f9b5e81cfabaed854188144c576b498a06e2769ceb4aae1` | `8e915c74c617f2f172f0da419b17fc5185078840459344c09b6d9818113c6fe7` | 21,791,706 |
| advected 50k | `333e487207ffab664e1519705953f8530af1f5bb180c7f30d72d3193607aae46` | `bd1c5ad3a38b85ca1b7adca2333331d39c033d8b9f326b8d416213a557259e7a` | `1b176356938914f036682127fbfb322595fb2b6112043bb3f44247c843990392` | 29,315,610 |

Both use two-byte neighbor IDs. Seed directed pairs / maximum degree are
`5,427,724 / 123` coherent and `5,471,308 / 123` advected. The complete
advected 32-step trace matches P1 exactly at every output, logical CSR and
canonical handoff.

## Raw report hashes

| Report | SHA-256 |
|---|---|
| coherent A | `0940ac52af946f0071e186a22a65a80950654b6e30d5e299093a2781719dac9d` |
| coherent B | `aef5e89774b91e689fc770841021d90c6a460b5cc96538420ba318c1b99be1c6` |
| advected A | `d0b270f665e4ce01c5ad137924295f512557173d8c1412579ed20250613ec8a8` |
| advected B | `e4919d11950a6ac8765e64d78cb673111eaeb63ffdaf93252f25225cfea39322` |

Raw JSON and binaries remain outside Git under `/tmp`.

## Closed alternatives and conditional branches

- P3 is not retained despite a `1.779x` permuted win because coupled controls
  exceeded their non-regression gates.
- P4 stops at a reproduced active-order mismatch; geometric pair coverage
  alone cannot preserve the retained cell-traversal reduction order.
- NP2 adaptive exit/Anderson is not activated: it is an algorithm identity
  intended only if fixed work misses the decision target.
- NP3 active-domain/adaptive resolution is not activated as a performance
  rescue and remains separate future scale research.
- Pairwise Descent stays a watch item until a public primary paper/code audit.

These branches are conditionally closed, not silently unfinished.

## Required work before any production claim

1. Draft a separate Proposed solver reclosure with the broader SPEC-38
   correctness corpus and explicit CPU/canonical versus GPU authority choice.
2. Specify public state, persistence/replay and fault/admission semantics.
3. Close one composition DAG with boundary/rigid-body/PhysX exchange identity.
4. Measure the complete ADR-081 `world-dynamics-step`, not this standalone lab.
5. Perform the deferred cross-target/Windows root if Windows re-enters scope.

Until those steps land, DFSPH remains the correctness reference and fallback;
the Nonlocal tool remains isolated and report-only.

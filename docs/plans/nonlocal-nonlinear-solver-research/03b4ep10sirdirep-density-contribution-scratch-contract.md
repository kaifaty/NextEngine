# NSR3-B4EP10SIRDIREP -- density-contribution scratch contract

Status: `CLOSED / FAIL / BASELINE_HEALTH_AND_CANDIDATE_STABILITY / IMPLEMENTATION_REVERTED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10sirdirep-density-contribution-scratch|v1|parent=dfc1bb3d154f9e406c89d0fde2304983c837d9f27d43888be542f6e0e5697e1d:a60d07709cd12449feb6fc58c519387841edb68be29dd7493e8dc242d9f4c556:b4f847cb4f19b09e951534649515a4504bc07044a13e6c636598b33f247777e9|negative=96a22c8e733cfd334db50bd4cace64cbdf6e49ad53e544aa50c6b461f68b8f99:default-path-regression|implementation=ecc9ab903b9919099d855a531b304506a4161184|commands=baseline:nominal-hydro-directed-scratch-reuse-8,candidate:nominal-hydro-directed-scratch-density-contribution-reuse-8|ownership=transaction-local;parallel-trace-adjacent;one-f64-vector;release-all-exits|scope=density-contribution-only;returned-workspaces+gradient+tape+directed-unchanged|work=calls226;full-init85716150;growth380511;writes85716150;payload3044088|semantics=sirdirea-liveness;pair-order-exact;density-fold-exact;roots-exact;old-command-exact|capacity=8388608;fail-closed|timing=external-monotonic+gnu-time;one-warmup-each;three-pairs=AB,BA,AB;serialized;affinity=0-7|gates=baseline-median-ns<=4720000000;baseline-range-ratio<=1.10;candidate-exact-3of3;wins3of3;median-paired-speedup>=1.02;candidate-range-ratio<=1.10;rss-delta-kib<=8192;median-total-cpu-ratio<=1.02|failure=retain-sirdi|reference=closed|credit=candidate-residual-attribution-research-only
```

Identity SHA-256:
`9b5d3f045adae33600fc21f604c657409247062acca49e8806892a0ce38e8d74`.

## Implementation boundary

Add only:

```text
--nominal-hydro-directed-scratch-density-contribution-reuse-8
```

The candidate adds one `std::vector<double>` adjacent to the existing
transaction-local directed scratch trace. Acquire it only inside the exact
SIRDI owner-parallel evaluation builder. Grow to `pair_count` when needed and
never shrink until transaction exit. Pair and density kernels may access only
the current `[0, pair_count)` prefix. Keep all returned vectors, sizes,
allocators, moves/releases, arithmetic and fold order unchanged.

An RAII guard must release the scratch on every transaction exit, including
failure. Existing commands cannot enable or touch it. Candidate mode is valid
only with split incoming, directed scratch reuse, no shadow audit and no timing.

## Exact work and lifetime

Require exactly:

- 226 acquire calls and evaluation calls;
- 85,716,150 requested full slots and active write slots;
- 380,511 high-water/growth slots and 3,044,088 maximum payload bytes;
- one storage release, maximum one live buffer and zero final live buffers;
- zero failures and an empty scratch vector after guard release.

The candidate must retain all SIRDI directed-scratch, query, workspace,
retention, topology, pair, coefficient, plan and HVP counts. Maximum added
payload stays within 67,108,864 bytes; this scratch alone stays within
8,388,608 bytes.

## Correspondence

Each candidate process must reproduce B4EP10SIRDI result
`b4f847cb...777e9`, B4EP10SII result, correspondence, five physics roots,
query chain and duration-free semantics. Old SIRDI must remain byte-exact.
Release builds pass `-Werror`.

## External A/B

Use one binary and fixed 8-worker OpenMP placement on physical cores `0..7`.
Run one warmup per command, then serialized `AB`, `BA`, `AB` pairs. Record
monotonic wall nanoseconds and GNU time user/system/RSS.

Before comparing candidates require baseline median wall no more than
4,720,000,000 ns and baseline range ratio no more than `1.10`. Then require
candidate exactness `3/3`, candidate wins `3/3`, median paired speedup at least
`1.02`, candidate range ratio at most `1.10`, median RSS delta no more than
8,192 KiB and candidate/baseline median total CPU ratio at most `1.02`.

## Exit

PASS selects the exact candidate for residual attribution research only. Any
baseline-health, semantic, lifetime, capacity or performance failure retains
SIRDI and stops this buffer-initialization branch. Do not lower gates or reopen
allocator/raw/returned-storage work. B4E2, broad corpus, runtime/GPU/schema and
production remain blocked.

The measured candidate is exact and wins all three pairs at median
`1.194152x`, but baseline median wall is `4.893718116 s` above the frozen
`4.72 s` health bound and candidate range ratio is `1.174767` above `1.10`.
The implementation was reverted. See the
[dated evidence](../../development/nonlocal-nsr3b4ep10sirdirep-density-contribution-scratch-evidence-2026-08-22.md).

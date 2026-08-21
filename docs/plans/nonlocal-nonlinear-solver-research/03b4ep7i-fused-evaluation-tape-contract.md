# NSR3-B4EP7I -- fused evaluation/tape A/B contract

Status: `CLOSED / PASS / B4EP8_PROFILE_DESIGN_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep7i-fused-evaluation-tape|v1|parent=261bcd72315c16836f782751738a4e4a1bcd8a10dd9ba5cab9ff5478ecb987d0:b8942b1bec2dffe3f1fabbbcef508a4ec1c82d02fdbbb57a52c7ee089ab0f471:a40cbb27244c6bbbd6bd6c8359e8c0e3779ec2e1b4a3592b79ea10d89fb74a13:9b5453d91a99fc3c21c5024e578d1d5a5593d1bcc14063a421442d25d98487fe|candidate=46224e0e70c3fa1a21b3a1fa3b8e5aec81f10fcc6312d99314caec6c91e45e40:5cd61e3eb82f6a3cedfe4d7d8e39cbb5aca65c6e00c5ef9cd9e7555a71b9bf23:dac62e7528e08bc6d9dec91458bd2f7d78a03b75c0e8554f86d1fda5e89ae73b|implementation=40ee0fe0e791d41262fd772b77218c4221bd3d14|fusion=transaction-only;flat-only;pair-pass=density+radius+gradient+second;center-pass=energy+gradient+compression;csr=validate-then-move;fallback=none|order=density-pairs-unchanged;gradient-centers+adjacency-unchanged;vector-grouping-unchanged|work=queries226;N=85716150;D=131987230;C=1356000;radius=85716150;gradient=85716150;second=85716150;compression=1356000;fused-builds226|physics=bit-exact-roots+counters;workspace-chain=b4ep5-exact|runs=fused2-byte-exact;regressions=b4ep1,b4ep3,b4ep3i,b4ep5,b4ep7d-byte-exact|timing=BASELINE,FUSED,FUSED,BASELINE,BASELINE,FUSED;gate=3/3-wins;median-speedup>=1.10|watchdog=900s|reference=closed|credit=b4ep8-profile-design-only
```

Identity SHA-256:
`441ac76483748631f94ae4f2d092b97ba4d271fb2c380d2b6e889245527821c9`.

## Implementation boundary

Add an opt-in fused flat-workspace builder and trace work counters. Add only:

```text
nonlocal-formula-reclosure --nominal-hydro-fused-evaluation-tape-ablation
```

The command executes the unchanged parent plus the exact B4EP5 transaction
with fusion enabled. No old/default command opts in. Missing/partial arrays,
invalid CSR, capacity or counter mismatch rejects the command; there is no
fallback.

## Correspondence and work

Require exact B4EP5 physics, schedule, topology cache, coefficient HVP
lookups, evidence ownership and query work-chain. Require exactly:

- 226 fused workspace builds;
- `N=85,716,150`, `D=131,987,230`, `C=1,356,000`;
- 85,716,150 radius, gradient and second evaluations each;
- 1,356,000 compression evaluations;
- unchanged 171,432,300 coefficient kernel-build evaluations and
  971,831,424 HVP coefficient lookups;
- zero fusion mismatch and fallback.

Run two independent clean Release builds and require byte-identical fused
stdout. Require exact B4EP1/B4EP3/B4EP3I/B4EP5/B4EP7D stdout hashes.

## Timing

From identical final Release binaries run:

```text
BASELINE, FUSED, FUSED, BASELINE, BASELINE, FUSED
```

Baseline is B4EP5. Pair `(1,2)`, `(4,3)` and `(5,6)`. Require three wins and
median paired baseline/fused speedup at least `1.10`. Timing/RSS are external
under the 900-second watchdog.

## Exit

PASS selects `EXACT_FUSED_EVALUATION_TAPE_CANDIDATE` and authorizes only
B4EP8 residual profiling/design. Exactness failure rejects speed; speed failure
preserves B4EP7D. B4E2, runtime/CUDA, references, parallelism, solver-policy
and production remain blocked.

Observed PASS: exact fused correspondence, three paired wins and median
`1.111111111111x`; median wall is 8.90 s versus 8.00 s. See the
[dated evidence](../../development/nonlocal-nsr3b4ep7i-fused-evaluation-tape-evidence-2026-08-22.md).

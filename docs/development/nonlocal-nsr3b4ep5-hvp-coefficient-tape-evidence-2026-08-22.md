# NSR3-B4EP5 HVP coefficient-tape evidence -- 2026-08-22

Status: `PASS / RESEARCH_CANDIDATE_SELECTED / NOT_PRODUCTION`

## Result

The optional transaction-only coefficient tape preserves the exact B4EP3I
physics and removes repeated scalar-kernel evaluation from the HVP hot path.
All three frozen Release pairs win. Median wall time falls from 10.61 s to
8.50 s, a paired speedup of `1.248235294118x` against the already cached
B4EP3I baseline.

This selects `HVP_INVARIANT_COEFFICIENT_TAPE_CANDIDATE` and authorizes only
B4EP6 residual profiling/design. It creates no default, runtime, CUDA,
reference, B4E2 or production authority.

## Frozen identity and implementation

- contract identity: `46224e0e70c3fa1a21b3a1fa3b8e5aec81f10fcc6312d99314caec6c91e45e40`;
- implementation commit: `7263d8929491b66ea94eb74713fab4c136efe5be`;
- candidate semantic result:
  `5cd61e3eb82f6a3cedfe4d7d8e39cbb5aca65c6e00c5ef9cd9e7555a71b9bf23`;
- candidate stdout SHA-256, including LF:
  `dac62e7528e08bc6d9dec91458bd2f7d78a03b75c0e8554f86d1fda5e89ae73b`;
- work-chain root:
  `6a220a4e6f4d6d06ab54fe043a9ddf49606aae40e598e43f1c331c78b7802991`;
- work receipt:
  `73c1356d1e6d0174a5740b34869f4183bfd995ab8c3b4e579c771ddab00b345b`.

Two independent clean Release builds are byte-identical:

- executable size: 3,777,848 bytes;
- executable SHA-256:
  `6c99c84906c1f01d55ea2a197f055a3e4e5a6b9676e4d8e462fcf1aef23785f7`;
- ELF Build ID: `e5cb1109c962d815f6804983b2742fcf59641f9c`;
- GCC 15.2.0, CMake 4.2.3, Ninja 1.13.2;
- Release flags: `-O3 -DNDEBUG -ffp-contract=off -fno-fast-math`.

Source SHA-256 values are:

- `boundary_reference.cpp`:
  `6fe62f0d4a095062059a5387d9c85e8d20ce5b96cb951575597a840c98304d63`;
- `boundary_reference.hpp`:
  `f7f6ea10b2a378ea7a291ae9924278b8c64c9b59e2958727f9b8052ea696511d`;
- `formula_reclosure_main.cpp`:
  `cdeaf706f179eb716257d55b5f2dbf0371add899978bd7c86d8ea8309b2288fa`.

External artifacts remain under
`/home/kaifaty/.cache/nextengine/external/b4ep5/`.

## Exact correspondence

The candidate reports:

- initial/selected substeps `14/28`;
- attempted/accepted/discarded substeps `42/28/14`;
- outer/rejected trials `221/0`;
- nonlinear/spectral HVP calls `411/48`;
- exact frame and aggregate roots
  `eaa6fe3...31eb5` / `8a634d69...a90dc`;
- exact trajectory, legacy-ledger and policy-ledger roots
  `4689e743...6fc4`, `df58c67e...c057`, `b8502f70...0444`;
- parent full-state hashes `1/0` and transaction hashes `0/226`.

The selected topology cache remains exact: 226 queries, one rebuild, 225
certified reuses, zero certificate failures/fallbacks, maximum degree 122 and
candidate/active visit ratio `1.0685913914705689`.

The new coefficient evidence is:

- 226 coefficient-tape builds over 85,716,150 pairs;
- 171,432,300 scalar kernel evaluations at tape construction;
- 459 HVP calls and 971,831,424 logical coefficient lookups;
- maximum added coefficient payload 6,088,176 bytes per workspace;
- zero mismatch and zero fallback.

The candidate stdout is byte-identical across the two Release builds. The old
command stdout hashes also remain exact:

- B4EP1: `4d63f5f05811357b958b18380ec483cd97073ae02c3a0228098e255d73da8112`;
- B4EP3: `4d62367830fd5a32f2f1ec32d07ebae6833cf2491c91af255022efdb988d7095`;
- B4EP3I: `b0ed87ff3e9cd1131b0c188c84634ab4c01e0453b91342abd4d6f5bb3ae99055`.

## Frozen timing gate

The exact process order was
`BASELINE,CANDIDATE,CANDIDATE,BASELINE,BASELINE,CANDIDATE`:

| Position | Command | Wall (s) | RSS (KiB) | CPU | Exit |
|---:|---|---:|---:|---:|---:|
| 1 | B4EP3I baseline | 10.61 | 62,368 | 99% | 0 |
| 2 | B4EP5 candidate | 8.50 | 63,196 | 99% | 0 |
| 3 | B4EP5 candidate | 8.50 | 63,328 | 99% | 0 |
| 4 | B4EP3I baseline | 10.61 | 63,252 | 99% | 0 |
| 5 | B4EP3I baseline | 10.61 | 63,504 | 99% | 0 |
| 6 | B4EP5 candidate | 8.50 | 62,816 | 99% | 0 |

All stderr files are empty. Pair positions `(1,2)`, `(4,3)` and `(5,6)` each
yield `1.248235294118x`; the median is the same and clears `1.10x`. Median RSS
is 63,252 KiB for baseline and 63,196 KiB for candidate; RSS is recorded as
an observation, not a memory-saving claim.

## Preserved negative evidence

The first instrumented smoke run failed only the frozen coefficient counter:
it recorded 728,873,568 reads after the implementation reused one local
`gradient` value for both second-pass expressions. The frozen 971,831,424
count represented four original logical call sites per active directed visit,
not three. The implementation was corrected to keep separate Jacobian and
tangential-gradient reads, restoring the original arithmetic graph. The
contract was not changed, and the second run passed every gate.

This matters because the selected change is now strictly a call-site
substitution: no common-subexpression, traversal or reduction rewrite is
silently credited to B4EP5.

## Evidence attestation

Exact projection, without final LF:

```text
nextengine.nonlocal.nsr3b4ep5-evidence|v1|identity=46224e0e70c3fa1a21b3a1fa3b8e5aec81f10fcc6312d99314caec6c91e45e40|implementation=7263d8929491b66ea94eb74713fab4c136efe5be|result=5cd61e3eb82f6a3cedfe4d7d8e39cbb5aca65c6e00c5ef9cd9e7555a71b9bf23|stdout=dac62e7528e08bc6d9dec91458bd2f7d78a03b75c0e8554f86d1fda5e89ae73b|binary=6c99c84906c1f01d55ea2a197f055a3e4e5a6b9676e4d8e462fcf1aef23785f7|regressions=4d63f5f05811357b958b18380ec483cd97073ae02c3a0228098e255d73da8112,4d62367830fd5a32f2f1ec32d07ebae6833cf2491c91af255022efdb988d7095,b0ed87ff3e9cd1131b0c188c84634ab4c01e0453b91342abd4d6f5bb3ae99055|speedups=1.248235294118,1.248235294118,1.248235294118|median=1.248235294118|rss=63252,63196|work=226,85716150,171432300,971831424,6088176|receipt=73c1356d1e6d0174a5740b34869f4183bfd995ab8c3b4e579c771ddab00b345b
```

SHA-256:
`9f47bda20166eb985b62b418f53acad2348311b309de66d01c53490b73abd8e9`.

## Decision

Retain B4EP5 as an internal exact research candidate. Before any further
mechanical optimization, profile the exact B4EP5 command again and select at
most one residual category under a new frozen B4EP6 contract. Do not infer
real-time, production or GPU readiness from this single nominal CPU corpus.

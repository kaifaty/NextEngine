# NSR3-B4EP7I fused evaluation/tape evidence -- 2026-08-22

Status: `PASS / EXACT_FUSED_WORKSPACE_CANDIDATE / NOT_PRODUCTION`

## Result

The transaction-only fused builder preserves exact B4EP5 physics, schedule,
work-chain and base work receipt while reducing duplicate evaluation/tape
work to the B4EP7D lower bound. All three frozen timing pairs win. Median
paired speedup is `1.111111111111x`; median wall falls from 8.90 s to 8.00 s.

This clears the `1.10x` gate by a small margin. B4EP7I is retained only as an
internal exact research candidate for the frozen nominal corpus. It authorizes
only B4EP8 residual profiling/design and creates no general performance,
runtime, GPU or production claim.

## Identity and reproducibility

- contract identity:
  `441ac76483748631f94ae4f2d092b97ba4d271fb2c380d2b6e889245527821c9`;
- implementation commit: `a3aa054217cfd9d21193f8943effe10b7ebce7bf`;
- semantic result:
  `e3453dc158a8eace69b468ac84641cccdcad221dcc05695d06b6f400039e775c`;
- stdout SHA-256, including LF:
  `8d3c8115861feb80e322a594be8f338dcab9622ac7b2839ec17a0f779fac2095`;
- base B4EP5 work receipt:
  `73c1356d1e6d0174a5740b34869f4183bfd995ab8c3b4e579c771ddab00b345b`;
- fused work receipt:
  `92dc0c1385f8040c6ce55c89d2052f0ed8c3775aaba355b5764753fa2a1c80b3`.

Two clean Release builds are byte-identical:

- executable size: 3,812,208 bytes;
- executable SHA-256:
  `7456e21f22a7900bf3f36ddf29afd9400c3897b13db348c557a4cacb9b34d6d2`;
- Build ID: `0082496dcc7508faf709d5be143d118e8959b9b2`;
- fused stdout size: 6,714 bytes.

Source SHA-256 values are:

- `boundary_reference.cpp`:
  `fb603f7729b1a653e0badec3dd06ef47a4f4458214c49b5678d63da79b474e67`;
- `boundary_reference.hpp`:
  `11df6d9de8f6c924ab080ca8bd852c52b43cedbba0a3e67b1ecc22539e9a8fd0`;
- `formula_reclosure_main.cpp`:
  `fc9e3e05d6e235d2546755d0918cabbc229d6d342654c4cc0328519df3e9f58e`.

External artifacts remain under
`/home/kaifaty/.cache/nextengine/external/b4ep7i/`.

## Exact correspondence

The candidate retains:

- 14 initial and 28 selected/accepted substeps;
- 42 attempted, 14 discarded substeps and 221/0 outer/rejected trials;
- 411 nonlinear and 48 spectral HVPs;
- exact frame/aggregate, trajectory and both ledger roots;
- parent full-state hashes `1/0`, transaction hashes `0/226`;
- B4EP5 query work-chain
  `6a220a4e6f4d6d06ab54fe043a9ddf49606aae40e598e43f1c331c78b7802991`.

Frozen fused work is exact:

- 226 builds;
- 85,716,150 pair/radius/gradient/second visits each;
- 131,987,230 active directed gradient visits served from the stored scalar;
- 1,356,000 center/compression visits;
- 171,432,300 coefficient-build kernel evaluations and 971,831,424 HVP
  coefficient lookups;
- zero mismatch and zero fallback.

Both fused reports are byte-identical. Old reports remain exact:

- B4EP1 `4d63f5f0...8112`;
- B4EP3 `4d623678...7095`;
- B4EP3I `b0ed87ff...9055`;
- B4EP5 `dac62e75...e73b`;
- B4EP7D `9b5453d9...87fe`.

## Timing

| Position | Command | Wall (s) | RSS (KiB) | CPU | Exit |
|---:|---|---:|---:|---:|---:|
| 1 | B4EP5 baseline | 8.90 | 62,020 | 99% | 0 |
| 2 | B4EP7I fused | 8.01 | 61,888 | 98% | 0 |
| 3 | B4EP7I fused | 8.00 | 61,856 | 99% | 0 |
| 4 | B4EP5 baseline | 8.90 | 61,816 | 99% | 0 |
| 5 | B4EP5 baseline | 8.50 | 62,628 | 99% | 0 |
| 6 | B4EP7I fused | 7.70 | 62,784 | 99% | 0 |

Pair speedups are `1.111111111111x`, `1.112500000000x` and
`1.103896103896x`; all win and the median clears `1.10x`. Median RSS is
62,020 KiB baseline versus 61,888 KiB fused; this is observational only.
Every stderr is empty and every JSON hash is exact.

## Evidence attestation

Exact projection, without final LF:

```text
nextengine.nonlocal.nsr3b4ep7i-evidence|v1|identity=441ac76483748631f94ae4f2d092b97ba4d271fb2c380d2b6e889245527821c9|implementation=a3aa054217cfd9d21193f8943effe10b7ebce7bf|result=e3453dc158a8eace69b468ac84641cccdcad221dcc05695d06b6f400039e775c|stdout=8d3c8115861feb80e322a594be8f338dcab9622ac7b2839ec17a0f779fac2095|binary=7456e21f22a7900bf3f36ddf29afd9400c3897b13db348c557a4cacb9b34d6d2|build-id=0082496dcc7508faf709d5be143d118e8959b9b2|work=85716150,131987230,1356000,85716150,85716150,85716150,1356000|base-receipt=73c1356d1e6d0174a5740b34869f4183bfd995ab8c3b4e579c771ddab00b345b|receipt=92dc0c1385f8040c6ce55c89d2052f0ed8c3775aaba355b5764753fa2a1c80b3|regressions=4d63f5f05811357b958b18380ec483cd97073ae02c3a0228098e255d73da8112,4d62367830fd5a32f2f1ec32d07ebae6833cf2491c91af255022efdb988d7095,b0ed87ff3e9cd1131b0c188c84634ab4c01e0453b91342abd4d6f5bb3ae99055,dac62e7528e08bc6d9dec91458bd2f7d78a03b75c0e8554f86d1fda5e89ae73b,9b5453d91a99fc3c21c5024e578d1d5a5593d1bcc14063a421442d25d98487fe|speedups=1.111111111111,1.112500000000,1.103896103896|paired-median=1.111111111111|wall-medians=8.90,8.00|rss-medians=62020,61888
```

SHA-256:
`de432459df752b4dd8d1b8ffc186718178fc68e65f19d503d1dffd0137f965bd`.

## Decision

Retain `EXACT_FUSED_EVALUATION_TAPE_CANDIDATE` for nominal research. Reprofile
the exact fused command before selecting any further serial change. Because
the timing margin is modest, broader corpus and variance evidence will be
required before this mechanism can enter a production roadmap.

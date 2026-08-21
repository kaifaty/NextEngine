# NSR3-B4EP7D evaluation/tape dataflow evidence -- 2026-08-22

Status: `PASS / EXACT_DUPLICATE_WORK_FROZEN / NO_FUSION_YET`

## Result

The two-build dataflow audit passes with exact B4EP5 physics, topology cache,
coefficient counts and ownership. It freezes
`D=131,987,230` active directed evaluation records across 226 transaction
workspaces.

An exact fused pair/center pipeline can therefore remove:

- 217,703,380 of 303,419,530 radius evaluations (`71.749956%`);
- 131,987,230 of 217,703,380 gradient-kernel evaluations (`60.627093%`);
- 1,356,000 of 2,712,000 compression evaluations (`50%`).

No fusion or timing was executed. PASS authorizes only a frozen B4EP7I exact
implementation/A-B design.

## Reproducibility

| Field | Value |
|---|---|
| identity | `261bcd72315c16836f782751738a4e4a1bcd8a10dd9ba5cab9ff5478ecb987d0` |
| implementation | `40ee0fe0e791d41262fd772b77218c4221bd3d14` |
| semantic result | `a40cbb27244c6bbbd6bd6c8359e8c0e3779ec2e1b4a3592b79ea10d89fb74a13` |
| stdout SHA-256 | `9b5453d91a99fc3c21c5024e578d1d5a5593d1bcc14063a421442d25d98487fe` |
| stdout size | 6,505 bytes |
| work receipt | `586d0e339e9ef0aedea40f0eec12c0b6c1b310024bc214dac94b75909752bf85` |
| executable SHA-256 | `540bc45039d0a174f2ee1343e81f4a3101eebed4c8d5ea7746375828992d8941` |
| executable size | 3,795,216 bytes |
| Build ID | `126a4b66943ec58bd374330b34132fdae5f2fb49` |

Both independent Release builds and both audit JSON files are byte-identical.
External artifacts remain under
`/home/kaifaty/.cache/nextengine/external/b4ep7d/` and the two B4EP5 build
directories.

Source SHA-256 values are:

- `boundary_reference.cpp`:
  `57dea1abed98680d76595162da513f265e601830aa8a033b9916628b3d1b5c19`;
- `boundary_reference.hpp`:
  `363b6de7a2397259d6d7807e32ce144379f533ce096802f2ccd9840a89b182b7`;
- `formula_reclosure_main.cpp`:
  `34db5b1267086264c0e99032b68888558bcecf3e3664c3cf002ef1f23878e342`.

## Exact dataflow

| Quantity | Current | Projected fused | Removed |
|---|---:|---:|---:|
| radius evaluations | 303,419,530 | 85,716,150 | 217,703,380 |
| gradient-kernel evaluations | 217,703,380 | 85,716,150 | 131,987,230 |
| compression evaluations | 2,712,000 | 1,356,000 | 1,356,000 |

The audit derives these totals after each valid tape, rather than incrementing
inside hot loops. Arithmetic overflow checks pass. Coefficient mismatch and
fallback remain zero.

All frozen regression reports remain exact:

- B4EP1: `4d63f5f05811357b958b18380ec483cd97073ae02c3a0228098e255d73da8112`;
- B4EP3: `4d62367830fd5a32f2f1ec32d07ebae6833cf2491c91af255022efdb988d7095`;
- B4EP3I: `b0ed87ff3e9cd1131b0c188c84634ab4c01e0453b91342abd4d6f5bb3ae99055`;
- B4EP5: `dac62e7528e08bc6d9dec91458bd2f7d78a03b75c0e8554f86d1fda5e89ae73b`.

## Evidence attestation

Exact projection, without final LF:

```text
nextengine.nonlocal.nsr3b4ep7d-evidence|v1|identity=261bcd72315c16836f782751738a4e4a1bcd8a10dd9ba5cab9ff5478ecb987d0|implementation=40ee0fe0e791d41262fd772b77218c4221bd3d14|result=a40cbb27244c6bbbd6bd6c8359e8c0e3779ec2e1b4a3592b79ea10d89fb74a13|stdout=9b5453d91a99fc3c21c5024e578d1d5a5593d1bcc14063a421442d25d98487fe|binary=540bc45039d0a174f2ee1343e81f4a3101eebed4c8d5ea7746375828992d8941|build-id=126a4b66943ec58bd374330b34132fdae5f2fb49|dataflow=85716150,131987230,1356000,303419530,85716150,217703380,85716150,2712000,1356000|fractions=0.71749956240456902,0.60627092698331098|receipt=586d0e339e9ef0aedea40f0eec12c0b6c1b310024bc214dac94b75909752bf85|regressions=4d63f5f05811357b958b18380ec483cd97073ae02c3a0228098e255d73da8112,4d62367830fd5a32f2f1ec32d07ebae6833cf2491c91af255022efdb988d7095,b0ed87ff3e9cd1131b0c188c84634ab4c01e0453b91342abd4d6f5bb3ae99055,dac62e7528e08bc6d9dec91458bd2f7d78a03b75c0e8554f86d1fda5e89ae73b|timing=none
```

SHA-256:
`b8942b1bec2dffe3f1fabbbcef508a4ec1c82d02fdbbb57a52c7ee089ab0f471`.

## Decision

Select `EXACT_EVALUATION_TAPE_FUSION_FEASIBLE` for B4EP7I design/A-B only.
The implementation must retain density and gradient accumulation order, move
the same flat CSR ownership, preserve all default/old command bytes and prove
bit-exact workspace/physics output before speed is considered.

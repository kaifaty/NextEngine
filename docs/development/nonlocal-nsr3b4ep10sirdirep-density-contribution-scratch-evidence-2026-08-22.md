# NSR3-B4EP10SIRDIREP density-contribution scratch evidence -- 2026-08-22

Status: `FAIL / BASELINE_HEALTH_AND_CANDIDATE_STABILITY / IMPLEMENTATION_REVERTED`

## Result

The candidate is mechanically exact. All three measured processes reproduce
candidate stdout SHA-256 `6a03e16a...a16a5`, identity
`9b5d3f0...e8d74`, result `a1355484...76e5`, the retained SIRDI/SII results,
correspondence and all five physics roots. Old SIRDI remains byte-exact at
stdout SHA-256 `539f1ec5...e7e7`; every measured stderr is empty.

The scratch also matches every frozen work/lifetime count: 226 evaluation
acquires, 85,716,150 requested and written slots, 380,511 growth/maximum
slots, 3,044,088 maximum payload bytes, one release, one maximum live lane,
zero final live lanes and zero failures.

The balanced `AB`, `BA`, `AB` run nevertheless fails two predeclared gates:

| Metric | Measured | Gate | Result |
|---|---:|---:|---|
| baseline wall ns | `5,316,432,703 / 4,850,720,305 / 4,893,718,116` | median `<= 4,720,000,000` | FAIL (`4,893,718,116`) |
| baseline range ratio | `1.0960089159376918` | `<= 1.10` | PASS |
| candidate wall ns | `4,452,055,142 / 4,567,911,618 / 3,888,353,938` | range `<= 1.10` | FAIL (`1.1747674442284786`) |
| paired speedup | `1.1941524831634711 / 1.06191203128484 / 1.2585577840985112` | wins `3/3`, median `>= 1.02` | PASS (`1.1941524831634711`) |
| median total CPU ratio | `0.906544647809207` | `<= 1.02` | PASS |
| median RSS delta | `-1,932 KiB` | `<= 8,192 KiB` | PASS |

The attractive relative result cannot override the failed baseline-health and
candidate-stability gates. The contract forbids a rerun, threshold change or
speed credit after observing these values.

## Reproducibility

The measured candidate was implementation commit
`c5a9a9c9f8430454c449bd76f5885abab0bf760b`. Its executable SHA-256 is
`fe30785bd8366267e0e786b2795d4ab49b8a7cbb6d462062f493b80cc20853a7`,
size 4,361,360 bytes and ELF Build ID
`b768bbb12be6e0793387258be404364eca276819`.

Candidate source hashes were:

- `boundary_reference.cpp`: `cbc671204ff99b7b649d7145be29839cdb1e4574df265412600074f584788bdc`;
- `boundary_reference.hpp`: `1160eaefc2ff14fe3f8c8a3c51f7324c18ae244b4b55f58d76dd1761313b7f37`;
- `formula_reclosure_main.cpp`: `17b4ab3b4545676daf2ddb104ff146ec78d1113ab06c3456a1dd1761313b7f37`;
- `compile_commands.json`: `78defa02edbba16b38737c574127a5d8474b026bfeebbdc14a2f5a9ea9f13a66`.

Raw A/B artifacts remain outside Git under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10sirdirep.tDk9Ls`.
The exact derived metrics file has SHA-256
`486be90c4620db7f367dce8a8bfe250fd7251505a862d96649702a3fdfdf7cff`.

The candidate code was reverted by
`b8a1eddffcf37a6281e2f67bc80e9a9ef07b2f04`. The rebuilt rollback command
again emits exact SIRDI stdout SHA-256 `539f1ec5...e7e7`, empty stderr and a
cold 4.79 s wall control under
`/home/kaifaty/.cache/nextengine/external/verify-nonlocal-b4ep10sirdirep-rollback.2mLc3L`.

## Decision

Close B4EP10SIRDIREP as FAIL, retain SIRDI and stop the evaluation-buffer
initialization branch. Do not retry allocator hooks, returned-workspace
pooling, this density scratch, lower gates or rerun the same A/B. Reconsider
storage representation only under a separately authorized ownership and
portability redesign.

No speed credit, B4E2 authority, broad-corpus authority, runtime/GPU/schema or
production authority is granted. Continue from unchanged SIRDI/SIRDIR residual
evidence and select a different bounded discriminator.

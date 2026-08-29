# NSR3-B4EP10SII split incoming plan evidence -- 2026-08-22

Status: `PASS / SPLIT_INCOMING_RESEARCH_PATH_SELECTED`

## Result

The topology-owned split incoming plan preserves the complete nominal Hydro
transaction exactly and clears the frozen performance gate:

- all three candidate reports are byte-identical at SHA-256
  `31990f6f01a4f71528f214f97860b5e9b4561651b3f4c5f4b0105fa524ed17ef`;
- frame, aggregate, trajectory and both ledger roots match B4EP10I;
- 226 plan builds serve 226 evaluation and 459 HVP calls;
- the exact floating fold scans 454,936,226 incoming entries and retains
  374,945,086 incoming plus 374,945,086 own-row contributions;
- all 4,089 executor regions and 261,696 logical partitions match the frozen
  construction boundary;
- candidate failure, order, coverage and fallback counts are zero;
- maximum actual added payload is 24,586,324 bytes, below 64 MiB.

The candidate wins all three balanced pairs. Median wall falls from
5.819690664 s to 5.536671494 s, a paired median speedup of
`1.0525214900893305x`. This narrowly exceeds the predeclared `1.05x` gate;
the result selects the path for further research but is not production or
broad-corpus evidence.

## External A/B

One warmup per command preceded serialized `AB`, `BA`, `AB` rounds on CPUs
`0..7`. Every process exited zero and every program stderr file is empty.

| Pair | B4EP10I wall ns | Split incoming wall ns | A/B |
|---:|---:|---:|---:|
| 1 | 5,819,690,664 | 5,516,705,323 | 1.054921429233642 |
| 2 | 5,827,465,731 | 5,536,671,494 | 1.0525214900893305 |
| 3 | 5,761,923,417 | 5,550,087,135 | 1.038168100220286 |

Candidate wall range ratio is `1.0060510413454251`. Median maximum RSS falls
from 96,560 to 91,708 KiB, a passing `-4,852 KiB` delta.

Median user CPU rises from 37.52 to 39.36 seconds and median system CPU from
0.74 to 1.15 seconds. The wall improvement therefore comes with more total
CPU work from construction and masked incoming scans. A residual timing stage
must distinguish whether construction, target scans or the unchanged HVP
directed work is now the next bounded target.

## Build and regressions

- implementation commit:
  `e7dec71bee66a72a497ef02315688995b675e345`;
- executable: 4,242,152 bytes,
  `c9964a6c29d0f4952668734aeea26e7671baea488a6918422e431dd68992d499`;
- Build ID: `34e3383dce4f69b570406873ceee1657ed1026cf`;
- `compile_commands.json` SHA-256:
  `39e46b8bf133bbee9298c948a2dd897722187f9696cd442198a2cbfedb9e2938`;
- raw A/B metrics SHA-256:
  `d4a34fdeb98a16f2f2f3af3c6a6004f53cc5ad27712e12b34adf8c2d54ced42d`.

Source hashes:

- `CMakeLists.txt`:
  `48301abd9ba4c4005c1b0d15714a2d4a2ecc07e20dffbc58707535d0bb99625c`;
- `boundary_reference.cpp`:
  `e9c2f4c5b083132d5af5bc2a8bab2834ce744cce8f5f3cbfe6a41a66801606e3`;
- `boundary_reference.hpp`:
  `b348627540fea05783cce5765cfbd2f9279cd3eee36328c42b8ff6ef12f02b44`;
- `formula_reclosure_main.cpp`:
  `a92d67ea96bd32378565fc20e14b70be4cbf95e1ac6d63b39c0c72bc063c9368`.

The final binary preserves exact B4EP10I worker-8 stdout
`c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3`,
B4EP10SICD stdout
`bf4164f6d293c69c5d9774b96d3ece10275b273a95d29c07d56c38fa166f6c5c`
and B4EP10R1 semantic result
`a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b`.

External artifacts remain outside Git under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10sii.wFdjYh`.

## Evidence attestation

Exact projection, without final LF:

```text
nextengine.nonlocal.nsr3b4ep10sii-evidence|v1|identity=9a496e6129ce4669af31aa056446f3743404bfe38ebf458edff1f6cb7b816774|implementation=e7dec71bee66a72a497ef02315688995b675e345|binary=c9964a6c29d0f4952668734aeea26e7671baea488a6918422e431dd68992d499|build-id=34e3383dce4f69b570406873ceee1657ed1026cf|compile=39e46b8bf133bbee9298c948a2dd897722187f9696cd442198a2cbfedb9e2938|metrics=d4a34fdeb98a16f2f2f3af3c6a6004f53cc5ad27712e12b34adf8c2d54ced42d|stdout=A:c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3,B:31990f6f01a4f71528f214f97860b5e9b4561651b3f4c5f4b0105fa524ed17ef|result=B:f7b1542f30fef20a08da57c83bb200878cdf426d87b627445e422c9a72825fb2|correspondence=1e4bedbb2c3ed7512ee0a31e879c9d7ba7887dca1fb5e833c89f0c45b108a35d|wall-ns=A:5819690664,5827465731,5761923417;B:5516705323,5536671494,5550087135|paired-speedups=1.054921429233642,1.0525214900893305,1.038168100220286;median=1.0525214900893305|medians-wall-ns=A:5819690664,B:5536671494|candidate-range-ratio=1.0060510413454251|rss-kib=A:96560,B:91708,delta:-4852|user-s=A:37.52,B:39.36|system-s=A:0.74,B:1.15|gates=exact3of3;wins3of3;speed-pass;range-pass;rss-pass|work=builds226;eval226;hvp459;incoming-full454936226;incoming-retained374945086;own-retained374945086;regions4089;partitions261696;payload24586324|regressions=bf4164f6d293c69c5d9774b96d3ece10275b273a95d29c07d56c38fa166f6c5c,a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b|decision=select-split-incoming-research-path;residual-timing-next
```

SHA-256:
`5c8ca71e2c6d7ebe77e3868b9077254694a6ac640300755d9ba06e293ff168cd`.

## Decision

Select the split incoming plan for the nominal 8-worker research path and
retain B4EP10I as the rollback. Freeze one candidate-specific residual timing
stage before changing construction or floating work again. B4E2, broad
corpus, runtime, GPU, schema and production remain blocked.

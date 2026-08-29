# Nonlocal corrected GPU neighborhood performance evidence — NCGP0

| Field | Value |
| --- | --- |
| Result | `50K_SUPPORTED_BOUNDED / 16K_TIMING_INCONCLUSIVE` |
| Repository checkpoint | detached `bb73075c29a5ec0b0a01c789487dd7850fb3aaf9` |
| Device | NVIDIA GeForce RTX 3080, SM 8.6, CUDA 13.3 |
| Scope | Exact canonical GPU neighborhood builder only |
| Product status | `REPORT_ONLY`; not full Nonlocal solve, water simulation or frame time |

## Outcome

An independent external prototype scaled the reviewed NCGA1 integer/stable-ID
semantics to exact 16k/50k profiles. All correctness and capacity gates passed.
The three 50k profiles reproduced p95 latency within `0.19..0.30%` across two
fresh processes. The 16k p95 differed by `10.79%` while process A crossed a
visible GPU DVFS transition, so that timing is explicitly inconclusive; no
third retry was used to turn it green.

| Profile | Process A p50/p95/p99 ms | Process B p50/p95/p99 ms | p95 spread |
| --- | ---: | ---: | ---: |
| 16k coherent | `0.581632 / 0.642048 / 0.660480` | `0.538624 / 0.576320 / 0.593920` | `10.79%` — inconclusive |
| 50k coherent | `0.963584 / 1.018880 / 1.027072` | `0.965632 / 1.021952 / 1.039360` | `0.30%` |
| 50k fixed permutation | `1.042432 / 1.091584 / 1.101824` | `1.039360 / 1.093632 / 1.110016` | `0.19%` |
| 50k advected snapshot | `1.126400 / 1.183744 / 1.186816` | `1.126240 / 1.181504 / 1.185792` | `0.19%` |

Thus the bounded current estimate for exact 50k neighborhood construction is
about `1.02..1.18 ms p95` on this RTX 3080. It cannot be added to an unmeasured
assembly/solver time.

## Frozen measurement protocol

- exact signed-micrometre predicate `distance_squared <= 100000^2`;
- stable `SampleId` owner rows and strictly ascending neighbor IDs;
- device key build, stable owner/cell radix sorts, two exact 27-cell traversals,
  scan and bounded device row sort;
- no host canonicalization or host round trip in the measured path;
- preallocated hot path, 256 conditioning iterations, 32 warmups and 128 CUDA
  event samples;
- nearest-rank p50/p95/p99 indices `63/121/126`, without outlier removal or
  retry.

The reviewed NCGA1 aggregate root reproduced exactly as
`c8b905f78a161fbb1fa40a4bd05c7458e9d47be98343e934f3c1e12c11ca01e8`.
The deliberate `<` radius identity was rejected. Coherent and fixed-permuted
50k inputs produced the same semantic graph root.

## Scale and capacity

| Profile | Samples | Directed neighbor IDs | Maximum degree | Graph root |
| --- | ---: | ---: | ---: | --- |
| 16k coherent | 16,000 | 484,952 | 33 | `f09a22c379679fecabf09da5098d54e2226cef346334da9ba549de1e420adc7c` |
| 50k coherent | 50,000 | 1,557,872 | 33 | `7bfe96dd72420e374306a193619dc56d7a1906cd5f8953b7e7f5b1863ff39488` |
| 50k fixed permutation | 50,000 | 1,557,872 | 33 | `7bfe96dd72420e374306a193619dc56d7a1906cd5f8953b7e7f5b1863ff39488` |
| 50k advected snapshot | 50,000 | 1,434,740 | 36 | `7983333a54ea839f4a38f3d66830cf9d79ba54a9a9ac31a55c75a84c592c129e` |

The prototype reserved 12,800,000 neighbor IDs (`50,000*256`) and
56,636,549 device bytes (54.01 MiB). The largest observed degree was 36.

## Bottleneck discriminator

The duplicate exact 27-cell count/fill traversals consume `63.76..72.00%` of
the sum of 50k p50 stage medians. Bounded row sorting consumes
`13.73..19.77%`; both radix sorts together consume `12.80..14.88%`.

This falsifies the frozen alternatives that row sorting or radix sorting is the
first bottleneck. The next performance optimization, if separately authorized,
should eliminate or amortize the duplicated spatial traversal. No bandwidth,
occupancy or cache-level explanation is claimed because profiler counters were
not captured.

## Artifacts

| Artifact | SHA-256 |
| --- | --- |
| Full external report | `899b28cb6c4999969327d5a38ee728e40160021207f825befe03f317547dd455` |
| Contract revision 1 | `ee56f9412ab84dc1755e486eda40c4187c6400ed7aaaf6213306b1f3723de7f5` |
| Percentile addendum revision 2 | `969717ba891c0181d0c36a33c191d1937d5030987ad9599d7c360a78ffd63c74` |
| Manifest revision 2 | `495c9b35fb61164d3f5490dd9e2a65d7b6090e3c0e1840f5e19eb12e76c9c402` |
| CUDA prototype source | `7d1310742cb6b4a19d3c0a81d90c8951ebfb50f699849b4acfcb42eee4e6a2ad` |
| Clean-build stripped binary A/B | `4bb8f268c60abff51833c4ea98d184349a00bdfbcd191a64a462ff84cec421c3` |
| Raw process A | `c75eb6022d8aacaaad04574412628eca521c7ed9394867937e2dd7d9c8cb92a3` |
| Raw process B | `7f65c45b42b2fbb4908f89ce52f46b67335fce59ed0c92511d0df328882e6680` |

The full report and raw artifacts remain outside Git under
`/tmp/nextengine-ncgp0-perf.pBPccQ`. `compute-sanitizer` was not run on this
scalable exploratory prototype, so this is bounded performance research rather
than durable production-benchmark evidence. Packaging the neighborhood stage
as a production-worthy reviewed benchmark is estimated at 4–6 engineering
days; corrected assembly and the full solver are separate work packages.


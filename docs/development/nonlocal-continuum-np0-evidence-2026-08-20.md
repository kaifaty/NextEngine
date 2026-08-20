# Nonlocal continuum NP0 evidence — 2026-08-20

Status: `NP0_COMPLETE / NP1_P1_AUTHORIZED / REPORT_ONLY / NO_W2_CREDIT`

## Decision

NP0 closes successfully. The separately rooted v1 corpus is reproducible and
the retained NR4 implementation is now measured at exact 50k on coherent,
permuted and sequentially advected states.

The fixed-work performance target is not yet met. Across two conditioned
fresh-process decision campaigns, the worse exact-50k results were:

| Profile | p95 | p99 | `4/6 ms` result |
|---|---:|---:|---|
| coherent 50k | `4.869248 ms` | `5.078048 ms` | p95 miss; p99 pass |
| advected 50k | `4.494592 ms` | `4.817280 ms` | p95 miss; p99 pass |

This is a valid NP1 denominator, not an NP4 terminal state. P1 must remove at
least about `17.9%` from the worse coherent p95 and `11.0%` from the worse
advected p95 to reach 4 ms, while preserving both profiles.

## Implementation delivered

- seven separately versioned v1 generators with exact 50k, stable-ID affine
  permutation, dynamic 32-step traces, coupled 16k controls, stiff i2 and 100k
  report-only scale;
- sequential device state publication: final position/velocity becomes the
  next substep reference/velocity and is included in step time;
- neighbor rebuild at every substep boundary and frozen membership only inside
  that substep's nonlinear iterations;
- grid-bound and bounded-CSR overflow detection before an invalid result can
  be admitted;
- canonical active-membership hashes separate from order-sensitive logical
  CSR hashes;
- min/mean/p95/max degree, pair count, locality, capacity and raw per-stage
  timings for every trace step;
- a 256-execution unmeasured conditioning window before the contract's formal
  warmups.

The retained NP0 binary SHA-256 is
`99fc0736200482e8a868700c6401974f08268d9c0b05e8ee6f81ba7cf17a562b`.
The machine remained Linux x86-64 with NVIDIA GeForce RTX 3080, CUDA 13.3,
driver `610.43.02`, 320 W limit and compute capability 8.6. Windows was not
run and no cross-target claim is made.

## Compatibility and correctness

All existing CPU, CPU-gather and retained CUDA self-tests passed. The closed
v0 water-48k identity remained exact across the pre-NP0 and NP0 binaries:

| Field | SHA-256 |
|---|---|
| profile | `cb1868b86b4d9d40e996ebfa8e529982f73e647e358dd9f0a7c9269fce1211e8` |
| input | `1be1a98e586bae70a2ffa72eb56d4b79071e404a546616811eea1764c1e1ce51` |
| ordered output | `03671a4fccf72958ff772dfde7870ad24e8a8678c669f2a1286a178676203d1b` |
| logical/physical CSR | `e1d202f46e16212736c7e1d72a4309ad9fca46b12ac683b51bc6c43ccb0f8c11` |

The final v0 repeatability report SHA-256 is
`911d98bab710adb2fb99c28a8c88e217a10b3c04036b1c5465f099fc9ad28b88`.

Coherent and permuted 50k fixtures also passed f32 correspondence after
lattice-ID remap and produced the same canonical topology:
`f6229662a6d11189d2175390e1e1a2959472d53a4a63660ee6867f52c13546a9`.
Every admitted trace was finite, symmetric, within its fixed capacity and
inside the momentum bound. The water trace's first topology-preserving
interval was `0 -> 1`; its first membership-changing interval was `2 -> 3`.

## Frozen dynamic samples

The complete water-50k advected trace SHA-256 is
`333e487207ffab664e1519705953f8530af1f5bb180c7f30d72d3193607aae46`.
Selected records are:

| Step | Directed pairs | Logical CSR SHA-256 | Active-membership SHA-256 | Ordered output SHA-256 |
|---:|---:|---|---|---|
| 0 | 5,471,308 | `1b176356938914f036682127fbfb322595fb2b6112043bb3f44247c843990392` | `2fb0aee8d3f886164528f4a37e2504f811d660c5fe0bf4d9ba31bec775f022de` | `bd1c5ad3a38b85ca1b7adca2333331d39c033d8b9f326b8d416213a557259e7a` |
| 1 | 5,471,308 | `7a2f8afc7cdd2707db023cad779a000313f88815647d8103218911c77056a8db` | `2fb0aee8d3f886164528f4a37e2504f811d660c5fe0bf4d9ba31bec775f022de` | `fb0642b90b974459615bc32b7edd9b46b7f41a37e4b61f5bc9d7900a6fef9` |
| 15 | 4,723,822 | `925d92442c42ae1d16082a920f38a7f57159f2bdc48523537d5db1910eac558c` | `fdda83f2e08c01006df1eb95b5a320a3c386b490a8fad5834f258bd6f78e674a` | `a6890e38fca738718fb104f5e1d105f3f8c0cff0e622c5ecd006c11a7d860d23` |
| 31 | 4,603,650 | `0d15f80ad994e4c2f239fabd6ce6a6c2c8f26ec6137a014d3d1daaa5501f80e1` | `4e75e6c09c9859da2ecea6c456814c1d37b2e3cc9476157da301c129d9f3c286` | `a89f39b2f6a924933480baaa747ad6f97068c2b448d4cee0d67e62cce48b1dd8` |

The pair count falls `15.9%` across the trace, so this is materially different
from repeating one coherent input.

## Adjacent denominator

After conditioning, each adjacent report used 32 formal warmups and 96
measured executions in one process:

| Profile | p95 | p99 | Seed mean/p95 storage distance |
|---|---:|---:|---:|
| coherent water 50k | `4.926720 ms` | `4.967584 ms` | `2840.06 / 5199` |
| permuted water 50k | `9.568960 ms` | `10.464832 ms` | `17045 / 39247` |
| advected water 50k | `4.530048 ms` | `5.019872 ms` | `2830.70 / 5199` |
| viscous 16k | `7.058080 ms` | `7.274944 ms` | `910.484 / 1679` |
| surface 16k (`gamma=100`) | `9.916448 ms` | `10.361664 ms` | `910.484 / 1679` |

HP-1 is confirmed. Permuting only stable-ID storage leaves canonical physics
and topology unchanged but makes total p95 `1.94x` slower, neighbor p95
`2.60x` slower and pair-stage memory access roughly `2x` slower. Dynamic
locality P3 therefore remains admissible; coherent O4 remains its negative
control.

## Decision reproducibility

Two independent conditioned decision processes used 64 formal warmups and 512
measured executions each:

| Profile | Run A p95/p99 | Run B p95/p99 | p95 spread |
|---|---:|---:|---:|
| coherent | `4.849472 / 5.078048 ms` | `4.869248 / 5.043136 ms` | `0.41%` |
| advected | `4.470304 / 4.817280 ms` | `4.494592 / 4.793472 ms` | `0.54%` |

Before conditioning, duplicate campaigns varied by about `12%`. Those samples
are diagnostic only. The telemetry log showed no active thermal/hardware
slowdown during the retained campaign; its SHA-256 is
`73ea62ce44823627c194d65350e5b93ab084cfea2126e024c73e7939f5f02dbf`.

## Negative result: stiff dynamic surface

The initially specified `gamma=1000`, i20, 32-step surface workload was not a
bounded performance fixture. At step one it reached 2,623,472 directed pairs
and degree 222 at p95, exceeding the 192-neighbor capacity. The diagnostic
later saturated the protected pair capacity and correctly failed topology.

NP0 did not widen capacity or tolerance to admit it. The dynamic profile now
uses `gamma=100`, while the separate coherent `gamma=1000`, i2 fixture passes
the CPU term oracle, repeated exact output, bounded topology and momentum
gates. This preserves surface sensitivity without benchmarking an already
invalid trajectory.

## Raw artifact hashes

Raw JSON and telemetry remain outside Git as required by the standalone lab.

| Artifact | SHA-256 |
|---|---|
| adjacent coherent | `b71c9fd03d55d850da55860c224fb7e1112b9dea7474a0403767fc2fdc546b5f` |
| adjacent permuted | `ed7b32f0c0ee105eba52613d57304efec58260138c35e5cf1d4f85402eb177c6` |
| adjacent advected | `d115da495b3a5d31be81081041ea67c4e8f55dd56685ee449e393a2634ce72eb` |
| adjacent viscous | `3dc963539d7b5811b2157255c072a941ad71f4c3982b75c584de7ee037577214` |
| adjacent surface | `2a305d7d6209c12f5b357b24411c11df48d884608402163d39cbb5cc406f2929` |
| decision coherent A/B | `5f558c86499265b4ec99a06aac78ef16d3cc4af6aee4a859cc1b233f207bcce6` / `5daa496a7b10dddd23e7fab34f7777a3b0d23548b740687b1f2ab93894eda77e` |
| decision advected A/B | `c0c93a6833634e713d462df5ff6218e98b5a84da2ce29b52c51b35656a711993` / `f6da5f62af47180427f79c3612d85617728400356d2413906c0ff9a1dbf7d75d` |
| stiff i2 | `69a2fec96a6092fae144c20877a4342e1820d115214236ca52a8b17d728618d2` |
| 100k report | `49cbf70c3f3239dec14cf4f0118526f8b50e937abb9256209de81966c090e58e` |

## Consequence

NP1-P1 is authorized against the frozen hashes above. It must preserve the
retained implementation as rollback, pass tiny/stiff/full correspondence and
run an alternating adjacent tournament. The permuted result prioritizes
memory traversal, but P1 remains first because the pair terms dominate both
coherent and dynamic totals. No runtime, W2 or production authority changed.

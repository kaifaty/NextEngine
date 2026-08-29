# Nonlocal continuum NP1-P2 evidence — 2026-08-20

Status: `P2_RETAINED / EXACT_WORK / P3_NEXT / REPORT_ONLY`

## Decision

Retain `compact-csr-u16-p2` on top of P1 for every fixture with at most
`65,535` samples. It passed exact output, ordered logical CSR, dynamic handoff,
capacity and memory gates on all frozen v1 controls. The adjacent exact-50k
total p95 improved by `19.9%` coherent and `28.0%` advected; no tested profile
regressed. The 100k report fixture selected the required `u32` fallback and
matched retained P1 exactly.

This is adjacent evidence, not the final 512-sample decision campaign and not
W2/runtime authority. Although both exact-50k candidates are already below the
`4/6 ms` p95/p99 thresholds in this 96-round tournament, that observation
cannot close NP4 before P3/P4 and the frozen finalist campaign.

## Artifact identity

- branch: `codex/nonlocal-continuum-n0`
- executable SHA-256:
  `a34b9ebb815a462a00aa66984b34d2d5c5ed1de4124f366f144c0838be5aa9b0`
- CUDA architecture: `sm_86`
- launch geometry: retained `256` threads; no block-size tournament was needed
- compiler semantics: `-O3 --fmad=false --prec-div=true --prec-sqrt=true --ftz=false`
- denominator: retained P1 with 32-bit neighbor IDs
- candidate: identical P1 arithmetic with direct 16-bit neighbor construction
- conditioning/formal/measured rounds: `256 / 32 / 96`, adjacent alternating

## Correctness and capacity

All commands passed:

- CPU self-test and independent CPU gather self-test;
- retained CUDA gather/pointer-swap/O2 self-test;
- compact tiny coupled CPU `f64` oracle preflight;
- stiff `gamma=1000`, i2 exact output/CSR and memory preflight;
- exact seed and all 32 dynamic trace-step outputs, logical CSR and handoffs;
- finite state, symmetric topology and normalized momentum bounds;
- exact memory accounting and no shadow 32-bit neighbor allocation.

Representative exact hashes:

| Profile | Ordered output SHA-256 | Logical CSR SHA-256 |
|---|---|---|
| stiff surface 16k i2 | `52a3d852c05b9cc7931a3133816e0ddb1445695b88ea25193d0981022080b65e` | `a0304020eeb98889b47c0e179c85aaf4f93fb671bf58f15a99d3e034f1ad4698` |
| water 50k coherent | `6b377e876439cbc72f9b5e81cfabaed854188144c576b498a06e2769ceb4aae1` | `8e915c74c617f2f172f0da419b17fc5185078840459344c09b6d9818113c6fe7` |
| water 50k advected seed | `bd1c5ad3a38b85ca1b7adca2333331d39c033d8b9f326b8d416213a557259e7a` | `1b176356938914f036682127fbfb322595fb2b6112043bb3f44247c843990392` |
| water 100k fallback | `28321986991c41579548af9f4845e7941771a428318ec940dd312d3b6d45ef07` | retained and fallback identical |

The 100k candidate reports `selected_neighbor_id_bytes=4`,
`fallback_u32=true` and equal `68,168,714`-byte allocations. The eligible
profiles report two-byte IDs and exactly `2 * pair_capacity` bytes saved:

| Capacity family | P1 bytes | P2 bytes | Saved |
|---|---:|---:|---:|
| water 50k coherent/permuted | 34,091,706 | 21,791,706 | 12,300,000 |
| water 50k advected | 48,515,610 | 29,315,610 | 19,200,000 |
| viscous/surface 16k advected | 15,533,578 | 9,389,578 | 6,144,000 |

## Adjacent tournament

Times are same-process p95 milliseconds. Speedup is P1/P2.

| Profile | P1 total | P2 total | Total speedup | P1 pair | P2 pair | Pair speedup | P2 p99 |
|---|---:|---:|---:|---:|---:|---:|---:|
| water 50k coherent | 4.6142 | 3.8496 | 1.1986x | 2.5487 | 2.0879 | 1.2207x | 3.9403 |
| water 50k permuted | 7.9780 | 6.2837 | 1.2696x | 3.1643 | 2.2794 | 1.3882x | 6.6772 |
| water 50k advected | 3.9589 | 3.0927 | 1.2801x | 2.0521 | 1.6027 | 1.2804x | 3.1808 |
| viscous 16k advected | 5.7756 | 5.0435 | 1.1452x | 3.0519 | 2.6542 | 1.1498x | 5.3823 |
| surface 16k advected | 8.5660 | 7.8988 | 1.0845x | 4.2898 | 4.0626 | 1.0559x | 8.5618 |

Every profile passed its target and regression gates. Compact IDs alone
therefore satisfy the P2 retention rule. Compile-time block-size variants were
not run: the specification permits them only after a miss or occupancy
discriminator, neither of which occurred.

## Raw report hashes

| Report | SHA-256 |
|---|---|
| coherent tournament | `ccb6b262539602f2bddb82b15158290736071a3a54cf586303848ea4410fb7d2` |
| permuted tournament | `a0901d40b6f21eec2f281366c8ba2a938d4adac342d06116b5a1d5b36fccc8f8` |
| advected water tournament | `1989ec6ac3535823f2d1482985c7b3c628bc4976a6765dc0973e5e6517c54a0a` |
| advected viscous tournament | `57ccc55ca0b7e506fd347b612692463b77343f34e3b4c92a2e7d6231a02c319a` |
| advected surface tournament | `3e06b88d41ea92484b4ff0ca20f741bedce602c098b4ee1fb9580dd7ed5cfa2e` |
| stiff check | `c5e23e1cfcd58e97c687bfd0efb43942ec566a3f9313c2fd0d3bd236ad9ed27e` |
| 100k fallback check | `5d5464e204366584b93a15abe01eaed66833d375817f98824f112514d4269fad` |

Raw JSON and binaries remain outside Git under `/tmp`; hashes bind this
evidence without promoting transient benchmark artifacts into the repository.

## Consequence

P3 uses P1+P2 as its adjacent denominator. The large permuted improvement does
not erase HP-1: permuted total remains about twice advected, so dynamic
cell-local storage is still a valid bounded experiment. Coherent storage stays
the negative control, and sorting/remap cost must remain inside timing.

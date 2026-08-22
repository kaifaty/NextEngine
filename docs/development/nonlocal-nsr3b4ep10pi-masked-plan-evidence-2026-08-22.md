# NSR3-B4EP10PI masked plan implementation evidence -- 2026-08-22

Status: `FAIL / PERFORMANCE_GATE / ACTIVE_PLAN_RETAINED`

## Functional result

The opt-in masked implementation is correct:

- all three measured candidate processes exit zero with empty stderr and
  identical stdout SHA-256
  `cab21b95218b72b6a7da74c1b7ec22d05b04c86fe2801a7cf1a9dc99e45f2ca2`;
- candidate semantic result is
  `ecf6c0056decd60ab5f1f5ceabb6d311f9282b5d136a9da24542b65c65a9dac5`;
- physical/output roots and frozen work receipts remain exact;
- active plan builds fall from 226 to zero, with one fixed build and 225
  reuses;
- exact work is 966,239,080 full scans, 749,890,172 retained entries and
  30,509,036 bytes actual maximum added payload;
- no candidate failure or fallback occurs.

The final binary also preserves baseline B4EP10I worker-8 stdout, B4EP10PD
stdout and B4EP10R1 semantic result exactly.

## Performance failure

The candidate wins wall time in all three same-round pairs, remains stable and
reduces RSS, but misses the frozen 5% median paired-speedup gate:

| Metric | Baseline active plan | Masked fixed plan |
|---|---:|---:|
| wall ns, round 1 | 5,802,134,960 | 5,590,945,530 |
| wall ns, round 2 | 5,800,166,741 | 5,633,770,976 |
| wall ns, round 3 | 5,740,248,824 | 5,568,750,965 |
| median wall ns | 5,800,166,741 | 5,590,945,530 |
| median user s | 37.31 | 39.27 |
| median RSS KiB | 96,296 | 91,052 |

Same-round speedups are `1.0377734729960784x`,
`1.0295354152145784x` and `1.0307964676599612x`; median is only
`1.0307964676599612x`. Candidate range ratio is `1.0116758697612185`,
and median RSS decreases by 5,244 KiB.

This is useful negative evidence: removing all repeated plan builds saves
wall and memory, but the 28.85% full-row scan expansion raises median user CPU
from 37.31 to 39.27 seconds and leaves only about 3.1% paired wall benefit.
Masked superset reuse is therefore not the selected execution path.

## Reproducibility

- implementation commit:
  `b55b0dc7594057323d766b22b626ecde3e464e29`;
- executable: 4,057,408 bytes,
  `b383ab49f932a96861dbf68f16a01951971458c3a8188d44af0f96185c5adfc5`;
- Build ID: `6e5eba3d58f773d9ff15244b68a6a2859914ff79`;
- `compile_commands.json` SHA-256:
  `89e42920e6bf45fdfbdd7fdb997aac903e68d2631e34f5695188897f807efa00`;
- raw metrics SHA-256:
  `e11bd6a6fb0b797ae2cf5316129600c4eac0ee891bf0c9bb9c5ab30855aa2625`.

Source hashes:

- `boundary_reference.cpp`:
  `42712386c79b8cbc402ee03324fa1123679c04c423dce1df5a656f467692724d`;
- `boundary_reference.hpp`:
  `b52df6259699206a0015e5ef6ebfbc9c085632803618324a5c43af957dc122c2`;
- `formula_reclosure_main.cpp`:
  `97074adc33e9948cd5c4a9e3eddafa71338bcd2b68cc5476110320748f915e17`.

External artifacts remain outside Git under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10pi.ol29hdyx`.

## Evidence attestation

Exact projection, without final LF:

```text
nextengine.nonlocal.nsr3b4ep10pi-evidence|v1|identity=45dcdee2ce3b5aee7b6f324abc68d9de392c6493a33c4a9d577d9013ca5e197b|implementation=b55b0dc7594057323d766b22b626ecde3e464e29|binary=b383ab49f932a96861dbf68f16a01951971458c3a8188d44af0f96185c5adfc5|build-id=6e5eba3d58f773d9ff15244b68a6a2859914ff79|compile=89e42920e6bf45fdfbdd7fdb997aac903e68d2631e34f5695188897f807efa00|metrics=e11bd6a6fb0b797ae2cf5316129600c4eac0ee891bf0c9bb9c5ab30855aa2625|stdout=A:c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3,B:cab21b95218b72b6a7da74c1b7ec22d05b04c86fe2801a7cf1a9dc99e45f2ca2|result=B:ecf6c0056decd60ab5f1f5ceabb6d311f9282b5d136a9da24542b65c65a9dac5|wall-ns=A:5802134960,5800166741,5740248824;B:5590945530,5633770976,5568750965|paired-speedups=1.0377734729960784,1.0295354152145784,1.0307964676599612;median=1.0307964676599612|medians-wall-ns=A:5800166741,B:5590945530|candidate-range-ratio=1.0116758697612185|rss-kib=A:96296,B:91052,delta:-5244|user-s=A:37.31,B:39.27|gates=exact3of3;wins3of3;speed-fail;range-pass;rss-pass|work=active-builds0;fixed=1,225;scans=966239080;retained=749890172;payload=30509036|regressions=ed205b78a818fbcef6a1b2edfef644af2451f422d809f37fa25696c1eb9f316e,a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b|decision=reject-selected-fast-path;parallel-active-plan-research
```

SHA-256:
`15d3ceb70c6c33d6069bdec1cf1cf25e7d780e90450bf215844b9a13349eadaa`.

## Decision

Reject B4EP10PI as the selected fast path and retain B4EP10I's active plan.
Keep the opt-in command only as reproducible negative evidence. Route next to
deterministic parallel active-plan construction research; do not relax the
5% gate. B4E2, broad corpus, runtime, GPU, schema and production remain
blocked.

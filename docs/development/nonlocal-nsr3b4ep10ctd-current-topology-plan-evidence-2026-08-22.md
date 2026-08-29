# NSR3-B4EP10CTD current-topology reverse-plan evidence -- 2026-08-22

Status: `FAIL_SCAN / FULL_CURRENT_PLAN_REJECTED`

## Result

The full current-topology reverse plan is structurally exact but misses the
frozen scan gate:

- all 226 active plans are exact stable compression-filtered subsequences;
- 226 evaluation and 459 HVP projections retain exactly 749,890,172 selected
  active entries;
- zero order, coverage or fallback mismatch;
- invalid source and target fixtures both reject;
- combined payload is 34,802,608 bytes, below 64 MiB;
- projected full scans are 909,872,452 entries, ratio
  `1.2133409477461454x` versus the frozen `1.20x` maximum.

Both fresh processes deterministically exit 1 with byte-identical stdout
SHA-256
`fbfc07a9f0ee7fb3b3cb495e4b5c76d73c8967f5ba84aa70c4c75c1ffd0c8663`
and empty stderr. Physical roots and work remain exact; this is a deliberate
structural-gate failure, not a solver failure.

## Scan decomposition

| Path | Full entries | Retained entries |
|---|---:|---:|
| evaluation | 301,691,992 | 263,974,460 |
| HVP | 608,180,460 | 485,915,712 |
| total | 909,872,452 | 749,890,172 |

Every directed value appears twice in a full target plan: once in its source
row and once in its participant row. The source-row half already exists as
the canonical flat adjacency. A split `self + incoming` representation would
need only 454,936,226 full incoming entries plus 374,945,086 retained own-row
entries, for a derived 829,881,312 visits and `1.1066704738730726x` ratio.
This is a work projection from exact counts, not implementation or timing
evidence.

## Build and regressions

- implementation commit:
  `88adf275d9b37de870c15dad5ab4583e76c0d607`;
- executable: 4,143,336 bytes,
  `4324f60e0a6870524829c505330b6499a1c9993c91f1e3e82e28ddd6834d582a`;
- Build ID: `8ef4003efa97b77149d2888bc5dd97575ab89a63`;
- `compile_commands.json` SHA-256:
  `89e42920e6bf45fdfbdd7fdb997aac903e68d2631e34f5695188897f807efa00`.

Source hashes:

- `CMakeLists.txt`:
  `48301abd9ba4c4005c1b0d15714a2d4a2ecc07e20dffbc58707535d0bb99625c`;
- `boundary_reference.cpp`:
  `c16e2bb6e8efceb36a98aef7abac34782b8fa923d5f48084db8feb03976eb591`;
- `boundary_reference.hpp`:
  `5aeea2c255596458c0e9e40a0a59082402d11840fa37aead0664df0eeef3f201`;
- `formula_reclosure_main.cpp`:
  `1f81711a69199bbd343b45caf8d99ae317f4a390ea481302f762a273a8b57d2c`.

The final binary preserves exact B4EP10I worker-8 stdout
`c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3`,
B4EP10PCI stdout
`e4559fcd5762cbf53a8ed5cdf57899dd2df1c364d0287525bcf55bef3eac8062`
and B4EP10R1 semantic result
`a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b`.

External artifacts remain outside Git under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10ctd.QOdSIe`.

## Evidence attestation

Exact projection, without final LF:

```text
nextengine.nonlocal.nsr3b4ep10ctd-evidence|v1|identity=d5d457d09b116496185e8eb11f97f80e27959757b22e6de5b674b12437ca7259|implementation=88adf275d9b37de870c15dad5ab4583e76c0d607|binary=4324f60e0a6870524829c505330b6499a1c9993c91f1e3e82e28ddd6834d582a|build-id=8ef4003efa97b77149d2888bc5dd97575ab89a63|compile=89e42920e6bf45fdfbdd7fdb997aac903e68d2631e34f5695188897f807efa00|stdout=fbfc07a9f0ee7fb3b3cb495e4b5c76d73c8967f5ba84aa70c4c75c1ffd0c8663|result=83c7bd52d183616af8be6f950bca1a822391cb1feb3491595a07ceb277f05d17|correspondence=e6abe4bffc499c00cdd5ade4f5712ca50162bb0ee6d07b155df46a70a036d7ca|runs=2;exit=1,1;stderr-empty|audit=plans226;eval226;hvp459|entries=eval:301691992,263974460;hvp:608180460,485915712;total:909872452,749890172|scan-ratio=1.2133409477461454;limit=1.20;fail|payload=8090048,34802608|mismatches=0,0,0|negatives=1,1|regressions=c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3,e4559fcd5762cbf53a8ed5cdf57899dd2df1c364d0287525bcf55bef3eac8062,a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b|decision=reject-full-current-plan;split-self-incoming-research
```

SHA-256:
`64e0898ac28ffa5061e1e743064f707ff0b970a562bc0c44163d3d722d0c85bb`.

## Decision

Reject the full current-topology plan and do not authorize its construction
implementation. Preserve the exact stable-subsequence result and research the
split self/incoming representation next. B4E2, broad corpus, runtime, GPU,
schema and production remain blocked.

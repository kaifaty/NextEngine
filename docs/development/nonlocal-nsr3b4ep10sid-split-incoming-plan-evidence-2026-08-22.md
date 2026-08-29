# NSR3-B4EP10SID split incoming plan evidence -- 2026-08-22

Status: `PASS / SPLIT_INCOMING_PLAN_CANDIDATE / B4EP10SIC_NEXT`

## Result

The split self/incoming representation reconstructs every selected active
target row exactly:

- 226/226 evaluation plans pass the three-part lower-incoming, own-row,
  upper-incoming sequence comparison;
- 459 HVP projections retain the same tape-owned active set;
- zero order, coverage or fallback mismatch;
- shifted own boundary and invalid incoming target both reject;
- all physical roots and frozen work remain exact.

Two fresh processes exit zero with byte-identical stdout SHA-256
`2a7f044d402f1f7d7b0a4fa5a2da288101dc21f257ab76d0f549482bac64548a`
and empty stderr. Correspondence SHA-256 is
`34a15ad1d08d86f736328b4b701b1518315396eec1cbd49e64c9278c14c36bbf`.

## Work and capacity

| Component | Entries |
|---|---:|
| full incoming | 454,936,226 |
| retained incoming | 374,945,086 |
| retained own row | 374,945,086 |
| projected visits | 829,881,312 |
| selected active baseline | 749,890,172 |

The exact projected visit ratio is `1.1066704738730726x`, below the frozen
`1.15x` gate and materially below B4EP10CTD's rejected `1.213341x` full-plan
ratio. Maximum incoming-plan payload is 5,409,132 bytes; conservative combined
payload is 32,121,692 bytes, below 64 MiB.

This proves slot order, work relations and nominal capacity. The audit still
builds a full plan beside the selected path, so its process duration is not
evidence. It does not yet prove an efficient topology-compaction builder or
floating-path speedup.

## Build and regressions

- implementation commit:
  `d5aa920a237618e2e9b0c2e47ebedd4623198f43`;
- executable: 4,160,168 bytes,
  `8459324a7290ba77f4f1277a1709f037f892fffbd9a36f84a473c4c31d0584b2`;
- Build ID: `db421d3275b9bfa3541f0cbee66dd8b425ccd7dd`;
- `compile_commands.json` SHA-256:
  `89e42920e6bf45fdfbdd7fdb997aac903e68d2631e34f5695188897f807efa00`.

Source hashes:

- `CMakeLists.txt`:
  `48301abd9ba4c4005c1b0d15714a2d4a2ecc07e20dffbc58707535d0bb99625c`;
- `boundary_reference.cpp`:
  `5e291061be2e8163a51d497bf6558845be11bad834b6586c67234f873505f850`;
- `boundary_reference.hpp`:
  `14021a31419e8e2566ac1bd5c2906b6ca9f17fc718bfc71876ef98796038030f`;
- `formula_reclosure_main.cpp`:
  `00e67d2255188c07f6db62562c63a1989d026471465ef82ff191a09f5b15b3cb`.

The final binary preserves exact B4EP10I worker-8 stdout
`c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3`,
B4EP10CTD expected-failure stdout
`fbfc07a9f0ee7fb3b3cb495e4b5c76d73c8967f5ba84aa70c4c75c1ffd0c8663`
and B4EP10R1 semantic result
`a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b`.

External artifacts remain outside Git under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10sid.tF6K42`.

## Evidence attestation

Exact projection, without final LF:

```text
nextengine.nonlocal.nsr3b4ep10sid-evidence|v1|identity=5b730b2e84676f8b47aa6b16ca2e519ae8b7dff43e935a1fc2ba642bed976b4c|implementation=d5aa920a237618e2e9b0c2e47ebedd4623198f43|binary=8459324a7290ba77f4f1277a1709f037f892fffbd9a36f84a473c4c31d0584b2|build-id=db421d3275b9bfa3541f0cbee66dd8b425ccd7dd|compile=89e42920e6bf45fdfbdd7fdb997aac903e68d2631e34f5695188897f807efa00|stdout=2a7f044d402f1f7d7b0a4fa5a2da288101dc21f257ab76d0f549482bac64548a|result=f1f5fbdce0e8c1e34cac7cf049c4d82628075ffa09a23e6af4b1ac95406d0d1b|correspondence=34a15ad1d08d86f736328b4b701b1518315396eec1cbd49e64c9278c14c36bbf|runs=2;exit=0,0;stderr-empty|audit=plans226;eval226;hvp459|entries=incoming-full454936226;incoming-retained374945086;own-retained374945086;visits829881312;baseline749890172|scan-ratio=1.1066704738730726;limit=1.15;pass|payload=5409132,32121692|mismatches=0,0,0|negatives=1,1|regressions=c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3,fbfc07a9f0ee7fb3b3cb495e4b5c76d73c8967f5ba84aa70c4c75c1ffd0c8663,a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b|decision=b4ep10sic-construction-research
```

SHA-256:
`a1fbe1e255ca3b95c97c4b446c0f750448faf82b1bb9a461f2acfd35882188c0`.

## Decision

Retain `SPLIT_INCOMING_PLAN_CANDIDATE`. B4EP10SIC may research how to emit
the incoming view during current topology compaction. No floating integration,
timing, B4E2, broad corpus, runtime, GPU, schema or production is authorized.

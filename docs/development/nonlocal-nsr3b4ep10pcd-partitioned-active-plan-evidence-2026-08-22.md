# NSR3-B4EP10PCD partitioned active-plan evidence -- 2026-08-22

Status: `PASS / PARTITIONED_ACTIVE_PLAN_CANDIDATE / B4EP10PCI_NEXT`

## Result

The stable partitioned counting-sort/CSR transpose is structurally exact for
the complete nominal transaction:

- all 226/226 candidate plans match the serial active plans;
- 150,845,996 directed slots contain 131,987,230 active directed slots and
  produce 263,974,460 target records;
- `source_by_slot`, `target_offsets`, `target_slots` and payload bytes match
  exactly, with zero order, coverage, plan or fallback mismatch;
- five isolated phases per plan add exactly 1,130 executor regions and 72,320
  logical partitions;
- corrupt local-count and partition-base fixtures both reject before an exact
  candidate can be reported.

The unchanged returned transaction preserves every physical/output root and
produces correspondence SHA-256
`82ce97a52b8f4fa9ccccb8e4e2452dd09238c1a8e52ca4dc53139e2b6956d92a`.
Two fresh pinned processes emit byte-identical stdout SHA-256
`a389e6491d3d13361f8ddf1a907eb0f8336623e991f14f8beebfc01676cfed42`
with empty stderr and semantic result
`ab1f608bb3574f3de50f0c6815cb6b1a81e787453edf61d48644d97c981801eb`.

## Work and capacity

| Item | Observed |
|---|---:|
| candidate plan builds | 226 |
| additional regions | 1,130 |
| additional logical partitions | 72,320 |
| total executor regions | 4,541 |
| total logical partitions | 290,624 |
| partition count/cursor matrix | 3,026,944 bytes |
| maximum candidate plan | 7,539,968 bytes |
| conservative combined payload | 37,279,472 bytes |

The conservative total remains below 67,108,864 bytes. This audit constructs
the candidate beside the serial plan and performs negatives; its process time
is deliberately inadmissible as a speed result. It proves correspondence and
bounded storage only.

## Build and regressions

- implementation commit:
  `74364cc514c7c21133b977d4d380a1d0e50a7c51`;
- executable: 4,101,856 bytes,
  `5d09b470e0492a3e71cc5cbf10e810c2b1ad4a0e2508ca0e13514d3cce15d400`;
- Build ID: `f08b67421dc82df3993e3e2c5613fcaf711f24cd`;
- `compile_commands.json` SHA-256:
  `89e42920e6bf45fdfbdd7fdb997aac903e68d2631e34f5695188897f807efa00`.

Source hashes:

- `CMakeLists.txt`:
  `48301abd9ba4c4005c1b0d15714a2d4a2ecc07e20dffbc58707535d0bb99625c`;
- `boundary_reference.cpp`:
  `64658dcd18cda2e266f99f9fe2e8a3076d093018908a4704441265c706032633`;
- `boundary_reference.hpp`:
  `4f7db69ede377352ad4fb68acbd3cbc20e908d497ab446a16b4f0c4644b71d6f`;
- `formula_reclosure_main.cpp`:
  `7d097a10dcc144ac14866b9a424573dd67c18ff3a88d95b9b0cadd8ae78104bc`.

The final binary preserves exact B4EP10I worker-8 stdout
`c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3`,
B4EP10PD stdout
`ed205b78a818fbcef6a1b2edfef644af2451f422d809f37fa25696c1eb9f316e`
and B4EP10R1 semantic result
`a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b`.

External artifacts remain outside Git under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10pcd.KJ074m`.

## Evidence attestation

Exact projection, without final LF:

```text
nextengine.nonlocal.nsr3b4ep10pcd-evidence|v1|identity=c6d14dd53d1669d2f370e057288567a284e7fde762a787f09d705ac0e45f5c06|implementation=74364cc514c7c21133b977d4d380a1d0e50a7c51|binary=5d09b470e0492a3e71cc5cbf10e810c2b1ad4a0e2508ca0e13514d3cce15d400|build-id=f08b67421dc82df3993e3e2c5613fcaf711f24cd|compile=89e42920e6bf45fdfbdd7fdb997aac903e68d2631e34f5695188897f807efa00|stdout=a389e6491d3d13361f8ddf1a907eb0f8336623e991f14f8beebfc01676cfed42|result=ab1f608bb3574f3de50f0c6815cb6b1a81e787453edf61d48644d97c981801eb|correspondence=82ce97a52b8f4fa9ccccb8e4e2452dd09238c1a8e52ca4dc53139e2b6956d92a|plans=226;directed=150845996;active=131987230;target=263974460|executor=1130,72320;total=4541,290624|payload=3026944,7539968,37279472|mismatches=0,0,0,0|negatives=1,1|regressions=c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3,ed205b78a818fbcef6a1b2edfef644af2451f422d809f37fa25696c1eb9f316e,a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b|decision=b4ep10pci-contract-research
```

SHA-256:
`34eecfa9116fc4de40098ecbec65077ef02ec95395da7c4eeb614139395efe6c`.

## Decision

Retain `PARTITIONED_ACTIVE_PLAN_CANDIDATE`. B4EP10PCI may now research and
freeze an opt-in candidate-path/A-B contract. No speedup is claimed; B4E2,
broad corpus, runtime, GPU, schema and production remain blocked.

# NSR3-B4EP10SICD incoming construction evidence -- 2026-08-22

Status: `PASS / INCOMING_CONSTRUCTION_CANDIDATE / B4EP10SII_NEXT`

## Result

The pair-endpoint builder reproduces B4EP10SID's incoming plan exactly for all
226 current topologies:

- exact `source_by_slot`, incoming target offsets and incoming target slots;
- 150,845,996 directed slots and exactly as many incoming entries;
- zero order, coverage or fallback mismatch;
- wrong pair endpoint and support cursor both reject;
- all physical roots and selected work remain exact.

Two fresh processes exit zero with byte-identical stdout SHA-256
`bf4164f6d293c69c5d9774b96d3ece10275b273a95d29c07d56c38fa166f6c5c`
and empty stderr. Correspondence SHA-256 is
`995283db5a74283459eadbbbbdf548074db48e7aa6d4da1e5d3e95b701275200`.

## Construction work and capacity

| Item | Aggregate |
|---|---:|
| current pair visits | 171,432,300 |
| support CSR records | 20,586,304 |
| endpoint writes | 150,845,996 |
| target-count visits | 171,432,300 |
| target-fill visits | 171,432,300 |
| added parallel regions | 678 |
| added logical partitions | 43,392 |

Maximum incoming plan is 5,409,132 bytes, construction scratch is 3,501,988
bytes and conservative combined payload with the selected owner path is
35,623,680 bytes, below 64 MiB.

The audit builder uses three isolated regions per topology. A later integrated
candidate may fuse source/endpoint mapping into existing topology row fill and
support CSR counting into current pair metadata; those savings are not yet
implemented or credited. Audit duration is not performance evidence.

## Build and regressions

- implementation commit:
  `41d90e89ab808e853ee4486ae25bcfdfc8363545`;
- executable: 4,203,960 bytes,
  `787b0c38dce5bcea337b217863a1d67a388d39f25d53ec6d2a527548a92c7fc7`;
- Build ID: `9370636b82caf0542ba943625c84c56075442c50`;
- `compile_commands.json` SHA-256:
  `89e42920e6bf45fdfbdd7fdb997aac903e68d2631e34f5695188897f807efa00`.

Source hashes:

- `CMakeLists.txt`:
  `48301abd9ba4c4005c1b0d15714a2d4a2ecc07e20dffbc58707535d0bb99625c`;
- `boundary_reference.cpp`:
  `38e40490b93a8f74f63501c8b39715207d0e13b15f8084f8521d6074c726da62`;
- `boundary_reference.hpp`:
  `85da483defd256557d864afe8d65b040d0ca31ea510571f816658102906e31df`;
- `formula_reclosure_main.cpp`:
  `2dd7c5fd95280a98bc6673ea411487fbd2ec760aa232c878c3e7f86dd5054be3`.

The final binary preserves exact B4EP10I worker-8 stdout
`c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3`,
B4EP10SID stdout
`2a7f044d402f1f7d7b0a4fa5a2da288101dc21f257ab76d0f549482bac64548a`
and B4EP10R1 semantic result
`a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b`.

External artifacts remain outside Git under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10sicd.8e7M7a`.

## Evidence attestation

Exact projection, without final LF:

```text
nextengine.nonlocal.nsr3b4ep10sicd-evidence|v1|identity=64dba1abc15451b19874df204411a88f34c99f60ca6f7ec296663708bc853267|implementation=41d90e89ab808e853ee4486ae25bcfdfc8363545|binary=787b0c38dce5bcea337b217863a1d67a388d39f25d53ec6d2a527548a92c7fc7|build-id=9370636b82caf0542ba943625c84c56075442c50|compile=89e42920e6bf45fdfbdd7fdb997aac903e68d2631e34f5695188897f807efa00|stdout=bf4164f6d293c69c5d9774b96d3ece10275b273a95d29c07d56c38fa166f6c5c|result=4c9f2d9e5d9eaae41146b9a2548fd4334fea83e58d4d9ce0470d1c7ea33c9fbb|correspondence=995283db5a74283459eadbbbbdf548074db48e7aa6d4da1e5d3e95b701275200|runs=2;exit=0,0;stderr-empty|audit=plans226;directed150845996;incoming150845996|work=pair-visits171432300;support20586304;endpoints150845996;target-count171432300;target-fill171432300|executor=678,43392|payload=5409132,3501988,35623680|mismatches=0,0,0|negatives=1,1|regressions=c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3,2a7f044d402f1f7d7b0a4fa5a2da288101dc21f257ab76d0f549482bac64548a,a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b|decision=b4ep10sii-integration-research
```

SHA-256:
`0062f28f10e1737b48016df723f8a44124cac25f9740236c03058a7941d515d2`.

## Decision

Retain `INCOMING_CONSTRUCTION_CANDIDATE`. B4EP10SII may research and freeze a
single opt-in floating integration/A-B contract. No speedup is claimed; B4E2,
broad corpus, runtime, GPU, schema and production remain blocked.

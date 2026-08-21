# NSR3-B4EP1 -- query-evidence separation contract

Status: `FROZEN / IMPLEMENTATION_AND_AB_AUTHORIZED / RESEARCH_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep1-query-evidence-separation|v1|parent=bf65f79d98c6c9802da5a853d57f77b5cdf8d8fac175da9c9703efde7e667b60:0b40131ee6820e75d3656aae744bc3e4f0a483758a7f0e127d12b1412df5ba5d|oracle=0cdc26e1d0b406fecc39c64cebfee080be32804b993e6d433fa0a74658efb4cc:54d42af619dbd48ae400ce4656ba155a8726dbd07b00754ecfc137ee864c111d:b9601aaad292c43201a5ab054192de4131478eb27e568e1eb78e6b613b07eecc|policy=preflight-full;transaction-work-only;workspace-state-hash=skip;work-chain=kind+counts|physics=bit-exact-roots+counters;full-policy-default-unchanged|runs=candidate2-byte-exact;timing=3-balanced-pairs;gate=3/3-wins;median-speedup>=1.10|watchdog=900s|reference=closed|credit=b4ep2-design-only
```

Identity SHA-256:
`470a0f4ec9b51ac57f1ecb69e937bb9d2756db41299a0dc01fa427ae63a78e99`.

## Implementation boundary

Add internal `FULL_STATE`/`WORK_ONLY` policy and computed/skipped counters to
`JointQueryTrace`. `FULL_STATE` remains the default and its chain bytes cannot
change. Under `WORK_ONLY`, `build_joint_query_workspace` must:

1. execute the same neighborhood/evaluation/tape path;
2. skip only `joint_workspace_hash` and its nested pair/tape hashes;
3. leave `state_sha256` empty and increment one skipped counter;
4. update a domain-separated work chain from query kind and existing numeric
   work facts, with no position/pair/tape serialization;
5. preserve lifecycle, work-reduced and all-pairs counters.

Add only the research command:

```text
nonlocal-formula-reclosure --nominal-hydro-query-evidence-ablation
```

It reuses exact B4E1M parent preflight with `FULL_STATE`, then calls the same
retained-flat macro transaction once with `WORK_ONLY`. Do not change formulas,
KKT/refinement policy, capacities, publication or final hashes.

## Exact candidate gate

Require all B4E1M physical/work gates and these frozen facts:

- selected level 1; initial/accepted/attempted/discarded `14/28/42/14`;
- outer/rejected trials `221/0`; nonlinear/spectral HVPs `411/48`;
- frame root
  `eaa6fe3aea4567594d82ce9fb76be16a4a99256e45b6ee10c4bc4c6417311eb5`
  and aggregate root
  `8a634d69ec2bdb692d08e2c1cdbbf4fd8ce5a4cc39289a7c05a9acd8f0ca90dc`;
- trajectory root
  `4689e74310815f413212d006d02347d598d38ce70b0ca44eee2253cddb1f6fc4`,
  legacy ledger
  `df58c67ec644ad717cf6d2192dd31743163bf65438289feedba1e93457f0c057`
  and policy ledger
  `b8502f70738b39cad693e515ba1546a3f5ae2e1dae432d597cc4ae7b0e0c0444`;
- output strain exactly `0.00045547995081940407`, zero energy creation and
  zero private/decoded penetration;
- parent full hashes computed/skipped `1/0`;
- transaction full hashes computed/skipped `0/226`;
- 227 total flat workspace builds including parent, zero nested/all-pairs work,
  balanced 42 retained transfers/reads/releases and zero final ownership.

Run the candidate twice in fresh processes and require byte-identical JSON.
Run the unchanged B4E1M command once after implementation and require its exact
6,151-byte stdout SHA
`b9601aaad292c43201a5ab054192de4131478eb27e568e1eb78e6b613b07eecc`.

## Timing gate

From the same Release binary, run six fresh processes in order:

```text
FULL, WORK_ONLY, WORK_ONLY, FULL, FULL, WORK_ONLY
```

Each has the existing 900-second watchdog. Capture `/usr/bin/time -v`
externally; clock/RSS never enter deterministic reports. Pair positions
`(1,2)`, `(4,3)` and `(5,6)` so order alternates. Require WORK_ONLY to win all
three pairs and median `FULL/WORK_ONLY >= 1.10`.

A correspondence failure rejects the candidate regardless of speed. A speed
gate failure preserves the exact negative and removes no full-state evidence.
PASS selects only `WORK_ONLY_NOMINAL_RESEARCH_CANDIDATE` and authorizes B4EP2
residual profiling/design. B4E2, references, runtime/CUDA and production work
remain blocked.

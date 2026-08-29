# NSR3-B4EP10PD masked superset plan evidence -- 2026-08-22

Status: `PASS / MASKED_SUPERSET_PLAN_CANDIDATE / B4EP10PI_NEXT`

## Result

The fixed masked plan is structurally exact for the complete nominal
transaction:

- one fixed superset plan build and 225 certified reuses;
- 226/226 active plan audits pass;
- 705,284 fixed directed slots and 1,410,568 fixed target entries;
- 318,788,368 fixed entries inspected by the structural audits;
- all 263,974,460 retained evaluation entries reproduce current target rows
  in exact source/current-slot order;
- zero order, coverage, mapping or fallback mismatch;
- corrupt current-slot mapping and fixed target entry both reject.

The unchanged returned transaction retains common B4EP10I correspondence
SHA-256
`917a04d31bb849a9bee5dd190ad6d15e07c9c90a9c2822130ae1adac6ebcb4ca`
and all physical/output roots. Two fresh processes emit byte-identical stdout
SHA-256
`ed205b78a818fbcef6a1b2edfef644af2451f422d809f37fa25696c1eb9f316e`
with empty stderr.

## Scan and capacity discriminator

A masked implementation would inspect 966,239,080 fixed target entries over
226 evaluation and 459 HVP gathers, versus 749,890,172 current active-plan
entries. Its exact scan expansion is `1.2885074589295991x`, below the frozen
`1.35x` gate.

The conservative capacity sum is:

| Component | Bytes |
|---|---:|
| fixed target plan | 8,510,708 |
| maximum current-slot mapping | 2,821,136 |
| existing owner plan/scratch peak | 26,712,560 |
| conservative combined total | 38,044,404 |

The total is below the 67,108,864-byte nominal gate. It deliberately double
counts the current active plan that a later implementation intends to replace;
the audit therefore does not understate peak memory.

This proves order, scan and nominal capacity admissibility. It does not prove
that the added mask checks and 28.85% extra target reads are faster than active
plan rebuilds.

## Build and regressions

- implementation commit:
  `a53b411b2b96f8b1f12555f022f27a8524400b6f`;
- executable: 4,034,328 bytes,
  `8bccb6746080e81008533acb7cddcaf0edc926be9139e048049aa1567a84860d`;
- Build ID: `09990af30968de020dbc086f1fa0544c64d87cc4`;
- `compile_commands.json` SHA-256:
  `89e42920e6bf45fdfbdd7fdb997aac903e68d2631e34f5695188897f807efa00`.

Source hashes:

- `boundary_reference.cpp`:
  `c7fe373fb75fe56039d2cc02fd253478a7d3357da48b30d19ebee9ba3578e980`;
- `boundary_reference.hpp`:
  `d7bf104c35731efa3ba58184b5c004327c1b3117a0a63d59339bf063dee65794`;
- `formula_reclosure_main.cpp`:
  `484302ebc660241b0b4a427c9a3ed1d9a847be16b4aa2e0a79b59cd4d0e054cf`.

The final binary preserves exact B4EP10I worker-8 stdout
`c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3`
and B4EP10R1 semantic result
`a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b`.

External artifacts remain outside Git under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10pd.M15i7k`.

## Evidence attestation

Exact projection, without final LF:

```text
nextengine.nonlocal.nsr3b4ep10pd-evidence|v1|identity=82be83e5131ae5eb3c49a687a764c851f79a81c107417922fd7285e35a986ce1|implementation=a53b411b2b96f8b1f12555f022f27a8524400b6f|binary=8bccb6746080e81008533acb7cddcaf0edc926be9139e048049aa1567a84860d|build-id=09990af30968de020dbc086f1fa0544c64d87cc4|compile=89e42920e6bf45fdfbdd7fdb997aac903e68d2631e34f5695188897f807efa00|stdout=ed205b78a818fbcef6a1b2edfef644af2451f422d809f37fa25696c1eb9f316e|result=bd6a3bb2f2e8c99dbfa78cfbeafbc77b29078650fdf60e179e9807b744a46a3d|correspondence=917a04d31bb849a9bee5dd190ad6d15e07c9c90a9c2822130ae1adac6ebcb4ca|cache=226,1,225|plan=1,225,226;slots=705284;entries=1410568;audit-scans=318788368;retained=263974460|projection=685;966239080;749890172;ratio=1.2885074589295991|payload=8510708,2821136,26712560,38044404|mismatches=0,0,0,0|negatives=1,1|regressions=c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3,a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b|decision=b4ep10pi-contract-research
```

SHA-256:
`c9b664a3f57864d8916f6db40f233e2019e0c83c9ef80e089fa6212d14b078ce`.

## Decision

Retain `MASKED_SUPERSET_PLAN_CANDIDATE`. B4EP10PI may now research and freeze
an opt-in implementation/A-B contract. No speedup is claimed; B4E2, broad
corpus, runtime, GPU, schema and production remain blocked.

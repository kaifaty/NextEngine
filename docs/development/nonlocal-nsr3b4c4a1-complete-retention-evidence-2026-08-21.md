# NSR3-B4C4A1 complete-lane retained-workspace evidence

Status: `PASS / STATIC-SUPPORT-INDEX DESIGN AUTHORIZED`

Date: `2026-08-21`

## Reproducible result

Full command:

```text
nonlocal-formula-reclosure --complete-retention-self-test
```

Two parent-gated reports executed concurrently and are byte-identical:

```text
status                 PASS / COMPLETE_RESEARCH_LANES_ONLY
raw JSON + LF          7d562356feec5ded444be6cf0b2943c0047a05221e50d420f2e7e172f1f094db
raw JSON without LF    243989062c55bccfbbf59aaa9119645c305cd5b2b233fb37bb7b3db8ce23cf39
semantic result        79847936f886b938b71adfb0b15780a465c31acd89ccd6e564204dc28d3c4e93
wall time              179.85 s / 179.84 s
CPU utilization        244% / 242%
maximum RSS            15,444 KiB / 16,148 KiB
parent B4C4A exact      true
```

The isolated probe also passed twice byte-identically at raw-with-LF
`436f7496bc9066a355731171a8aba526daa313a2110eda83b157f62ca2b7516a`,
raw-without-LF
`2c21f0a5a35e0965d9173c0beaaf20bb6ee8662223bd89bf65f06846817f5fdd`
and the same semantic result. Rebuilt regression probes preserve the exact
B4C4A hashes `15a91fac...ce9` / `7da604bc...e2b5` and B4C3MAR hashes
`60e44aaf...554` / `9e186b1e...4ff` with/without the final LF.

## Exact complete-lane work result

| Lane | Legacy builds | Candidate builds | Removed | Reduction |
|---|---:|---:|---:|---:|
| P1 adaptive, 8 frames | `2,368` | `1,924` | `444` | `18.75%` |
| P1 fixed 48 | `1,941` | `1,557` | `384` | `19.78%` |
| P1 fixed 96 | `3,705` | `2,937` | `768` | `20.73%` |
| P1 fixed 192 | `6,167` | `4,631` | `1,536` | `24.91%` |
| P2 adaptive, 16 frames | `447` | `323` | `124` | `27.74%` |
| P2 fixed 48 | `1,668` | `900` | `768` | `46.04%` |
| P2 fixed 96 | `3,287` | `1,751` | `1,536` | `46.73%` |
| P2 fixed 192 | `6,521` | `3,449` | `3,072` | `47.11%` |

Every removed build has exactly one transfer, retained diagnostic read,
release and ordered receipt. Maximum retained ownership is one, maximum total
live workspaces is two and every lane ends with zero live workspaces. The
result applies no wall-time threshold: it proves exact work elimination and
ownership composition, not a product performance claim.

## Physical and durable correspondence

Legacy, candidate and repeated candidate lanes match bit-exactly for adaptive
schedules, attempts, accepted/discarded work, fixed-lane runs, private
physical diagnostics, contact/onset state, aggregate budgets, published
position/velocity, canonical frame roots, ledgers and trajectories.

The selected adaptive roots remain:

| Case | Trajectory | Legacy ledger | Policy ledger |
|---|---|---|---|
| P1 | `852759dc...0d5` | `78bda3a7...74ad` | `bab793e5...b23` |
| P2 | `de782684...883` | `9273756e...a1c` | `d03a5985...3cd` |

The candidate query roots intentionally differ from the legacy transcript
because equal-state diagnostic rebuilds no longer occur. Candidate query and
aggregate receipt roots repeat exactly and remain bound to the preserved
physical state.

## Failure composition

Forced adaptive prepublication rollback performs `63` transfers, reads and
releases, preserves the committed prefix and roots, publishes no failed
transaction and ends with zero retained and total live workspaces.

## Decision

Select `COMPLETE_LANE_RETAINED_ACCEPTED_WORKSPACE_CANDIDATE`. Authorize only a
separately frozen B4C4B immutable static-support-index design. B4C4C flat-only
CSR, B4D, nominal corpus, CUDA, runtime and production remain blocked.

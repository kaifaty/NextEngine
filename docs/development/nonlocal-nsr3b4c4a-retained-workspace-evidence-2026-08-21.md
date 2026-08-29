# NSR3-B4C4A retained accepted-workspace evidence

Status: `PASS / COMPLETE-LANE DESIGN AUTHORIZED`

Date: `2026-08-21`

## Reproducible result

Full command:

```text
nonlocal-formula-reclosure --retained-workspace-self-test
```

Two parent-gated reports executed concurrently and are byte-identical:

```text
status                 PASS / ONE_MACRO_PRIVATE_OWNERSHIP_ONLY
raw JSON + LF          fdd2050a7401305b64b67035b71038618322bf2e244f34b078d07c6e7414c136
raw JSON without LF    452245c1e1a8631541db57943c8aab441bfdbaf203eac141986820dba5899166
semantic result        a1d35119d7620113a98c4a77359ac282a8a7a0ca7142f3b883150a30d8fbf3fc
wall time              115.79 s / 116.51 s
CPU utilization        304% / 306%
maximum RSS            16,400 KiB / 16,660 KiB
parent B4C4M0 exact    true
```

The isolated probe passes twice byte-identically in `1.54 s` at raw-with-LF
`15a91faca2fa5c1daa1c98d6fccaad96d80fd99f71f4c19285763554ff002ce9`,
raw-without-LF
`7da604bc9994bf4beead2113b5fe1a923b1fb15f4a85c56c30c978e78de5e2b5`
and the same semantic result. The rebuilt legacy B4C4M0 probe preserves its
exact raw hashes `a51c413a...8501` with LF and `7bd7ab05...ff8e` without LF.

## Exact work result

| Case | Legacy builds | Candidate builds | Removed | Transfer/read/release |
|---|---:|---:|---:|---:|
| P1 supported | `327` | `264` | `63` (`19.27%`) | `63 / 63 / 63` |
| P2 released | `12` | `9` | `3` (`25%`) | `3 / 3 / 3` |

Only `SUBSTEP_DIAGNOSTIC` builds disappear. P1 retains exact category counts
`1/1/2/63/195/0/2/0`; P2 retains `1/1/2/3/0/0/2/0`. Every other category is
identical to the legacy transcript. Maximum total live workspaces remains two
for P1 and one for P2; maximum retained ownership is one and both final live
counts are zero.

## Correspondence and receipts

The candidate and legacy transactions are bit-exact across every adaptive
attempt and selected gate, private physical diagnostic, committed position and
velocity, contact/topology state, canonical frame and publication ledger.
Trajectory, legacy-ledger and policy-ledger roots are unchanged.

Removing queries correctly changes the query-chain roots:

| Case | Legacy query root | Candidate query root | Retained-read receipt |
|---|---|---|---|
| P1 | `4229fbab...be62` | `7108e9c9...c62f` | `3e179e99...f81f` |
| P2 | `23489df6...e235` | `74bfe7de...b446` | `0cec65a9...1626` |

Each receipt binds the ordered retained workspace state hash and exact
binary64 mechanical-energy/density-strain values. Candidate query and receipt
roots repeat exactly.

## Failure ownership

The valid consumer-abort control transfers one workspace and releases it
without a read or publication, ending with zero live objects. A non-finite
current input fails as `CURRENT_WORKSPACE:JOINT_POSITION_INVALID`, creates no
retained ownership and also ends at zero live objects.

## Decision

Select `ONE_MACRO_RETAINED_ACCEPTED_WORKSPACE_CANDIDATE`. Authorize only a
separately frozen complete adaptive/macro-fixed lane correspondence stage.
Do not yet combine immutable static-support indexing or flat-only CSR with
this result. B4D, nominal corpus, CUDA, runtime and production remain blocked.

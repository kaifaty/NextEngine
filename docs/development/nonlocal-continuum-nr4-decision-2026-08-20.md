# Nonlocal continuum NR4 architecture decision — 2026-08-20

Status: `NONLOCAL_48K_RECLOSURE_CANDIDATE / REPORT_ONLY / NO_W2_CREDIT`

## Decision

NR4 selects exactly one state from the frozen
[NR0 decision contract](../plans/nonlocal-continuum/00-research-contract.md#decision-states):

```text
NONLOCAL_48K_RECLOSURE_CANDIDATE
```

The retained research identity is:

```text
nuv-gather-directed-r0
+ pointer-swap-o1
+ nuv-terms-specialized-o2
+ stable-sample-v0
```

This is the strongest satisfied state. The local-domain state is not selected
even though its scale cutoff also passes, because the retained 48k profile
satisfies the stronger reclosure condition. The stop and incomplete states do
not apply.

## Frozen gate evaluation

| NR4 condition | Evidence | Result |
|---|---|---|
| Applicable correctness gates pass | CPU pair/iteration algebra, CUDA tiny `11/11`, cold stiff-surface i2/i20 repeatability, water-16k/48k and viscous-16k controls pass for the retained identity | PASS |
| Retained NR2 fixed-work speedup is at least `2.0x` | HN-3 geometric mean is `3.27688x` over the two correctness-valid source-atomic denominator profiles | PASS |
| `nuv-water-48k.v0` total p95 is at most `8 ms` | Final retained total p95 including neighbor construction is `4.019520 ms` | PASS |
| Reproducible baseline and required source/tool support exist | Primary paper and pinned code are audited; fixed inputs, binary, raw reports and profiler captures are hash-bound | PASS |

The unavailable Nsight Compute hardware counters recorded as
`ERR_NVGPUCTRPERM` do not select `NONLOCAL_EVIDENCE_INCOMPLETE`: the contract
requires stage timing and one profiler capture, both of which exist. No
occupancy, throughput or roofline claim is inferred from unavailable counters.

## What this authorizes

This result authorizes only the next report-only work:

1. draft a new Proposed Nonlocal solver/profile reclosure with fresh identity;
2. define an independent correctness and performance corpus at the exact
   `50,000`-sample product scale;
3. research implementation and algorithmic performance changes under separate
   candidate identities and rollback baselines;
4. return to architecture review after the new corpus and stop gates close.

The follow-up may reuse the standalone lab and retained implementation as an
experimental baseline. It may not inherit DFSPH roots or ProductCheck credit.

## What this does not authorize

- no amendment or promotion of SPEC-38 or ADR-076;
- no W2 PASS, W3 coupling or production/runtime integration;
- no GPU authority, public contract, persistence format or world-dynamics
  schedule change;
- no reduction of the exact `50k` standalone `4/6 ms` p95/p99 target to the
  48k research fixture;
- no claim about integrated `world-dynamics-step` performance;
- no adaptive, learned, thermal, plastic, solid or universal-material claim;
- no Windows work in the current research phase.

DFSPH remains the current continuum correctness reference and fallback. The
Nonlocal branch remains a separately rooted candidate until a later explicit
consumer-backed architecture decision.

## Evidence closure

The full fixed-work evidence is recorded in
[NR2-O4/final evidence](nonlocal-continuum-nr2-o4-evidence-2026-08-20.md).
NR1 source-atomic stiff-surface failure and NR2-O3 numeric mismatch remain
negative evidence; NR4 does not relabel either result as passing.

The next work item is the separately rooted
[Nonlocal performance reclosure roadmap](../plans/nonlocal-continuum-performance/README.md);
it is not part of this closed NR0–NR4 task.

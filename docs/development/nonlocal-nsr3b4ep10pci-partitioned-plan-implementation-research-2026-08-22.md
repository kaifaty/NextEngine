# NSR3-B4EP10PCI partitioned plan implementation research -- 2026-08-22

Status: `COMPLETE / OPT_IN_CANDIDATE_AB_SELECTED`

## Input

B4EP10PCD proves that a 64-partition stable counting-sort/CSR transpose
reconstructs every one of the 226 serial active plans exactly. The audit does
not answer whether five small parallel phases per plan cost less than the
serial count, fill and validation loops they replace.

B4EP10PI is preserved as negative evidence: eliminating builds by scanning a
masked superset raises CPU work and misses the frozen 5% gate. B4EP10PCI must
retain compact active rows; it does not retry or tune that approach.

## Candidate boundary

Add one opt-in research command:

```text
--nominal-hydro-partitioned-active-plan-8
```

Only this command replaces `build_joint_owner_gather_plan` inside the exact
B4EP10I worker-8 transaction with the already audited partitioned builder.
It constructs 226 compact plans directly. It must not first construct the
serial plan, run the side-by-side audit, or enter masked-superset logic.

The unchanged canonical energy fold and owner gathers consume the resulting
arrays. Stable target-row order remains partition ordinal, source centre and
directed slot; therefore no floating-point accumulation order changes.

## Expected work and storage

The candidate retains the exact B4EP10PCD structural totals:

- 226 plan builds over 150,845,996 directed slots;
- 131,987,230 active directed slots and 263,974,460 target records;
- 64 fixed logical partitions, independent of physical worker count;
- five builder regions per plan, for 4,541 total transaction regions and
  290,624 total logical-partition executions;
- one serial checked prefix over at most 11,824 target degrees per plan.

Peak candidate-owned plan/matrix/scratch storage must be measured and remain
below 64 MiB. Audit-side duplication and corrupt fixtures are absent from the
candidate path. Old commands must not allocate the matrix.

## A/B design

Use one final Release binary, physical CPUs `0..7`, fixed OpenMP placement and
an otherwise idle host. Run one warmup of each command, then serialized pairs
in order `AB`, `BA`, `AB`, where A is B4EP10I worker-8 and B is the partitioned
candidate. Record external monotonic wall time plus GNU Time user/system/RSS.

Correspondence precedes duration. Candidate output must be byte-identical in
all measured runs, preserve exact physics/work roots and report zero fallback.
The candidate must win all three pairs, reach at least `1.05x` median paired
speedup, keep max/min wall at most `1.10`, and add no more than 16 MiB median
RSS. Thresholds are inherited from B4EP10PI so the two plan architectures are
judged consistently.

## Decision tree

- Functional mismatch: reject the implementation and keep B4EP10I.
- Exact but below performance gate: retain B4EP10PCD as structural evidence,
  record a performance failure and keep the serial active plan selected.
- PASS: retain the opt-in candidate and freeze residual timing before another
  optimization.

No outcome enables a default/runtime path, B4E2, broad corpus, GPU, schema or
production work.

## Decision

Freeze B4EP10PCI before implementation. This is the only next code change.

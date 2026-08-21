# ADR-093: Deterministic worker placement for the R5 release-performance workload

| Field | Value |
|---|---|
| ID | ADR-093 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-21 |
| Last verified | 2026-08-21 |
| Normative dependencies | [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-058](058-physx-production-backend.md), [ADR-062](062-r5-physx-humanoid-performance-authority.md), [ADR-063](063-run-level-performance-evidence-and-fixed-gate-batches.md), [ADR-091](091-linux-release-performance-authority.md), [ADR-092](092-dimensional-relative-performance-comparison.md) |
| Supersedes | Narrowly supersedes the unpinned-worker condition of the `r5-physics-16` workload identity accepted under ADR-062/091; all absolute budgets, statistical method, evidence-unit rules and no-retry controls remain Accepted |

## Context

After ADR-092 removed the false derived-ratio regression, two independent v10
R5 baseline/gate sets on commits `9dc6919` and `aac4fc6` still returned
`WARNING` solely on 8-worker direct-cost rows (`+589bp`, then `+364bp`/
`+262bp`), every confidence interval crossing zero while every absolute
ADR-062 budget passed with at least `2.2x` headroom.

Bounded research over 36 stored runs established:

- documented host quiescence collapsed 4-worker spread `35% -> 7%` and
  1-worker spread `29% -> 6%` but left 8-worker spread at `28–34%`;
- slow runs shift whole distribution segments (decile medians up to `2–3x`
  typical for `250`-frame plateaus), so each run lands in one persistent
  process-wide scheduling state rather than collecting random spikes;
- no thermal drift exists (pre/post clocks flat `85–89%`) and desktop load is
  refuted as the primary cause;
- the motor workload spawns un-pinned `std::thread` workers and no engine
  crate contains any thread-affinity API, while the release host exposes four
  L3 CCX domains (`0–3`, `4–7`, `8–11`, `12–15` plus SMT siblings);
- a whole-process probe pinned to one CCD pair made 8-worker runs slower and
  less stable (`2503/2557/7132 us` versus `1378–1843 us` unpinned), so
  restricting headroom is not an acceptable control.

Un-pinned worker pools therefore draw a placement lottery on the dual-CCD SMT
release host, and the accepted three-member gate cannot distinguish that
lottery from a real regression.

## Decision

The `r5-physics-16` workload places its worker threads deterministically
before measurement:

1. Detect the online CPU topology from the operating system: for every online
   logical CPU record its physical-core identity and last-level-cache domain,
   failing closed with a stable diagnostic when the platform cannot describe
   it.
2. Group logical CPUs into physical cores ordered by `(cache-domain id,
   core id)`, then interleave the domains round-robin into one deterministic
   core sequence.
3. Assign worker `i` of a `k`-worker configuration to the `i`-th entry of that
   sequence and pin the worker to that core's representative logical CPU at
   thread start, before warm-up. Worker counts above the physical-core count
   are a configuration error.

This spreads every supported worker count evenly across cache domains and
keeps workers off each other's SMT siblings. Placement changes no simulated
value: authoritative roots must remain byte-exact across worker counts, and
the existing root-parity invariant enforces that on every run.

### Reviewed unsafe boundary

Applying the mask requires `sched_setaffinity`. Workspace `unsafe_code =
"forbid"` stays intact everywhere else; a new dedicated crate
`next_cpu_affinity` joins the existing `[workspace.metadata.nextengine.ffi]
allowed_crates` with `ffi_adr = "ADR-093"`, following the reviewed-crate
policy already enforced by `boundary-scan` (`unsafe_code = "warn"`,
`unsafe_op_in_unsafe_fn = "deny"`, documented blocks). Its entire unsafe
surface is one function applying one CPU mask to the current thread and
reading the mask back to verify success.

Failure semantics are fail-closed: unsupported platforms, unreadable
topologies, invalid masks or verification mismatches produce typed errors that
fail the workload run before any timing is measured. No silent unpinned
fallback exists.

### Workload identity

Because measured conditions change, the scenario preimage advances to
`nextengine.performance.r5-physics-16.v3` with
`worker-placement=deterministic-physical-core-v1`. v2 baselines and reports
become incompatible through the existing scenario-hash admission and are not
relabeled. Absolute budgets, methodology v10 comparison classes, ten-run
calibration and isolated fresh-process gates are unchanged.

## Consequences

- The 8-worker placement lottery is removed by construction instead of being
  absorbed by wider thresholds; direct-cost relative checks keep their
  sensitivity for real regressions.
- Release evidence collected under ADR-091/092 for R5 must be recollected on a
  commit carrying this change; earlier v10 R5 WARNING batches remain immutable
  historical evidence.
- Other performance workloads keep their current behavior; extending
  deterministic placement to them requires its own consumer-driven decision.
- The engine gains one small, reviewed, tested CPU-affinity boundary; gameplay
  simulation never depends on it.

## Product checks

| Check | Expected |
|---|---|
| Focused placement tests | Synthetic topologies produce the documented interleave, distinct physical cores, domain-balanced assignment and typed errors for oversubscription or missing domains |
| Boundary scan | `next_cpu_affinity` passes the FFI allowlist policy; no other crate regains unsafe |
| Root parity | 1/4/8-worker runs publish the exact same authoritative roots as before placement |
| Fresh R5 baseline/gate | Ten-run calibration plus one isolated fixed three-run gate under v10 on one clean commit pass all absolute and relative hard metrics |

## Rejected alternatives

- **Raise thresholds or drop w8 relative rows.** Rejected: weakens regression
  sensitivity for direct costs, explicitly rejected again in ADR-092.
- **Pin whole process to one CCD.** Rejected by probe evidence: slower and
  less stable than unpinned.
- **Accept WARNING gates as release evidence.** Rejected: roadmap and
  ADR-091 require passing hard gates; warnings stay blocking evidence.
- **Retry unchanged sets until green.** Rejected by the no-retry policy.

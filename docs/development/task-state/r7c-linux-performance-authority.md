# R7c Linux performance authority — task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / R5_MEMORY_OPTIMIZED / CLEAN_EVIDENCE_NEXT` |
| Updated | 2026-08-21 |
| Task key | `r7c-linux-performance-authority` |
| Scope | Accept one exact Linux release-performance profile and numeric policy, then collect compatible ten-run baselines and fixed three-run hard gates for the representative R2, R3, R4 and R5 workloads |
| Definition of done | One Accepted Linux performance contract and one exact clean Linux commit produce strict compatible baselines plus `PASS` gates for every required workload, with unchanged roots, pre/post environment evidence and no retry-to-green |
| Authority | Working context only; accepted ADRs, specifications and `docs/roadmap.md` remain normative |

## Resume in 60 seconds

- **Current conclusion:** ADR-091 accepts exact Linux profile
  `ref-linux-b550i-3950x-rtx3080-v1`, canonical R2–R5 budgets and strict
  Performance V6/methodology v9. R3 and R4 now have complete clean baseline and
  gate evidence on `2bdd20c`. The first R5 calibration run on that commit is an
  immutable `FAIL`: peak process working set is `355,880,960` bytes against the
  accepted `335,544,320`-byte ceiling; every timing, restore, logical-memory,
  root and environment row passes.
- **Why:** Diagnostic R3/R5 already passed their accepted rows and exact-root
  closure. Cached immutable catalog revisions reduce R4 navigation from roughly
  `4.24/4.79 ms` to `0.67/0.76 ms` and integrated production ticks from
  `15.43/16.10 ms` to `2.25/2.39 ms` p95/p99 without root changes.
- **Next action:** Publish the verified checkpoint-retention optimization as a
  new clean commit, then collect a fresh complete exact-commit evidence set.
- **Current blocker:** R2 only: the Linux session currently exposes no active
  physical display (`xrandr` 0×0, empty Mutter display state, NVIDIA display
  inactive), so the production Vulkan workload correctly returns `NOT_RUN`.
  R3/R4/R5 evidence can proceed independently.
- **Do not retry:** Do not run `ref-win-thoth-v1`, reuse old reports as Linux
  evidence, use a virtual/software display for R2, or rerun an unchanged failed
  calibration/gate set to obtain a greener sample.
- **Reconsider when:** The active Linux release machine materially changes or a
  future Accepted ADR changes the release-performance target.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| ADR-091 | Accepted exact Linux fingerprint, preflight, canonical R2–R5 budgets and V6/v9 evidence semantics | Normative authority is complete; Windows/THOTH stays historical |
| Performance V6 implementation | Linux-only baseline/gate prerequisites, budget-bearing baseline V2 metrics, canonical hard/diagnostic policy and R2–R5 gate routing compile and pass focused tests | Clean evidence may now be collected without relabelling V5 |
| R3/R5 diagnostics | R3 completes in `868,855 us`; R5 passes every existing row with exact worker roots and clean environment boundaries | Accepted budgets have observed headroom |
| R4 pre-optimization diagnostic | Exact roots and zero defer/drop/starvation/fabrication, but navigation `4,236/4,793 us` and integrated `15,425/16,100 us` exceed ADR-016 | Optimize; do not widen budgets |
| R4 optimized V6 diagnostic | Outer `PASS`, no diagnostics; navigation `668/760 us`, cognition `23/29 us`, integrated `2,252/2,391 us`; roots unchanged | R4 is ready for clean ten-run evidence |
| R2 desktop prerequisite | GNOME/Wayland variables exist, but no active display is published by Xwayland/Mutter/NVIDIA | Keep R2 `NOT_RUN` until the physical monitor is OS-visible |
| R3 clean calibration on `368d216` | Ten reports are clean, ready, exact-root and individually within `867,328–896,795 us`, but baseline publication rejects all ten because device/Vulkan unavailability was left attached to the CPU-only workload | Preserve the set; fix the workload resource declaration and collect a new set on a new commit |
| R3 resource-boundary fix | Focused contract test passes; a production diagnostic reports `device_resident_bytes = 0`, zero Vulkan queries, no unavailable counters and no diagnostics | New clean R3 evidence may be collected after commit |
| R3 clean evidence on `2bdd20c` | Ten-run baseline accepted; fixed gate `PASS`, p95/p99 `885,128 us`, exact streaming root, zero diagnostics | R3 portion of R7c is complete for this commit |
| R4 clean evidence on `2bdd20c` | Ten-run baseline accepted; fixed gate `PASS`, navigation `679/777 us`, cognition `24/30 us`, integrated `2,278/2,443 us`, all eight roots exact and no defer/drop/starvation/fabrication | R4 portion of R7c is complete for this commit |
| R5 first calibration run on `2bdd20c` | Immutable `FAIL`: process peak working set `355,880,960 / 335,544,320` bytes; all other rows and root parity pass under ready pre/postflight | Preserve the run; reduce live checkpoint retention on a new commit, never widen the accepted budget or retry unchanged |
| R5 checkpoint-retention optimization | Production diagnostic `PASS`; peak working set falls to `191,184,896` bytes, exact root remains `6b6fee7492dc5ac600a6e75aa8a0c1aab830ee5174e797bbd1eaafe758c19f31`, worker parity remains true and diagnostics remain empty | Optimization is sufficient without changing budgets or authoritative results; commit before new evidence |

## Decisions that constrain the work

### D-001 — Version the Linux authority explicitly

- **Observation:** Current strict reports and baselines encode a single
  Windows/THOTH authority in their validation semantics.
- **Evidence:** Windows target checks and THOTH fingerprint validation are
  mandatory in baseline construction, comparison and gate prerequisites.
- **Decision:** Introduce an explicit successor wire/methodology for the active
  Linux release-performance authority; keep V5 only as historical evidence.
- **Rejected alternatives:** Relabel THOTH, weaken strict validation, or accept
  Linux under unchanged V5 semantics.
- **Consequences:** Run/baseline readers, tests, documentation and scenario
  hashes must advance together before calibration begins.
- **Uncertainty:** None for the accepted V6 boundary; evidence remains to run.
- **Reconsider when:** The source audit proves a smaller versioned profile layer
  can preserve strict old/new wire separation without ambiguity.

### D-002 — Preserve the accepted statistical method

- **Observation:** The existing v8 method already separates calibration from
  acceptance and aggregates independent runs conservatively.
- **Evidence:** Ten single-run reports build a baseline; a fixed three-run gate
  uses run-level medians, worst per-run absolute tails and deterministic
  bootstrap intervals.
- **Decision:** Change host authority and numeric policy without weakening the
  statistical, provenance, environment or no-retry controls.
- **Rejected alternatives:** A single benchmark run, pooled-sample percentiles,
  retry-to-green, or relative-only acceptance.
- **Consequences:** Any required tooling split is semantic plumbing, not a
  shortcut around evidence quality.
- **Uncertainty:** None for the accepted methodology; natural run variance will
  be preserved by the no-retry evidence sets.
- **Reconsider when:** A later Accepted methodology ADR provides stronger
  equivalent controls.

## Resolved hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: the current host exposes a stable exact Linux fingerprint through existing probes | Probe records exact CPU/GPU/RAM/storage/OS/kernel/BIOS/driver/governor values across diagnostics | None observed | Resolved by ADR-091 exact profile |
| H2: R2 product frame deadlines can remain absolute Linux gates | SPEC-04 deadlines are product-facing and ADR-091 accepts them independently | R2 measurement still awaits an active display | Authority resolved; evidence pending |
| H3: R3/R4/R5 can use explicit Linux budgets without weakening product behavior | ADR-091 accepts R3 ceiling and unchanged ADR-016/062 rows; diagnostics show headroom after R4 optimization | None after optimized R4 diagnostic | Resolved; clean evidence pending |

## Required context

Read these sources in precedence order before acting:

1. `AGENTS.md`, `docs/architecture/agent-routing.md`, ADR-090 and ADR-091
2. SPEC-00, SPEC-04, SPEC-09, SPEC-12, SPEC-15, SPEC-29 and SPEC-35
3. ADR-001, ADR-003, ADR-016, ADR-028, ADR-030, ADR-035, ADR-036,
   ADR-038, ADR-045, ADR-049, ADR-060 through ADR-063 and ADR-074
4. `docs/roadmap.md` R7/R7c and the completed R7a/R7b task states
5. Current performance run, host, baseline, workload and statistics modules

## Next action

Commit the verified checkpoint-retention optimization, then collect a fresh
complete R2–R5 exact-commit set. R2 still waits for an OS-visible physical
display.

## Do not retry

- `ref-win-thoth-v1` or any Windows performance command — out of scope.
- Historical THOTH baselines/reports as Linux release evidence — incompatible
  authority even when the workload name matches.
- A virtual/software R2 display or headless substitute — it cannot prove the
  production presentation path.
- Any unchanged calibration/gate retry after a completed negative result.
- Baseline publication from the rejected `368d216` R3 set after changing only
  the publisher. The workload must emit complete evidence itself on a new clean
  commit.
- Any further R5 run on unchanged `2bdd20c`, or widening the 320 MiB ceiling to
  relabel its recorded failure.

## Handoff

- **Workspace state:** The ADR-091/V6/R4 boundary is commit `368d216`. Its first
  clean R3 set found a missing CPU-only device declaration. Commit `2bdd20c`
  fixes that boundary and closes R3/R4 evidence, but its first R5 report fails
  only the process peak. The checkpoint-retention optimization is implemented
  and verified, reducing the diagnostic peak from `355,880,960` to
  `191,184,896` bytes without changing the authoritative root.
- **Checks:** Focused contracts/world/agent/verification tests pass; xtask
  performance lib tests pass `46/46`; workspace clippy is warning-free and the
  broad Linux `host-check` passes. Optimized dirty-worktree R4 report passes
  every numeric/root/instrumentation check with no diagnostics.
- **Remaining risk:** Physical-display availability for R2 and total clean
  evidence runtime.
- **Promotion needed:** Exact clean R2–R5 baselines/gates, then record their
  immutable evidence commit/hashes and close B-12.

# R7c Linux performance authority — task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / ADR093_PLACEMENT_IMPLEMENTED / EVIDENCE_PENDING` |
| Updated | 2026-08-21 |
| Task key | `r7c-linux-performance-authority` |
| Scope | Accept one exact Linux release-performance profile and numeric policy, then collect compatible ten-run baselines and fixed three-run hard gates for the representative R2, R3, R4 and R5 workloads |
| Definition of done | One Accepted Linux performance contract and one exact clean Linux commit produce strict compatible baselines plus `PASS` gates for every required workload, with unchanged roots, pre/post environment evidence and no retry-to-green |
| Authority | Working context only; accepted ADRs, specifications and `docs/roadmap.md` remain normative |

## Resume in 60 seconds

- **Current conclusion:** ADR-091/092 accept exact Linux profile
  `ref-linux-b550i-3950x-rtx3080-v1`, canonical R2–R5 budgets and strict
  Performance V6/methodology v10. R3 and R4 have historical clean baseline and
  gate evidence on `2bdd20c` under v9; the methodology bump makes that
  historical evidence incompatible with v10 gates, so all final R2–R5 evidence
  must be recollected on one exact v10 commit. The first R5 calibration run on
  `2bdd20c` is an immutable `FAIL`: peak process working set is
  `355,880,960` bytes against the accepted `335,544,320`-byte ceiling; every
  timing, restore, logical-memory, root and environment row passes. Commit
  `57739ea` fixes the retention defect, but its first evidence set is an
  immutable preflight rejection: post-build one-minute CPU load was 41%, so no
  workload executed. The next clean `feae4f0` baseline is valid, but its sole
  fixed gate is `WARNING`: same-process members accumulate Linux
  high-water/vendor-runtime state unlike the ten fresh-process calibration
  runs. Fresh-process `3bbc19e` fixes that asymmetry, and its sole R5 gate
  `FAIL` was a generic percentage comparison double-normalizing an already
  normalized scaling ratio. Commit `22c8049` implements the v10 dimensional
  classes (the three normalized R5 ratios are absolute-only with
  `relative: null`; direct costs keep whole-run relative bootstrap), and
  commits `26097e2`/`896e000` bind v9-baseline rejection to v10 gate admission
  by focused tests and keep the codegen workload-unavailability test honest
  under the production `physx` feature.
- **Why:** Direct physics-substep cost changed by `-0.25%`, one-worker cost
  improved about 5% and eight-worker cost improved about 7.5% in the recorded
  `3bbc19e` gate, so the derived-ratio `FAIL` was a measurement-semantics
  defect, not a slower workload. Cached immutable catalog revisions reduce R4
  navigation from roughly `4.24/4.79 ms` to `0.67/0.76 ms` and integrated
  production ticks from `15.43/16.10 ms` to `2.25/2.39 ms` p95/p99 without
  root changes.
- **Next action:** On a new exact clean v10 commit collect the fresh R5
  ten-run baseline plus one isolated fixed three-run gate, then re-collect R3
  and R4 evidence on that same commit. R2 still waits for an OS-visible
  physical display.
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
| ADR-091/092 | Accepted exact Linux fingerprint, preflight, canonical R2–R5 budgets and V6/v10 dimensional evidence semantics | Normative authority is complete; Windows/THOTH stays historical |
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
| R5 calibration set on `57739ea` | Ten reports are typed `NOT_RUN` at preflight with CPU load `41%`; zero metrics and roots prove the workload never started | Preserve the rejected set; allow the one-minute load to settle, record the transition on a new commit and do not treat these entries as calibration runs |
| R5 clean evidence on `feae4f0` | Ten-run baseline accepted; the sole fixed gate preserves exact root and passes every absolute row, but returns `WARNING` for relative 4/8-worker and peak-RSS variance | Preserve the gate; fix baseline/gate process-lifetime asymmetry on a new commit rather than rerunning it |
| R3 isolated-gate evidence on `3bbc19e` | Ten-run baseline plus three fresh-process members produce `PASS`, six ready boundaries and exact streaming root | Fresh-process aggregation is functional |
| R5 isolated gate on `3bbc19e` | Ten-run baseline accepted; fresh-process gate passes every absolute row/root/environment check and direct costs are flat or faster, but generic relative comparison returns `FAIL` only for 4-worker scaling inefficiency `1108 -> 1535 bp` | Preserve the gate; adopt ADR-092 dimensional comparison on a new methodology/commit, never retry v9 |
| ADR-092/v10 implementation `22c8049`/`26097e2`/`896e000` | Methodology identity advances to `nextengine-performance-v10`; the three normalized R5 ratios keep budgets but skip relative comparison (`relative: null`); direct costs retain whole-run bootstrap; focused tests bind ratio classes and v9-baseline rejection to v10 admission; codegen workload-unavailability test is honest under `physx`; xtask lib tests pass `113/113` under `desktop-sdl-ash,physx` with clean fmt/clippy | Implementation authority is complete; all final R2–R5 release evidence must be recollected under v10 on one exact clean commit |
| First uninstrumented R5 set on `9dc6919` | Ten report runs pass every absolute row, but baseline publication rejects them: runs lacked `NEXTENGINE_PERFORMANCE_PROFILER=on` and the explicit `--target ref-linux-b550i-3950x-rtx3080-v1`, so each carries `PERF_PROFILER_DISABLED` and `observed-host-v1` target identity | Preserve under `target/perf/r5-v10-cal-uninstrumented`; hard evidence requires the profiler env and exact target id; not a calibration set |
| R5 v10 baseline/gate set 1 on `9dc6919` | Ten-run baseline published (`f18fa243…cb85`); isolated fresh-process gate (`25171f98…1a00`) returns `WARNING`: only `worker-8.physics-motor-frame` is relative-warning at `+589bp` (CI `[-1952, +884]`); all 16 absolute rows PASS with headroom, exact root `6b6fee74…` unchanged, peak RSS `199,487,488` bytes; the three normalized ratios are absolute-only `PASS` with `relative: null`, so the v9 derived-ratio defect is gone | Preserve as complete immutable evidence; do not rerun this baseline/gate pair |
| Set-1 w8 warning analysis | Gate member run-p95s `[1381, 1652, 1674]` lie fully inside the baseline run-p95 range `[1391–1787]`; nearest-rank p50 of ten takes the lower-middle order statistic `1560` while the three-member median is `1652`, so an order-statistic gap plus real host noise yields `+589bp`; during collection the desktop ran `localsearch-3` (~13%) and `steamwebhelper` (~12%) alongside the agent process | The warning is measurement-noise sensitivity of the accepted conservative policy, not a slower workload; quiesce optional desktop load and recollect on a new commit instead of retrying unchanged |
| Stale-build rejection on `aac4fc6` | First set-2 attempt returned typed `NOT_RUN`: binary provenance `9dc6919…` versus runtime commit `aac4fc6…` (`PERF_RUNTIME_COMMIT_MISMATCH`) | Preserve under `target/perf/r5-v10-cal2-stale-build`; rebuild xtask on the exact evidence commit before collection |
| R5 v10 baseline/gate set 2 on `aac4fc6` | Ten-run baseline published after rebuild; isolated fresh-process gate returns `WARNING` again, now on three w8-config rows (`worker-8.physics-motor-frame` `+364bp`, `physics-substep-cost`/`motor-frame-cost` `+262bp`), every CI crossing zero; all absolute rows PASS with headroom, ratios stay absolute-only `PASS`, peak RSS improved | Second coherent occurrence under improved conditions triggers the pre-declared two-strike stop: no further collection attempts |
| Set-2 w8 research mining | Host quiescence collapsed w4 spread `35% -> 7%` and w1 spread `29% -> 6%`, but w8 spread stayed `28% -> 34%`; within slow runs entire distribution segments shift (decile medians up to `2–3x` typical for `250`-frame plateaus, clustered early and in bursts); no monotone thermal drift; recorded pre/post clock/load percentages do not discriminate; no thread-affinity API exists anywhere in engine, motor workers are unpinned `std::thread`s, PhysX bridge uses `PxDefaultCpuDispatcherCreate(1)` | Residual w8 variance is structural per-process state (consistent with core/CCD placement lottery on the dual-CCD SMT 3950X), not desktop load or thermal ramp; discriminating experiment would be an affinity A/B probe plus an Accepted authority decision before any new evidence set |
| Non-evidence affinity probe `256a64a` | Topology shows four L3 CCX domains (cores `0–3/4–7/8–11/12–15`, siblings `+16..+31`). Pinning the whole process to `0–7` (one logical CPU per physical core, one CCD pair) made w8 slower and less stable: p95s `2503/2557/7132` versus unpinned `1378–1843`; a fourth attempt was a typed preflight rejection (`PERF_CPU_LOAD_LIMIT_EXCEEDED`) | Naive single-mask pinning refuted: restricting headroom oversubscribes the mask and moves the w8 path onto a scheduling knee. The untested variant is deterministic per-worker placement spread over all physical cores inside the workload/engine, which changes measured conditions and requires Accepted authority regardless of outcome |
| ADR-093 implementation | Product owner chose deterministic per-worker placement. New Accepted ADR-093; new reviewed `next_cpu_affinity` crate joins the FFI allowlist (`boundary-scan PASS`) with one unsafe mask application plus read-back verification; motor workload detects topology, interleaves cache domains round-robin, pins each worker before warm-up and fails closed via `MOTOR_PERF_WORKER_PLACEMENT_FAILED`; scenario preimage advances to `r5-physics-16.v3`; focused placement tests pass on synthetic topologies, root parity holds with real pinning, end-to-end probe keeps root `6b6fee74…` with w8 p50 `1242 us` and zero diagnostics | The placement lottery is removed by construction; all prior R5 v10 baselines are incompatible via the scenario-hash bump and the next evidence set must be collected on the commit carrying this change |
| First v3 collection attempt on `8166eb9` | Ten report runs typed `NOT_RUN` at preflight with `PERF_GPU_LOAD_LIMIT_EXCEEDED`: the desktop session was in active use (GPU utilization 44%, image viewer/file manager/IDE/Telegram holding GPU contexts) and every workload honestly refused to start; preserved under `target/perf/r5-v3-cal-preflight-rejected` | Complete negative evidence for that window; collect on a new commit during a genuinely idle session instead of treating these entries as calibration runs |
| R5/R3 v10+ADR-093 release evidence on `0e47362` | Idle-window session: R5 ten-run baseline published, isolated fixed three-run gate returns **`PASS`** — all sixteen rows green, w8 direct costs flat (`-15bp/-8bp`), normalized ratios absolute-only `PASS`, exact root preserved; R3 ten-run baseline plus fixed gate also **`PASS`** (`889,314 us` p95 against the `1,500,000 us` ceiling, `+0bp`). ADR-093 placement collapsed w8 calibration spread from `28–34%` to `8%` | The R5 and R3 portions of R7c are closed on this exact commit under methodology v10 |
| R4 first collection interrupted on `0e47362` | Runs 01–07 completed clean, then runs 08–10 returned typed preflight rejections when desktop GPU load spiked to 45% mid-collection; partial set preserved under `target/perf/r4-v10-cal-interrupted` | Do not assemble a baseline across an environmental disturbance; record the boundary and recollect all ten after the host settles again |

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

1. `AGENTS.md`, `docs/architecture/agent-routing.md`, ADR-090 through ADR-092
2. SPEC-00, SPEC-04, SPEC-09, SPEC-12, SPEC-15, SPEC-29 and SPEC-35
3. ADR-001, ADR-003, ADR-016, ADR-028, ADR-030, ADR-035, ADR-036,
   ADR-038, ADR-045, ADR-049, ADR-060 through ADR-063 and ADR-074
4. `docs/roadmap.md` R7/R7c and the completed R7a/R7b task states
5. Current performance run, host, baseline, workload and statistics modules

## Next action

Commit the ADR-093 implementation, then collect the fresh R5 ten-run baseline
plus isolated fixed three-run gate on that exact commit under the documented
host discipline (indexer stopped, load settled). After R5 closes, collect R3
and R4 evidence on the same final commit. R2 still waits for an OS-visible
physical display.

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
- Any further calibration run on unchanged `57739ea`; its ten-entry preflight
  rejection is complete negative evidence even though the workload never ran.
- Any further R5 gate on unchanged `feae4f0`; its `WARNING` is a complete fixed
  batch and cannot be selected or repeated.
- Any further R5 gate on unchanged `3bbc19e`; its derived-ratio `FAIL` is a
  complete v9 batch even though all absolute/direct-cost rows pass.
- Reusing the v9 R3/R4 evidence on `2bdd20c` as final release evidence; the
  methodology bump makes it incompatible with v10 gates even though its
  absolute rows passed.
- Rerunning either recorded `9dc6919` or `aac4fc6` baseline/gate pair; both
  `WARNING` batches are complete immutable evidence.
- Further R5 collection attempts before the authority decision in the research
  checkpoint below: the two-strike criterion fired, and a third similar
  attempt would be retry-to-green.

## Research checkpoint (2026-08-21)

**Problem restated:** w8-config direct-cost rows trip relative warnings in
every v10 R5 gate although absolute budgets pass with `>=2.2x` headroom and
all confidence intervals cross zero. Two coherent remediations (v10 dimensional
classes; documented host quiescence on a fresh commit) moved w4/w1 variance to
`<=7%` but left w8 at `34%`.

**Hypotheses after mining 36 stored runs:**

| Hypothesis | Evidence for | Evidence against | Status |
| --- | --- | --- | --- |
| H-A core/CCD placement lottery of unpinned workers | Quiescence fixed w4/w1 but not w8; plateaus persist for whole process lifetimes; four-L3 topology gives a rich placement space | Whole-process pinning to one CCD pair made w8 slower and less stable (`2503/2557/7132`), so simple locality is not the mechanism | Refined: scheduling-headroom/knee sensitivity of the 8-worker path, placement family still leading |
| H-B desktop interference bursts | Set-1/uninstr sets ran under load average `3–5`; burst plateaus exist | Same background left w4/w1 stable in set-2; bursts persist when quiet | Refuted as primary cause |
| H-C thermal/frequency ramp | None | No directional drift across sequences; pre/post clocks flat `85–89%` | Refuted |
| H-D warm-up transients (`240` substeps = only `60` motor frames) | Decile-0 medians elevated in several runs | Mid-run plateaus need a second cause | Partial contributor |

**Decision:** pause R5 evidence collection. Two authority paths remain for the
product owner: (1) accept an engine/workload change that places workers
deterministically on distinct physical cores (a production-semantics change
requiring its own ADR and fresh evidence), or (2) amend the accepted warning
semantics for noise-dominated direct rows (CI-based instead of point-estimate).
Whole-process mask pinning was probed and rejected. R3/R4 stay paused because
final release evidence must share one exact commit with R5.

**Missing evidence:** the bounded web-search backend returned `403` on this
host, so no external prior art (Linux scheduler/CCD placement, PhysX worker
variance) was consulted; H-A rests on local evidence only.

## Handoff

- **Workspace state:** The ADR-091/V6/R4 boundary is commit `368d216`. Its first
  clean R3 set found a missing CPU-only device declaration. Commit `2bdd20c`
  fixes that boundary and closes v9 R3/R4 evidence, but its first R5 report
  fails only the process peak. Commit `57739ea` contains the verified
  optimization, but its first ten-entry R5 set was rejected before execution
  because the post-build one-minute CPU load was 41%. Clean `feae4f0` publishes
  a valid R5 baseline and one immutable `WARNING` gate; fresh-process member
  isolation is implemented by `3bbc19e`, whose immutable R5 gate exposes the
  percent-over-percent derived-ratio defect. Commits `22c8049`/`26097e2`/
  `896e000` implement and test-verify ADR-092/methodology v10.
- **Checks:** Focused contracts/world/agent/verification tests pass; xtask lib
  tests pass `113/113` under `desktop-sdl-ash,physx`; workspace clippy is
  warning-free; fmt is clean. The broad Linux `host-check` passed earlier in
  R7c and has not been rerun after the test-only commits.
- **Remaining risk:** Physical-display availability for R2 and total clean
  evidence runtime.
- **Promotion needed:** Exact clean R2–R5 baselines/gates, then record their
  immutable evidence commit/hashes and close B-12.

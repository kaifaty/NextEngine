# R7c Linux performance authority — task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / V11_R3_R4_R5_CLOSED_ON_8498001 / R2_ROOT_CAUSE_FIXED_ON_df964af2 / FINAL_EVIDENCE_CAMPAIGN_PENDING` |
| Updated | 2026-08-24 |
| Task key | `r7c-linux-performance-authority` |
| Scope | Accept one exact Linux release-performance profile and numeric policy, then collect compatible ten-run baselines and fixed three-run hard gates for the representative R2, R3, R4 and R5 workloads |
| Definition of done | One Accepted Linux performance contract and one exact clean Linux commit produce strict compatible baselines plus `PASS` gates for every required workload, with unchanged roots, pre/post environment evidence and no retry-to-green |
| Authority | Working context only; accepted ADRs, specifications and `docs/roadmap.md` remain normative |

## Resume in 60 seconds

- **Current conclusion:** ADR-091/092/093/094 form the accepted Linux release
  performance authority. On commit `8498001` the R3/R4/R5 portions closed with
  hard v11 `PASS`, but R2 stayed blocked by desktop-session presentation state.
  Post-restart diagnosis isolated two environmental mechanisms: the Wayland
  driver rebuilt the frame plan because the compositor reconfigured the surface
  after rendering started (`cache_miss=2`, `plan_invalidate=0` ⇒ swapchain
  extent changed mid-run), and the x11 driver delivered a decoration/dock-clamped
  drawable (`[1920,1011]` instead of `[1920,1080]`). Root cause shared by both:
  a floating monitor-sized window cannot guarantee a stable declared extent.
- **Fix (product owner approved):** commit `df964af2` adds opt-in presentation
  stabilization to `DesktopRunOptions`
  (`prefer_borderless_fullscreen_when_display_matches`): when the declared
  extent equals the display bounds the adapter starts borderless fullscreen,
  absorbs the initial compositor configure before swapchain creation, flushes
  pre-run window events, and fails closed with typed
  `PLATFORM_FULLSCREEN_START_EXTENT_UNAVAILABLE` if the extent never settles.
  Only the timing workload path enables it; the live game default is unchanged.
- **Validation so far:** on quiet host, clean HEAD `df964af2`: Wayland soak
  `PASS cache_miss=1`; x11 soak `PASS cache_miss=1`; full six-window
  `r2-alpha-render` report run `PASS`/`REPORT_ONLY` with zero diagnostics.
- **Why this direction:** environment toggles proved unreproducible across
  reboots; relaxing the exactly-one-plan-build invariant was rejected as
  evidence weakening. Fullscreen start is production-faithful (the shipped game
  presents fullscreen) and keeps every strict check intact.
- **Next action:** recollect ALL FOUR workloads (R2–R5) on the final exact
  clean commit `df964af2` — ten-run baselines plus fixed gates, admission-timed,
  no retry-to-green (HEAD moved from `8498001`, so prior v11 CPU-side sets are
  incompatible with the one-commit rule). Then close B-12 in roadmap and hand
  off to R7d hardening.
- **Current blocker:** none technical; campaign needs a sustained-quiet desktop
  session (~2–3 h wall clock).
- **Do not retry:** Do not run `ref-win-thoth-v1`, reuse old reports as Linux
  evidence, use a virtual/software display for R2, rerun any recorded failed
  or warned baseline/gate set unchanged, assemble a baseline across an
  environmental disturbance, or treat preflight-rejected entries as
  calibration runs.
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
| Final aligned evidence on `0e8db75` | Admission-timed attempts (each run fires only when desktop GPU load is below `35%`; interrupted attempt sets preserved as `.interrupted*`, no process touched). Ten-run baselines plus isolated fixed gates all return hard `PASS` with zero diagnostics on the same commit: R4 baseline `ced35608…c685` / gate `dd496319…a396f`; R5 baseline `1c338c6e…2b2ab` / gate `c5307655…03d27`; R3 baseline `2d4dc44a…b83e5` / gate `578c1480…f924f`. R4 navigation `686 us`, cognition `24 us`, integrated `2310 us` (`-761bp`); R5 w8 p95 `1283 us` (`-7bp`), ratios absolute-only; R3 total `869,479 us` (`-13bp`) | The R3, R4 and R5 portions of R7c are closed on one exact clean commit; only the R2 display-blocked workload remains for B-12 |
| Display returns; realignment on `f3cb715` started | HDMI-A-1 became OS-visible (`1920x1080`), unblocking R2. Repo frozen at docs commit `f3cb715`, binary provenance verified (`f3cb715b6`); admission threshold raised from `35%` to `38%` desktop GPU (still strictly inside the accepted `<40%` preflight). R4 ten-run calibration plus gate completed first-attempt **`PASS`** (`final4-r4-*`); R5 ten-run calibration plus baseline published clean | R4 portion re-closed on the new commit; R5 baseline ready for its gate |
| R5 gate attempt 1 on `f3cb715` incomplete | During a desktop-active window one fresh-process member executed the full workload with verdict **`FAIL`** and exited `PERFORMANCE_GATE_FAILED`; the parent rejected its stdout as `PERF_GATE_MEMBER_REPORT_INVALID: trailing characters at line 2 column 1`, so the batch never assembled and no aggregate verdict was published. Member metrics are not recoverable (captured stdout dropped by the parent error path); the exact log strings are preserved in `/tmp/opencode/r5-gate.log`. Root cause of the stdout pollution is unresolved | Recorded as an incomplete negative attempt under desktop-load burst, consistent with prior interrupted-attempt precedent: never rerun this attempt, but a NEW complete gate batch under sustained quiet is the documented continuation; if a cleanly assembled batch fails, that verdict is immutable and stops R7c |
| R5 gate attempt 2 on `f3cb715` completed `WARNING` | Sustained-quiet admission (GPU `<30` for 60s) admitted a fully assembled three-member batch: **immutable** `WARNING` (`final4-r5-gate-attempt1`), solely `worker-4.physics-motor-frame` at `+205bp` with CI `[-293, +782]`; every absolute row PASS, w8 improved `-509bp/-604bp`, substep-cost flat `+12bp`. Baseline analysis: w4 between-run p95 spread is `~±10–15%` even under ADR-093 placement (morning median `2331`, members `2370–2404`); the accepted policy warns at point-estimate `>=200bp` regardless of CI while FAIL already requires CI-low `>=500bp` | Third independent event of the relative-noise class across rows; blind recollection is coin-flipping immutable artifacts, so further collection pauses pending the product-owner decision on CI-gated warning semantics |
| ADR-094 accepted and implemented `4827ca1` | Product owner chose CI-gated warnings: warning requires change `>=200bp` AND CI95-low `>=200bp`, symmetric with FAIL; methodology advances to `nextengine-performance-v11`; failed commands emit diagnostics on stderr only, fixing the stdout pollution that discarded a failing member's metrics. Focused verdict tests encode the recorded noise shapes; host-check, clippy and `113/113` xtask tests pass; docs (README 2.62, traceability V9.1, SPEC-04/09/12, routing) updated in the same change | v10 evidence is incompatible by design; all four workloads recollected on the new commit |
| V11 R3/R4/R5 closure on `4827ca1` | Admission-timed collection with sustained-quiet gate precondition: R4 baseline `e67316a9…4e06` / gate attempt2 `c9190cdc…8d32…` **PASS** (attempt1 was a typed environment `NOT_RUN`, retried per policy); R5 baseline `cfc427c7…453ca…` / gate `062de9fb…448b…` **PASS**; R3 baseline `8ffff7ac…453ca…` / gate `9a4b07cf…f924f…` **PASS** — all zero diagnostics on one exact clean v11 commit | The three CPU-side portions of R7c are closed under the final authority |
| R2 blocked by desktop session presentation state | Display is OS-visible (`HDMI-1 1920x1080@199.92`), Vulkan loader and devices healthy, but every production desktop workload fails identically under both SDL video drivers: default Wayland path renders no smoke frame; forced `SDL_VIDEODRIVER=x11` renders completely (240-frame soak PASS) yet R2's declared-profile extent check fails. Root environmental factor identified: Mutter experimental features `scale-monitor-framebuffer` + `xwayland-native-scaling` are enabled in this session; the Wayland breakage appeared after the display reconnection | Engine code is unchanged and not implicated (CPU scenarios pass end to end); R2 needs either mutter feature toggle or a fresh graphical session before its ten-run baseline/gate can be collected; user processes remain untouched per instruction |
| Fullscreen-start fix on `df964af2` (2026-08-24) | Opt-in `prefer_borderless_fullscreen_when_display_matches` in `DesktopRunOptions`: borderless fullscreen start when declared extent equals display bounds, bounded stabilization for the initial configure, pre-run window-event flush, typed closed failure `PLATFORM_FULLSCREEN_START_EXTENT_UNAVAILABLE`; timing workloads enable it, live game default unchanged. Focused tests + clippy clean, xtask lib `113/113` (`desktop-sdl-ash,physx`). Probes on clean quiet host: Wayland soak `PASS cache_miss=1`, x11 soak `PASS cache_miss=1`, full six-window R2 report `PASS` zero diagnostics | Both presentation blockers are removed by construction; final R2–R5 evidence must be recollected on `df964af2` (one-commit rule) and the campaign may proceed whenever the desktop is sustained-quiet |
| Mutter features toggled off; x11 path heals, Wayland stays broken | Product owner approved disabling the features (previous value preserved in `/tmp/opencode/mutter-features-backup.txt`). After the toggle the x11 driver produces a fully clean soak on rebuilt binary `84980001` (`PASS`, zero diagnostics — the earlier extent mismatch is gone), while the default Wayland path still renders no smoke frame; that anomaly now requires a graphical-session restart to diagnose further and is recorded as an open host finding | Evidence collection proceeds pinned to `SDL_VIDEODRIVER=x11` (real display, real compositor presentation, production NVIDIA Vulkan); Wayland presentation remains an open host issue to recheck after the next session restart |
| V11 campaign on `8498001` paused by legitimate CPU contention | R4 closed first-attempt **`PASS`** (baseline `fin8498-r4-baseline`, gate attempt1). Then two R5 calibration attempts produced immutable absolute-budget FAILs: `worker-8.scaling-inefficiency` `4582bp`/`4511bp` against the `4500bp` ceiling with zero diagnostics and ready preflight. Host at that moment: load average `~8`, a `python` process at `350%` CPU plus two 100% workers (user's important computation, untouched per instruction), `Tctl 81.6°C`; w1 motor-frame doubled to `~19 ms`. The `<40%` preflight cannot see topology-local contention or CPU thermal state | Both contaminated runs preserved as exact negative evidence under `.interrupted` sets; collection paused until the user's compute finishes. Resume script ready: rerun `collect_fin8498_resume.sh` unchanged on frozen HEAD `8498001` (binary already built there), then finalize docs; do not widen budgets or treat contaminated runs as calibration entries |
| V11 campaign completed on `8498001`; R2 deferred | After the user's computation finished, the resumed admission-timed run closed R5 (ten clean runs; gate attempt1 **`PASS`**, baseline `6f1998de…4e3c`, gate `78c5f333…176`) and R3 (gate attempt1 **`PASS`**, baseline `e55020ed…3a8`, gate `06e7c6fa…e28`) with zero diagnostics, alongside R4 (baseline `47dc4717…e3c`, gate `bf1059ee…dfd`). The remaining R2 still failed under both drivers (`720p30` declared-extent mismatch persists under x11 even though the `1080p` soak is clean), and the product owner chose to defer it rather than restart the graphical session now | R3/R4/R5 hold hard v11 gates on one exact commit; B-12 stays open solely for R2, whose evidence requires a healthy desktop presentation path (session restart recommended before recollection) |
| Post-restart R2 diagnosis (`2026-08-23`, temp-instrumented, discarded) | Machine rebooted for the session restart. Temporary env-gated instrumentation in the desktop smoke (committed as `cb22abde`, then `git reset --hard 901e5046`; never evidence) revealed, on the Wayland driver: all frames render (`rendered=240 timings=240 tsq=480 dropped=0 ui_frames=240`) yet the run fails because the **frame plan is built twice** (`cache_miss=2 cache_hit=238` against the required exactly-one-build invariant) — a late compositor-side surface change rebuilds the plan mid-run. On the x11 driver the drawable arrives pre-shrunk by Mutter decoration chrome (`1853x1011` inside a `1903x1098` frame; horizontal clamp matched Ubuntu Dock's reserved `67px`, dock-fixed) so declared extents can never match. Neither mutter features, extensions (all disabled for a probe), dock autohide nor session restart removed either effect; user environment was restored afterwards | The blocker is an interaction between this desktop session's late surface reconfiguration/decorations and the strict single-frame-plan-build invariant — not engine rendering correctness. Next escalation level is adapter-internal instrumentation of the plan-rebuild trigger (which input changed), or a product-owner decision on that invariant; do not rerun blind driver/extension probes |

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

Run the final evidence campaign on exact clean commit `df964af2` (release
xtask already built there; verify provenance before starting): for each of
R2 (`r2-alpha-render`), R4 (`r4-100npc`), R5 (`r5-physics-16`) and R3
(`r3-multiregion-streaming`), collect a ten-run calibration set with
admission-timed runs, publish the baseline, then run one isolated fixed
three-run gate against it. Collect R2 first while the freshly fixed
presentation path is stable; any cleanly assembled negative verdict is
immutable and stops that workload per policy. After all four gates `PASS`,
close B-12 in `docs/roadmap.md`, mark R7c complete, and hand off to R7d.

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

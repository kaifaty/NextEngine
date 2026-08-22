# Nonlocal nonlinear solver research -- current task state

| Field | Value |
|---|---|
| Status | `ACTIVE / NSR3B4E2D7R_FAIL_INNER_FLOOR / B4E2D7R1_RESEARCH / SHARED_HOST_PERFORMANCE_STOP` |
| Updated | `2026-08-22` |
| Task key | `nonlocal-nonlinear-solver-research` |
| Scope | Fundamental solver research over the verified Nonlocal variational objective, isolated from runtime and the stopped SISSM lineage |
| Definition of done | NSR0--NSR6 select a production-roadmap candidate or stop at an exact reproducible boundary |
| Authority | Working context only; Accepted architecture, SPEC-38/ADR-076/ADR-081 and frozen stage contracts outrank this file |

## Resume in 60 seconds

- **Current conclusion:** B4C4 packaging is complete. The B4C4C1 packaged
  runner still reproduces identity `66e318cb...d6f3c` and semantic result
  `b4d52600...550c`; formula/profile/source hashes are unchanged.
- **Current conclusion:** B4D executes twice deterministically at stdout SHA
  `fbeb4040...ddd9` and fails closed at
  `CW-HYDRO-001:MISSING_ARTIFACT`. All three required external `CWREFV1`
  files are absent; `trajectory_started=false` and B4E is not authorized.
- **Current conclusion:** B4DR0 finds neither exact payloads nor the recorded
  adapter diff/source/binary. Matching GCC and prose are insufficient to
  reproduce the historical bytes.
- **Current decision:** select `NEW_REFERENCE_PROFILE_REQUIRED`. Preserve B4D
  FAIL and historical W0I/W1, then freeze a new reproducible external
  generator/profile; never synthesize old hashes or inherit W1 credit.
- **Current conclusion:** B4DR1 R1A passes after explicit provenance
  correction. One originally retained source copy lacked a complete Git
  object database; a verified true-full-clone build reproduces all eight
  static artifacts, and the historical R1C1 build reproduces executable/report
  roots exactly. The incomplete-clone fact remains recorded.
- **Current constraint:** upstream configure writes `Utilities/Version.h` into
  the source tree, and its revision probe rejects linked Git worktrees. Every
  profile build starts from an ordinary clean full clone and binds the
  generated header separately.
- **Current decision:** R1B freezes an independent standalone contact tool,
  separate validator, six parent vectors and internal-face/one-ulp restart
  sentinels. Process preflight must fail before contact on float/OMP/locale/ABI
  mismatch.
- **Current correction:** R1B v2 separates the unit outer-clamp fixture from
  the `[0,2] x [0,1] x [0,1]` orifice box. Rejected v1 placed wall `x=1` on
  the outer face and never reached implementation.
- **Current conclusion:** R1B passes. Two builds reproduce executable SHA
  `c1150fad...a5da`, two processes reproduce stdout SHA `c6a4950d...a8a`, all
  eight contact cases pass, and float/locale/OpenMP mutations reject before
  contact. No trajectory ran.
- **Negative result:** first R1C manifest-only run stopped at
  `CW-DAM-001:FLUID_ROOT`, with `simulation_created=false` and
  `trajectory_started=false`. The shortened ID conflicts with roots computed
  from normative `CW-DAMBREAK-001`; this is not a physics failure.
- **Current decision:** reject R1C identity `a061f43e...99d0` and select the
  narrow R1C1 manifest-identity reclosure `865570e1...8927`.
- **Current conclusion:** R1C1 passes. Two builds reproduce executable SHA
  `c8933e01...b6ca`; two processes reproduce report SHA `6d293328...8f3`;
  all roots/mutations pass and forced mismatch rejects before Simulation.
- **Negative result:** the first authorized Hydro trajectory exits at
  `PRESSURE_NOT_CONVERGED`; report root `1d3f7c4e...67e`, no stderr and zero
  output entries. No repeat, Dam or Orifice ran. R1C and R1D are blocked.
- **Current decision:** freeze R1C2 failure observability only. Preserve all
  physics/profile bytes and run one diagnostic Hydro process exposing the
  failing step, phase, iteration/residual/convergence and time-step bits.
- **Current conclusion:** R1C2 shows step 1 pressure reaches cap 100 with
  residual `0.8274405823` versus threshold `0.1`; divergence converges in one
  iteration with zero residual and timestep bits remain exact.
- **Current decision:** select an ascending one-step R1C3 pressure-cap sweep
  over `25,50,75,100,125,150,200,300`, changing no other profile value.
- **Current conclusion:** R1C3 residual decreases monotonically; cap 300
  first permits convergence at iteration 220 with residual `0.0992042`.
- **Calibration conclusion:** `spacing^3` yields infinite-lattice density
  ratio `0.999972`; upstream's 0.8 startup heuristic yields `0.799978` and
  changes mass by 20%, so it is not selected for this comparator.
- **Current decision:** reclose R1C4 with pressure cap 300 only, new profile
  identity and otherwise byte-identical physics/serialization.
- **Current conclusion:** R1C4 Hydro and Dam pairs pass byte-identically. The
  first Orifice step converges, then contact rejects because analytical
  `x_max` was wrongly derived as 1 m from source-support `boundary_nx=20`.
- **Current decision:** R1C4 fails overall. R1C5 separates domain extent from
  boundary lattice extent, keeps Orifice support unchanged and uses a new
  global profile identity.
- **Current conclusion:** R1C5 passes all three pairs byte-identically. All six
  processes exit zero with empty stderr; Orifice completes with 28 receiver
  samples under analytical `x_max=2.0` and unchanged one-metre source support.
- **Current decision:** R1D uses new schedule-consistent scenario manifests,
  unchanged frame format/physics and separate q99-x/q99-y/receiver roots.
  Independent one-thread scenarios run in two waves of at most three.
- **Current conclusion:** R1D passes. All full scenario reports/payloads are
  pairwise byte-exact and published as verified regular content-addressed
  files. Orifice ends with 1,172 receiver samples.
- **Performance fact:** two three-process waves reduce harness wall by about
  `1.58x`; Hydro nevertheless takes 7:43 per 1,200-step process, exposing a
  real late-state DFSPH reference cost rather than I/O or memory starvation.
- **Current decision:** R1E uses a separate no-SPlisHSPlasH C++17 reader with
  descriptor-safe path admission, full independent parse, canonical decoded
  root, regenerated aggregates and serialized/decoded mutation controls.
- **Current conclusion:** R1E passes. Two builds reproduce reader SHA
  `8c4e7d61...55ea`; two positive processes reproduce report SHA
  `60e5575b...630e`; all semantic/aggregate/mutation gates pass and four
  external negative fixtures reject deterministically.
- **Current decision:** select `NEW_EXTERNAL_DFSPH_REFERENCE_CANDIDATE` and
  authorize only B4E nominal-corpus research and contract design.
- **Current conclusion:** B4E research finds exact Hydro/Dam initial geometry,
  constants and macro-step alignment, but the packaged entry point still owns
  only tiny P1/P2 fixtures. Per-particle and solver-iteration comparisons are
  rejected; canonical q99/COM/curve aggregates are selected.
- **Current decision:** use a B4E0 zero-trajectory alignment gate, then a
  one-macro cost probe and first-output pilots before any full run. Orifice is
  deferred to B4O; a projected combined cost above four machine-hours routes
  to B4EP optimization without becoming a physics failure.
- **Current conclusion:** B4E0 passes twice and across two identical builds.
  Hydro/Dam need at most 118/117 neighbors, about 47 MiB RSS and 0.4 s for the
  zero-step preflight; all roots and mutations are exact.
- **Numerical diagnostic:** Hydro has nine active centres at only
  `6.6613381477509392e-16` positive strain. This is machine-floor branch
  sensitivity, not pressure evidence; B4E1 must show its spectral cost.
- **Current decision:** split the resource probe. Because epsilon-active Hydro
  forces the 48-HVP spectrum, B4E1S measures that path before any KKT work.
  The derived initial count must be at most 96 so an adjacent fine level can
  remain within the selected 192-substep cap.
- **Current conclusion:** B4E1S passes across two independent builds and two
  fresh processes. Four bit-exact 48-HVP estimates give maximum eigenfrequency
  `499.43728723929792 s^-1` and 14 initial substeps, with zero all-pairs work.
- **Performance fact:** a process containing two complete B4E1S estimates
  takes 0.51 s wall and about 64 MiB RSS at 99% CPU. This is spectrum preflight
  cost, not macro-step or production throughput.
- **Current decision:** B4E1M reuses the complete retained-flat adaptive
  transaction at exact nominal Hydro. It may attempt only `14,28,56,112`,
  commits one selected fine step, and runs once per fresh process under an
  external 900-second watchdog.
- **Current conclusion:** B4E1M passes byte-exactly across two builds/processes.
  Levels 14/28 pass, strain is `4.55e-4`, energy creation and penetration are
  zero, and the strict ledger residual is `1.55e-11`.
- **Performance boundary:** the macro takes 48.83/48.80 s at 99% of one CPU
  core. It performs 221 outer trials, 459 total HVPs and 227 flat workspace
  builds, materializing 151,461,068 directed records. One unrepeated full
  Hydro+Dam pair projects to about 26 machine-hours versus the 4-hour gate.
- **Current decision:** hold B4E2 execution and route early to B4EP. First
  freeze an attribution profiler; do not choose parallelism, reuse or GPU work
  until measured stage costs identify the dominant paths.
- **Attribution constraint:** process `perf` is blocked by host
  `perf_event_paranoid=4`; do not change the sysctl. B4EP0 uses a separate
  GCC `-pg`/gprof build and requires byte-exact B4E1M stdout correspondence.
- **Current conclusion:** B4EP0 matches B4E1M stdout exactly and records 3,138
  samples. SHA-256 is 41.36% self time; HVP, neighborhood construction and
  evaluation are about 23.4%, 22.7% and 8.3% total respectively.
- **Current decision:** select B4EP1 query-evidence separation first. Keep
  full-state hashing as the oracle/default; a work-only nominal transaction may
  skip inner workspace/pair/tape hashes but must reproduce all physics roots
  and counters exactly in Release.
- **Current decision:** the frozen B4EP1 work-only policy skips only transient
  workspace/pair/tape hashes, retains a small domain-separated work chain and
  leaves parent/final publication evidence unchanged.
- **Current conclusion:** B4EP1 passes byte-exactly across both builds. All
  three alternating timing pairs win; median paired speedup is `3.0168x`,
  median wall falls from 48.74 s to 16.15 s and median RSS from 93,060 KiB to
  62,016 KiB.
- **Current constraint:** this removes only transient research evidence cost.
  It changes no physics and does not make the remaining 16.15 s macro
  production-ready; HVP and topology remain live residual categories.
- **Current decision:** B4EP2 freezes one exact-output GCC/gprof run over the
  work-only command. It ranks HVP, topology/CSR, evaluation/tape and nonlinear
  bookkeeping without changing source or host policy.
- **Current conclusion:** B4EP2 matches B4EP1 stdout exactly. Workspace
  construction is 58.33% inclusive (40.97% topology, 16.84% evaluation/tape),
  HVP is 39.64% and SHA is 0.46% of 1,728 samples.
- **Preserved negative:** NP1-P4 stopped because anchor-cell order changed
  f32 association after a cell crossing. Current CPU/f64 pairs are explicitly
  lexicographically sorted, but B4EP3 must prove this distinction rather than
  inherit credit.
- **Current decision:** B4EP3 uses fixed `0.04h` skin and the conservative
  `4*d_max^2` certificate. It is an offline audit over the captured query
  sequence, not a solver cache implementation.
- **Current conclusion:** all 227 B4EP3 states match exact pair/CSR/evaluation/
  tape. One superset build serves 226 reuses; maximum degree is 122,
  candidate/active visits `1.0689` and construction-work ratio `0.1998`.
- **Current decision:** B4EP3I is transaction-local and query-trace-owned;
  defaults remain null, parent remains canonical/full-state, and certificate
  or capacity errors fail closed without fallback.
- **Current conclusion:** B4EP3I preserves exact B4EP1 physics and old report
  bytes. One superset build serves 225 certified reuses; all three timing pairs
  win with median paired speedup `1.5899x`, and median wall falls from 16.71 s
  to 10.40 s.
- **Next action:** reprofile the exact cached command in B4EP4 and distinguish
  residual HVP, evaluation/tape, superset filtering/CSR and nonlinear-control
  cost before selecting another implementation.
- **Current decision:** B4EP4 uses one exact-output GCC/gprof profile and
  selects a next design only when one comparable category leads by at least
  `1.20x`; otherwise it routes to scoped internal phase timing.
- **Current conclusion:** B4EP4 matches cached stdout exactly and records 1,112
  samples. HVP is 62.14% inclusive versus 35.52% complete cached workspace;
  its `1.749x` lead selects B4EP5 HVP research/design only.
- **Current decision:** B4EP5 optionally caches only `weight_gradient(radius)`
  and `weight_second(radius)` once per pair/tape. All vector arithmetic,
  traversal and reduction order remain unchanged; defaults remain empty.
- **Current conclusion:** B4EP5 preserves exact physics and all old command
  bytes. Three of three Release pairs win; median wall falls from 10.61 s to
  8.50 s (`1.2482x`) with zero coefficient mismatch/fallback.
- **Next action:** freeze one B4EP6 exact-output residual profile of the B4EP5
  command before selecting another implementation. Do not stack changes.
- **Current decision:** B4EP6 compares HVP, complete workspace and residual
  control with a `1.20x` leader rule; workspace has three explicit
  subcategories. No leader routes to internal phase timing.
- **Current conclusion:** exact B4EP6 profiling selects workspace over HVP
  `1.3507x`, then evaluation/base tape over topology/CSR `2.5046x`.
- **Next action:** audit a single exact evaluation/base-tape discriminator;
  do not implement fusion or another cache before its arithmetic/ownership
  boundary is frozen.
- **Current decision:** B4EP7D counts active directed records only after each
  valid tape, derives `2N+D -> N` radius and `N+D -> N` gradient work, and
  performs no fusion or timing.
- **Current conclusion:** B4EP7D freezes `D=131,987,230`; projected exact
  fusion removes 71.75% radius and 60.63% gradient-kernel evaluations.
- **Next action:** freeze a B4EP7I exact fused evaluation/tape A/B contract;
  implementation must follow only after its operation/ownership boundary.
- **Current decision:** B4EP7I uses one canonical pair pass and one unchanged
  center/adjacency pass, moves CSR only after validation, and fails closed.
- **Current conclusion:** B4EP7I is bit-exact and wins 3/3 pairs; median wall
  falls 8.90 -> 8.00 s with median paired `1.1111x`. Margin is modest.
- **Next action:** reprofile the exact fused command before another change;
  do not generalize the nominal result to production or a broad corpus.
- **Current decision:** B4EP8 uses one exact-output gprof run and a `1.20x`
  leader rule across HVP, fused workspace and control; no leader selects
  internal phase timing.
- **Current conclusion:** workspace/HVP are 3.61/3.59 s (`1.0056x`), and
  inlined fused pair/center work is not separable by gprof. No optimization is
  selected.
- **Current decision:** B4EP9 adds opt-in transaction-only steady-clock
  timers with exact call counters and excludes all durations from the semantic
  result. Three fresh processes route only to CPU-parallel architecture
  research or serial-residual research using the frozen `0.80/0.75/0.05`
  median/minimum/range gate.
- **Current conclusion:** all three B4EP9 Release processes preserve semantic
  result `44e93e6e...9cd72`; parallelizable fraction is 92.14--92.22%, median
  92.19%, with only 0.08 percentage-point range. All six old-command stdout
  hashes remain exact.
- **Current decision:** select B4EP10 deterministic CPU-parallel architecture
  research only. Freeze ownership, partition, reduction, scheduling, failure
  and A/B gates before implementing threads.
- **Current decision:** select 64 fixed logical partitions and owner-computes
  gathers. Reject atomics and per-worker floating partials because neither
  preserves the serial addition order. OpenMP/static is only a future research
  backend after a serial dataflow proof.
- **Current conclusion:** B4EP10D reproduces all 226 topology, 226 evaluation
  and 459 HVP oracle results bit-for-bit. Zero mismatch/fallback occurs across
  485,915,712 HVP target gathers; maximum added payload is 29,557,700 bytes.
- **Current decision:** B4EP10I uses research-only OpenMP, 64 fixed logical
  partitions, `schedule(static,1)`, explicit `1/2/4/8/16` workers and
  unique-owned output slices. Prefix/metadata, owner-plan construction and
  energy fold remain serial initially.
- **Current conclusion:** B4EP10I passes at all five worker counts with common
  correspondence SHA `917a04d3...b4ca`, zero executor mismatch and a
  byte-identical 16-worker repeat. No timing claim was made.
- **Current decision:** B4EP10S uses two unmeasured warmups and three balanced,
  serialized S/1/2/4/8/16 rounds pinned to distinct physical cores. It selects
  the smallest count within 3% of the fastest median only after speed,
  utilization, stability and exactness gates.
- **Current conclusion:** B4EP10S selects 8 workers. Median wall falls from
  7.313827799 s serial to 5.912456915 s (`1.237020x`) with 6.421 effective
  cores; 16 workers are only 1.70% faster at 12.488 effective cores.
- **Performance problem:** owner-computes at one worker takes 11.416808981 s,
  exposing about 4.10 s of transpose/region/memory overhead before scaling.
  The result is far below B4EP9's idealized parallel-fraction ceiling.
- **Current decision:** B4EP10R uses one unmodified gprofng 2.46 8-worker
  profile with 1 ms clock samples and native synchronization tracing. Exact
  output is required; profile overhead grants no speed credit.
- **Negative result:** B4EP10R target output is exact, but the collector says
  its interval changed from 1000 us to zero and marks the profile unreliable.
  The 60.91% anonymous libgomp sample is diagnostic only and grants no route.
- **Current conclusion:** B4EP10R1 passes three exact processes. Median
  orchestration/imbalance shares are only 1.11%/6.55%, while evaluation leads
  HVP by `1.916851x`; `evaluation_plan` is 42.33% of evaluation.
- **Current decision:** research one B4EP10P architecture for the active
  owner-plan plus canonical energy-fold subphase.
- **Research conclusion:** repeated plan construction performs at least
  546.8M record visits versus 1.356M centre energy-fold visits. The existing
  225-reuse Verlet superset can own one fixed target CSR if stable active-mask
  filtering reproduces every current target row exactly.
- **Current conclusion:** B4EP10PD passes twice byte-identically. One fixed
  705,284-slot plan reproduces all 226 active row sets; full-scan expansion is
  `1.288507x`, conservative added payload 38,044,404 bytes and both corrupt
  structure negatives reject.
- **Current decision:** B4EP10PI keeps the fixed plan cache-owned, mapping
  workspace-owned and canonical energy fold unchanged. It must eliminate all
  226 active plan builds and pass three balanced external A/B pairs.
- **Negative result:** B4EP10PI is exact and wins all three pairs, but median
  paired speedup is only `1.030796x`, below 1.05. RSS falls 5,244 KiB while
  median user CPU rises from 37.31 to 39.27 s due to full masked scans.
- **Current decision:** retain B4EP10I active plans. Research deterministic
  parallel construction of only the active transpose; do not relax the gate.
- **Research conclusion:** use stable counting-sort/CSR transpose with 64
  fixed contiguous source partitions, target-owned integer prefix/base work
  and no atomics. The bounded matrix is 3,026,944 bytes.
- **Current decision:** B4EP10PCD builds the partitioned plan beside all 226
  serial plans, compares all arrays byte-exactly and admits no timing.
- **Negative result:** B4EP10PCI is exact and wins all three pairs, but its
  `1.033650x` median speedup misses the frozen `1.05x` gate while CPU rises.
- **Negative result:** the exact full current-topology plan scans `1.213341x`
  retained work and misses its frozen `1.20x` structural gate.
- **Current conclusion:** split self/incoming reconstruction and the
  pair-endpoint incoming builder both pass exact audits over all 226 plans.
- **Current conclusion:** B4EP10SII preserves all physics/work roots, wins
  `3/3` A/B pairs and passes narrowly at `1.052521x` median speedup. Median
  wall falls 5.820 -> 5.537 s and RSS falls 4,852 KiB, while total CPU rises.
- **Current decision:** select split incoming only for the nominal research
  path, retain B4EP10I as rollback and remeasure the candidate residual before
  another code change.
- **Current decision:** B4EP10SIR reuses the exact hierarchical timers and
  reconstructs disjoint topology, source-local, target-fold and control
  categories across three fresh processes. Route only through its frozen
  stability and `1.20x` leader rule.
- **Current conclusion:** B4EP10SIR passes three exact processes. Source-local
  work has median transaction share `0.587730` and leads topology `2.918189x`;
  every category-share range is below `0.0018`.
- **Rejected routes:** median executor orchestration/imbalance are only
  `0.011919/0.071954`, so persistent-region and partition-balance work are not
  selected. Research one narrower source-local discriminator before code.
- **Current decision:** B4EP10SIRD reuses the unchanged timing command and
  reduces source-local into directed, setup, compression and local-scalar
  groups. Execute three fresh processes; no implementation change is allowed.
- **Current conclusion:** B4EP10SIRD passes. Directed evaluation/HVP work has
  median transaction share `0.389073`, range `0.000870`, and leads setup by
  `4.094282x`.
- **Current decision:** research one timing-free directed scratch-liveness
  audit. Do not infer arithmetic dominance from a phase that also contains
  allocation and value-initialization.
- **Current decision:** B4EP10SIRDA proves active-slot write-before-read,
  inactive-slot liveness, target assignment and transaction-local high-water
  capacity without changing the returned path. Implement/run the audit only.
- **Current conclusion:** B4EP10SIRDA passes byte-identically. A full 454.9M
  slot initialization stream reduces structurally to 670,229 high-water growth
  slots (`0.001473x`); all 374.9M active writes have exactly two valid reads.
- **Current decision:** research one opt-in transaction-local directed scratch
  reuse path and external A/B contract. Do not fuse arithmetic or expand scope
  to compression/target buffers.
- **Current decision:** B4EP10SIRDI owns one transaction-local high-water
  directed buffer, releases it on every exit, and changes no compression,
  target or arithmetic path. Implement then run the frozen balanced A/B.
- **Current conclusion:** B4EP10SIRDI is exact and wins `3/3` pairs. Median
  wall falls 5.514 -> 4.290 s (`1.284223x`), total CPU ratio is `0.784012`,
  candidate range `1.002851` and RSS delta only +496 KiB.
- **Current decision:** select directed scratch reuse for nominal research,
  retain B4EP10SII/B4EP10I as rollbacks, and remeasure the candidate residual
  before changing compression, target buffers or arithmetic.
- **Current conclusion:** B4EP10SIRDIR passes three exact processes.
  Source-local remains the stable leader at `0.447174`, but directed and setup
  are now close; executor orchestration/imbalance remain below `0.15`.
- **Current decision:** split `evaluation_setup` validation, capacity/control
  and buffer preparation with timing only before changing ownership or code.
- **Current conclusion:** B4EP10SIRDIRE passes three exact processes. Buffer
  preparation has median setup share `0.894957`, leads validation by
  `8.531659x`, and every setup-share range is below `0.009`.
- **Current decision:** research one timing-free write-before-read,
  ownership-lifetime and high-water audit for the seven evaluation buffers.
  Do not implement reuse or remove initialization from timing alone.
- **Current decision:** B4EP10SIRDIREA freezes per-index coverage, exactly two
  returned-workspace lanes, one builder-local ephemeral lane, explicit receipt
  release and checked high-water projection. Implement/run only this audit.
- **Current conclusion:** B4EP10SIRDIREA passes byte-identically. All seven
  roles are fully overwritten; 226 workspace receipts peak at two live, the
  ephemeral lane peaks at one, and both finish at zero live receipts.
- **Performance projection:** two workspace lanes plus one ephemeral lane grow
  22,067,592 bytes versus 2,828,746,176 repeated initialization bytes
  (`0.007801x`). This is work projection, not speed credit.
- **Current decision:** research/freeze one opt-in two-lane workspace plus one-
  lane density-contribution reuse implementation/A-B contract. SIRDI remains
  rollback until exact external A/B passes.
- **Design correction:** an ordinary vector pool cannot preserve the audit's
  high-water initialization projection when pair `.size()` shrinks and grows;
  keeping high-water size would change tape semantics. Do not implement that
  pool in this bounded candidate.
- **Current decision:** B4EP10SIRDIREI freezes candidate-only overwrite
  construction for six audited `double` roles, leaving gradient, sizes,
  ownership and arithmetic unchanged. Implement then run the balanced A/B.
- **Negative result:** the stateful allocator implementation keeps bytes exact
  but changes libstdc++ to elementwise construction. Unchanged SIRDI regresses
  4.290 -> 10.43 s (`2.431x`); candidate 9.63 s is not admissible speed credit.
- **Current decision:** close SIRDIREI FAIL and revert it before commit. Do not
  run balanced A/B against a corrupted baseline or widen into raw storage.
- **Current decision:** research only one transaction-local high-water reuse
  path for builder-local density contribution. It changes no returned vector
  size, workspace ownership or arithmetic.
- **Current decision:** B4EP10SIRDIREP freezes one ordinary high-water
  `std::vector<double>` for density contribution only, plus explicit baseline
  health before relative A/B. Implement and measure only this candidate.
- **Negative result:** the candidate is exact and wins `3/3` at median
  `1.194152x`, but baseline median `4.893718116 s` exceeds `4.72 s` and the
  candidate range ratio `1.174767` exceeds `1.10`.
- **Current decision:** close SIRDIREP FAIL and revert it. Retain exact SIRDI,
  stop the complete evaluation-buffer initialization branch and route only
  from unchanged SIRDIR residual evidence.
- **New uncertainty:** accepted SIRDI used 31.58 s median total CPU, while the
  later exact default uses about 36 s. Three dormant instrumentation/audit
  additions may have changed default-path cost despite byte-exact output.
- **Current decision:** B4EP10SIRDIREQ freezes independent accepted/current
  source builds and balanced same-command qualification. Do not optimize a
  residual until code drift is confirmed or rejected.
- **Qualification result:** both checkpoints are exact and current is not
  slower in paired wall/CPU ratios, but accepted median `4.801272953 s`
  exceeds its `4.72 s` host-health gate.
- **Current decision:** do not source-bisect or run short-margin wall A/B on
  this shared desktop host. Research an opt-in process/thread CPU-time residual
  attribution lane; it may route structural work but grants no wall credit.
- **Current decision:** B4EP10SIRDIREQ1 freezes process CPU clocks for the
  existing disjoint phases and thread CPU clocks for all worker-active
  intervals. Implement/run only this timing command; exclude stopped
  evaluation setup from routing.
- **Measurement result:** Q1 passes `3/3`; all CPU-share ranges are within
  `0.03`, GNU/internal CPU ratio median is `1.007535`, and exact result is
  `e5ddff76...14f`.
- **Current decision:** topology is the eligible leader at median `0.301239`
  and `1.629380x` over target fold. Research and freeze one timing-free
  topology structural audit; do not implement a topology change yet.
- **Research result:** the accepted incoming builder performs 665,142,896
  standalone entry visits and 678 regions after topology already owns the
  needed pair order, row counts and slots.
- **Current decision:** B4EP10SIRDIREQ2 freezes one shadow fusion audit. It
  piggybacks degree/source/endpoint data on existing topology passes and uses
  one 85,716,150-pair canonical fill; implement the audit, not the fast path.
- **Measurement result:** Q2 passes `2/2` byte-identically. All 226 plans are
  exact, standalone work ratio is `0.12886877468807845`, added regions are
  zero and combined payload is 21,758,020 bytes.
- **Current decision:** research and freeze one separate opt-in fused-plan
  consumer contract. Preserve SIRDI and do not use unqualified wall timing as
  speed evidence.
- **Research result:** Q3 selects direct publication through the existing
  neighborhood-to-tape path; the audit trace is not a production data owner.
- **Current decision:** implement two opt-in Q3 commands. First require two
  exact candidate processes; only then run one balanced process-CPU A/B with
  no wall-speed credit.
- **Measurement result:** Q3 exact passes twice, but CPU A/B wins only `1/3`;
  median paired speedup is `0.983844x` and paired range ratio `1.565388`.
- **Current decision:** close Q3 FAIL and revert the candidate/harness. Retain
  SIRDI and Q2 structural evidence. Research measurement-lane qualification
  before another performance implementation; do not rerun Q3.
- **Research result:** in Q1, transaction CPU range is `1.248912x` and
  in-region non-active residual range is `1.999653x`, while
  `transaction - region + active-worker` ranges only `1.012483x`.
- **Current decision:** Q4 freezes a no-code qualification of this
  executor-adjusted CPU surrogate over three fresh existing Q1 commands. It
  grants no wall credit and cannot reopen Q3.
- **Measurement result:** Q4 exact/accounting and `O/A` gates pass, but
  adjusted CPU range is `1.031236x` versus the frozen `1.03` limit.
- **Current decision:** close Q4 FAIL without repeat. No further CPU/wall
  candidate A/B is admissible on this shared host; a dedicated/quiescent
  performance lane is required. Fundamental non-speed research may continue.
- **Current decision:** separate reference extraction from physical execution.
  B4E2R freezes a standalone timing-free Dam-step-4/Hydro-step-24 slice
  extractor with complete-hash admission, canonical micrometre aggregates and
  mutation controls. It runs no Nonlocal trajectory.
- **Current conclusion:** B4E2R passes across two independent Release builds
  and fresh processes. Both emit report SHA `6f2d0ffb...40f1`; Dam step 4
  aggregate is `b8ad20e8...750c` and Hydro step 24 is `d9a113a3...3c47`.
- **Current decision:** B4E2D freezes a four-macro Dam-first pilot with exact
  decoded-canonical handoff, global roots, cumulative physical budgets and
  the independent step-four centre/q99 envelope.
- **Architecture correction:** retain one immutable static support index across
  all four steps, but make the certified dynamic topology cache transaction-
  local. The reference displacement exceeds its cross-step anchor limit and
  certificate failure has no fallback.
- **Negative result:** B4E2D process A exits before the trajectory because one
  alignment predicate is false, then its failure report calls trajectory-root
  on an empty prefix and throws. Stdout is empty, stderr SHA is
  `b469c089...08af`; process B was not run.
- **Current conclusion:** B4E2D0 passes twice byte-identically and isolates
  only pair root/count drift: decoded frame zero has `342502/611520/120`
  versus raw-lattice B4E0 `335814/596256/117`.
- **Binary64 diagnostic:** all 6,000 external Dam frame-zero positions match
  integer-micrometre decode bit-for-bit, while only 384 also match the current
  addition-built lattice. Horizon-boundary pairs expose the one-ulp change.
- **Current conclusion:** B4E2D1 passes across two builds/processes. External
  root `0d567ba5...74d7` equals micrometre division for all 6,000 vectors and
  differs from addition root `b7063e2b...8dc4`; old first-output bytes remain
  exact.
- **Current conclusion:** B4E2D2 passes twice. Addition/decoded share 315,522
  pairs with `20,292/26,980` one-sided pairs, all within five epsilons of the
  horizon; both initial pressure evaluations are exactly inactive.
- **Negative result:** B4E2D3 step one commits exactly. Step two solves and
  publishes at 80 accepted substeps, but maximum positive density strain is
  `0.0011747197409319732`, above the frozen `0.001` material gate. Its KKT,
  penetration, closure and work gates pass. Process B was correctly skipped.
- **Current decision:** preserve B4E2D3 FAIL. Research and freeze a fixed
  80/160/320-substep step-two discriminator from the exact committed step-one
  state. Do not change `KAPPA`, tolerances, formulas or physical gates.
- **Next action:** distinguish premature temporal admission from converged
  finite-penalty compressibility before any controller or model redesign.
- **Current decision:** B4E2D4 freezes exact reproduction plus private fixed
  80/160/320 step-two lanes. Existing 160/320 state convergence and a
  predeclared `5e-5` peak-strain delta route controller admission, finite
  penalty compressibility or unresolved temporal error.
- **Current conclusion:** B4E2D4 passes twice byte-identically and selects
  `FINITE_PENALTY_COMPRESSIBILITY`. Private strain converges from
  `0.0011740875` at 80 to `0.0011739238` at 320; the 160/320 delta is only
  `5.67e-8`. Publication adds `6.32e-7` but is not the cause.
- **Next action:** research a formulation discriminator comparing the minimum
  sufficient bulk-penalty increase and its stiffness cost with constrained or
  augmented-Lagrangian incompressibility. Do not alter the adaptive controller.
- **Architecture result:** penalty-only pressure is identically zero at zero
  compression and its required coefficient scales linearly with pressure/head.
  Select a unilateral PHR augmented-Lagrangian pressure state for the next
  scalar discriminator; retain semismooth primal-dual as fallback.
- **Next action:** implement/run B4E2D5 without a trajectory or physics
  mutation. PASS may authorize only a tiny dense AL oracle contract.
- **Current conclusion:** B4E2D5 passes twice byte-identically. PHR recovers
  the penalty at zero multiplier and carries `11.516 kPa` at zero strain with
  `lambda=1.439524 J`, exact feasibility and complementarity. Select explicit
  augmented-Lagrangian pressure state as a new solver identity.
- **Next action:** research/freeze a tiny dense AL oracle with multiplier
  convergence, warm-start, inactive-state and rollback controls. No nominal
  trajectory or runtime pressure schema is authorized.
- **Scope correction:** B4E2D6 first freezes an eight-constraint, one-primal-
  DOF path oracle over the true B2 density kernel. This isolates multiplier
  convergence before a complete dense vector AL oracle.
- **Next action:** implement/run B4E2D6 with analytic density-path derivatives,
  bracketed inner solves, cold/warm/inactive/reset/rollback controls and no
  trajectory.
- **Current conclusion:** B4E2D6 passes twice byte-identically and selects
  `AL_PATH_VIABLE`. Cold start reaches `1.58e-11` primal violation in three
  outer updates; warm start needs one. Inactive, reset, rollback and mutation
  controls pass over the true eight-centre density path.
- **Next action:** research/freeze B4E2D7 as a dense-vector AL oracle with
  analytic gradient/HVP and trust-region inner solve. Keep nominal Dam and
  runtime pressure persistence blocked.
- **Current decision:** B4E2D7 freezes 24 primal coordinates, eight pressure
  multipliers, analytic full-curvature AL HVP, dense derivative correspondence
  and trust/outer residual gates over the immutable B2 fixture.
- **Negative result:** B4E2D7 derivatives, full-curvature inner solves and all
  cold/inactive/reset/rollback controls pass. Cold primal decreases
  monotonically to `1.67e-9`, but one further warm update changes the pressure
  multipliers by `3.50e-7 J`, above the frozen `1e-8 J` state gate.
- **Current decision:** preserve B4E2D7 FAIL. Its cold gate mixed
  `delta_lambda/beta` with an absolute-joule warm correspondence gate, so it
  admitted a pressure state before that state was stable at commit precision.
  Do not tune `beta`, relax the warm gate or select semismooth from this result.
- **Next action:** research/freeze B4E2D7R with one absolute multiplier-update
  admission and a private confirmation update under a new identity. It must
  reproduce all B4E2D7 derivative facts and the first eight cold records.
- **Current decision:** B4E2D7R freezes `<=1e-8 J` absolute multiplier and
  `<=1e-8 dx` position-update admission, followed by one complete private
  confirmation update. Only the confirmed state may commit; total cap is 14.
- **Negative result:** B4E2D7R reproduces the D7 prefix exactly and keeps the
  primal sequence monotone, but outer 9 accepts a zero-work inner result at
  stationarity `5.24e-9 <= 1e-8`, then changes `lambda` by `3.50e-7 J` again.
  The following inner solve reaches `REJECT_LIMIT`; zero state commits.
- **Current decision:** preserve B4E2D7R FAIL as `INNER_ACCURACY_FLOOR`. Do
  not lower the tolerance or increase the outer cap without resolving whether
  raw objective subtraction can represent the required correction.
- **Next action:** research/freeze a replay-only B4E2D7R1 observability oracle
  over the exact post-outer-9 state and failed trust trials.
- **Do not run:** B4E2D/H or broader B4E corpus, CUDA, runtime/schema, PhysX
  coupling, persistence or production work before a preflight reclosure.

## Current selected lineage

| Boundary | Selected result | Allowed claim |
|---|---|---|
| NSR0--NSR2C2 | analytic full HVP, safeguarded trust-region Newton-CG and numerical-floor stop | full-curvature tiny/scaling solver candidate |
| NSR3A/A1/A2 | serial baseline, allocation-free HVP workspace and outer-state Hessian tape | exact CPU research implementation and local cost evidence |
| NSR3B0R | `nuv-variational-fcr2` lattice-normalized cubic objective | formula/density/gradient/HVP correspondence |
| NSR3B1S3/R1 | embedded controller with selected-fine ownership | bounded report-only temporal controller |
| NSR3B2--B3R | split static support/contact plus owned-residual globalization | tiny static-boundary composition candidate |
| NSR3B4B2 | feasible contact-onset pressure forecast | tiny P1/P2 pressure/contact candidate |
| NSR3B4C3MC1 | balanced macro publication and tiny physical accuracy | P1/P2-only adaptive macro candidate; no temporal-equivalence claim |
| NSR3B4C4A/B/C | retained workspace, immutable support index and flat CSR ownership | packaged complete adaptive/fixed research runner |
| NSR3B4D | fail-closed external reference reader | deterministic missing-artifact boundary only |
| NSR3B4DR1A | reproducible strict external DFSPH library bootstrap | build/toolchain candidate only; no adapter or trajectory |
| NSR3B4DR1B | reproducible standalone contact/validation adapter | contact algebra and ABI gate only; R1C design authorized |
| NSR3B4DR1C contract | frozen manifests, source patch and short-trajectory format | manifest-only implementation; trajectory conditional on preflight PASS |
| NSR3B4DR1C1 | corrected normative dam scenario identity | repeat manifest-only gate; no solver object yet |
| NSR3B4DR1C1 PASS | reproducible zero-physics manifest preflight | R1C trajectory implementation only; no full schedules |
| NSR3B4DR1C trajectory | first Hydro rejects at pressure convergence with no payload | R1C/R1D blocked; do not advance scenarios |
| NSR3B4DR1C2 contract | observability-only diagnostic reclosure | one Hydro diagnostic; no solver tuning or R1C credit |
| NSR3B4DR1C2 PASS | step-1 cap hit isolated to pressure; divergence/dt exact | R1C3 cap-sweep design only |
| NSR3B4DR1C3 contract | fixed one-step pressure-cap sweep | diagnose bounded convergence; no payload or R1C credit |
| NSR3B4DR1C3 PASS | monotone residual; first convergence at iteration 220 | R1C4 cap-300 profile reclosure only |
| NSR3B4DR1C4 contract | cap 300 and new trajectory identity | paired short scenarios; R1D still blocked |
| NSR3B4DR1C4 execution | Hydro/Dam exact; Orifice fails before contact at extent mismatch | R1C4 FAIL; no R1D |
| NSR3B4DR1C5 contract | explicit domain/support extent ownership | rerun all pairs under new identity |
| NSR3B4DR1C5 PASS | all short pairs exact; Orifice reaches 28 receiver samples | R1D full generation only |
| NSR3B4DR1D contract | full schedule manifests, aggregate roots and verified publication | implement preflight/generator, then run full pairs |
| NSR3B4DR1D PASS | three full pairs exact and content-addressed | R1E contract design only |
| NSR3B4DR1E contract | actual reference closure plus independent fail-closed reader | implement/attest only; no B4E execution |
| NSR3B4DR1E PASS | independently decoded full DFSPH references and deterministic negative controls | new reference candidate; B4E contract design only |
| NSR3B4E research | staged Hydro/Dam aggregate comparison and cost ladder | B4E0 alignment preflight only; Orifice remains B4O |
| NSR3B4E0 contract | exact zero-trajectory nominal alignment | implement preflight only; B4E1 still blocked |
| NSR3B4E0 PASS | exact 6k Hydro/Dam inputs and capacity-valid flat neighborhoods | B4E1 one-macro contract design only |
| NSR3B4E1S contract | isolate nominal Hydro 48-HVP spectrum and temporal capacity | implement/execute spectrum only; no KKT or trajectory |
| NSR3B4E1S PASS | deterministic 48-HVP spectrum; 14 initial substeps | B4E1M one-macro contract design only |
| NSR3B4E1M contract | one nominal retained-flat Hydro transaction | implement/execute step 1 only; reference remains closed |
| NSR3B4E1M PASS | exact physical step-1 macro at 48.8 s serial cost | correctness candidate; B4EP required before B4E2 execution |
| NSR3B4EP0 contract | exact-output gprof attribution of B4E1M | external profile only; no optimization or B4E2 |
| NSR3B4EP0 PASS | SHA 41.36%, HVP 23.4%, neighborhood 22.7%, evaluation 8.3% | B4EP1 query-evidence separation design only |
| NSR3B4EP1 contract | full-state default plus work-only nominal transaction | implement/A-B only; physics and B4E2 unchanged |
| NSR3B4EP1 PASS | exact physics with median `3.0168x` paired speedup | B4EP2 residual attribution/design only |
| NSR3B4EP2 contract | exact-output work-only residual gprof | one external profile only; no optimization/B4E2 |
| NSR3B4EP2 PASS | workspace 58.33%, HVP 39.64%, SHA 0.46% | B4EP3 exact topology-reuse audit design only |
| NSR3B4EP3 contract | fixed-skin 227-state canonical superset audit | implementation/two runs only; no hot-path cache or timing claim |
| NSR3B4EP3 PASS | one build, 226 exact reuses, work proxy `0.1998` | B4EP3I internal cache design/A-B only |
| NSR3B4EP3I contract | transaction-local cached work-only A/B | implement and time only; no default/runtime/B4E2 change |
| NSR3B4EP3I PASS | exact cached transaction and median `1.5899x` paired speedup | B4EP4 residual profiling/design only |
| NSR3B4EP4 PASS | exact-output profile; HVP 62.14% versus workspace 35.52% | B4EP5 HVP research/design only |
| NSR3B4EP5 contract | optional invariant scalar tape over selected topology cache | implement/A-B only; no default/runtime/B4E2 change |
| NSR3B4EP5 PASS | exact candidate and median `1.2482x` paired speedup | B4EP6 residual profiling/design only |
| NSR3B4EP6 contract | one exact-output coefficient-candidate gprof profile | execute profile only; no optimization/B4E2 change |
| NSR3B4EP6 PASS | workspace `4.66s` over HVP `3.45s`; eval/base `2.73s` | B4EP7 evaluation/base-tape research/design only |
| NSR3B4EP7D contract | post-tape derived duplicate-work audit | implement/run twice only; no fusion/timing |
| NSR3B4EP7D PASS | `D=131,987,230`; 71.75% radius and 60.63% gradient removable | B4EP7I exact fusion design/A-B only |
| NSR3B4EP7I contract | transaction-only exact flat evaluation/tape fusion | implement/A-B only; no default/runtime/B4E2 change |
| NSR3B4EP7I PASS | exact fused transaction; median paired `1.1111x` | B4EP8 residual profiling/design only |
| NSR3B4EP8 contract | one exact-output fused residual profile | execute profile only; no optimization/B4E2 change |
| NSR3B4EP8 PASS | workspace/HVP balanced `3.61/3.59s`; fused subphases inlined | B4EP9 scoped internal phase timing only |
| NSR3B4EP9 contract | opt-in non-overlapping transaction phase timers | implement/measure only; no parallel implementation or throughput claim |
| NSR3B4EP9 PASS | stable conservative parallelizable fraction, median `0.921879` | B4EP10 deterministic CPU-parallel architecture research only |
| NSR3B4EP10D contract | serial topology-plan and owner-computes exactness audit | implement/run only; no threads/timing/production claim |
| NSR3B4EP10D PASS | exact `226/226/459` owner dataflow; 28.19 MiB added peak | B4EP10I parallel contract research only |
| NSR3B4EP10I contract | opt-in OpenMP owner-computes at `1/2/4/8/16` workers | implement/correspondence only; timing deferred to B4EP10S |
| NSR3B4EP10I PASS | common exact correspondence at all worker counts; exact 16-worker repeat | B4EP10S serialized scaling contract research only |
| NSR3B4EP10S contract | three balanced serialized physical-core rounds | execute timing only; host-specific selection or serial fallback |
| NSR3B4EP10S PASS | 8 workers, median `1.237020x`, 6.421 effective cores | B4EP10R selected-count residual profiling research only |
| NSR3B4EP10R contract | one exact gprofng clock/sync profile at 8 workers | attribution only; route one next design target |
| NSR3B4EP10R FAIL | exact target, unreliable collection-interval warning | preserve B4EP10S; internal parallel phase timing research only |
| NSR3B4EP10R1 contract | opt-in hierarchical stages plus worker active intervals | implement/run three only; duration has no speed credit |
| NSR3B4EP10R1 PASS | orchestration 1.11%, imbalance 6.55%; evaluation leads `1.916851x` | B4EP10P evaluation-plan architecture research only |
| NSR3B4EP10PD contract | one masked superset target CSR versus 226 active plans | structural/capacity audit only; no timing or fast path |
| NSR3B4EP10PD PASS | exact stable rows; `1.288507x` scan, 38.0 MB conservative peak | B4EP10PI implementation/A-B contract research only |
| NSR3B4EP10PI contract | opt-in cache-plan gather and three balanced A/B pairs | implement/time only; old path remains rollback |
| NSR3B4EP10PI FAIL | exact and `3/3` faster, but median only `1.030796x` | retain B4EP10I; parallel active-plan research only |
| NSR3B4EP10PCD contract | stable 64-partition active CSR transpose audit | implement/compare only; no fast path or timing |
| NSR3B4EP10PCD PASS | exact active transpose across all plans | B4EP10PCI opt-in A/B only |
| NSR3B4EP10PCI FAIL | exact, but median `1.033650x` misses speed gate | retain B4EP10I; current-topology representation research only |
| NSR3B4EP10CTD FAIL | exact full current plan scans `1.213341x` | split self/incoming audit only |
| NSR3B4EP10SID PASS | exact lower/own/upper fold at `1.106670x` projected visits | incoming construction audit only |
| NSR3B4EP10SICD PASS | exact pair-endpoint incoming builder | B4EP10SII opt-in integration/A-B only |
| NSR3B4EP10SII PASS | exact candidate; median `1.052521x`, RSS `-4,852 KiB` | candidate residual timing research only |
| NSR3B4EP10SIR PASS | stable timing; source-local median `58.77%`, `2.918x` lead | one source-local discriminator research only |
| NSR3B4EP10SIRD contract | four disjoint source-local groups from existing timers | run three exact processes; no code or speed credit |
| NSR3B4EP10SIRD PASS | directed median `38.91%`, stable `4.094x` lead | one directed scratch-liveness audit research only |
| NSR3B4EP10SIRDA contract | shadow slot liveness plus high-water work projection | implement/run twice only; no scratch fast path or timing |
| NSR3B4EP10SIRDA PASS | exact liveness; 454.3M repeated init slots removable | directed scratch reuse implementation/A-B research only |
| NSR3B4EP10SIRDI contract | opt-in directed high-water buffer plus balanced A/B | implement/measure only; B4EP10SII remains rollback |
| NSR3B4EP10SIRDI PASS | exact candidate; median `1.284223x`, CPU `0.784x` | candidate residual attribution research only |
| NSR3B4EP10SIRDIR PASS | source-local `44.72%`; setup/direct work no longer has a clear leader | evaluation-setup timing discriminator research only |
| NSR3B4EP10SIRDIRE PASS | buffer setup `89.50%`, stable `8.532x` lead | one buffer liveness/high-water audit research only |
| NSR3B4EP10SIRDIREA contract | seven-role write coverage plus two workspace/one ephemeral lane receipts | implement/run twice only; no reuse or timing |
| NSR3B4EP10SIRDIREA PASS | full coverage; two workspace/one ephemeral lane; projected `0.007801x` initialization bytes | buffer reuse implementation/A-B contract research only |
| NSR3B4EP10SIRDIREI contract | overwrite construction for 345,576,600 audited `double` slots; no pool/size change | implement and run frozen balanced A/B only |
| NSR3B4EP10SIRDIREI FAIL | default SIRDI regresses `2.431x`; candidate-relative probe invalid; code reverted | ephemeral density-contribution scratch research only |
| NSR3B4EP10SIRDIREP contract | one local pair buffer; 85.7M repeated slots -> 380,511 growth slots; baseline-health gate | implement and run balanced A/B only |
| NSR3B4EP10SIRDIREP FAIL | exact and `3/3` faster, but baseline health and candidate stability fail; code reverted | retain SIRDI; different residual discriminator research only |
| NSR3B4EP10SIRDIREQ contract | accepted `f33bf3a` versus reverted `b8a1edd`, same exact SIRDI command | build and run balanced qualification only; no speed credit |
| NSR3B4EP10SIRDIREQ HOST_UNQUALIFIED | accepted median `4.801273 s` misses health; current/accepted wall/CPU `0.977/0.982` | no source bisection or short-margin wall A/B |
| NSR3B4EP10SIRDIREQ1 contract | process/thread CPU clocks over exact SIRDIR hierarchy | implement/run three only; no wall or speed credit |
| NSR3B4EP10SIRDIREQ1 PASS | topology median `30.12%`, stable `1.629x` lead; CPU cross-check `1.0075` | one timing-free topology structural audit research only |
| NSR3B4EP10SIRDIREQ2 contract | topology metadata/row-fill piggyback plus one canonical pair fill | implement/run shadow audit twice; no timing or fast path |
| NSR3B4EP10SIRDIREQ2 PASS | all 226 plans exact; standalone work `0.128869x`; zero added regions | fused-plan consumer implementation-contract research only |
| NSR3B4EP10SIRDIREQ3 contract | direct neighborhood publication; no SICD/fallback; paired process CPU | implement exact stage, then CPU A/B only |
| NSR3B4EP10SIRDIREQ3 FAIL | exact path; CPU wins `1/3`, median `0.983844x`, range `1.565388`; reverted | retain SIRDI; measurement-lane qualification research only |
| NSR3B4EP10SIRDIREQ4 contract | `E = transaction - region + active-worker`, existing exact Q1 command | execute three fresh qualification runs only |
| NSR3B4EP10SIRDIREQ4 FAIL | exact accounting; adjusted range `1.031236x` misses `1.03` | dedicated/quiescent host before another candidate A/B |
| NSR3B4E2R PASS | exact Dam-step-4/Hydro-step-24 canonical reference slices | B4E2D Dam-first contract research only |
| NSR3B4E2D4 PASS | converged step-two strain remains `0.0011739238` | finite penalty, not temporal admission, causes the material-gate failure |
| NSR3B4E2D5 PASS | unilateral PHR supports hydrostatic pressure at zero compression | augmented pressure-state oracle research only |
| NSR3B4E2D6 PASS | actual-kernel scalar path converges with exact controls | dense-vector AL oracle research only |
| NSR3B4E2D7 FAIL | dense derivatives/inner/cold pass; committed multiplier state is not warm-stable | dimensionally consistent commit reclosure only |
| NSR3B4E2D7R contract | absolute pressure-state admission plus private confirmation | implement/run tiny dense oracle only |
| NSR3B4E2D7R FAIL | D7 prefix exact; fixed inner accuracy alternates zero-work admission and reject limit | inner-floor observability research only |

Candidate solver identity remains:

```text
formula  = nuv-variational-fcr2+split-static-boundary-r0
solver   = nuv-newton-krylov-r0+outer-state-hessian-tape-v1
publish  = balanced macro-boundary canonical publication
package  = complete-lane flat-adjacency candidate
```

No current runtime owner, public schema, save format, gameplay mutation or
production authority is created by this lineage.

## Evidence that still constrains work

| Evidence | Result | Instruction |
|---|---|---|
| [FCR3-B2](../nonlocal-continuum-fcr3b2-chebyshev-evidence-2026-08-20.md) | fixed Chebyshev directions become non-descent | never retune/relabel stopped SISSM lineage |
| [B1D1](../nonlocal-nsr3b1d1-floor-oracle-evidence-2026-08-20.md) | temporal stiffness is real below acoustic Courant about one | retain adaptive temporal policy |
| [B4C3TR](../nonlocal-nsr3b4c3tr-fixed-reference-evidence-2026-08-21.md) | per-substep micrometre publication destroys refinement reference | do not restore per-private-substep publication |
| [B4C3P](../nonlocal-nsr3b4c3p-publication-cadence-evidence-2026-08-21.md) | macro publication restores order but old scalar tube fails | retain mixed physical/temporal budget, not coefficient 32 widening |
| [B4C3MC1](../nonlocal-nsr3b4c3mc1-adaptive-accuracy-budget-evidence-2026-08-21.md) | unchanged B4B physical budgets pass on P1/P2 | expand diversity before physical production claims |
| [B4C4C1](../nonlocal-nsr3b4c4c1-complete-flat-adjacency-evidence-2026-08-21.md) | all eight lanes/rollback exact; zero final ownership | B4C4 is closed; keep legacy path as oracle/rollback |
| [B4D](../nonlocal-nsr3b4d-reference-reattestation-evidence-2026-08-21.md) | local identities exact, all external files missing | no nominal trajectory until reference closure is restored |
| [B4DR1A](../nonlocal-nsr3b4dr1a-external-bootstrap-evidence-2026-08-21.md) | strict closure reproduces 8/8 artifacts from a verified full clone after [provenance correction](../nonlocal-nsr3b4dr1a-full-clone-provenance-correction-evidence-2026-08-21.md) | require complete-object `fsck` before configure; never use linked upstream worktrees |
| [B4DR1B](../nonlocal-nsr3b4dr1b-contact-adapter-evidence-2026-08-21.md) | adapter binary/output reproduce and all contact/preflight gates pass | freeze R1C manifests before the first DFSPH step; do not inherit W1 credit |
| [B4DR1C research](../nonlocal-nsr3b4dr1c-trajectory-preflight-research-2026-08-21.md) | warm starts and hidden convergence diagnostics violate the intended profile | use only the tracked equation-preserving patch; pass manifest preflight first |
| [B4DR1C negative](../nonlocal-nsr3b4dr1c-manifest-preflight-negative-evidence-2026-08-21.md) | shortened dam ID contradicts frozen fluid/boundary roots; stopped before Simulation | preserve rejection; use only R1C1 normative ID reclosure |
| [B4DR1C1](../nonlocal-nsr3b4dr1c1-manifest-preflight-evidence-2026-08-21.md) | two builds/reports exact; roots and negative mismatch gate pass without Simulation | apply frozen patch in a fresh clone and implement short trajectory only |
| [B4DR1C trajectory](../nonlocal-nsr3b4dr1c-trajectory-negative-evidence-2026-08-21.md) | first Hydro fails at pressure convergence; current report hides solver fields | preserve FAIL; instrument only R1C2 observability before any tuning |
| [B4DR1C2](../nonlocal-nsr3b4dr1c2-failure-observability-evidence-2026-08-21.md) | pressure hits 100 iterations at `8.2744x` threshold on step 1; divergence and dt exact | sweep pressure cap before changing calibration/tolerance |
| [B4DR1C3](../nonlocal-nsr3b4dr1c3-pressure-cap-evidence-2026-08-21.md) | residual falls monotonically and crosses threshold at iteration 220; physical volume is lattice-normalized | reclose cap 300 only; retain mass/volume and cold policy |
| [B4DR1C4](../nonlocal-nsr3b4dr1c4-trajectory-evidence-2026-08-21.md) | Hydro/Dam pairs exact; Orifice domain extent conflated with source support | preserve partial evidence but grant no pass; separate ownership in R1C5 |
| [B4DR1C5](../nonlocal-nsr3b4dr1c5-trajectory-evidence-2026-08-21.md) | all three pairs byte-exact; corrected Orifice crosses into receiver | execute R1D full schedules; retain all earlier negative evidence |
| [B4DR1D](../nonlocal-nsr3b4dr1d-full-generation-evidence-2026-08-21.md) | all full pairs exact; verified external publication | freeze R1E reader/profile contract over actual roots |
| [B4DR1E](../nonlocal-nsr3b4dr1e-reference-attestation-evidence-2026-08-21.md) | independent reader accepts all full references; both mutation layers and four external negatives reject | design B4E against the new candidate; no execution before a frozen comparison contract |
| [B4E research](../nonlocal-nsr3b4e-nominal-corpus-research-2026-08-21.md) | Hydro/Dam align at input/step level; nominal entry point and cost evidence are missing | execute B4E0 alignment before any trajectory |
| [B4E0](../nonlocal-nsr3b4e0-nominal-alignment-evidence-2026-08-21.md) | exact roots/aggregates/mutations; nominal degrees 118/117; no trajectory | design one-macro Hydro resource probe only |
| [B4E1S research](../nonlocal-nsr3b4e1s-spectrum-research-2026-08-21.md) | epsilon-active parent forces spectrum before KKT; initial count must be <=96 | execute isolated spectrum before one-macro design |
| [B4E1S](../nonlocal-nsr3b4e1s-hydro-spectrum-evidence-2026-08-21.md) | four bit-exact estimates give 14 initial substeps and zero all-pairs work | design one Hydro macro transaction only |
| [B4E1M research](../nonlocal-nsr3b4e1m-hydro-macro-research-2026-08-21.md) | complete transaction can isolate levels `14,28,56,112`, fine commit and cost | execute one step-1 macro per fresh process |
| [B4E1M](../nonlocal-nsr3b4e1m-hydro-macro-evidence-2026-08-21.md) | physical/root PASS but one macro is 48.8 s and 227 workspace builds | hold B4E2; profile B4EP first |
| [B4EP0 research](../nonlocal-nsr3b4ep0-attribution-research-2026-08-21.md) | four competing serial-cost hypotheses; perf events unavailable | run exact-output gprof attribution only |
| [B4EP0](../nonlocal-nsr3b4ep0-attribution-evidence-2026-08-21.md) | exact-output profile selects SHA as largest leaf with `~1.70x` ceiling | freeze one Release evidence-policy ablation |
| [B4EP1 research](../nonlocal-nsr3b4ep1-query-evidence-research-2026-08-21.md) | inner hashes are non-physical and separable from parent/final roots | implement work-only policy and balanced A/B |
| [B4EP1](../nonlocal-nsr3b4ep1-query-evidence-evidence-2026-08-21.md) | exact roots/counters with `3.0168x` median paired speedup | retain full default; profile residual work-only cost before one next optimization |
| [B4EP2 research](../nonlocal-nsr3b4ep2-residual-attribution-research-2026-08-21.md) | HVP/topology/evaluation/control remain competing residual hypotheses | run one exact-output work-only gprof profile |
| [B4EP2](../nonlocal-nsr3b4ep2-residual-attribution-evidence-2026-08-21.md) | workspace pipeline leads; topology and HVP nearly tie | audit canonical superset feasibility before implementing reuse |
| [B4EP3 research](../nonlocal-nsr3b4ep3-canonical-superset-research-2026-08-21.md) | current lexicographic pair sort may avoid old anchor-cell order failure | run fixed `0.04h` 227-state audit; no tuning on failure |
| [B4EP3](../nonlocal-nsr3b4ep3-canonical-superset-evidence-2026-08-21.md) | 227/227 exact; one rebuild, 226 reuse; candidate work ratio `0.1998` | design optional internal hot-path cache and controlled Release A/B |
| [B4EP3I research](../nonlocal-nsr3b4ep3i-hotpath-cache-research-2026-08-21.md) | trace ownership avoids global/persistent state and signature fan-out | implement dedicated cached candidate; defaults remain byte-exact |
| [B4EP3I](../nonlocal-nsr3b4ep3i-hotpath-cache-evidence-2026-08-22.md) | exact physics, one build/225 reuse and median `1.5899x` paired speedup | retain cache for nominal research and reprofile optimized residual before another change |
| [B4EP4 research](../nonlocal-nsr3b4ep4-cached-residual-attribution-research-2026-08-22.md) | HVP, filtered workspace and nonlinear control remain competing residual costs | execute one frozen exact-output gprof profile; select no optimization before attribution |
| [B4EP4](../nonlocal-nsr3b4ep4-cached-residual-attribution-evidence-2026-08-22.md) | HVP leads complete cached workspace `1.749x` with exact output | research one HVP-only mechanical discriminator before implementation |
| [B4EP5 research](../nonlocal-nsr3b4ep5-hvp-coefficient-tape-research-2026-08-22.md) | repeated invariant kernel coefficients dominate the safe HVP opportunity | implement optional two-scalar tape and controlled exact A/B only |
| [B4EP5](../nonlocal-nsr3b4ep5-hvp-coefficient-tape-evidence-2026-08-22.md) | exact physics/old bytes and median `1.2482x` paired speedup | retain coefficient tape for research; exact-profile residual before one next change |
| [B4EP6 research](../nonlocal-nsr3b4ep6-coefficient-residual-attribution-research-2026-08-22.md) | old profile is invalid after 800M removed kernel calls | execute one exact-output profile with frozen leader rule |
| [B4EP6](../nonlocal-nsr3b4ep6-coefficient-residual-attribution-evidence-2026-08-22.md) | workspace leads; evaluation/base tape dominates its subcategories | audit one exact fusion/reuse discriminator before implementation |
| [B4EP7D research](../nonlocal-nsr3b4ep7d-evaluation-tape-dataflow-research-2026-08-22.md) | radius, gradient kernel and compression repeat across evaluation/tape | run derived-counter audit; freeze exact removable work before fusion |
| [B4EP7D](../nonlocal-nsr3b4ep7d-evaluation-tape-dataflow-evidence-2026-08-22.md) | exact duplicate-work counts with all regressions unchanged | freeze one fused evaluation/tape A/B contract |
| [B4EP7I research](../nonlocal-nsr3b4ep7i-fused-evaluation-tape-research-2026-08-22.md) | exact pair/center order permits one fused builder | implement dedicated command and controlled exact A/B |
| [B4EP7I](../nonlocal-nsr3b4ep7i-fused-evaluation-tape-evidence-2026-08-22.md) | bit-exact fused path and median `1.1111x` paired speedup | retain internally; reprofile exact fused residual before another change |
| [B4EP8 research](../nonlocal-nsr3b4ep8-fused-residual-attribution-research-2026-08-22.md) | fusion invalidates B4EP6 function attribution | run one exact-output profile and route by frozen leader rule |
| [B4EP8](../nonlocal-nsr3b4ep8-fused-residual-attribution-evidence-2026-08-22.md) | no top-level leader and gprof cannot split fused subphases | design isolated phase timers; no optimization selected |
| [B4EP9 research](../nonlocal-nsr3b4ep9-fused-phase-timing-research-2026-08-22.md) | Amdahl-ready phase boundary and stability gate frozen | implement opt-in timers and run three fresh processes only |
| [B4EP9](../nonlocal-nsr3b4ep9-fused-phase-timing-evidence-2026-08-22.md) | exact semantics; 92.19% median conservative parallelizable fraction | research/freeze deterministic CPU parallel architecture before threads |
| [B4EP10 research](../nonlocal-nsr3b4ep10-cpu-parallel-architecture-research-2026-08-22.md) | fixed partitions plus target-owned canonical gathers selected | prove serial dataflow exact before linking OpenMP |
| [B4EP10D](../nonlocal-nsr3b4ep10d-owner-computes-dataflow-evidence-2026-08-22.md) | exact topology/evaluation/HVP owner dataflow with zero mismatch | freeze OpenMP A/B/capacity/failure contract before parallel code |
| [B4EP10I research](../nonlocal-nsr3b4ep10i-openmp-implementation-research-2026-08-22.md) | OpenMP static logical-partition executor selected | implement exact cross-count gate; no timing until B4EP10S |
| [B4EP10I](../nonlocal-nsr3b4ep10i-owner-parallel-evidence-2026-08-22.md) | exact cross-count correspondence and fail-closed negatives | freeze balanced serialized scaling before any speedup claim |
| [B4EP10S research](../nonlocal-nsr3b4ep10s-scaling-design-research-2026-08-22.md) | physical-core affinity and short balanced matrix selected | execute frozen scaling contract without concurrent conditions |
| [B4EP10S](../nonlocal-nsr3b4ep10s-owner-parallel-scaling-evidence-2026-08-22.md) | 8-worker host-specific knee passes every frozen gate | attribute selected parallel residual before another change |
| [B4EP10R research](../nonlocal-nsr3b4ep10r-selected8-profile-research-2026-08-22.md) | unmodified clock/sync profile selected | run exact profile and route one next design only |
| [B4EP10R](../nonlocal-nsr3b4ep10r-selected8-profile-evidence-2026-08-22.md) | target exact but PC samples marked unreliable | discard attribution; design internal timing |
| [B4EP10R1 research](../nonlocal-nsr3b4ep10r1-internal-parallel-timing-research-2026-08-22.md) | active/imbalance/orchestration timing selected | implement isolated command and run three exact processes |
| [B4EP10R1](../nonlocal-nsr3b4ep10r1-internal-parallel-timing-evidence-2026-08-22.md) | stable evaluation-plan leader; region and balance hypotheses rejected | research one exact plan architecture before implementation |
| [B4EP10P research](../nonlocal-nsr3b4ep10p-masked-superset-plan-research-2026-08-22.md) | stable superset subsequence can remove repeated transpose construction | run structural/scan/capacity audit before fast path |
| [B4EP10PD](../nonlocal-nsr3b4ep10pd-masked-superset-plan-evidence-2026-08-22.md) | all active target rows exactly filter one fixed plan within gates | freeze opt-in implementation/A-B before speed claim |
| [B4EP10PI research](../nonlocal-nsr3b4ep10pi-masked-plan-implementation-research-2026-08-22.md) | cache/workspace lifetime and exact masked gather selected | implement candidate then run frozen external A/B |
| [B4EP10PI](../nonlocal-nsr3b4ep10pi-masked-plan-evidence-2026-08-22.md) | exact masked reuse misses frozen speed gate and raises CPU work | retain active plan; research deterministic parallel rebuild |
| [B4EP10PC research](../nonlocal-nsr3b4ep10pc-partitioned-active-plan-research-2026-08-22.md) | stable partition-local histogram preserves canonical source order | run builder audit before candidate integration |
| [B4EP10PCD](../nonlocal-nsr3b4ep10pcd-partitioned-active-plan-evidence-2026-08-22.md) | exact stable parallel transpose | freeze opt-in replacement/A-B only |
| [B4EP10PCI](../nonlocal-nsr3b4ep10pci-partitioned-plan-evidence-2026-08-22.md) | exact candidate misses 5% speed gate | preserve negative; do not lower gate |
| [B4EP10CTD](../nonlocal-nsr3b4ep10ctd-current-topology-plan-evidence-2026-08-22.md) | exact full reverse plan misses scan gate | split self from incoming before construction |
| [B4EP10SID](../nonlocal-nsr3b4ep10sid-split-incoming-plan-evidence-2026-08-22.md) | exact three-part fold and bounded visit ratio | prove independent incoming construction |
| [B4EP10SICD](../nonlocal-nsr3b4ep10sicd-incoming-construction-evidence-2026-08-22.md) | exact pair-endpoint construction | integrate opt-in floating path and A/B |
| [B4EP10SII](../nonlocal-nsr3b4ep10sii-split-incoming-plan-evidence-2026-08-22.md) | exact and narrowly passes frozen speed gate | retain for research; profile candidate residual next |
| [B4EP10SIR research](../nonlocal-nsr3b4ep10sir-split-incoming-residual-research-2026-08-22.md) | existing timers can isolate four architecture categories | implement/run frozen timing command only |
| [B4EP10SIR](../nonlocal-nsr3b4ep10sir-split-incoming-residual-timing-evidence-2026-08-22.md) | source-local work is the stable leader; executor hypotheses fail | research one narrower source-local discriminator |
| [B4EP10SIRD research](../nonlocal-nsr3b4ep10sird-source-local-discriminator-research-2026-08-22.md) | existing subphases separate directed/setup/compression/local-scalar work | execute the frozen reduction only |
| [B4EP10SIRD](../nonlocal-nsr3b4ep10sird-source-local-discriminator-evidence-2026-08-22.md) | stable directed leader, but timer includes initialization | research an exact scratch-liveness audit |
| [B4EP10SIRDA research](../nonlocal-nsr3b4ep10sirda-directed-scratch-audit-research-2026-08-22.md) | split incoming folds permit a complete write/read certificate | implement the frozen shadow audit only |
| [B4EP10SIRDA](../nonlocal-nsr3b4ep10sirda-directed-scratch-audit-evidence-2026-08-22.md) | exact liveness and bounded high-water projection pass | research one opt-in scratch reuse path |
| [B4EP10SIRDI research](../nonlocal-nsr3b4ep10sirdi-directed-scratch-reuse-research-2026-08-22.md) | transaction-local directed-only ownership selected | implement and execute frozen A/B only |
| [B4EP10SIRDI](../nonlocal-nsr3b4ep10sirdi-directed-scratch-reuse-evidence-2026-08-22.md) | exact reuse passes all wall/CPU/RSS gates | reprofile exact candidate before another change |

Detailed stage order, every intermediate negative and all evidence links remain
in the [research roadmap](../../plans/nonlocal-nonlinear-solver-research/README.md).
Git history before the B4D checkpoint retains the superseded long-form task
diary; it is not current authority.

## Current exact external boundary

Frozen W0I attestation root:
`186e1e31c0aa2636525bbc54e4fe4335b8432e7e99eddf3221d08e0380b65c90`.

| File | Required SHA-256 | Current state |
|---|---|---|
| hydro | `84ae867f5b336cd0bd51be6f29a6a2a1f27f702c424f1dbd0a8f735b9f4bb435` | missing |
| dam break | `853d965489a40082a024aeee5a19f98aef054417014af8556fa212687d88d12c` | missing |
| orifice | `60e9b3538d621ef1a3f1ae77569640740df471fbe4e5ef8eaa813d3751930849` | missing |

B4D identity:
`47c78bdb115c0e5d7ed7132a6de3e62e3ba9533396a7f346b510b354dc5222fe`.

The external payloads are deliberately outside Git. Local searches found no
copy in `/tmp`, NextEngine worktrees, Downloads, desktop/trash or exact-size
Git blobs. Exact-hash web search returned no result and is not proof of global
absence. Unreachable Git blobs contain neither a full payload nor the recorded
adapter/source/binary SHA-256 values.

The separate new-root R1E reference candidate is available under profile
`ba34b4e3b12986ebc831320d6811551d5311a6774a64f079aabe3a5eaa6bb746`
and is attested by identity
`9cf5fc571fee7bc0be5585d9b467f90cd8a27f40d9cf39d6be999b0c466cbccc`.
It does not replace the missing historical W0I bytes or inherit their credit.

## Decisions

### D-001 -- Preserve the new nonlinear lineage

- **Observation:** full coupled curvature plus trust-region globalization
  closes the stopped local-SISSM failure on bounded controls.
- **Decision:** retain `nuv-newton-krylov-r0`; Pairwise Descent remains
  monitor-only until public formulas/code exist.
- **Rejected:** Chebyshev radius sweeps, larger caps and silent fallback.

### D-002 -- Keep normalized corrected physics

- **Observation:** the raw FCR cubic integrates to `1/8`; the author's fixed
  lattice normalization restores the declared density.
- **Decision:** all current work uses `nuv-variational-fcr2`; no old source-
  shaped coefficient, root or performance result transfers.

### D-003 -- Publish only at macro boundaries

- **Observation:** canonical microunit noise scales with private substep count.
- **Decision:** private adaptive/fixed levels remain binary64 inside a macro
  transaction; only the accepted endpoint is balanced and published once.
- **Rejected:** epsilons, per-substep canonical continuation and fitted scalar
  amplification tubes.

### D-004 -- Close B4C4 packaging without a whole-solver claim

- **Observation:** retained workspaces, static support indexing and CSR
  ownership preserve all complete-lane roots and remove duplicate work.
- **Decision:** select the composed B4C4 candidate. Local construction timing
  improves `1.2274x/1.2055x` on P1/P2; this is not whole-solver performance.

### D-005 -- Preserve missing B4D evidence and recover reproducibility

- **Observation:** source/solver identities match, but the three `/tmp`
  payloads and their original external adapter artifacts are unavailable.
- **Decision:** keep B4D fail-closed and research artifact/generator recovery.
- **Rejected:** placeholder files, reconstructed aggregate curves, old
  non-clearance references, hash-manifest-only PASS and B4E without input.
- **Reconsider when:** exact W0I files or exact source/diff/binary lineage is
  restored; otherwise only a newly rooted reference profile may proceed.

### D-006 -- Select a strict complete-clone external build profile

- **Observation:** GCC/CMake/Ninja and all required pinned dependencies
  reproduce byte-identically with binary64, AVX/FMA/fast-math disabled from a
  verified complete clone. One originally retained source copy was not a
  complete object database and is preserved as negative provenance evidence.
- **Decision:** select B4DR1 R1A and require an ordinary clean full Git clone
  for every reference build; bind generated `Utilities/Version.h` separately.
- **Rejected:** lazy blob-by-blob checkout and linked Git worktrees.

### D-007 -- Select the independent contact adapter

- **Observation:** the v2 geometry, all eight branch cases and independent
  clearance/chord validator pass under a reproducible linked DFSPH ABI.
- **Decision:** select R1B only as the contact/preflight boundary and authorize
  R1C scenario-manifest design.
- **Rejected:** the v1 outer-box geometry, a weakened header-only ABI anchor,
  and starting trajectories before manifest/serialization closure.

### D-008 -- Freeze a cost-aware short external trajectory gate

- **Observation:** pinned upstream enables warm starts and does not expose the
  complete convergence state required for a fail-closed comparator.
- **Decision:** bind the tracked cold-start/diagnostic-only patch and exact
  R1C manifests/serialization. Require a zero-physics manifest preflight
  before any solver object or trajectory.
- **Rejected:** accepting default warm starts, inferring convergence from one
  iteration counter, running full schedules first, or modifying DFSPH
  equations.

### D-009 -- Reject the inconsistent shortened dam identity

- **Observation:** `CW-DAM-001` generates neither of the dam roots frozen by
  R1C; normative `CW-DAMBREAK-001` generates both exactly.
- **Decision:** preserve the failed R1C identity and reclose only the scenario
  ID and derived manifest root under R1C1.
- **Rejected:** changing the expected roots to fit the shortened ID, editing a
  frozen contract into an apparent PASS, or treating the manifest failure as
  DFSPH evidence.

### D-010 -- Select the corrected zero-physics manifest gate

- **Observation:** two independent builds/processes reproduce every corrected
  root and mutation; forced mismatch stops before Simulation creation.
- **Decision:** select R1C1 and authorize implementation of the already-frozen
  short trajectory path in a separately patched full clone.
- **Rejected:** mutating a clean R1A clone or jumping directly to R1D.

### D-011 -- Preserve the first physical failure and reclose observability

- **Observation:** the first Hydro process fails at pressure convergence and
  publishes no payload, while the adapter discards its populated diagnostic
  fields when constructing the failure report.
- **Decision:** R1C fails and R1D remains blocked. Select R1C2 as a report-only
  reclosure followed by exactly one Hydro diagnostic process.
- **Rejected:** a blind iteration/tolerance change, warm-start restoration,
  retry under the failed identity, or advancing to Dam/Orifice.

### D-012 -- Measure the pressure convergence boundary before remediation

- **Observation:** R1C2 isolates a finite pressure-only cap hit at step 1 but
  provides only one convergence endpoint.
- **Decision:** run a fixed ascending one-step R1C3 cap sweep, stopping at the
  first converged result and changing no other profile field.
- **Rejected:** immediate cap promotion, tolerance loosening or simultaneous
  mass/volume/boundary changes.

### D-013 -- Reclose the short reference profile at cap 300

- **Observation:** cap 300 permits normal step-1 exit at iteration 220, while
  the selected physical volume gives an almost unit lattice density sum.
- **Decision:** create new-root R1C4 by changing only pressure maximum to 300;
  repeat paired short scenarios before R1D.
- **Rejected:** upstream 0.8 underdensity heuristic, warm starts, tolerance
  loosening and inheriting any payload/root from failed R1C.

### D-014 -- Separate analytical domain from boundary support

- **Observation:** Orifice intentionally has a two-metre analytical box but
  only one-metre source-side Akinci support; one `boundary_nx` field cannot own
  both meanings.
- **Decision:** add explicit scenario `domain_x_max`, retain boundary roots and
  reissue the global R1C5 payload identity before rerunning all pairs.
- **Rejected:** extending receiver-side Akinci support, weakening contact's
  two-metre assertion or inheriting R1C4 Hydro/Dam payload roots.

### D-015 -- Admit corrected short trajectories to full generation

- **Observation:** all six R1C5 processes pass with byte-identical paired
  reports/payloads; Orifice records 28 final receiver samples.
- **Decision:** authorize only R1D full external generation under the R1C5
  profile and cap 300.
- **Rejected:** inheriting short payloads as full references, starting R1E
  early or converting this research pass into runtime/production authority.

### D-016 -- Reclose manifests at the full schedule

- **Observation:** R1C blocks normatively claim 24 steps/every-step output and
  cannot be embedded unchanged in truthful full-schedule payloads.
- **Decision:** reissue only scenario schedule blocks and profile identity;
  retain all R1C5 physical bytes and the frame layout. Run independent
  scenarios concurrently, never parallelizing one solver process.
- **Rejected:** a contradictory embedded manifest, OpenMP reduction changes,
  `/tmp`-only output or publication before pair equality.

### D-017 -- Admit full references to attestation design

- **Observation:** all three full pairs and reports compare byte-for-byte;
  final content-addressed files rehash to their reported roots.
- **Decision:** select R1D PASS and authorize only R1E reader/profile contract
  design over the actual closure.
- **Rejected:** direct B4E use without a fail-closed reader, old W1 credit or a
  production claim from external-reference generation.

### D-018 -- Keep reader independent from generator

- **Observation:** generator parser reuse could reproduce a common layout bug
  while still matching complete-file hashes.
- **Decision:** implement a standalone reader that canonically reconstructs
  every decoded field and independently regenerates aggregate roots.
- **Rejected:** generator-source reuse, filename/report trust, path checks
  separated from open, or full hash without decoded semantic controls.

### D-019 -- Admit the new reference candidate to B4E design

- **Observation:** the independent reader reproduces all three decoded and
  aggregate roots twice; missing, symlink, oversized and complete-size mutated
  fixtures all fail closed.
- **Decision:** select `NEW_EXTERNAL_DFSPH_REFERENCE_CANDIDATE` and begin only
  B4E nominal-corpus research/contract design.
- **Rejected:** inheriting historical W1 credit, executing an unfrozen
  comparison, or treating reference integrity as runtime/production evidence.

### D-020 -- Stage nominal comparison behind alignment and cost gates

- **Observation:** the packaged path has nominal capacities and avoids
  all-pairs candidate HVPs, but only P1/P2 commands have executed; its active
  macro frame still requires spectral and nonlinear HVP work.
- **Decision:** run zero-trajectory B4E0, one-macro B4E1 and first-output B4E2
  before projecting or starting the full Hydro/Dam pair.
- **Rejected:** full-run-first execution, cross-solver iteration/density or
  per-particle gates, Orifice before B4O, and timing as a physics tolerance.

### D-021 -- Admit nominal alignment to one-macro cost design

- **Observation:** both 6k cases reproduce exact R1D roots and remain below
  flat-neighborhood capacities; two processes/builds agree exactly.
- **Decision:** select B4E0 PASS and design one Hydro macro step with timing
  outside deterministic physics/work evidence.
- **Rejected:** treating the 0.4 s zero-step preflight as solver throughput or
  interpreting machine-floor Hydro active centres as physical pressure.

### D-022 -- Isolate spectral cost before one macro

- **Observation:** any Hydro macro attempt first pays 48 HVPs solely because
  nine parent centres are epsilon-positive; a whole step cannot attribute
  that cost or preflight the 192-substep policy capacity.
- **Decision:** execute B4E1S spectrum twice and require derived initial count
  at most 96 before B4E1M design.
- **Rejected:** timing the whole macro first or clipping the active set under
  the already frozen solver identity.

### D-023 -- Admit the nominal spectrum to one-macro design

- **Observation:** four independent same-state estimates agree bit-for-bit,
  use exactly 48 joint HVPs each and derive 14 initial substeps versus the
  frozen capacity boundary of 96.
- **Decision:** select `NOMINAL_HYDRO_SPECTRUM_CANDIDATE` and design B4E1M as
  exactly one Hydro macro transaction with timing kept external.
- **Rejected:** interpreting the spectrum probe as macro throughput, opening
  the reference curve early or advancing directly to a multi-step run.

### D-024 -- Bound the first nominal nonlinear transaction

- **Observation:** the existing adaptive path must test adjacent temporal
  levels, so B4E1M can cost more than the 14-substep spectrum forecast alone.
- **Decision:** run exactly one retained-flat step-1 transaction per process
  over levels `14,28,56,112`, with a non-physical 900-second watchdog.
- **Rejected:** two in-process macros, a forced-failure duplicate, opening the
  first reference output early or treating a watchdog exit as physics FAIL.

### D-025 -- Route nominal execution to performance remediation early

- **Observation:** step 1 passes every physical/root gate but takes 48.8 s on
  one core; a one-pair Hydro+Dam projection is about 26 machine-hours, already
  `6.5x` beyond the four-hour routing boundary.
- **Decision:** preserve B4E1M as the exact oracle, hold B4E2 execution and
  profile stage costs/workspace rebuilds in B4EP0 before selecting a remedy.
- **Rejected:** spending about 46 minutes on B4E2 repeats before attribution,
  treating process-level parallelism as reduced machine-hours or assuming the
  227 rebuilds dominate without a profiler.

### D-026 -- Attribute serial cost without changing host policy

- **Observation:** work counts alone cannot distinguish SHA/stream cost,
  neighborhood rebuild, HVP traversal and nonlinear-control overhead; Linux
  perf events are blocked by the current host policy.
- **Decision:** use one separate GCC `-pg`/gprof build, require byte-exact
  B4E1M stdout and select only one follow-up Release ablation from self time.
- **Rejected:** changing `perf_event_paranoid`, optimizing from code inspection
  alone, comparing gprof wall time as Release throughput or starting B4E2.

### D-027 -- Remove inner evidence hashing before harder solver changes

- **Observation:** SHA-256 consumes 41.36% sampled self time and is dominated
  by per-workspace/pair/tape evidence; HVP and topology remain comparably large
  follow-up categories.
- **Decision:** first A/B a work-only transaction trace while retaining full
  hashing as default and requiring exact B4E1M physical roots/counters.
- **Rejected:** claiming the `1.70x` ceiling closes the roadmap, deleting final
  publication hashes, changing formulas/solver policy or parallelizing before
  this lower-risk cost is separated.

### D-028 -- Preserve full evidence as default during the hash ablation

- **Observation:** transient workspace roots do not feed physics, but deleting
  them globally would destroy the exact B4E1M oracle and existing diagnostics.
- **Decision:** add an internal work-only policy used by one candidate command;
  keep full-state bytes unchanged and require exact durable physics roots.
- **Rejected:** replacing final hashes, silently changing existing commands,
  timing different binaries or granting B4E2 authority from this ablation.

### D-029 -- Retain work-only for nominal research and reprofile the residual

- **Observation:** all three controlled pairs win at about `3.0x` with exact
  physics, but the remaining macro still takes 16.15 s and constructs all 227
  neighborhoods/workspaces.
- **Decision:** use work-only only in dedicated nominal research commands and
  freeze B4EP2 residual attribution before choosing topology reuse, HVP
  traversal or nonlinear-control work.
- **Rejected:** changing the default oracle, extrapolating this result to GPU
  or production, or starting B4E2 before the next cost boundary is measured.

### D-030 -- Audit canonical topology reuse before optimizing it

- **Observation:** workspace construction is 58.33% inclusive, but topology
  alone (40.97%) nearly ties HVP (39.64%). An older GPU/f32 Verlet candidate
  failed exact order after cell crossing.
- **Decision:** exploit the current builder's explicit lexicographic final
  pair sort only after a 227-state filtered-superset audit proves pair/CSR
  equality, certificate coverage, capacity and useful reuse.
- **Rejected:** transferring P4 credit, assuming geometric coverage implies
  reduction-order equality, or implementing/timing cache reuse first.

### D-031 -- Admit exact superset reuse to a hot-path A/B only

- **Observation:** every captured state reproduces exact topology/evaluation/
  tape, one list covers the sequence and the candidate work proxy is 0.1998.
- **Decision:** integrate an optional internal cache into the research query
  trace, leave all defaults/full parent unchanged and require exact physical
  correspondence before balanced Release timing.
- **Rejected:** treating the audit's 31-second dual-path process as a speed
  result, exposing a runtime option, or dropping per-query CSR/tape refresh.

### D-032 -- Retain the transaction-local cache and reprofile

- **Observation:** the cached command preserves all B4EP1 physical roots and
  counters, performs one build plus 225 certified reuses, and wins all three
  Release pairs at median `1.5899x` paired speedup.
- **Decision:** select `HOTPATH_CANONICAL_SUPERSET_CANDIDATE` for nominal
  research commands and run B4EP4 residual attribution before choosing the
  next optimization.
- **Rejected:** enabling the cache by default/runtime, claiming audit work as
  whole-solver speed, starting B4E2 at 10.4 s per macro, or assuming HVP is now
  dominant without a fresh profile.

### D-033 -- Select HVP as the next residual research target

- **Observation:** B4EP4 assigns 6.91 s (62.14%) inclusive to 459 exact HVPs
  and 3.95 s (35.52%) to complete cached workspaces; HVP leads `1.749x`.
- **Decision:** research one HVP-only mechanical discriminator that preserves
  exact arithmetic, call schedule and solver policy before implementation.
- **Rejected:** another topology/evaluation optimization, solver trial-policy
  change, CPU parallelism or GPU work before the HVP dataflow is isolated.

### D-034 -- Admit deterministic owner-parallel execution to scaling design

- **Observation:** the selected OpenMP owner-computes path reproduces one
  common exact correspondence hash at `1/2/4/8/16` workers, with zero team,
  coverage or worker mismatch and a byte-identical 16-worker repeat.
- **Decision:** retain the opt-in research backend and freeze a balanced,
  serialized B4EP10S scaling experiment before selecting a worker count.
- **Rejected:** using unordered correspondence-process durations as a speedup
  result, enabling the backend by default or inferring production readiness.

### D-035 -- Select the 8-core knee and attribute parallel overhead next

- **Observation:** 8 workers improve exact whole-macro wall by `1.237020x`
  and use 6.421 effective cores. Sixteen workers reduce wall only another
  1.70% while raising effective use to 12.488 cores; one-worker owner dataflow
  is about 4.10 s slower than serial.
- **Decision:** retain 8 workers for this host/nominal research path and
  profile the selected parallel residual before changing code again.
- **Rejected:** selecting 16 from minimum wall alone, treating 23.70% wall
  improvement as production readiness or optimizing without attribution.

### D-036 -- Reject unreliable worker-thread sampling

- **Observation:** gprofng preserves exact target output and records 32.302
  sampled CPU seconds, but its own header says the 1 ms interval changed to
  zero and the profile may be unreliable; native sync tracing is empty.
- **Decision:** reject all sampled percentages for routing, preserve B4EP10S
  and research opt-in internal parallel phase timing.
- **Rejected:** waiving the frozen reliability gate because the raw 60.91%
  libgomp sample appears plausible or treating the warning as a solver fail.

### D-037 -- Route to evaluation owner-plan architecture

- **Observation:** direct instrumentation puts orchestration at 1.11% and
  imbalance at 6.55% median worker capacity. Evaluation is 54.63% of the
  transaction, `1.916851x` HVP, and its plan/energy subphase is 42.33%.
- **Decision:** reject persistent-team and partition-balance work; research
  one exact owner-plan architecture before changing implementation.
- **Rejected:** routing from unreliable gprofng samples, optimizing OpenMP
  barriers, or treating instrumented durations as a speed result.

### D-038 -- Audit one masked superset transpose

- **Observation:** the cache already has one certified superset and 225
  reuses; current active rows preserve its order. Rebuilding the active plan
  visits at least 546.8M records versus 1.356M energy-fold centres.
- **Decision:** audit a fixed superset target CSR filtered by current slot and
  compression masks before implementing it.
- **Rejected:** repeated parallel rebuild as the first choice, atomic scatter,
  or assuming the pair-superset ratio bounds target-gather bandwidth.

### D-039 -- Admit masked plan implementation experiment

- **Observation:** all 226 active plans are exact filtered views of one fixed
  705,284-slot plan; projected target scan grows `1.288507x` and conservative
  payload remains 38.0 MB.
- **Decision:** retain masked reuse for one opt-in implementation/A-B contract.
- **Rejected:** claiming speed from structural evidence or promoting the
  audit-only structures into runtime/default ownership.

### D-040 -- Reject masked plan as selected fast path

- **Observation:** masked reuse wins `3/3` and lowers RSS, but median paired
  speedup is `1.030796x` and median user CPU increases by 1.96 s.
- **Decision:** preserve the exact negative candidate, retain B4EP10I active
  plans and research a stable parallel active transpose.
- **Rejected:** lowering the predeclared 5% gate after seeing the result or
  hiding added scan work behind the 3.6% aggregate-median wall change.

### D-041 -- Admit partitioned transpose to candidate-path A/B

- **Observation:** all 226 stable partitioned plans reproduce the serial
  active-plan arrays exactly; both corruption fixtures reject and conservative
  combined storage is 37.3 MB.
- **Decision:** freeze one opt-in implementation that replaces, rather than
  duplicates, serial plan construction and measure it in balanced Release
  pairs against B4EP10I worker-8.
- **Rejected:** using the side-by-side audit duration as performance evidence,
  changing the canonical energy fold, or promoting the builder to a default
  or runtime path before exact A/B gates pass.

### D-042 -- Reject partitioned plan as selected fast path

- **Observation:** the exact partitioned builder wins `3/3`, but median paired
  speedup is `1.033650x`; median user/system CPU increase by 2.06/0.63 seconds.
- **Decision:** retain B4EP10I. Research a current-topology reverse plan that
  can be produced with topology compaction and masks compression during
  gathers, avoiding both superset expansion and five per-plan build regions.
- **Rejected:** lowering the 5% gate, selecting from wall wins alone, or
  returning immediately to generic persistent-team work after B4EP10R1 found
  low orchestration cost on the selected path.

### D-043 -- Reject full current plan; split self from incoming

- **Observation:** the current-topology full plan is exact and bounded, but
  scans `1.213341x` the retained work, narrowly failing its `1.20x` gate.
- **Decision:** exploit the source CSR for self contributions and retain only
  participant/incoming reverse entries. Exact counts project `1.106670x`
  visits before implementation; audit the three-part fold order next.
- **Rejected:** relaxing the scan gate after observing the result or proceeding
  directly to topology construction based on a near miss.

### D-044 -- Admit split incoming view to construction research

- **Observation:** all 226 three-part reconstructions are exact; projected
  visits are `1.106670x`, both corruption fixtures reject and conservative
  payload is 32.1 MB.
- **Decision:** research a deterministic topology-compaction builder that
  emits one participant slot per directed source slot without active-set or
  floating work.
- **Rejected:** timing the audit's full side-by-side plan, integrating the
  floating fold before construction correspondence, or claiming the work
  proxy as speedup.

### D-045 -- Admit pair-endpoint builder to integration research

- **Observation:** all 226 incoming plans match byte-for-byte; pair endpoint
  and support cursor corruptions reject, with 35.6 MB conservative payload.
- **Decision:** freeze one opt-in floating split-fold implementation and
  balanced A/B against B4EP10I, preserving the exact construction and target
  fold order.
- **Rejected:** timing the side-by-side audit, enabling the path by default,
  or fusing construction phases before the isolated implementation has exact
  correspondence.

### D-046 -- Select split incoming for residual research

- **Observation:** the integrated candidate is bit-exact, wins all three
  balanced pairs and reaches `1.052521x` median paired speedup with stable wall
  and lower RSS. User/system CPU still rise.
- **Decision:** retain the split incoming representation for the nominal
  8-worker research path, keep B4EP10I as rollback and remeasure candidate
  subphases before another optimization.
- **Rejected:** production/default promotion from one nominal macro, lowering
  earlier gates, or immediately fusing construction without attribution.
- **Reconsider when:** candidate-specific timing identifies one bounded leader
  and the next exact mechanical discriminator is frozen.

### D-047 -- Select evaluation-buffer structural audit

- **Observation:** three exact timing processes put buffer preparation at
  median `89.50%` of `evaluation_setup`, `8.531659x` above validation, with
  every setup-share range below `0.009`.
- **Decision:** freeze one timing-free per-buffer write-before-read,
  ownership-lifetime and simultaneous-live high-water audit before designing
  reuse.
- **Rejected:** removing value initialization from timing alone, assuming one
  scratch bundle is sufficient, or extending the earlier directed-scratch
  lifetime proof to returned evaluation/tape storage.
- **Reconsider when:** the audit proves exact write/read coverage and a bounded
  workspace release protocol for accepted, rejected and failure exits.

### D-048 -- Admit two-workspace buffer reuse to contract research

- **Observation:** all seven roles are fully overwritten; 226 workspace
  receipts peak at two live and one ephemeral receipt peaks at one. Projected
  high-water growth is 22.07 MB versus 2.829 GB repeated initialization.
- **Decision:** research and freeze one opt-in two-lane returned-workspace pool
  plus one transaction-local density-contribution scratch implementation/A-B
  contract, preserving SIRDI as rollback.
- **Rejected:** one shared returned buffer, directed-slot sizing, timing the
  shadow audit, or changing workspace ownership and arithmetic together.
- **Reconsider when:** the implementation preserves every receipt/root/work
  identity and wins its predeclared balanced external A/B gates.

### D-049 -- Replace direct pool candidate with overwrite construction

- **Observation:** pair extent varies (average 379,275, maximum 380,511).
  Ordinary vector shrink/regrow reintroduces value initialization; retaining
  high-water `.size()` changes published tape semantics.
- **Decision:** first test a candidate-only C++17 allocator mode that omits
  redundant value assignment for six fully overwritten `double` roles. This
  covers 2,764,612,800 of 2,828,746,176 measured initialization bytes without
  changing size, capacity, ownership or arithmetic.
- **Rejected:** indexing beyond vector size, raw storage, span conversion, or
  combining allocator and pool changes in one A/B.
- **Reconsider when:** exact A/B passes and candidate-specific reprofiling still
  identifies allocation/lifetime work as a bounded leader.

### D-050 -- Reject user-allocator overwrite before A/B

- **Observation:** semantic correspondence passes, but the custom allocator
  forces per-element construction and regresses unchanged SIRDI from 4.290 to
  10.43 s. Candidate 9.63 s is relative to this corrupted baseline.
- **Decision:** fail and revert SIRDIREI before commit/A-B. Preserve SIRDI and
  narrow research to the builder-local density-contribution scratch lane.
- **Rejected:** promoting the apparent `1.083x` probe win, comparing regressed
  A/B paths, or replacing returned storage with raw memory after this failure.
- **Reconsider when:** a separately frozen representation redesign has broader
  ownership/portability authority; this performance lineage does not.

### D-051 -- Admit local density scratch with baseline health

- **Observation:** density contribution is builder-local, fully overwritten
  and has exactly one live lane. Its 85,716,150 repeated slots project to only
  380,511 high-water growth slots without changing returned storage.
- **Decision:** freeze one transaction-local standard-vector reuse candidate
  and require the SIRDI baseline itself to remain within 4.72 s median before
  any relative A/B gate is considered.
- **Rejected:** shared allocator/type changes, returned-workspace pooling,
  comparing against a degraded baseline, or a 5% gate inconsistent with this
  isolated role's measured upper contribution.
- **Reconsider when:** exact balanced A/B either selects the candidate for
  residual attribution or stops this buffer-initialization branch.

### D-052 -- Reject density scratch and stop buffer initialization work

- **Observation:** the candidate preserves every exact root, count and
  lifetime invariant and wins `3/3` at median `1.194152x`, but the baseline
  median is `4.893718116 s` versus the frozen `4.72 s` limit and candidate
  range ratio is `1.174767` versus `1.10`.
- **Decision:** fail the contract without rerun, revert commit `c5a9a9c` via
  `b8a1edd`, retain SIRDI and stop the setup-buffer initialization branch.
- **Rejected:** lowering gates, repeating the observed A/B, promoting the
  relative win, retrying allocator/raw/returned-storage variants or combining
  buffer work with another optimization.
- **Remaining uncertainty:** density scratch may reduce CPU work, but this
  experiment cannot distinguish it from code-layout and host-variance effects.
- **Reconsider when:** a separately authorized representation/ownership
  redesign supplies a new portability boundary and independent evidence.

### D-053 -- Qualify default-path source drift before residual work

- **Observation:** the accepted SIRDI checkpoint measured 31.58 s median
  total CPU, while later exact default measurements are about 36 s after 999
  added source lines across residual timing, setup timing and liveness audit.
- **Decision:** compare independent `f33bf3a` and `b8a1edd` Release builds of
  the identical SIRDI command in frozen balanced pairs.
- **Rejected:** assuming host noise, assuming dormant code is free, selecting
  topology from a possibly regressed baseline or rerunning SIRDIREP.
- **Decision criterion:** healthy/stable accepted timing plus at least 1.05
  wall slowdown and total-CPU ratio confirms drift; failed health gates route
  to host qualification; both ratios below 1.05 return to topology research.

### D-054 -- Reject source attribution on an unqualified host

- **Observation:** both builds are exact and stable, and current/accepted wall
  and total-CPU ratios are `0.977436/0.982361`; accepted median nevertheless
  misses the absolute host gate at `4.801272953 s`.
- **Decision:** close `HOST_UNQUALIFIED`, retain SIRDI and do not bisect the
  999-line source delta. Replace preemption-sensitive wall attribution with one
  opt-in process/thread CPU-time discriminator before structural routing.
- **Rejected:** treating current's apparent 2% win as speed credit, relaxing
  4.72 s after observation, rerunning until quiet or modifying unrelated user
  processes/host policy.
- **Reconsider when:** an independently admitted quiet wall window passes the
  original health boundary.

### D-055 -- Replace residual wall attribution with CPU clocks

- **Observation:** Linux process CPU time covers all process threads and
  thread CPU time covers one worker while excluding external descheduling;
  both clocks report 1 ns resolution on the active host.
- **Decision:** add one opt-in exact CPU trace over 7,275 phase/total, 4,089
  region and 32,712 worker intervals. Route only from stable disjoint CPU
  shares and retain wall throughput as unqualified.
- **Rejected:** using process total CPU without phase attribution, modifying
  unrelated process affinity/priorities, interpreting CPU time as frame latency
  or allowing stopped evaluation setup to select another buffer experiment.

### D-056 -- Route the stable CPU leader to topology structure

- **Observation:** Q1 passes all exact gates in three fresh processes;
  topology has median share `0.301239`, every share range is at most
  `0.024734`, and external/internal CPU ratio median is `1.007535`.
- **Decision:** authorize exactly one timing-free topology structural audit.
  It may prove redundant work or ownership but cannot change code or claim
  speed until a separate contract is frozen.
- **Rejected:** another buffer experiment, using the noisy wall values,
  directly implementing a presumed topology optimization, or promoting this
  research command toward production.

### D-057 -- Audit topology/incoming construction fusion

- **Observation:** split-incoming construction performs 665,142,896 standalone
  entry visits and 678 regions after topology has already established active
  pair identity, endpoint order, degrees and directed rows.
- **Decision:** shadow incoming degree during metadata, source/endpoints during
  row fill, and fill exact target rows with one canonical 85,716,150-pair
  pass. Compare all 226 plans byte for byte without timing.
- **Rejected:** masked superset, partitioned active and full-current plans are
  already negative evidence; current-active topology caching lacks a horizon-
  margin certificate and cannot be inferred from stable pair counts.
- **Reconsider when:** the audit passes every order/lifetime/capacity gate and
  a separate candidate implementation/measurement contract is frozen.

### D-058 -- Admit fused-plan consumer contract research

- **Observation:** both audit reports are byte-identical; all 226 shadow plans
  match SICD exactly, every corruption/lifetime gate passes, standalone work
  falls from 665,142,896 entries to 85,716,150 and no region is added.
- **Decision:** research and freeze one opt-in candidate that publishes the
  fused plan directly and skips the redundant SICD builder. Preserve SIRDI as
  rollback and require exact roots, plans, ownership and work accounting.
- **Rejected:** enabling the candidate directly from audit evidence, claiming
  `7.76x` solver or frame speed from the structural ratio, or running a
  short-margin wall A/B on the currently unqualified shared host.
- **Reconsider when:** a separate frozen contract defines candidate lifetime,
  failure rollback and a measurement route that does not overclaim wall
  throughput.

### D-059 -- Select direct publication and paired process CPU

- **Observation:** `JointNeighborhood` already owns the accepted SICD arrays
  until the unchanged evaluation path moves them into the pressure tape.
- **Decision:** publish the fused plan through that existing boundary, skip
  SICD only under an opt-in flag and fail closed without fallback. Prove two
  candidate processes exact before one `CLOCK_PROCESS_CPUTIME_ID` A/B with
  warmups and `AB,BA,AB` pairs.
- **Rejected:** making the Q2 audit trace a real owner, rebuilding inside
  evaluation, caching active plans, parallel target fill in the same change or
  treating process CPU as frame latency.
- **Reconsider when:** the frozen exact and CPU gates select or reject the
  candidate without weakening thresholds.

### D-060 -- Reject fused candidate under unstable CPU evidence

- **Observation:** exactness and structural reduction pass, but the candidate
  wins only one CPU pair. Baseline/candidate samples span 30--39 s, median
  paired speedup is `0.983844x` and range ratio is `1.565388`.
- **Decision:** close Q3 FAIL without rerun, revert commits `237fc12` and
  `517c9ad`, retain SIRDI and preserve Q2 only as structural evidence.
- **Rejected:** median-of-path reinterpretation after observation, lowering
  the 1.03/1.10 gates, translating `0.128869x` entries to solver speed or
  keeping an unselected dormant fast path.
- **Reconsider when:** a new, separately frozen measurement lane can bound
  worker wait/spin and shared memory/frequency interference. It must not reuse
  Q3 as an unrecorded rerun.

### D-061 -- Qualify executor-adjusted algorithmic CPU

- **Observation:** exact Q1 accounting decomposes process CPU as
  `E + X`, where `E = transaction - region + active-worker` ranges only
  `1.012483x` while excluded in-region residual `X` ranges `1.999653x`.
- **Decision:** freeze three fresh no-code Q1 executions with exact integer
  accounting and `E/O/A` stability gates of `1.03/1.05/1.05`.
- **Rejected:** calling `E` wall speed, crediting excluded runtime residual,
  retrospectively applying it to Q3 or implementing another candidate before
  qualification.
- **Reconsider when:** Q4 passes and a distinct future candidate contract is
  researched, or Q4 fails and a dedicated host becomes available.

### D-062 -- Stop shared-host performance selection

- **Observation:** Q4 passes exact accounting and external CPU cross-check,
  but adjusted range `1.031236x` exceeds its frozen `1.03` gate.
- **Decision:** close Q4 FAIL without repeat and require a dedicated/quiescent
  performance lane before another CPU/wall candidate A/B.
- **Rejected:** rounding to `1.03`, using the retrospective `1.012483x`
  window, loosening the gate after observation or substituting more structural
  counts for measured speed.
- **Reconsider when:** an external performance runner supplies bounded host
  health, explicit worker placement and repeatable wall/CPU evidence.

### D-063 -- Extract immutable first-output references before physics

- **Observation:** R1E fully attests the payloads, but the old B4E2 outline
  would first parse those files inside the same stage that advances four or
  twenty-four nonlinear macros.
- **Decision:** freeze a separate standalone B4E2R extractor for Dam step 4
  and Hydro step 24. Require complete hashes, exact layout, stable IDs,
  ties-to-even micrometre aggregates and two-build byte equality before any
  candidate trajectory.
- **Rejected:** embedding hand-copied q99 values, modifying the frozen R1E
  reader, decoding references after trajectory start or using shared-host
  elapsed time as speed credit.
- **Reconsider when:** B4E2R passes and its immutable slice roots can become
  the parent of a separately frozen Dam-first physical pilot.

### D-064 -- Admit only Dam-first physical-pilot contract research

- **Observation:** B4E2R independently closes path, complete-hash, frame
  layout, stable-ID and canonical aggregate extraction without starting a
  trajectory.
- **Decision:** use the frozen Dam step-4 slice as the only external parent of
  the next physical stage. Research state handoff and cumulative transaction
  gates before implementing four macros.
- **Rejected:** starting Dam from the extractor PASS alone, combining Dam and
  Hydro in one command, comparing DFSPH iterations/density or treating a
  coarse watchdog as performance evidence.
- **Reconsider when:** a B4E2D contract freezes exact solver lineage,
  comparison observables, tolerances, failure order and execution bound.

## Performance facts retained

- B4C4BM candidate construction wins all `63/63` paired rounds per fixture;
  median process speedups are `1.1399x/2.5952x` P1/P2.
- B4C4CM candidate neighborhood+evaluation+tape construction wins all `63/63`
  rounds; medians are `1.2274x/1.2055x`.
- Six independent fixed-reference lanes used about `3.06x` wall parallelism;
  that validates harness utilization, not runtime solver throughput.
- A B4E1S process containing two complete nominal 48-HVP estimates takes
  0.51 s wall and about 64 MiB RSS at 99% CPU; no KKT solve runs.
- B4E1M takes 48.83/48.80 s, about 92 MiB RSS and 99% of one core; 42
  attempted substeps contain 221 outer trials, 459 total HVPs and 227 flat
  workspace builds.
- B4EP1 work-only retains the same physical/work counts and wins all three
  Release pairs with `3.0168x` median paired speedup; median wall/RSS are
  16.15 s and 62,016 KiB versus full-state 48.74 s and 93,060 KiB.
- B4EP2's exact-output gprof records 1,728 samples: workspace construction is
  58.33% inclusive, HVP 39.64%, and residual SHA self time is 0.46%.
- B4EP3I cached work-only retains exact physics and wins all three Release
  pairs. Median paired speedup is `1.5899x`; median wall/RSS change from
  16.71 s/62,672 KiB to 10.40 s/62,416 KiB.
- B4EP4's exact cached profile records 1,112 samples: HVP is 62.14%, complete
  workspace 35.52%, evaluation/tape 24.64% and filter/CSR 10.25% inclusive.
- No multi-macro nominal, 50k, GPU or production performance result exists
  for this corrected Nonlocal lineage; B4E1M is one CPU research macro only.
- B4EP9 measures a stable 92.19% median conservative parallelizable fraction
  over one exact nominal transaction. This is an architecture discriminator,
  not parallel throughput or production evidence.
- B4EP10S selects 8 physical-core workers: median wall is 5.912456915 s versus
  serial 7.313827799 s (`1.237020x`), with 6.421 effective cores. Sixteen
  workers reach 5.813467075 s but consume 12.488 effective cores.
- B4EP10R1 measures stable internal shares without PC sampling. Evaluation is
  54.63% of transaction and its owner-plan/energy fold is the 42.33% leading
  subphase; orchestration and imbalance are only 1.11% and 6.55%.
- B4EP10SII is exact and wins `3/3`: median wall changes from 5.819690664 s
  to 5.536671494 s (`1.052521x`) and median RSS changes from 96,560 to
  91,708 KiB. Median user/system CPU rise to 39.36/1.15 s, so the path still
  needs residual attribution.

## Required context

1. `docs/architecture/agent-routing.md`, SPEC-38, ADR-076 and ADR-081.
2. `docs/plans/nonlocal-continuum-formula-reclosure/README.md` and
   `00-formula-contract.md`.
3. Stopped formula-reclosure task state and FCR3-B2 evidence.
4. `docs/development/nonlocal-nonlinear-solver-research-2026-08-20.md`.
5. `docs/plans/nonlocal-nonlinear-solver-research/README.md` and current frozen
   stage contract.
6. W0I reference contract and B4D design/evidence.

## Exact next action

1. Do not run another CPU/wall candidate A/B on this shared host.
2. Preserve SIRDI, Q2 structural evidence and the Q3/Q4 negative results.
3. Preserve B4E2D3's exact step-one prefix and step-two strain failure.
4. Preserve B4E2D7's convergent dense AL result and hard state-commit failure.
5. Research/freeze B4E2D7R with absolute multiplier-update units and a private
   confirmation update; do not change `beta`, formulas or start a trajectory.

## Reconsideration triggers

- Exact W0I files become available: rerun B4D twice; do not redesign first.
- Original adapter/diff/binary becomes available: verify recorded hashes, then
  regenerate outside Git and require all three historical payload hashes.
- Pairwise Descent paper/code becomes public: compare only after its exact
  formula and identity are reviewable; it does not bypass B4D references.

# Nonlocal nonlinear solver research -- current task state

| Field | Value |
|---|---|
| Status | `ACTIVE / NSR3B4EP10R_FROZEN / SELECTED8_PROFILE_EXECUTION` |
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
- **Next action:** execute the frozen B4EP10R profile and route by normalized
  sync wait or a `1.20x` exclusive-CPU category leader.
- **Do not run:** unfrozen B4E corpus, CUDA, runtime/schema, PhysX coupling,
  persistence or production work.

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

1. Execute the frozen B4EP10R selected-8 gprofng profile and exact-output gate.
2. Aggregate the frozen CPU/synchronization categories and route one next
   design; do not infer production readiness.

## Reconsideration triggers

- Exact W0I files become available: rerun B4D twice; do not redesign first.
- Original adapter/diff/binary becomes available: verify recorded hashes, then
  regenerate outside Git and require all three historical payload hashes.
- Pairwise Descent paper/code becomes public: compare only after its exact
  formula and identity are reviewable; it does not bypass B4D references.

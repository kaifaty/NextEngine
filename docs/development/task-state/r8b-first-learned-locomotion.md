# R8b first learned locomotion — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE_R&D / V3_STANDING_NOMINAL_GATE_PASS / V5_CPU_REACHABILITY_PASS / PAIRED_ACTION_AUDIT_FAILED / NO_RUNTIME_AUTHORITY` |
| Updated | 2026-09-04 |
| Task key | `r8b-first-learned-locomotion` |
| Scope | First learned standing, then bounded forward start/stop on a physically meaningful humanoid |
| Definition of done | Five complete canonical standing episodes, then five forward/start/stop episodes with at least 3 m travel and 180 final zero-command ticks, all without safety events |
| Authority | Working context only; Accepted SPEC/ADR, exact manifests and evidence outrank this file |

## Resume in 60 seconds

- **Implemented:** Walking V4 removes absolute forward-position feedback.
  Walking V5 adds BodySchema V4's narrower thigh/shank/knee collision proxies
  and a bounded fourfold residual. Old standing and walking identities remain
  intact. V5 is a Proposed fresh-weight recipe, not an active training run.
- **Measured:** CPU zero/left/right probes survive 105 ticks. Left/right
  continuous single support lasts 31/34 ticks. This is foot-release
  reachability, not a complete alternating gait.
- **Blocker:** Exact actions and initial targets agree, but physical states
  differ from tick 1. Isaac GPU ends on joint safety at tick 96; Isaac CPU
  does so at tick 105. Explicit canonical damping does not close the gap.
- **Next action:** Under ADR-107 freeze and run the direct CPU V5 experiment
  with CUDA PPO. Corrected adapter control passes 5,120 exact raw transitions
  and 399 resets; eight adapter/PPO tests pass. No policy quality yet.
- **Training:** No optimizer run started after the V4/V5 corrections. The
  original paired alternation/correspondence criterion remains unsatisfied.
  ADR-107 permits one direct-CPU discriminator, not an Isaac retry.
- **Do not retry:** Walking V1/V2/V3 unchanged, PPO/noise tuning, CPU PCM
  enablement, tolerance relaxation, standing-weight initialization into V5,
  or shortening the probe until it passes.

## Required context

1. [Routing](../../architecture/agent-routing.md), the motor/training rows,
   [SPEC-35](../../architecture/35-deterministic-humanoid-training-substrate.md)
   and [roadmap R8](../../roadmap.md).
2. [ADR-106](../../architecture/adr/106-walking-reference-and-leg-clearance-audit.md),
   [ADR-102](../../architecture/adr/102-biomechanics-neutral-self-clearance-successor.md)
   and [ADR-105](../../architecture/adr/105-r8b-dense-tracking-walking-counterfactual.md).
3. [Current action investigation](../r8b-walking-action-basis-2026-09-04.md),
   [walking research](../r8b-walking-training-research-2026-09-04.md) and
   [paired mirror investigation](../r8b-model-mirror-p1-investigation-2026-09-04.md).
4. Training-runner/current-contract before selecting any new run; diagnostics
   before changing an experiment. Earlier detailed task history is retained
   in Git at `5b9f6491` and the linked dated reports.
5. [ADR-107](../../architecture/adr/107-canonical-cpu-walking-learner.md) and
   [direct-CPU investigation](../r8b-canonical-walking-learner-2026-09-04.md).

## Current evidence

External current evidence is under
`/home/kaifaty/NextEngine-training/r8b-walking-action-basis-v5/evidence/`.
Exact file hashes and the interpretation of development report labels are in
the current action investigation. These are development-tree diagnostics,
not clean-commit generation/run manifests.

| Evidence | Result | Consequence |
| --- | --- | --- |
| V3 standing final `model_249.pt`, `254f9d3d…f41d` | One complete GPU and five 3,600-tick CPU episodes, zero safety, continuous two-sole CPU contact | Retain as nominal standing evidence only; reset seeds are not perturbation diversity |
| Standing paired mirror | Joint/root-velocity RMSE `0.05634 rad / 0.06655 m/s`; opposite-plane action tapes terminate at 171/116 | MODEL-MIRROR-P1 failed |
| Walking V1, V2, V3 | GPU survives but has negligible travel; V3 CPU falls at 156 in all five episodes | All three optimizer remedies rejected |
| Body V4 CPU reachability | 105 ticks safe; left/right single support 31/34; forward displacement `0.045964/0.115136 m` | Anatomy/action space can expose foot release |
| Zero CPU control | 105 ticks safe, double support, `0.061948 m` forward drift | Positive displacement alone is not walking |
| Isaac GPU corrected recorder | Exact actions; first joint-safety terminal 96; joint/root/velocity RMSE `0.05247/0.06926/0.18366` | Paired gate failed |
| Isaac CPU control | First joint-safety terminal 105; RMSE `0.05883/0.08140/0.21948` | GPU alone is not the explanation |
| Explicit `0.05` damping counterfactual | Terminal 92; RMSE `0.04889/0.06160/0.16246` | Insufficient fix; not promoted |

## D-016 — Correct action and geometry, retain the failed paired gate

- **Observation:** Origin feedback changes ankle action meaning with travel;
  old thigh/shank collision proxies have neutral gaps below the pinned
  40 mm pair contact distance. Broad residual probes encounter leg
  self-contact before a useful swing.
- **Evidence:** ADR-106 and the exact current action report. Body V4 changes
  six proxies only; tests preserve all other body fields, joints, actuators,
  materials, exclusions, effectors and mass properties.
- **Conclusion:** These are concrete action/reference and proxy defects.
  Correcting them enables bilateral CPU foot release but does not repair
  the independent Isaac discrepancy.
- **Decision:** Keep V4/V5 as separately hashed diagnostics. Use the smaller
  `WALKING-ACTION-REACHABILITY-P0` only to establish foot release; do not
  substitute two independent swings for the original within-episode
  alternation criterion or full correspondence.
- **Rejected alternatives:** Retuning PPO before an executable action control,
  selecting a shorter green prefix, transferring standing weights into a
  fourfold action range/new body, or redefining contact/safety tolerances.
- **Consequences:** No gait-reward implementation or optimizer run yet.
  Fix report defects and localize the first physical divergence.
- **Uncertainty:** Which loaded scene/solver/version difference causes the
  remaining mismatch. Damping is a real missing explicit parameter but its
  isolated correction was insufficient.
- **Reconsider when:** One causal mirror correction passes the original
  paired reproduction and standing non-regression; then test alternation
  and isolate duration-aware gait reward before a small fresh run.

## D-017 — Learn directly on canonical physics

- **Observation/evidence:** First substep starts with equal effort; divergence
  persists airborne. Explicit canonical contact offset .02, position iterations
  16 and damping .05, alone/together, still terminate. Loaded solver mass/COM/
  inertias match. Exact evidence is in the direct-CPU investigation.
- **Decision:** ADR-107 admits one fresh canonical CPU V5/CUDA PPO run, unchanged
  body/actions/reward/safety, 1,024,000 samples, then five fixed evaluations.
- **Rejected:** More speculative mirror tuning or claiming parameter matching
  solved correspondence; transferring incompatible standing weights.
- **Consequence:** Mirror remains failed but is not needed for this one direct
  CPU experiment. No runtime authority, no sweep/resume or reward override.
- **Uncertainty:** V5's instantaneous support reward may favor standing; a full
  alternating gait is still unproved. Diagnose the final run before successor.
- **Reconsider mirror when:** One demonstrated correction passes the original
  tape and unchanged standing control, not a shorter or improved prefix.
- **Next:** Clean-commit generation freeze, one bounded run, final evaluation.
- **Infrastructure repair:** Generation-01 failed before the first PPO update
  because native responses sort by `(episode ordinal, slot)`, not slot alone.
  Align by explicit slot; retain duplicate/missing/ordinal checks. Do not
  repeat the two-slot-only control. Generation-02 binds the repaired adapter
  to the unchanged profile and unexecuted optimizer budget.

## Active hypotheses

| Hypothesis | Update | Next test |
| --- | --- | --- |
| Action order/quantization/scaling causes first drift | Initial targets and all Q1.30 commands agree; disfavoured | Preserve exact comparison |
| Reset or velocity-frame reporting causes failure | Recorder defects repaired; physical terminal remains | Retain terminal commit and compare world-frame velocities |
| GPU-only numerical behavior causes failure | Isaac CPU also diverges | Compare loaded Isaac/native parameters and first substep |
| Missing linear damping is sufficient | Counterfactual fails | Do not repeat as the sole remedy |
| Solver/contact/version or another scene parameter differs | Consistent with first-tick physical drift | Inspect actual applied properties/efforts, then a one-variable correction |

## Earlier constraints that remain active

- R8b is an independent command-only lineage. The R141 reference problem is
  `INVALID / STOP_NO_RETRY`; R123–R141 artifacts, rejected TRAIN-5 checkpoints
  and the old reference corpus are never inputs.
- The 24-sphere/all-X-axis Stage 0 body is retired for humanoid learning.
  Biomechanics V2/standing V1 failed on contact-complete evaluation; their
  checkpoints cannot seed successors.
- CPU PhysX is canonical. Isaac outputs remain R&D until the fixed
  correspondence and CPU quality matrix pass.
- V3 standing is nominal evidence, not robustness or runtime promotion.
  The current walking body differs and cannot inherit its quality claim.
- Kimodo remains a separately proposed offline reference source. It supplies
  neither a controller nor a current R8b input. See
  [Kimodo research](../kimodo-motion-training-research-2026-09-03.md).
- Forward only, fixed 0.5 m/s lesson, no yaw/strafe/backward/terrain/recovery,
  no new reference corpus, and no runtime policy activation in this task.
- No optimizer/tolerance sweeps. Keep the original 3 m travel and final
  180-tick stop gate. A successful manual probe is not a learned policy.
- Full MODEL-MIRROR-P1 requires 256 × 600 samples and fixed thresholds.
  An early terminal or diagnostic prefix is a failed/incomplete corpus.

## Verification and remaining work

- Native motor tests (116) and focused Python tests (59) pass. Old standing
  descriptor is byte-exact. Play/content/reference replay pass; host-check
  was interrupted (143), and explicit PhysX runtime replay fails on bootstrap
  profile closure, cause unresolved. Exact scope is in the current report.
- Isaac audit now has an external result supervisor: the real failed tape
  returns exit 4. Do not rely on Kit's raw exit status; fast shutdown can
  return zero, while the tested non-fast shutdown segfaults on this host.
- Remaining implementation: execute ADR-107's direct CPU run and canonical
  evaluation; if needed, isolate duration-aware gait credit under a successor.
  A demonstrated mirror correction remains separately necessary for Isaac
  correspondence and promotion, not this direct CPU experiment.
- No full V5 training generation has been activated and no new checkpoint
  exists. The user has authorized necessary fixes and training; the blocker
  is technical evidence, not missing permission.

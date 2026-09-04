# R8b — actual sole support and walking-training method

Status: `REPORT_ONLY / V6_WALKING_FAILED / NEXT_LESSON_NOT_ADMITTED`.
This investigation changes diagnostic rendering, not the plant, reward, safety,
training profile or acceptance gates. It answers the user's toe-standing
question before any successor optimizer run.

## Closed V6 result

External root: `/home/kaifaty/NextEngine-training/r8b-canonical-walking-v2/`.
`generation-01/runs/TRAIN-1` binds clean code `88b6a43d`, profile
`canonical-rsl-rl-walking.v2.json` and the previously recorded descriptor.
All declared artifact hashes verify; all 1,000 metric rows have consecutive
iterations. Training completed 4,096,000 transitions. The final checkpoint,
not a selected intermediate, fails all five nominal evaluations identically:
382 ticks, 0.092032 m forward, `terminal.contact-impact`, no single support,
no support switches. These identical reset seeds are not robustness evidence.

| Artifact relative to external root | SHA-256 |
| --- | --- |
| `generation-01/runs/TRAIN-1/run-manifest.json` | `fb348f6a6da74abbb9df1302df1fbf3fd9309de260a0340cdd1159018c79bf18` |
| `generation-01/runs/TRAIN-1/model_999.pt` | `cca258c3f42fe066decae011b453f417d5bbb09385f2922d890c4c395fb372ba` |
| `generation-01/runs/TRAIN-1/evaluation.json` | `85dacd40f899455534926d83817b70fa1a1bc28bedd654c2904f789486955d60` |
| `evidence/final-1001-native-trace-02.json` | `b66acc527118432c1ae2298e29d6da0a4e34500bf337a966152d412efddb8fd7` |
| `evidence/final-1001-native-trace-03.json` | `4fa4f853febd498bc475505cf6e4705f858560b02ce310bceaffd1b54fa86fa7` |
| `evidence/contact-audit-03/report.json` | `5957a667a41da403016a5a3674fe2df860f3da0338af30453fbe633bfcd73140` |
| `evidence/reachability-control-native-trace.json` | `204219ec3f69b8499d202014fc2538737ce149011441a2a637b678613ffc349f` |
| `evidence/zero-v6-382-native-trace.json` | `4a26788179e59bae1e5394653f7b9680c9885179950c5704882fb52961553e7c` |

The diagnostic native executable closes its own hash and exact Q1.30 tape.
Root position, quaternion, linear velocity, all actuator-ordered joint positions,
commands and contact flags match the stored evaluation exactly. The original
headless executable remains `58b9d650…0325`; rebuilding the separate example
does not replace or relabel the training executable. Earlier trace/figure
versions remain immutable. Version -02 adds world contact positions; figure
audit -03 uses equal geometric aspect and the accurate `zero_command` label.
The final native build also rejects a non-string profile ID before producing
output; its trace -03 reproduces every frame, case and input identity from -02
exactly, while retaining the new executable hash.

## Competing explanations and discriminator

| Hypothesis | Prediction | Observed update |
| --- | --- | --- |
| The stick plot makes flat feet look like toes | Actual sole boxes are horizontal despite slanted body-origin connections | Supported for the apparent persistent toe stance; the original plot omitted soles |
| Anatomy/reset forces the body to start on its toes | Both heel ends are already elevated in the initial state/zero control | Refuted for the recorded nominal reset: initial toe-down angles are about 0.013 degrees |
| The learned motion sometimes rocks onto a toe edge | Heel-to-toe height difference and force location agree | Supported on the left: maximum 10.183 mm / 2.245 degrees, 28 moving ticks above 5 mm with pressure in the front quarter |
| Foot release happened but the contact margin hid it | Native box clearance is materially positive despite both flags | Not supported as an explanation of failed walking: left max 0.0045 mm, right max 4.025 mm; neither reaches 5 mm |

Measurements use the native body poses and descriptor collider transforms,
not forward kinematics from an independently reconstructed skeleton. Each foot
is one 260 × 110 × 60 mm box, with its local forward end designated the toe.
Heel/toe are geometric box ends, not anatomical toe joints. World up is +Y,
ground surface Y=0. Heel raise is the difference between the mean Y of the
two heel and two toe sole corners. Positive toe-down angle raises the heel.
The minimum corner is measured separately: raising the heel is not lifting
the whole foot. Pressure location weights actual contact positions by absolute
vertical impulse from the **last physical substep only**. It is not a whole-tick
support classifier. The 1/5 mm and front-quarter thresholds are report-only;
no acceptance threshold is replaced by them.

The left heel first exceeds 5 mm at tick 214 and last does so at 294; the
maximum is at 225. Right heel-above-toe never reaches 5 mm. At zero command,
maximum toe-down angles are below 0.05 degrees on both feet. Several flat-foot
frames have force concentrated at a toe contact: a force location alone does
not demonstrate a raised heel.

Two controls bound the interpretation:

- The existing exact 105-tick zero/left/right tapes reproduce safe bilateral
  release. Whole-sole maximum clearance is 115.85 / 149.85 mm in the separate
  left/right cases, with 29 / 32 ticks above 60 mm; zero has none. The recorder
  can detect a genuine lift. This is not evidence for a safe return or gait.
- V6 zero residual on the same seed, requested horizon 382, falls at 361.
  Both heels first exceed 5 mm at 332 and tilt about 35 degrees near the fall.
  The learned unilateral rocking starts earlier and differs from this late
  passive/control failure. Neither experiment supports a claim that anatomy
  imposes persistent toe standing. Zero residual is not guaranteed balance,
  but its failure is not, by itself, a bug in a residual-RL architecture.

PhysX generates contacts within a contact distance, before shapes touch;
presence and nonzero load are different observables. See the actual
[PhysX 5.4.1 collision documentation](https://nvidia-omniverse.github.io/PhysX/physx/5.4.1/docs/AdvancedCollisionDetection.html)
and [contact-point fields](https://nvidia-omniverse.github.io/PhysX/physx/5.1.0/_build/physx/latest/struct_px_contact_pair_point.html).
Here the geometry measurement, not that general fact, rejects hidden useful lift.

## Research: what successful training recipes actually specify

Primary sources accessed 2026-09-05; mutable repository examples are method
references, not pinned NextEngine inputs. No third-party implementation is copied.

| Method/source | Relevant mechanism | Bounded lesson for this engine |
| --- | --- | --- |
| [Siekmann et al., 2021, sections IV–V](https://arxiv.org/html/2011.01387v2) | Reference-free periodic force/speed costs, observable gait phase, recurrent policy, PPO; Cassie uses 40 Hz policy / 2,000 Hz PD and 150 million samples | Supports the phase idea, not our simplified load-ratio reward or our model's learnability; 4.096 million is not an equivalent training budget |
| [Humanoid-Gym, 2024, section III](https://arxiv.org/html/2404.05695v2), [native environment](https://raw.githubusercontent.com/roboterax/humanoid-gym/main/humanoid/envs/custom/humanoid_env.py) | PD joint-position actions, phase-conditioned procedural joint reference, contact pattern, clearance and velocity objectives, observation history | A successful humanoid lesson specifies actual step structure, not only weight transfer. Reference actions are optional; joint-reference rewards are distinct from adding a reference to actions |
| [Humanoid-Gym configuration](https://raw.githubusercontent.com/roboterax/humanoid-gym/main/humanoid/envs/custom/humanoid_config.py) | 4,096 environments, 60 steps/update, configured 3,001 updates, 15 actor history frames; 1 kHz simulation / 100 Hz policy; target foot height 60 mm | Configured ceiling is 737,525,760 transitions, not a claim that each paper run used them all. Never copy its gains, frequencies or 12-DOF assumptions blindly to our 23-DOF plant |
| [DeepMimic, 2018, sections 5–6 and 10.4](https://xbpeng.github.io/projects/DeepMimic/DeepMimic_2018.pdf) | PPO with task plus motion-imitation rewards; reference-state initialization and early termination | Reference motions can make natural style explicit. RSI is particularly important for dynamic skills, not universally necessary for walking; paper's walking ablation is similar without it. Retargeting and physical validity remain separate work |
| [Rudin et al., CoRL 2021/PMLR 2022](https://proceedings.mlr.press/v164/rudin22a.html) | Massive GPU simulation parallelism and terrain curriculum | GPU speeds data collection. The reported fast results are for quadrupedal ANYmal, not a transferable humanoid convergence guarantee |
| [HUGWBC, equations 8–9 and Table I](https://hugwbc.github.io/resources/HugWBC.pdf) | Distinct periodic contact-swing and foot-trajectory/height terms | Force timing and swing geometry are separate learning signals. Their combination motivates a bounded candidate, not a ready-made solution here |

The full common workflow is: validate the physical model and action interface;
choose explicit observations/control timing; define a reachable task and gait
objective or a retargeted motion prior; train actor/critic on many simulated
transitions; diagnose physical outcomes separately from reward; then test
unseen starts/commands/perturbations and the target simulator. Curriculum and
reference assistance are tools, not universal prerequisites. A lifted heel can
be valid during a walking rollover; demanding flat soles at every phase would
itself damage the walking objective. Quiet standing and swing/stance phases
need distinct expectations.

## Assessment of our approach and next decision

The infrastructure evidence supports using canonical physics, checked actions
and PPO. It does **not** establish a good locomotion lesson. The exact V6
formula is independent of foot height and fore-aft foot progression when
impulses and planar foot speeds are held fixed. Thus load credit cannot by
itself distinguish unloaded-but-grounded from lifted feet. This is a direct
inspection of the formula, not a proof that force-based RL can never walk.
Native evaluation shows weight transfer without useful clearance, matching
that limitation. The graphical omission hid an important diagnostic observable.

Do not launch another unchanged V6 run, declare the body infeasible, loosen
safety, impose an all-phase flat-foot constraint, or presume a single clearance
bonus solves the task. No V7 implementation, weights initialization or larger
training budget is admitted by this report.

Smallest next action: define one phase-conditioned lift-and-return lesson
that distinguishes the current rocking control from actual bilateral release
and safe re-contact. Test geometric reward observables against the existing
positive single-swing controls and negative unloading/flight/wrong-phase cases;
retain the failed return safety evidence. Select the new objective only after
these discriminators. If a procedural joint/foot reference is used, validate it
on this morphology rather than recycling the failed manual return or retired
R141 corpus. Then freeze a separate environment/run with explicit diagnostic
milestones and a justified sample budget. The original final travel, start/stop,
single-support and safety matrix remains unchanged.

Remaining `UNRESOLVED`: whether the next lesson learns a coordinated safe return;
whether observation history is useful for the rate-limited actuator state;
how many samples this actual plant needs. These are experiments, not conclusions
drawn from another robot's hyperparameters. CPU/Isaac correspondence remains
failed and is not altered by the present diagnostic work.

## Verification / reproduction

Use `PYTHONPATH=lab` and the pinned external Isaac Python. Invoke the module
`python -m lab.scripts.cpu_walking_contact_audit prepare` with `--run`, `--seed`
and fresh `--tape`; replay that tape with the native
`audit_biomechanics_action_tape` example; invoke `analyze` with the same inputs,
`--trace` and a fresh `--output`. The companion tests check real box geometry,
pitch sign, weighted pressure, contact/clearance separation and fail-closed
exact replay under pose/velocity/joint/command/contact corruption.

The first two corruption-test fixtures used integer arrays and truncated the
intended floating corruption back to zero. Correcting fixture dtype makes
those negative tests effective; no simulator or evidence was changed.

- PASS: 26 focused Python tests, Ruff, formatting and native diagnostic clippy.
- PASS: exact final-evaluation native replay; successful 105-tick reachability
  control and full retained zero-action failure.
- PASS: V6 implementation's prior 120 motor tests, five headless tests and
  complete Linux `host-check` (`evidence/host-check.log`).
- FAILED: learned walking quality, independently of successful execution.
- NOT RUN: successor training, runtime/export admission and renewed mirror
  gate. New changes are diagnostic-only; broad checks are not repeated solely
  for a report/renderer change.

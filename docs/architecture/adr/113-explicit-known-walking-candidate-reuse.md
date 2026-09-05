# ADR-113: Explicit known walking candidate reuse

| Field | Value |
| --- | --- |
| ID | ADR-113 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-05 |
| Dependencies | [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [ADR-111](111-final-weight-corrected-walking-evaluation.md), [ADR-112](112-prospective-validated-walking-training.md) |
| Supersedes | Blanket exclusion of separately reusing old intermediate weights in ADR-109/111/112, only for the explicit candidate below; no old selection rule, result, safety gate or statistical claim changes |
| Superseded by | none |

## Product decision and evidence

The user wants a model that walks. The closed V7 diagnostic model 3999 already
completed 1,200 ticks and 6.135 m with real bilateral foot release, while final
9999 failed. Requiring a successful *last* state is a frozen experiment rule,
not a physical requirement for a useful saved model. The old V7 run and ADR-111
final-only evaluation remain failed; an exploratory reuse must not relabel them
or claim unbiased held-out quality.

Admit one explicitly **known-result-selected** candidate evaluation under
[known-walking-candidate.v1.json](../../../lab/profiles/known-walking-candidate.v1.json).
It binds model 3999 SHA-256
`606cc8101f1cc299d0be1ff3b952c751ac16953592116e19022f497db53e90c3`,
the entire completed source-run/generation closure and the exact V8 descriptor.
Validate all source artifacts through the unchanged final-source validator,
then separately require this candidate's declared artifact hash and integer
iteration. No candidate search, optimizer, normalization update, initialization,
source mutation or claim that old final-only validation passed is admitted.

## Same physical outcome, honest claim boundary

Execute the policy closed-loop on fresh V8 scenes with deterministic mean
actions. Preserve every ADR-111 threshold and all five seeds 1001..1005:
1,200 safe ticks, >=3 m, forward velocity MAE <=0.2 m/s, 180 final applied zero
commands, stop speed MAE <=0.1 m/s, >=8 continuous single-support ticks per side
and >=2 qualified support switches. Require exact native replay of every
action tape, all-four-substep exclusive loaded support and positive canonical
post-step whole-foot clearance. An old successful open-loop tape is not a
substitute for this closed-loop evaluation. Retain the complete video, not a
selected prefix, and visually check that the measured behavior is walking.

Passing yields an explicitly selected, bounded nominal walking artifact. It
does not establish statistical generalization, algorithm convergence, another
body's standing retention, natural style, arbitrary commands, terrain/recovery,
Isaac correspondence or runtime/export promotion. Repeating identical nominal
starts does not create robustness evidence. Failure remains a failed candidate;
do not scan other checkpoints under this decision.

The separate ADR-112 fresh run may continue while this read-only evaluation
runs. If the candidate fully meets the user's bounded walking outcome, it is
permissible to stop redundant compute through the existing trainer's handled
interrupt, retaining its closed interrupted run and all checkpoints. Never
rewrite it as a completed successful training experiment.

## Checks and rollback

Use focused exact-candidate/iteration/path/hash tests plus the unchanged source
closure, native corrected-support and validation-state/RNG controls. Native
body/actions/safety are unchanged; no new broad host-check is necessary.
Record clean code, candidate profile, source and tool hashes before evaluation.
Preserve all full results externally. Cost is five <=1,200-tick rollouts and
their native replays, bounded O(5 * 1,200). Rollback rejects this candidate while
leaving both old results and the independent fresh run intact.

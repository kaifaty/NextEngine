# Walking action-basis corrections — 2026-09-04

## Result

The origin-dependent walking reference and insufficient leg-proxy clearance
have concrete corrections under ADR-106. Canonical CPU now reaches sustained
single support on both sides. The paired action audit still fails: Isaac
diverges before any walking command and later triggers joint safety. No new
optimizer run was started and no walking-quality claim is made.

## Changes and controls

Walking environment V4 removes only the standing reference's absolute
forward-position term. V5 retains that correction, widens the normalized
residual by four and uses BodySchema V4's narrower thigh/shank/knee proxies.
These are isolated successor identities; old environments remain available
with their original meanings.

On the original geometry, leg probes first trigger thigh/thigh contact and
then shank/shank contact. The old neutral gaps (`4.52 mm` thigh, `39.02 mm`
shank) are smaller than the pinned `40 mm` pair contact distance. The new
gaps are `44.52 mm` thigh, `59.02 mm` shank and `49.02 mm` knee. Tests compare
all body fields after restoring only the old colliders and verify identical
joints, actuators, material/collision policy and source mass properties.
The revised proxies are a simulation approximation; no new anthropometric
accuracy claim is inferred from these dimensions.

The manual action bank is a reachability probe selected after bounded CPU
exploration, not held-out gait evidence. Three independent episodes use zero,
left swing and its anatomical action mirror. Two mirrored successful episodes
do not satisfy the earlier requirement for alternation within one episode.
The original alternation/full-correspondence gate is therefore still open.

## Retained exact evidence

External root:
`/home/kaifaty/NextEngine-training/r8b-walking-action-basis-v5/evidence/`.
These are development-tree diagnostics, not clean-commit training manifests.
Copied reports preserve their original temporary trajectory paths; the NPZ
files are retained beside them under the same basenames.

| Input or report | SHA-256 |
| --- | --- |
| `nextengine-v5-descriptor.json` | `0f4610fe25d52cd128b651a1533544f45bf20c855dd18d917e82eaf65fdcdb03` |
| canonical raw Q1.30 tape | `bdcaf944de58ade7472d166581618e5227ffb9121b715ac47f559ec9b2bec097` |
| `nextengine-walking-v5-cpu-audit-v2.json` | `4d68d5464f899e7d76b3bf69bec1d6de50a8938e97b68b5ed26fe89519616b3f` |
| `nextengine-walking-v5-gpu-audit-v2.json` | `8c26dd2d25a6eda13c879eb1c7af530701354edc400cc0af707253552bf9638f` |
| `nextengine-walking-v5-isaac-cpu-audit.json` | `7b973fb2d620991b1dc564df89acdaa9ed61152d65fc09d506947eaca4dd2ab0` |
| `nextengine-walking-v5-damping-audit.json` | `63605e717f5de09b5bcaa82e474e844f4e0ab89f26b48d9ac15328c07a2e041a` |
| `cpu-final.json` | `2e3f579b97df345ef69f42e09cb88f2af8957b52dcbb69f54fb7c48db43df8e5` |
| `cpu-final.npz` | `24fe4a63322045181bf139466e3328c64d8354aab739a05aceecdec65c6a59dc` |
| `gpu-supervised.json` | `ace0c6b6dbabe8d37d5c0335320471a601f5b2f0ffdc38496af44b6be8c68a3c` |
| `gpu-supervised.npz` | `423a6a67e1af2f140dfb35d8d099291ad33e9cd4a6864031ddad81e884998247` |

The final CPU binary SHA-256 is
`e3c967db3e5c92fcd43e130ca768ee6b1c86958e26a9f75f3bf8c38aca9a55e0`.
The USD translation bundle is preserved under this evidence root's `derived/`
directory. Its `humanoid.usda` SHA-256 is
`fd668b64b50541d31549c014463bdae9d790eca2fcba11318249658c1d6eac33`.

The final script names the restricted check
`WALKING-ACTION-REACHABILITY-P0`. Development reports above used the earlier
`WALKING-ACTION-BASIS-P1` diagnostic name; neither is MODEL-MIRROR-P1.
The Isaac CPU development report also retained the earlier generic
`isaac-gpu-mirror` plane label; its explicit `device: cpu` is the actual
execution plane. The final recorder fixes the label.

BodySchema hash is
`30c47391f84dd6e68249c2fb88a69917cc2ab7a14ef7990e75870ee59366784a`;
environment manifest is
`576189dfcdc43e7215957fdc16ef724e094a6e1551431332049a0384cae230b8`;
action layout is
`c1db18e9b6b0dacd0a2f30292a47129d60f6e284ae2521910acaec626d539d2f`.

## Measured outcomes

| Execution | Observed horizon | Left/right continuous single support | First terminal | Joint / root-position / root-velocity RMSE against canonical CPU |
| --- | ---: | ---: | --- | --- |
| Canonical CPU PhysX | 105 | 31 / 34 ticks | none | reference |
| Isaac GPU | 96 | 34 / 31 ticks | right joint safety, 96 | `0.05247 rad / 0.06926 m / 0.18366 m/s` |
| Isaac CPU | 105 | see retained report | left joint safety, 105 | `0.05883 rad / 0.08140 m / 0.21948 m/s` |
| Isaac GPU, explicit CPU damping probe | 92 | 26 / 27 ticks | left joint safety, 92 | `0.04889 rad / 0.06160 m / 0.16246 m/s` |

CPU forward displacement is `0.045964 m` for left swing and `0.115136 m` for
right swing; zero action also moves `0.061948 m`. All are measured from the
first committed frame to the last. This demonstrates foot release and
positive movement, not useful walking or superior progress to zero action.

The corrected GPU recorder captures action/contact facts before automatic
reset and uses committed `last_step` world-frame snapshots. The first version
read reset state and compared GPU root-local velocity to CPU world velocity;
that version's derived trajectory metrics are invalid and are not used here.
The corrected action bytes match exactly. Target angles match through ticks
1 and 2; physical joint position already differs by up to `0.004674 rad` at
tick 1. Only ankle reference targets diverge from tick 3 as physical feedback
diverges. At tick 96, zero action has moved roughly `0.332 m` in Isaac versus
`0.060 m` canonically. This is present before the first nonzero command.

## Hypotheses and decision

| Hypothesis | Prediction | Observation / update |
| --- | --- | --- |
| Action order, quantization or scale is the first error | Initial targets/actions differ | Falsified at the action boundary: exact tape and initial targets agree |
| Automatic reset explains the reported failure | Correct terminal recording removes physical failure | Reporting defect fixed; joint-safety failure remains |
| GPU alone causes the drift | Isaac CPU tracks canonical CPU closely | Disfavoured: Isaac CPU also diverges and terminates |
| Omitted linear damping is sufficient | Explicit canonical `0.05` damping closes the gap | Falsified as a sufficient repair; RMSE improves slightly but still fails |
| Solver/contact/version or another scene parameter differs | Same action/targets produce different early physical state on both Isaac modes | Supported as the remaining class of causes, exact cause unresolved |

The CPU compiler explicitly supplies `0.05` linear/angular damping; the
current biomechanics USD does not author linear damping. The diagnostic
override is not activated in the production environment because it has not
established correspondence and needs complete descriptor/translator closure.
Earlier paired research also found CPU PCM changes destroy the standing
control; do not retry that rejected mutation or relax tolerances.

Next: compare loaded link mass/CoM/inertia, joint axes/frames/limits, damping,
contact settings and per-substep effort at the first two motor ticks, against
the actual canonical descriptors. Isolate one demonstrated mismatch with an
exact input golden before changing the mirror. Rerun this tape and the standing
control after a causal correction; only then reconsider the full alternating
action audit and separate gait-credit training discriminator.

## Verification

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| Native `next_motor` / `next_headless` all-target Clippy, warnings denied | PASS |
| `cargo test -p next_motor --features physx-sdk --lib` | PASS: 116 tests, zero ignored |
| Focused Python audit, mirror, training, safety-contact and USD translation tests | PASS: 59 tests |
| Original standing descriptor against retained standing artifact | PASS: byte-exact `cmp` |
| `play`, `content-package`, reference `persistence-replay` | PASS |
| `platform` | PASS portable contract; SDL/Ash candidate NOT_RUN_ADAPTER_DISABLED |
| Full `host-check` | INCOMPLETE: signal 143 during workspace tests, after formatting/Clippy |
| Explicit PhysX `persistence-replay --backend physx` | FAILED: `runtime bootstrap world/profile closure does not match`; cause not established |
| Final canonical action tape | PASS: complete 105-tick reachability probe |
| Final supervised Isaac action tape | FAILED: right joint safety at 96; command correctly exits 4 |
| Diff whitespace and changed documentation links | PASS |

Logs are preserved beside the reports. No claim is made that the separate
PhysX runtime bootstrap failure is pre-existing or caused by this change;
the native motor reset/replay tests do pass. Full workspace verification
remains incomplete.

Isaac's default fast shutdown exits Python with status zero before a later
`SystemExit(4)` can propagate. A bounded `fast_shutdown=False` trial wrote the
same failed trajectory but then segfaulted on shutdown (exit 139); it is not
retained as the implementation. The audit now runs Kit in a child process
and checks the fresh report in its parent. A child error, missing/malformed
report, non-passing status or false/empty gates fails closed. Tests include
stale-report rejection; the real `gpu-supervised.json` run returned 4.

No optimizer, full MODEL-MIRROR-P1/P2 corpus, within-episode alternating gait,
learned walking evaluation, performance check or runtime promotion was run.
A passing source-level or reachability check does not imply any of them.

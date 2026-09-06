# Physical sound V25 — M0 implementation-conformance rebaseline

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `M0_CONTRACT_DEFECT / VALUES_UNOPENED / SUPERSEDED_BEFORE_OFFICIAL_EXECUTION / M0A_PROTOCOL_REQUIRED` |
| Affected package | Roadmap V25 `M0-I` |
| Product effect | None; no model, metric, protected role, generated sound or runtime value was opened |

## Observation

The frozen V24 M0 protocol requires the candidate to satisfy isolated causal
frequency counterfactuals for Young's modulus `E`, density, thickness and
length/planar scale. The same protocol defines the model input as a 24-value
geometry descriptor plus material/support one-hot-and-known-mask features.

That input can vary geometry and categorical material identity, but it contains
no independently variable `E` or density coordinate. Holding material identity
fixed while changing only `E` or only density therefore leaves the complete
model input byte-identical. A deterministic model must return the same
frequency vector for both sides of either counterfactual, while the required
analytic ratios are respectively `2.0` and `0.5` for a `4x` change.

## Evidence

- [M0 protocol](physical-sound-v24-m0-exact-object-neural-student-protocol-2026-09-01.md)
  freezes material/support as one-hot-plus-known-mask inputs and separately
  requires isolated `E` and density exponents within `10%`.
- [T0 teacher](physical-sound-v24-t0-analytic-teacher-result-2026-09-01.md)
  proves those analytic controls, but its V3 row exposes only `material_id`,
  not independently mutable material parameters.
- No official M0 checkpoint, prediction, development metric, method holdout or
  admission-shadow value exists. The candidate family is still unspent as a
  quality experiment.

## Conclusion

The original M0 representation cannot satisfy its own causal gate. Treating
the already-passed T0 teacher control as the M0 candidate control would make
the gate vacuous: it would verify the teacher twice without verifying what the
model learned. Switching between material one-hot values would change several
physical properties simultaneously and would not be an isolated `E` or
density counterfactual.

This is a protocol-conformance defect, not a model-quality failure. It was
found before model values, so the safe correction is a preregistered successor,
not a post-result retry.

## Decision

[M0a](physical-sound-v25-m0a-causal-material-neural-student-protocol-2026-09-02.md)
supersedes M0 before implementation completion. It retains the same seed,
capacity ceiling, architecture widths, optimizer schedule, losses, evidence
roles, access order, resource limits, controls and stop policy, but adds a
hash-bound physical-material feature vector with an explicit known mask.

T0 rows receive the exact analytic material parameters already frozen by the
teacher implementation. X0 Glass retains a known semantic label but receives
zero physical values with `physical_parameters_known=0`; no real parameter is
guessed. The implementation fixture must demonstrate that independent `E`,
density, thickness and scale mutations reach distinct candidate inputs before
any official run.

## Rejected alternatives

- **Count the T0 teacher's analytic control as the candidate gate:** rejects
  because it does not test the learned representation.
- **Use material one-hot changes as counterfactuals:** rejects because `E`,
  density, Poisson ratio and damping change together.
- **Infer Glass constants from its label:** rejects because the internet source
  does not establish composition or those numeric axes.
- **Delete the causal gate:** rejects because causal material response is the
  reason to prefer a physical modal student over an unconstrained sound model.
- **Open official values and decide later:** rejects because it would turn a
  contract correction into result-driven tuning.

## Remaining uncertainty and next action

The new coordinates make the gates observable; they do not guarantee the
compact model will learn them or beat ridge/nearest controls. The smallest next
action is to complete the M0a full-entry contract fixture, prove role privacy
and deterministic bytes, commit it, and only then run official A/B.

Reconsider the added physical vector only if a future source supplies a
different authoritative material parameterization or the unopened M0a
contract fixture proves another value-independent inconsistency.

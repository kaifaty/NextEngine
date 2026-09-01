# Roadmap V24: data-first offline neural acoustic assets

| Field | Value |
| --- | --- |
| Rebaseline date | `2026-09-01` |
| Status | `ADOPTED / V23_IMPLEMENTATION_REJECT / QUALITY_UNOBSERVED / D0_T0_X0_REPEAT_EXACT_PASS / COMBINED_V3_VALIDATED_AXIS_INCOMPLETE / M0_PROTOCOL_FREEZE_NEXT / DATA_FIRST_ML / INTERNET_ONLY_REAL_EVIDENCE / AUTOMATED_VALIDATOR_REQUIRED / CLIP_FALLBACK / RUNTIME_NOT_AUTHORIZED` |
| Replaces | [Roadmap V23](physical-sound-synthesis-roadmap-v23.md) as planning authority; its frozen P2a protocol remains binding for the one bounded V23 closeout |
| Evidence basis | [X0 result](../development/physical-sound-v24-x0-blue-bowl-pilot-result-2026-09-01.md), [T0 result](../development/physical-sound-v24-t0-analytic-teacher-result-2026-09-01.md), [D0 result](../development/physical-sound-v24-d0-neural-evidence-plane-result-2026-09-01.md), [V23 F2a result](../development/physical-sound-v23-f2a-fixed-feature-ridge-result-2026-09-01.md), [V23 P2a](../development/physical-sound-v23-p2a-fixed-feature-ridge-protocol-2026-09-01.md), [V23 research](../development/physical-sound-v23-closed-form-field-research-2026-09-01.md), [V22 result](../development/physical-sound-v22-f1r-resource-bounded-result-2026-09-01.md), [M0b automatic-metric result](../development/physical-sound-v20-m0b-confound-resistant-metric-result-2026-09-01.md), [neural acoustic-field strategy](../development/physical-sound-neural-acoustic-field-strategy-2026-08-30.md) and the existing internet-source registry evidence referenced by SPEC-45 |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed`; no current public/runtime contract |
| Mandatory fallback | Existing authored/recorded clip for every reject, OOD, missing source, unsupported material or tooling failure |

## Product-shaped outcome

Build one end-to-end rigid-impact authoring vertical in which:

1. geometry, material/support metadata and a contact query enter an external
   offline generator;
2. a compact neural model predicts a bounded modal source description rather
   than an unconstrained waveform;
3. an independently frozen automatic validator decides `Pass`, `Reject` or
   `FallbackOutOfDomain` without asking the user to approve each sound;
4. accepted output is deterministically cooked into ordinary audio assets;
5. one demo prop plays those assets through the existing presentation-only
   audio path and retains a complete authored clip fallback.

This roadmap is complete for the first vertical only when one protected Metal
shadow passes the frozen automatic admission and the cooked result runs in the
demo. Glass and Wood then repeat independent admission. It does not promise a
universal material model, runtime inference, rolling, scraping, fracture,
fluids, fire, cloth or biological sound.

## Why the plan changes

- B0, C0 and M0b prove useful deterministic pieces: global modal fitting,
  coverage/OOD structure and phase-tolerant metrics.
- F0 is reproducible but violates the strict topology-gradient gate. V21,
  V22 and V23 never published fresh quality because execution failed first. Repeatedly
  replacing only the residual formula is not a path to the final product.
- The limiting resource is now labelled, causally useful evidence, not model
  capacity. Internet recordings are plentiful but often omit synchronized
  geometry, contact, force, support or listener axes.
- Synthetic physics can supervise causality; real internet recordings can
  calibrate realism, material identity, domain gap and rejection risk. Neither
  evidence class may silently claim the other's missing axes.
- A neural generator and its validator must be separate. A model cannot certify
  its own output, and a single generic audio embedding is not acceptance
  authority.

## Target system boundary

```text
published geometry/transfer/audio evidence    deterministic synthetic teacher
                    \                         /
                     hash-closed research corpus
                                |
                 offline contact-conditioned student
                                |
              modal frequencies + damping + gain field
                   + bounded residual + uncertainty
                                |
        independent hard/physical/perceptual/OOD validator
                         |                  |
                       Pass          Reject / OOD
                         |                  |
             deterministic clip cooker   authored fallback
                         |
                 existing AudioMixerV1/demo
```

Research datasets, feature caches, checkpoints, generated WAVs and validator
weights remain outside Git. Runtime receives no model and no research registry.
Only a future promoting ADR may define an engine-owned public content contract
or committed contact projection.

## Evidence lanes

### Lane A — causal teacher evidence

Use deterministic analytic/FEM/DiffSound-style offline controls to vary known
geometry, thickness, support, material parameters and contact position. The
teacher provides supervised modal frequencies, damping and contact gains. Its
claim is synthetic representation and sensitivity, never real-material truth.

### Lane B — exact real transfer evidence

Use only internet sources whose immutable revision exposes enough geometry,
impact position, listener/transfer and provenance axes. REALIMPACT-style rows
are the current strongest candidates. Missing raw force or composition remains
explicit and narrows the loss/claim instead of receiving inferred metadata.

### Lane C — identified real recordings

Use AV-MSF, YCB Impact, Heller, Freesound and other already reviewed adapters
for real acoustics, material identity, temporal envelope and validator
calibration. These recordings do not supervise contact transfer unless their
source actually exposes that axis.

Every lane uses publisher/project/revision/object-disjoint roles. Training,
development, calibration, holdout, validator and untouched shadow identities
are hash-frozen before their values can affect a decision. The user records no
audio locally.

## Learned representation

The first neural student is object-specific and contact-conditioned at one
canonical listener. It consumes bounded mesh/shape descriptors, declared
material/support metadata and an intrinsic contact query, and predicts:

- sorted modal frequencies and positive damping;
- signed, normalized modal gains at the contact;
- a small coloured-residual descriptor;
- calibrated uncertainty/OOD evidence.

Training combines synthetic parameter supervision with phase-tolerant real
losses: multi-resolution log spectrum, decay-envelope slope, modal peak
consistency, contact continuity and remesh consistency. Unknown excitation
gain, microphone response and alignment remain declared nuisance variables.

The first model comparison contains only:

1. frozen B0/F0-compatible classical modal and continuous controls;
2. one compact continuous contact-query neural field;
3. causal ablations without geometry, contact or residual information.

A shared unseen-object geometry encoder is a later package. It cannot be
selected merely because the exact-object student passes. Direct waveform
generation remains a report-only upper bound and cannot satisfy cooker
admission.

## Automatic validator

The validator is an independently versioned ensemble, frozen before candidate
holdout or shadow values:

1. **hard validator:** schema, finiteness, bounds, hashes, provenance, access,
   deterministic render and exact fallback behavior;
2. **physical validator:** modal order, damping/decay, force-scale relations,
   contact continuity, remesh invariance and counterfactual sensitivity;
3. **real-acoustics validator:** temporal-spectral/decay descriptors plus at
   least one frozen pretrained audio representation calibrated only on real
   roles;
4. **mutation validator:** known bad frozen-spectrum, shuffled-envelope,
   stationary-noise, ringing, clipping, silence and metadata/identity
   corruptions;
5. **selective-risk validator:** group-disjoint OOD calibration and a
   predeclared confidence bound on false passes.

No single component can create `Pass`. Generator checkpoints, synthetic teacher
targets and candidate development outputs are forbidden validator inputs.
Human listening is optional diagnosis and demo review; it never accepts an
individual asset or changes a threshold.

## Work packages and observable gates

| ID | Package | State | Observable exit criterion |
| --- | --- | --- | --- |
| C0 | V23 bounded closeout | `COMPLETE / IMPLEMENTATION_CONFORMANCE_REJECT / NO_ARTIFACT` | [F2a result](../development/physical-sound-v23-f2a-fixed-feature-ridge-result-2026-09-01.md) records the missing inherited `analytic_surface` callback, spent `2601…2712`, absent artifact, unstarted B and retired unopened later roles. No quality inference or V23 repair is allowed. |
| D0 | External dataset contract | `COMPLETE / REPEAT_EXACT_PASS / CONTRACT_ONLY` | [D0 result](../development/physical-sound-v24-d0-neural-evidence-plane-result-2026-09-01.md) records two clean `10/10` focused executions, twice-exact full `run_cli` output, V2 compatibility, three claim-safe lanes, protected-role privacy and disabled training authority. |
| T0 | Synthetic teacher corpus | `COMPLETE / REPEAT_EXACT_PASS / SYNTHETIC_ONLY` | [T0 result](../development/physical-sound-v24-t0-analytic-teacher-result-2026-09-01.md) records two byte-identical 207-file runs: twelve analytic plate/beam objects, 144 contact rows, exact remesh/force controls and all authority flags disabled. |
| X0 | Exact real-object pilot | `COMPLETE / REPEAT_EXACT_PASS / DISCLOSED_DEVELOPMENT_ONLY` | [X0 result](../development/physical-sound-v24-x0-blue-bowl-pilot-result-2026-09-01.md) records two exact 4+3 lane builds and a 151-row combined V3 owner pass; incomplete real axes remain absent and contact `4` stays sealed. |
| M0 | Exact-object neural student | `PROTOCOL_FREEZE_NEXT / VALUES_UNOPENED` | Frozen compact student beats the frozen classical controls on unopened synthetic contact/remesh gates and improves disclosed real development metrics without weakening physical gates. |
| V0 | Independent validator v1 | `PROTOCOL_MAY_START / SEALED_BEFORE_M1` | Frozen real-only calibration, held positives, adversarial negatives and OOD groups establish declared coverage plus false-pass confidence; all hard/mutation cells pass twice exactly. |
| M1 | Cross-object Metal candidate | `BLOCKED_BY_M0_AND_V0` | A preregistered geometry-conditioned successor passes object-disjoint development and one-shot holdout; no role-specific tuning or per-object threshold exists. |
| A0 | Protected Metal admission | `BLOCKED_BY_M1` | Independent validator processes one untouched Metal shadow exactly once and returns `Pass`; reject/OOD leaves the material fallback-only. |
| K0 | Deterministic cooker | `BLOCKED_BY_A0_PASS` | Same accepted record cooks byte-identical clips and provenance twice; invalid/unknown records select the authored fallback and publish no partial asset. |
| D1 | One demo prop | `BLOCKED_BY_K0` | Opt-in demo uses cooked clips through the existing audio presentation path; disabling the feature and every fault reproduce the ordinary authored clip behavior. |
| G0 | Glass admission | `BLOCKED_BY_METAL_VERTICAL` | New source roles, model decision, validator calibration/holdout/shadow and cooker evidence; no Metal threshold or owner decision is inherited. |
| W0 | Wood admission | `BLOCKED_BY_METAL_VERTICAL` | Same independent process as Glass; existing pleasant audition clips are controls, not automatic pass evidence. |
| P3 | Production promotion | `POST_RESEARCH / ADR_REQUIRED` | Concrete consumer, admissible evidence, Linux cost, committed contact projection, content/fault/migration contract and complete fallback justify a separate Accepted ADR and ProductChecks. |

## Ordered critical path

1. Treat the [V23 F2a implementation reject](../development/physical-sound-v23-f2a-fixed-feature-ridge-result-2026-09-01.md)
   as final: no repair, B or quality claim; keep `2801…2912` unopened and
   retired.
2. Keep the completed external V3 dataset record as the sole corpus authority.
   Source acquisition may proceed in parallel with step 3 against its frozen
   three-lane contract.
3. Produce the deterministic teacher corpus and one exact-real-object pilot.
   Do not train when either record is structurally invalid.
4. Freeze the compact neural student, controls, losses, resource ceiling and
   unopened roles; train/evaluate M0 once.
5. Freeze the independent validator before cross-object candidate values. Its
   thresholds come from real controls and mutations, never candidate outputs.
6. Run Metal development, one-shot holdout and protected shadow in order.
7. On pass only, cook clips and wire one opt-in demo prop with fallback.
8. Admit Glass and Wood independently. Only after those verticals is a broader
   material/shape model worth evaluating.

## Branch and stop rules

- **V23 implementation failure (observed):** the execution family is closed.
  D0 must prove its complete owning entry point on value-independent records
  before opening fresh teacher/real/model identities; V23 is not repaired.
- **Internet source lacks a required axis:** narrow the claim or return
  `FallbackOutOfDomain`; never infer geometry, force, support or contact from a
  material label.
- **M0 fails synthetic controls:** stop before real holdout and change the
  representation through a new preregistered hypothesis, not capacity search.
- **M0 passes synthetic but fails disclosed real data:** investigate source,
  excitation/listener nuisance and domain gap before increasing model size.
- **Validator cannot bound false-pass risk:** no admission is possible. Grow
  independent real/mutation groups; do not ask the user to approve the queue.
- **Protected holdout/shadow rejects:** the material remains fallback-only;
  opened values never select a retry.

## Delivery checkpoints

| Checkpoint | User-visible meaning |
| --- | --- |
| V23 closeout | The formula-only branch is closed without a quality claim; its values and failures cannot drive another nearby retry. |
| Dataset + teacher | The system can turn internet evidence and controlled physics into reproducible training records. |
| M0 + V0 | A model can generate contact-dependent modal assets, and a different system can reject bad ones automatically. |
| Metal A0 + K0 | One material passes an untouched automatic test and becomes deterministic ordinary clips. |
| Demo D1 | The demo scene uses generated sound while remaining fully safe offline. |
| Glass/Wood | The same machinery generalizes material-by-material without manual per-sound validation. |

## Definition of done

V24 is done when the V23 baseline has a final recorded result, one Metal model
is trained from hash-closed synthetic plus internet evidence, an independently
frozen validator accepts one untouched Metal shadow exactly once, and a
deterministic cooker supplies one opt-in demo prop with a complete authored
fallback. No user microphone recording or per-sound human approval is required.

Glass and Wood are follow-on admissions, not conditions for claiming the first
Metal vertical. A public/runtime physical-sound feature remains blocked until a
separate architecture promotion.

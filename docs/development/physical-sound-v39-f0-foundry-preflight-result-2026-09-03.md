# Physical sound V39 F0 foundry preflight result

Date: `2026-09-03`

Result: `COMPLETE / REPEAT_EXACT_MECHANICS_READY / TARGET_FREE / NO_PRODUCT_AUTHORITY`

Roadmap authority: [Roadmap V39](../plans/physical-sound-synthesis-roadmap-v39.md)

## Claim

F0 proves only that the two-lane foundry machinery is reproducible before any
real source payload, signal, feature, model target, candidate output or
protected role is opened. It does not prove that a generated sound resembles
Steel, Glass, Wood or any real material; it does not authorize training,
validator release, admission, cooking, demo integration or a public contract.

## Frozen implementation

- owner: `lab/scripts/physical_sound_v39_f0_foundry_preflight_v1.py`,
  `sha256=4c1f458dd32de8e5e197a574c02b1259aef384fbca762248414b51aeed41f1f2`,
  `26,903 bytes`;
- profile: `lab/profiles/physical-sound-v39-f0-foundry.v1.json`,
  `sha256=a00e348e8ba7549a5c32ae90ca2e001a127762438e87c1b90159dedd6e3b99a7`;
- the profile binds the existing dataset contract, exposure ledger, research
  record and Rust neural data-plane owners by exact path, byte count and hash;
- the only successful F0 terminal is `MechanicsReady`; the next authorized
  stage is `V39-F1-source-frontier-automation`.

The profile freezes two candidate families, at most eight development
iterations per family, one seed, `20,000` training steps, `1,800 s`, `2 GiB`
peak RSS and protected-work WIP `1`. These are future ceilings, not training
performed or authorized by F0.

## Role and access proof

The synthetic fixture contains one project in each of six roles:

- permanent disclosed: `generator_train`, `generator_development`,
  `validator_calibration`;
- protected and sealed: `generator_method_holdout`,
  `validator_qualification`, `joint_admission_shadow`.

Project/revision, object-parent and recording-parent sets are pairwise
disjoint. A disclosed project is permanently ineligible for a protected role.
Generator and validator namespaces share neither learned artifacts nor
thresholds.

Both aggregate and per-project counters are exact zero for network requests,
source bytes, PCM samples, force samples, feature values, model targets,
candidate outputs and protected signal values. Protected roles remain unopened;
no generator or validator release is frozen.

## External A/B evidence

The official external root is:

`/home/kaifaty/.codex/experiments/nextengine-physical-sound-v39-f0-2026-09-03`

`run-a` and `run-b` were generated independently from the tracked profile.
`run-validate` then consumed the canonical `run-a` role plan and access ledger.
All three five-file trees are byte-identical. The deterministic tree root is:

`155025ddddf04f2ff63ef9ba76c4f0ee295f97a305c96310ad2dc03f9385b508`

| Artifact | SHA-256 |
| --- | --- |
| `access-ledger.json` | `0940675c5e7cf3b8ab7f945796978960a17cbd251b5f2721dc9c4fc9c09e2f2e` |
| `descriptor.json` | `cf33f958f02e1b86aeeba8f1617c663118d44f346b9ce326401ab42d5594c287` |
| `profile.json` | `a00e348e8ba7549a5c32ae90ca2e001a127762438e87c1b90159dedd6e3b99a7` |
| `report.json` | `961fc3a0191f47d9e23bae88751297418e0f9e6de60c6e4f74ad89800f0ebd7f` |
| `role-plan.json` | `ac5293dc394fc81cdeaa1e66fec807176054f1a3106752be9757f36d1493e1bb` |

## Failure coverage

Nine focused Python tests cover fixture/validate repeatability, exact output
shape, project/object/recording leakage, wrong role protection or permanence,
opened signal, incomplete role roster, non-zero forbidden access classes,
access-ledger mismatch, forbidden terminal transitions, profile budget,
authority and dependency drift, non-canonical or duplicate-key JSON, in-repo,
occupied and symlink outputs, and cleanup after a late publication failure.

The existing Rust neural data-plane suite also passes `10/10`, including
projection repeatability, protected-row privacy, all three evidence lanes,
cross-role leak rejection and atomic failure cleanup.

`cargo run -p xtask -- boundary-scan` remains red on the pre-existing
`SOURCE_LAYOUT_ESCAPE_HATCH` in
`tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`.
F0 adds no Rust escape hatch and does not depend on that fixture, so this is a
reported repository debt rather than F0 capability credit.

## Decision and next action

F0 closes as `MechanicsReady`. F1 may now automate the metadata-only source
frontier and must reproduce the immutable V30 baseline plus IETeasy increment,
including the exact protected deficit `6 exact-Steel / 23 non-Metal`. F1 still
may not decode signal, assign disclosed data back to protection or grant
training, validator, admission, cooker, demo or runtime authority.

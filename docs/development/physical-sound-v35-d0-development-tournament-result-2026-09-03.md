# Physical sound V35 D0 development-tournament result

| Field | Value |
| --- | --- |
| Date | `2026-09-03` |
| Decision | `REPEAT_EXACT_POST_ACCESS_OWNER_FAULT / HARD_GATE_REJECT` |
| Scientific consequence | `NO_QUALITY_INFERENCE / V35_FAMILY_CLOSED_BEFORE_H0` |
| Owner commit | `35a07761313bdad4e1f633a6f4c45cbeb7cf0113` |
| Owner SHA-256 | `a08edd1b5764fe44b340c2e12972be80b94985aa65dc9bb0b53ec00fba69ff04` |
| Claim | `SYNTHETIC_EXECUTION_FAILURE_ONLY / NO_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY` |

## Outcome

The frozen owner ran in two fresh CPU processes. Both processes materialized all
`6,480` train and `4,320` development target rows, initialized and trained all
four declared models with `6,972` total parameters, and then failed in the
development-prediction stage with the same diagnostic:

```text
TypeError: object of type 'RoleData' has no len()
```

The terminal publisher converted the exception into an atomic
`PostAccessOwnerFault` / `HardGateReject`. Both processes returned exit code
`1` and published the same three files, stdout and complete artifact tree
byte-for-byte. No method-holdout builder, real signal, protected signal,
network request or prior-generation value was accessed.

The failure occurred after official target access. V35 train/development are
therefore spent even though no metric, prediction or weight payload was
published. Repairing the expression and rerunning on V35 would be an
inadmissible post-development retry. H0 remains unopened and V35 is closed.

## Root cause and allowed inference

`development_predictions` passed the `RoleData` aggregate itself to `len()`
while constructing two zero-valued prediction vectors. `RoleData` exposes its
rows through `row_ids` and implements no length protocol. The exact code path
was absent from the discarded I0 rehearsal even though I0 matched official row
counts, candidate variants and terminal shapes.

This is an executable-owner coverage and Python interface error, not evidence
for or against the frozen geometry-conditioned hybrid. The process trained on
official targets internally, but none of its values or metrics escaped the
atomic failure artifact. A successor may preserve the unchanged scientific
hypothesis only with fresh roles, while thresholds, model/training seeds,
features, model capacity and controls remain fixed without using V35 values.

One earlier candidate invocation failed before access with
`train-role:KeyError:'v'`; its ledger was all zero and it created no output
directory. The missing causal context field was corrected before the official
A/B owner identity was frozen. This does not authorize the later post-access
repair or a V35 retry.

## Access and repeatability

| Observation | Run A | Run B |
| --- | ---: | ---: |
| Exit code | `1` | `1` |
| Artifact files / bytes | `3 / 2,360` | `3 / 2,360` |
| Artifact-tree SHA-256 | `c3c9c8b36f9638eb49d5b27b55bfc3e59aef8b0fa35b57eed9f4fd487497055c` | same |
| stdout bytes / SHA-256 | `1,049 / 470a6d1c…633a` | same |
| Outer wall / max RSS | `103.71 s / 915,748 KiB` | `104.10 s / 914,960 KiB` |
| Train / development targets | `6,480 / 4,320` | same |
| Initialized model parameters | `6,972` | same |
| Method-holdout / real / protected / network access | `0 / 0 / 0 / 0` | same |

Artifact identities:

| Artifact | SHA-256 |
| --- | --- |
| `evidence.json` | `aa5f4daea3531a372184ee3d82ca7f5031d0dd350993ea7e4cd3816e099f026f` |
| `post-access-owner-fault.json` | `34cd4b01848956be2b36e9dd8cd49788f6c0a60f4bcc19668b0b61eaf39d692b` |
| `report.json` | `470a6d1cde90a9e7510e5b9167e0e0190433c2c3094fdcd2c7c4278482af633a` |

## Successor boundary

The next protocol must make the rehearsal and official experiment use one
entry point, one concrete role container and one prediction/report path. A
surrogate provider may change values and role identity, but it may not select a
different owner branch. Strict static typing, runtime shape checks, fault
injection and a full-count/full-step discarded rehearsal must all pass twice;
the exact owner bytes are then sealed before any fresh target capability can be
issued.

The V35 scientific knobs are not reconsidered because no quality observation
exists. A new one-use role namespace is mandatory. Any rehearsal/official trace
drift or post-access exception closes the successor as before.

SPEC-45 remains `Proposed`. P1 and authored clips remain the complete product
fallback; this result creates no model, real-material, validator, cooker, demo,
runtime or ProductCheck credit.

## Verification

- official process A/B: `PASS / REPEAT_EXACT_TERMINAL_FAULT`, exit `1`, three
  artifact files and stdout byte-identical;
- method-holdout, real/protected signal and network access: exact zero;
- V35 train/development target access: `10,800` rows per process, now spent;
- owner wall/RSS envelope: diagnostic only because the owner faulted before a
  scientific terminal result;
- Ruff `0.16.3` format/check, Python compile and the focused D0 suite: `PASS`,
  `7/7` tests;
- focused xtask physical-sound registry suite: `PASS`, `161/161` tests;
- `git diff --check`, changed-link and task-state-bound checks: `PASS`;
- `cargo run -p xtask -- boundary-scan`: `FAIL`, the known unchanged
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
- no ProductCheck applies because there is no production consumer or promoting
  ADR.

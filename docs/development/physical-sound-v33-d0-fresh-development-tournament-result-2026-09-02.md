# Physical sound V33 D0 fresh-development tournament result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Decision | `REPEAT_EXACT_CONTRACT_REJECT / NO_ARTIFACT` |
| Scientific consequence | `NO_QUALITY_INFERENCE / V33_FAMILY_CLOSED_BEFORE_HOLDOUT` |
| Owner commit | `2139266e462ae5354c1f56c0088e2095dede5924` |
| Owner SHA-256 | `b2c00accf27f4782954e1eefdc363c648d0e3e5b659ce2b1cd0cad55941e866a` |
| F0 profile SHA-256 | `4c1869101af3ec2264771e17af5c2e652e5861e36c3a8ab61333e588db88fd52` |
| Claim | `SYNTHETIC_PROTOCOL_FAILURE_ONLY / NO_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY` |

## Outcome

The frozen D0 owner ran in two fresh CPU processes without any owner, profile,
seed, feature, model, threshold or role change between them. Both processes
trained the frozen candidate and raw-MLP control, opened the fresh development
role internally and terminated with exit code `2` and the same byte-exact
diagnostic:

```text
physical-sound-v33-d0: ContractReject: P1 hard gate failed: nodal_zero_exact
```

Neither process published an artifact directory. Candidate/control metrics,
weights and predictions therefore remain unavailable, but this is not a valid
zero-access run: the owner had already materialized train/development targets,
trained both models and evaluated development before entering the hard-gate
function. Those roles are spent and cannot authorize a repaired D0 retry.

The method holdout builder was never called. Its target access remains exact
zero, H0 stays unopened and no candidate freeze exists.

## Root cause and allowed inference

`hard_gate_conformance` requires at least one development mode whose original
contact participation is exactly zero. It initializes `nodal_zero_exact` to
true, counts exact nodal modes while composing development cases, forces the
check false when that count is zero and raises before evidence/report
publication. The frozen development corpus contains no such structural
witness. I0 passed because its discarded non-official fixtures did contain a
nodal mode; it did not prove that every official role carried every hard-gate
witness.

This is a protocol-coverage defect, not evidence for or against the V33
surface-spectral representation. Development metrics were computed in process
but never published and must not be reconstructed, guessed or used to select a
successor. The intended seven-file terminal-reject closure was also bypassed,
so D0 earns no artifact-repeat or resource-envelope pass.

The reusable lesson is narrower and mechanical: before target values or model
training, every official role must publish a signal-blind structural witness
census for every hard gate, and the complete owner must prove that all terminal
decisions publish an atomic report rather than raising after role access.

## Repeatability and resources

| Observation | Run A | Run B |
| --- | ---: | ---: |
| Exit code | `2` | `2` |
| stdout bytes / SHA-256 | `0` / `e3b0c442…b855` | `0` / `e3b0c442…b855` |
| stderr bytes / SHA-256 | `77` / `f608505c…caf5` | `77` / `f608505c…caf5` |
| Artifact directory | absent | absent |
| Outer wall / max RSS | `83.25 s` / `887,756 KiB` | `87.80 s` / `889,324 KiB` |

The repeated exit/stdout/stderr prove a deterministic failure signature only.
Outer timing files are diagnostic and intentionally differ. Since the owner
raised before its own resource and artifact report, these observations do not
convert into a D0 resource pass.

## Conclusion and next boundary

V33 is closed before holdout. Repairing the corpus, changing the gate or moving
the check and rerunning against the opened V33 roles would be post-development
tuning even though no metric was printed. P1 and authored clips remain the only
valid path.

A successor may preserve the untested representation hypothesis only with
fresh one-use roles and a new preregistered protocol. Its first executable
milestone must be a zero-target structural preflight that enumerates hard-gate
witnesses per role, followed by a discarded-fixture whole-owner terminal-path
test. No model training is authorized until both pass twice exactly.

SPEC-45 remains `Proposed`. This result creates no public schema, real-material
quality claim, validator release, admission, cooker, demo, runtime model or
ProductCheck credit.

## Verification

- official process A: `REPEATABLE_CONTRACT_REJECT`, exit `2`, no artifact;
- official process B: same exit, stdout and stderr bytes, no artifact;
- method-holdout target access: exact zero by control-flow inspection;
- Ruff format/check and Python compile: `PASS`;
- focused V33 F0/I0/D0 Python suite: `PASS`, `14/14` tests;
- focused xtask physical-sound registry suite: `PASS`, `161/161` tests;
- `git diff --check` and direct documentation path/task-state bounds: `PASS`;
- `cargo run -p xtask -- boundary-scan`: `FAIL`, the known unchanged
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
- no ProductCheck applies because there is no production consumer or promoting
  ADR.

# Physical sound V42 R0 domain/information audit result

Date: `2026-09-03`

Result: `COMPLETE / REPEAT_EXACT / CORPUS_SIGNAL_INSUFFICIENT / C1_NEXT / V0_READY / NO_CANDIDATE_TRAINED`

Planning authority: [Roadmap V42](../plans/physical-sound-synthesis-roadmap-v42.md)

## Outcome

R0 distinguishes the two leading explanations for B0 without fitting a new
generator. Acoustic pseudo-targets strongly identify their source project, but
coarse material identity does not generalize between projects and an oracle
project-mean subtraction does not recover it. The exact terminal decision is
`CorpusSignalInsufficient`.

This means that a larger material-only neural network is not the next useful
experiment. C1 must add source-backed, runtime-available object descriptors and
enough independent internet parents before B1 can test descriptor signal.
Independent V0 validator mechanics may proceed because they use neither
generator outputs nor protected data.

## Frozen implementation

- owner: `lab/scripts/physical_sound_v42_r0_domain_information_audit_v1.py`,
  `43,845 bytes`,
  `sha256=490d7c2047b5192abf3eeaf015d4beb5afd419dddef9e0e0a26b70d302cc2e2e`;
- profile: `lab/profiles/physical-sound-v42-r0-domain-information-audit.v1.json`,
  `6,263 bytes`,
  `sha256=c8dc3ee0e1224f39084998ece14b1b78ac39121c46aebe24b068ef47b8cb21b1`;
- tests: `lab/tests/test_physical_sound_v42_r0_domain_information_audit_v1.py`,
  `12,703 bytes`,
  `sha256=fe86d0c6a23a070e6524ca0dd07fc55097a308a14a5751e44f7a54c61fbda00f`.

The profile binds SPEC-45, the C0 and B0 result documents, the B0/R0 owners,
the tracked B0 profile, exact C0 train/development projections and exact B0
profile/report/metrics/representation artifacts. The living roadmap is not a
hash dependency, so this terminal result remains replayable after its status is
advanced.

## Audit population

R0 keeps B0's 105-dimensional masked target and generator-train scaler. It
combines only permanently disclosed generator roles:

| Population | Count |
| --- | ---: |
| train records / parents | `44 / 26` |
| development records / parents | `70 / 30` |
| combined parents | `56` |
| single-source parents | `55` |
| source projects | `7` |
| cross-source physical anchors | `1` |
| strict leave-project-out supported parents | `46` |
| project-classifier eligible parents | `42` |
| source-centred supported parents | `44` |

The REALIMPACT/ObjectFolder Blue Bowl is excluded from project statistics and
reported separately. Its two source-specific views differ by standardized
RMSE `1.461631096`; because one side is a transfer and the other recorded
impacts, this is a confounded warning rather than a pure microphone estimate.

## Project/source signal

A leave-one-parent-out nearest-project-centroid classifier operates only within
the same coarse material. Project labels are shuffled within that material
4,096 times with seed `4200`.

| Metric | Observed | Permutation null |
| --- | ---: | ---: |
| accuracy | `0.880952381` | median `0.428571429` |
| balanced accuracy | `0.763333333` | median `0.253333333` |
| balanced null P95 | — | `0.450000000` |
| permutation p-value | `0.000488162` | — |

The frozen domain-signal gate passes. On the common 51-parent
leave-one-parent-out support, the project prototype median RMSE is
`1.025653611`, versus global `1.153717680`; the project-and-material prototype
is `1.003175263`. These are diagnostic values, not deployable generator inputs.

Marginal non-orthogonal explained fractions are `0.306264490` for project,
`0.258635038` for coarse material, `0.394109641` for exact material label and
`0.153300489` for generator role. They intentionally do not sum to one and
cannot identify a causal capture effect by themselves.

## Cross-project material signal

For every query parent, all parents from its project are removed. Global and
coarse-material prototypes are then fitted to the other single-source
projects.

| Control | Mean RMSE | Median | P90 |
| --- | ---: | ---: | ---: |
| global | `1.243964047` | `1.272392831` | `1.526724792` |
| coarse material | `1.324254769` | `1.310740776` | `1.730002597` |

Material improves only `16/46` parents (`0.347826087`). Its median paired
relative improvement is `-0.122864043`. The deterministic 8,192-sample parent
bootstrap with seed `4201` gives 95% interval
`[-0.168145282, -0.050277002]`: on this surface coarse material is consistently
worse than the global control.

Only Glass shows a positive per-material median (`+0.145662100`, 15 parents).
Metallic is `-0.105770513`, polymer `-0.204502221` and Wood `-0.151675703`.
These strata remain descriptive because project/material support is sparse and
unbalanced. In particular, generator train still contains no exact Steel.

## Normalization discriminator

R0 also subtracts each project's oracle target mean and reruns the same
leave-project-out comparison on projects with at least two parents. This view
is explicitly non-deployable because a new authored object has no target audio
from which to estimate that mean.

The centred global median is `0.944699071`; centred material is worse at
`1.078863402`. Only `6/44` parents improve and median relative improvement is
`-0.137309564`. Therefore the normalization-recovery gate fails. Source signal
exists, but a simple source offset is not sufficient to reveal a reusable
material mapping.

## C1 planning floor

The current 46 paired cross-project deltas have standard deviation
`0.199519878`. A frozen normal-approximation planning calculation for a 5%
relative improvement, two-sided alpha `0.05` and power `0.80` gives a floor of
`125` supported evaluation parents. This is a corpus-planning estimate, not an
admission bound. C1 also requires at least two independent generator-train
projects per priority material; exact Steel remains transfer/OOD until that
minimum is met.

## External A/B evidence

External root:

`/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v42-r0-2026-09-03`

`run-a` consumes C0/B0 `run-a`; `run-b` consumes C0/B0 `run-b`. Recursive
comparison is empty. Each output contains six files and `61,027` bytes. The
SHA-256 of the canonical sorted `path / bytes / sha256` inventory is:

`a957d4aedc424b75209a856e2b02e77028724ba4ac0ad3208fdecb5691f39455`

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `access_ledger.json` | 1,067 | `9172eaa8963e65de5ba5a9b887a2c7ed737c4ebdd9f42d8db4463e08afbf32da` |
| `audit.json` | 22,348 | `2d1a15e1b37bf2b25aeae20b6a77b331a4797203a568af737703e7872d44cd5a` |
| `parent_inventory.json` | 27,914 | `4bb891efa024c2c60474a70198ae17624ea1714568006b66a2fc6b8b261db7c5` |
| `power_plan.json` | 539 | `5c8475c501fb7e6ad89dec93ddd241265555a1f3a9244cddbbea9df0f855ca81` |
| `profile.json` | 6,263 | `c8dc3ee0e1224f39084998ece14b1b78ac39121c46aebe24b068ef47b8cb21b1` |
| `report.json` | 2,896 | `56baaf55a183d6b863a234267f46aca169c457e85bdee5d6ed94d341be36f10e` |

## Verification and authority

Both runs record zero PCM, validator, protected, candidate-model and network
access. Parent/component isolation, the B0 floor, permutation/bootstrap
completion, finite output and atomic publication are conjunctive.

- focused R0 suite: `PASS`, `7/7` tests;
- combined D0/D1/C0/B0/R0 lineage suite: `PASS`, `40/40` tests;
- Ruff `0.14.1` format/check, Python compile and canonical profile validation:
  `PASS`;
- official external A/B and recursive byte comparison: `PASS`;
- `cargo run -p xtask -- boundary-scan`: `FAIL` on the known tracked
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
  R0 adds no source-layout escape hatch and receives no boundary-scan credit;
- R0 trains no candidate and freezes no descriptor, recipe or validator;
- runtime ProductChecks: `NOT_RUN`, because R0 is external research tooling
  under SPEC-45 `Proposed`;
- authored fallback remains mandatory.

R0 authorizes only C1 descriptor/disclosed-source growth and the already
independent V0 mechanics work. M0 remains blocked until a later B1 proves that
runtime-available descriptors beat the global floor on grouped cross-project
evidence.

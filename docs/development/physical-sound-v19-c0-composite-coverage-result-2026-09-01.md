# Physical sound V19 C0 — composite coverage result

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `COMPLETE / REPEAT_EXACT / C0_CAPABILITY_PASS` |
| Protocol | [V19 P0a](physical-sound-v19-p0a-composite-coverage-protocol-2026-09-01.md), SHA-256 `ea28f844a63036fc6273f829baa6ff8f405050152367da7c8530a1683c315e5d` |
| Implementation | Git `e27ff732`; two deterministic Python modules and ten development-only focused tests |
| Allowed claim | Structural closure plus set-level intrinsic fill plus local graph reachability detects every frozen unsupported-context family on fresh bounded synthetic meshes |
| Product effect | None; P0b may be frozen, but F0/I0, real validation, cooker and runtime promotion remain unauthorized |

## Execution identity

The implementation and focused successful controls were committed before the
test band was generated. Two complete official executions wrote only to fresh
independent external roots:

- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/v19-c0-composite-coverage-run-a`;
- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/v19-c0-composite-coverage-run-b`.

Each root contains exactly eight files, `33,670,296` file bytes, 24 meshes,
216 complete case records and `62,655` composite per-query decisions. Every
corresponding file is byte-identical. The complete-file-map tree digest is
`ac0f3a2893ed40415cfb5cd84b736d335a74b37603c65307401fcbe81be49ace`.

| Artifact | SHA-256 |
| --- | --- |
| `manifest.json` | `659974cc37f19103a5b273bffb9792520e91bd34347a291e575fccc6e9c07987` |
| `report.json` | `ea0e84f4950f59e360b74d3d6c70006efaf9d31547ef09d71af3f4609b942ec5` |
| `corpus.json` | `65e958f9ba3f260cb9c8102fbd2e2d8e1f1f0a032e5a8300fb5d49f5e1129229` |
| `calibration.json` | `203bb317a18e222d78b01e695da6bbd1ceee8cc433b7500b15d2d54fb439447c` |
| `decisions.jsonl` | `64e86dfb94d78f7f07ddbd20956bce75f11a574204bd396406759d26e9a19df5` |
| `scores.npz` | `a80f0e0a2ef91f420b30b9d52cd553a8f79e4a75eb501e276d7e8a5124cec87e` |
| `geometry.npz` | `4154b97b0b25098c536632201c34ccaec27b3314a26b7de31e7d974acd57eb4b` |
| `access-ledger.json` | `4d70e3d90d6f1aa015d995010ec52383ab6ee70af8b35f099829eb78fdbedc50` |

Implementation hashes embedded in both manifests are:

| File | SHA-256 |
| --- | --- |
| `physical_sound_v19_c0_common.py` | `1224038462ba54e94986419f5db0510125474fbb9487d964066a6b3200548024` |
| `physical_sound_v19_c0_oracle.py` | `be586995efd17efda30bf6631526a6315a2c04b65228d046d8218cb46b6c5e88` |

The environment is CPython `3.12.13`, NumPy `2.5.2`, SciPy `1.18.0`, float64
CPU and one numerical-library thread. The pinned V18 geometry dependency is
`baa201197a9790c7c827e36d043ccdbf3362b7458f207b4cd7a3d5f41cecccad`.

## Result

All `15/15` single-run gates pass in both roots, and the complete repeat gate
passes. Frozen development calibration remains exact:

```text
intrinsic local threshold = 0.11869598258542362
intrinsic global threshold = 0.1332105169640669
euclidean local threshold = 0.16904819773036295
euclidean global threshold = 0.19098768258200008
```

Fresh test outcomes are:

| Endpoint | Composite | Raw local intrinsic | Euclidean local |
| --- | ---: | ---: | ---: |
| Valid false OOD | `0.0` | `0.0` | `0.0` |
| Numerical mutation rejection | `1.0` | `0.9617058215130023` | `0.7609267132435648` |
| Rejection-minus-false-rejection utility | `1.0` | `0.9617058215130023` | `0.7609267132435648` |

Every one of the 16 mutation/topology cells and all 12 mutation objects reject
at `1.0`. The ordered reasons also pass exactly:

- every thinning query is `OOD_CONTEXT_BUDGET`;
- every component-isolation query is `OOD_DISCONNECTED`;
- RolledSheet ambient shortcut rejects at `1.0` through intrinsic fill/gap and
  strictly beats the Euclidean control;
- all 48 test structural mutation records reject every query as
  `OOD_CONTEXT_BUDGET` before a distance evaluation;
- removing structural checks accepts the identity-mismatch family, while
  removing graph checks accepts intrinsic cap, proving both layers are active;
- all valid objects and every topology have zero false OOD.

All score values are finite or explicitly null for non-evaluated/unreachable
records; NaN/infinity is never serialized. All real/source/protected/network,
opened-artifact, B0/F0/integration and premature-test counters are exactly zero.

## Verification

- `py_compile`: pass for both modules and focused tests;
- focused unittest: `10/10 PASS` using development only;
- Ruff `0.16.3` check and format: pass for all three files;
- official run A/B: single-run pass in both roots;
- complete directory compare: `byte_identical=true`, eight files;
- mapped boundary scan: unchanged repository-level
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
  C0 adds no finding.

## Decision

C0 is accepted as the frozen deterministic coverage input to later field and
integration work. It closes V18's thinning gap without threshold repair and
without delegating exact input integrity to a neural confidence score.

This unseals only P0b protocol work on the reserved all-new F0/I0 bands. It
does not unseal a model test directly: exact signed-gain truth, masks,
classical/neural controls, seeds, gates and integration endpoints must first be
frozen. A later F0 must consume the exact C0 protocol/implementation/result
hashes and cannot retrain or replace this certificate.

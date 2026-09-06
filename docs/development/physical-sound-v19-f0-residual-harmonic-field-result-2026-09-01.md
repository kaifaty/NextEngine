# Physical sound V19 F0 — residual harmonic field result

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `COMPLETE / REPEAT_EXACT / F0_CAPABILITY_PASS` |
| Protocol | [V19 P0b](physical-sound-v19-p0b-field-integration-protocol-2026-09-01.md), SHA-256 `1c2681a3ac4d505118d08e485cff9e38b23e72a897e8d149b21c909cd11ac24d` |
| Implementation | Git `b815cddb`; three deterministic modules and ten focused tests |
| Allowed claim | On the bounded synthetic truth, a learned shared prior plus intrinsic harmonic extension of observed object residual recovers fresh signed modal-gain fields and beats every frozen compatible control |
| Product effect | None; I0 implementation may start, but real-material credit, validator release, cooker, demo, public schema and runtime inference remain unauthorized |

## Execution identity

The implementation and development-only tests were committed before any F0
test mesh, truth, prediction or metric was generated. Two official executions
wrote to independent new external roots:

- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/v19-f0-residual-harmonic-run-a`;
- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/v19-f0-residual-harmonic-run-b`.

Each root contains exactly eight files and `11,347,962` file bytes. Every
corresponding file is byte-identical. The complete-file-map tree digest is
`7918d8b4fd29f5aabb201b1dcc51d85fce9df44463ecb72ebd4eceb42d608718`.

| Artifact | SHA-256 |
| --- | --- |
| `manifest.json` | `e2e94a4b815bfe926b23879ce49bae0744442bb1fd7df142435a322fdb965ee7` |
| `report.json` | `1fa32ba99e0c212fc6efe2de23e10b18b6ee35b967ae27fb75939d2ab1486bad` |
| `model.npz` | `855e81c38563aad47b4393b954b4bac2caa85c573683fe792d32fe9e62c2b00c` |
| `predictions.npz` | `7c236a9e8ddb066ae336244c742e983eb6a29a442ea77ea8bf2b362c7d662d35` |
| `geometry.npz` | `912f5261284b371c331121fcb848eab43aeeab499cd91cd209c2dced15451c2e` |
| `corpus.json` | `7075675bd066ada673ae4327aef483b80affc7847f08eb9ac80bb930e150ab11` |
| `decisions.jsonl` | `325f78669945073590d6fa2d38308153d0dc7c0777f16ddb941cf454d04d5938` |
| `access-ledger.json` | `43e8104cc281f061680214d9a855af3ad4616905ea518ae98130775bf5288534` |

The manifest pins implementation commit
`b815cddb1f1ac89f623f63b012c73a6f24df7866` and:

| File | SHA-256 |
| --- | --- |
| `physical_sound_v19_f0_common.py` | `84b742502f7be48742a5ee7d42d7073fd2f0061cb9fc43d8283bbbe7125942e7` |
| `physical_sound_v19_f0_model.py` | `ce48434d14a445949ad8e8d0516ac34306f3fd4d1bcbc623ba14daad27e884be` |
| `physical_sound_v19_f0_oracle.py` | `7a9107f292bfe33a3bf14de900adec667a95ac8c9d16fb9784441aeb81cf57f4` |

The exact B0 tree digest is `bffd8bf5…1111`, C0 tree digest is
`ac0f3a28…9ace`, and protocol hash is `1c2681a3…c24d`. Environment is
CPython `3.12.13`, NumPy `2.5.2`, SciPy `1.18.0`, PyTorch `2.13.0+cu130`,
float64 deterministic CPU and one numerical-library thread.

## Result

All `17/17` single-run gates pass in both roots, followed by the complete
byte-exact repeat gate.

| Method | Gain NRMSE mean/max | Edge-gradient p99 mean/max |
| --- | ---: | ---: |
| candidate harmonic residual | `0.168309918 / 0.242008841` | `0.374579795 / 0.653363207` |
| prior + Euclidean RBF residual | `0.292866986 / 0.392526785` | `0.487700867 / 0.770257752` |
| prior + geodesic RBF residual | `0.306856608 / 0.417896817` | `0.501266992 / 0.757800052` |
| learned prior only | `0.392271596 / 0.513093604` | `0.611262899 / 0.846915881` |
| raw graph harmonic | `0.519836026 / 0.615677815` | `0.739618485 / 0.927589137` |

The candidate is `0.5747x` the best compatible gain control and `0.7680x` the
best compatible gradient control. Relative to prior-only it is
`0.4291x/0.6128x`; relative to raw harmonic it is `0.3238x/0.5065x`.
It wins both endpoints against prior-only and prior-plus-Euclidean residual on
all `12/12` primary physical groups.

Topology means also pass:

| Topology | Gain NRMSE | Edge-gradient p99 |
| --- | ---: | ---: |
| Plate | `0.162508` | `0.272906` |
| Cylinder | `0.177736` | `0.530437` |
| Bowl | `0.216178` | `0.501820` |
| RolledSheet | `0.116818` | `0.193156` |

Coverage and invariance remain inside the frozen boundary:

- all `24/24` primary/twin views pass global C0 fill;
- maximum local false OOD is `0.0075188`, versus the `0.05` gate;
- mean/max primary/twin probe disagreement is `0.0995447/0.1445574`;
- maximum relative remesh gain-metric drift is `0.0579663`;
- the model artifact round-trips with bit-exact predictions.

Query shift, context-value shift and alternating context mode-sign flip each
quality-reject `12/12` primary groups. All five structural corruption families
reject every primary object before inference, for `60/60` exact pre-inference
rejects. Every reported value is finite.

The trained prior has 41,992 float64 parameters and 335,936 raw parameter
bytes. Training repeats with initial/final loss
`1.1344933938/0.0139918136`. The external model file is 338,014 bytes.

## Access and verification

The access ledger records exactly 12 test physical groups and 24 mesh views
generated after the implementation commit. All forbidden counters are zero,
including integration, B0 external artifact, opened V16/V17/V18 artifact,
network/source, real waveform, force, generator-real, protected calibration,
method holdout and admission shadow access.

Verification:

- `py_compile`: pass for all four implementation/test files;
- focused unittest: `10/10 PASS`, train/development only;
- Ruff `0.16.3` check and format: pass;
- production-shaped development preview: `17/17 PASS` without F0 test values;
- official run A/B: `17/17 PASS` in both roots;
- complete directory compare: `byte_identical=true`, eight files.

The mapped repository boundary scan remains a separate handoff check; its known
pre-existing `SOURCE_LAYOUT_ESCAPE_HATCH` is not F0 code.

## Decision

F0 passes and its exact model/normalization/prediction/report identities are
frozen. This closes the synthetic field observability question: context is
materially useful, graph-harmonic residual transport is materially useful, and
the result is not explained by extra neural capacity or coordinate-only prior.

Only I0 implementation is now unsealed. I0 must load the exact B0/C0/F0 hashes,
train nothing, generate only `1601…1612` plus twins, and pass the already-frozen
modal/waveform/remesh/corruption gates twice exactly. F0 cannot be refit or
reselected there.

This remains synthetic capability evidence, not a real material formula.
Published-source growth, protected validator/shadow admission, clip cooker,
demo integration and runtime promotion keep their independent blockers and
authored fallback remains mandatory.

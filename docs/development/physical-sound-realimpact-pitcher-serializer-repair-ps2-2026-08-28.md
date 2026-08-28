# REALIMPACT Pitcher serializer repair — PS-2 — 2026-08-28

## Outcome

The one preregistered Pitcher range request succeeded and the frozen prefix
decoded the complete 600-row calibration block. The first offline analysis
completed its projection/Bempp/cooker computations but failed before report
publication because Python's standard JSON encoder rejected one
`numpy.bool_` comparison result. No calibration decision was published.

A narrowly scoped repair revision now casts gate comparison results to built-in
`bool`. It changes no decoder, input, numeric model, solver, threshold, split or
gate. It prohibits both `acquire` and `decode`, so it cannot spend another
request or replace the frozen decoded block. Two local repair preflights repeat
at report SHA-256
`b24e7c903089123462f4978771b7713e7a8605ab58e0940bbca207b012fc5036`.

## Opened Pitcher lineage

The original execution manifest remains immutable at
`8e791327595f43c672361e486ed50aa8512f1c8068de5efdbe6212a81df8ba45`.
Its single HTTPS request returned the exact declared identity:

| Artifact or field | Frozen result |
| --- | --- |
| HTTP status | `206` |
| Content-Range | `bytes 4900621-541771532/2379553389` |
| Network request count | `1` |
| Compressed prefix bytes | `536870912` |
| Compressed prefix SHA-256 | `a0dd700645c89ab5f2d93779cb16177da8464bb0544d4501fd9c0287eda36cf5` |
| Acquisition report SHA-256 | `899fbbe9b6f098f72438817d27d701d0eabc5b5c4a7a0cbbd45de1ad74a5c819` |
| NPY header SHA-256 | `160607c53d9027204d87376ede85f983dce543f4cb1c29a2d83bd887be557cbe` |
| Decoded block bytes | `553128000` (`600 × 230470 × sizeof(f32)`) |
| Decoded block SHA-256 | `182f2010410bf3061176830309b83c3dad482447c8dd2e5f40346f81e0591e0f` |
| Decode report SHA-256 | `29496c6f8f8f5aebb0525d056b7bdf40575ea2b66fc04f1687e449e13e359eef` |
| Planter audio payload bytes | `0` |

The prefix and decoded block live only in the external experiment store. They
are not repository content and are not distributable runtime assets.

## Observed failure

The first `analyze` invocation ran for approximately fourteen minutes and used
the frozen Rust 600-row projection followed by the sixteen Bempp solves. It
then reached `canonical_json(report)` and raised:

```text
TypeError: Object of type bool is not JSON serializable
```

The reported type name is NumPy's scalar display; the failing value was a
`numpy.bool_` produced by a comparison involving NumPy scalar metrics. The
runner's exception path removed the staging directory, so there is no partial
report, projection or decision to admit. This failure is execution-harness
evidence, not a physical-model rejection.

## Repair identity and controls

Repair manifest
`pitcher-execution-report-serializer-repair-manifest.json` has SHA-256
`603c1185884e50de0fda0df08ff98214d9ba56b70c325bb424c869e6f98128e3`.
It references the original execution manifest, acquisition report, decode
report and decoded block hashes. The repaired script SHA-256 is
`3315a8ddaa4739aa14b502d78655d5311cdd4c647b05a6ae7dcea464491fa1c5`.

The script reconstructs the effective execution contract from the original
manifest and rejects any change beyond the new script hash, repair revision and
declared repair record. Its only numeric-path edits are `bool(...)` wrappers on
cooker, held-stratum, comparison, frequency and final conjunction results.

Both preflight runs retained:

- network requests: `0`;
- additional reserved audio payload bytes read: `0`;
- Planter audio payload bytes read: `0`;
- `audio_acquisition_authorized: false`;
- `offline_analysis_authorized: true`.

An explicit attempt to invoke `--stage acquire` with the repair manifest exits
before networking with `serializer-repair revision prohibits acquire and
decode`.

## Decision boundary and next action

Current status is
`PITCHER_PREFIX_ACQUIRED / ROWS_DECODED / SERIALIZER_REPAIR_PREFLIGHT_SUPPORTED /
CALIBRATION_DECISION_NOT_PUBLISHED`. Planter remains sealed. Run the repaired
analysis twice, sequentially, from decoded block `182f2010…1e0f` and require
byte-identical reports. Only a conjunctive calibration pass may trigger a
separate preregistered Planter holdout package; any numerical, repeat or gate
failure stops with the authored-clip fallback.

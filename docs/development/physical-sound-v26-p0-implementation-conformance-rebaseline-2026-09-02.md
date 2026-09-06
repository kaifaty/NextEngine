# Physical sound V26 P0 — implementation-conformance rebaseline

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `P0_ALIGNMENT_CONTRACT_DEFECT / MODEL_VALUES_UNOPENED / P0A_REQUIRED` |
| Frozen predecessor | [P0 m0b-v1.0](physical-sound-v26-p0-barycentric-surface-query-protocol-2026-09-02.md), SHA-256 `4fff3ebd7b788e527b524ff8615d9e8a449b40121e194ba1eb947c5ea81e65e7` |
| Trigger | I0 complete-preprocessing smoke after the off-vertex row began passing |
| Product effect | None; no candidate, checkpoint, metric, sound, public schema or runtime path |

## Observation

The new barycentric evaluator resolves the original first failure
`t0-train-t-beam-a-c00` on both meshes and returns ten finite interpolated gain
targets. The next complete-preprocessing boundary then reaches the unchanged
M0a REALIMPACT alignment helper and rejects before training:

```text
M0 transfer peak cannot satisfy the frozen alignment window
```

The three legal adaptation-context rows are each `230,215` samples, while the
frozen window is `144,000` samples with the absolute peak at output sample
`512`:

| Context row | Source peak | Unpadded start | Required leading zeros |
| --- | ---: | ---: | ---: |
| `contact-000` | `87` | `-425` | `425` |
| `contact-001` | `73` | `-439` | `439` |
| `contact-002` | `39` | `-473` | `473` |

All three independently falsify P0's assumption that the inherited helper
already implemented the declared peak-to-512 semantics. The contract fixture
generated a pulse far enough from the source boundary and did not exercise
leading padding.

The disclosed development query was inspected only after the padding algorithm
had already been written in the bounded diagnostic and agreed with the same
condition. It did not select the algorithm or any model/quality parameter. M0b
still treats that query as disclosed development, never protected admission.

## Hypotheses and alternatives

| Option | Evidence | Decision |
| --- | --- | --- |
| Source/hash corruption | combined, T0/X0 lineage and exact transfer lengths/hashes pass | Rejected |
| Change anchor to `0` or shorten window | avoids leading padding | Rejected; changes frozen representation and acoustic context |
| Increase a crop-search range | could find a convenient onset | Rejected; creates a selection surface and no longer means absolute-peak alignment |
| Reflect/repeat samples before index `0` | fills the window | Rejected; fabricates acoustic history |
| Bounded zero padding | preserves peak at `512`, original sample order and exact window | Selected |

## Selected correction

P0a keeps every P0 surface rule and every M0a model/value choice. It defines a
zero-filled `144,000`-sample output, copies the maximal contiguous source slice
that places the first absolute-maximum peak at output `512`, and permits at most
`512` samples of padding on either side. At least `142,976` original samples
must remain. Silence, non-finite samples, excessive padding or wrong
length/dtype reject.

On the three contexts this produces leading pads `425/439/473`, no trailing
pad, copy counts `143,575/143,561/143,527` and output peak exactly `512`.
These source facts are conformance evidence only and cannot tune the model.

## Decision

P0 m0b-v1.0 remains an exact historical protocol but is superseded before I0
completion and before any M0b implementation root or model value. Do not commit
or run the partial v1.0 implementation as M0b.

Freeze [P0a m0b-v1.1](physical-sound-v26-p0a-barycentric-and-padded-alignment-protocol-2026-09-02.md),
then implement a separate alignment helper plus early/center/late/tie/silence/
short-source mutations. The non-regression check is the unchanged 48-query
surface fixture plus all three context rows aligned to sample `512` without
sample reordering or nonzero fabricated data.

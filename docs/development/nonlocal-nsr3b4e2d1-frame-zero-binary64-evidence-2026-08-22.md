# NSR3-B4E2D1 frame-zero binary64 evidence

Date: `2026-08-22`

Status: `PASS / MICROMETRE_DIVISION_SELECTED / NO_SOLVER`

## Result

The independent standalone reader proves that the admitted external Dam
frame-zero state is bit-exact to integer-micrometre division, not to the
addition-built B4E0 lattice:

| Fact | Value |
|---|---|
| external raw-bit root | `0d567ba5512ba237a48e5e0b828a670a398f1bf23a35ac269729cad535f374d7` |
| micrometre-division root | same |
| addition-lattice root | `b7063e2b024c90acce762533ca612382321a677eee46443942e0eea1a7e28dc4` |
| matching complete position vectors | 6,000 / 384 |
| velocity bits | all positive zero |
| selected representation | `MICROMETRE_DIVISION` |

Exactly one complete candidate root matches. Serialized-position-bit,
candidate-position-bit and ID-swap controls all reject.

## Reproducibility

Raw evidence:
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d1.iQyl2y`

Two independent GCC 15.2 Release builds pass CTest `3/3`, produce the same
117,792-byte executable SHA
`8caa180a7ef6c3d420a6cf4a2979811fef9dd99ea064d2460efe7093aeea3952`
and GNU Build ID `9945bee07af995efb74ae45f0d430219de4af8e3`.

Both fresh processes exit zero, have empty stderr and emit the same 1,082-byte
report SHA
`5a9d2f67550b73169c7405c1c46fb6ee5eeebeb539915d296620e1c407922c07`.
The semantic result is
`2c340f6a9cb570d57a70c3cceed9ab81d8e6a9fb11d278ec3d8d7ad2aefda9ce`.

The inherited `--first-output` command remains byte-exact at report SHA
`6f2d0ffb...40f1` in both new builds. No solver, neighborhood, trajectory or
timing path ran.

## Decision

Select `MICROMETRE_DIVISION` as the external frame-zero binary64 authority.
The old B4E0 topology root remains valid evidence for its addition-built
fixture but cannot gate the external-reference physical pilot. B4E2D2 must
reclose only the decoded topology/capacity identity and audit that all pair-set
changes lie at the compact-support horizon before B4E2D can be repaired.


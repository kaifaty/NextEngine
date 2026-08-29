# NSR3-B4E2D0 preflight observability evidence

Date: `2026-08-22`

Status: `PASS / PAIR_IDENTITY_MISMATCH_ISOLATED / NO_TRAJECTORY`

## Result

Two fresh Release processes exit zero with empty stderr and byte-identical
1,502-byte reports at SHA-256
`be8e9ecf9bc35204ed422ecdb35032e5dabaa3688e5b7f788826d78ef0626d4e`.
The semantic result is
`ee0c8d94799420a112103917252a4bc0e3b31a62bc652c7ebda7e03ccf23845f`.
Neither process starts a trajectory or calls a trajectory-root function.

All B4E2D preflight facts except the initial pair identity/counts are exact:

| Fact | Expected B4E0 | Observed after canonical frame-0 decode |
|---|---:|---:|
| pair root | `c330a0ae...93889` | `fb2b8f8b4c0227cf5d8a7a43ce518ed72b2e5d5cda31fb8e8c17a5727c24ba13` |
| unique pairs | 335,814 | 342,502 |
| directed records | 596,256 | 611,520 |
| maximum degree | 117 | 120 |

Projection, OpenMP policy, scenario, initial aggregate, initial canonical
layout, static index, sample/support counts, finite evaluation and one static
index build all pass. The decoded and raw in-process binary64 lattice states
are explicitly unequal.

## Root cause direction

An independent read-only inspection of the admitted external Dam payload's
6,000 frame-zero position vectors finds:

- all 6,000 are bit-exact to `integer_micrometres / 1,000,000`;
- only 384 are also bit-exact to the current `0.025 + index*0.05`
  construction;
- none match neither representation.

For example sample 1 x is external/decoded bits `3fb3333333333333`, while
the addition-built raw lattice produces `3fb3333333333334`. These one-ulp
changes alter inclusion of pairs lying exactly on the finite-horizon boundary.
This inspection is diagnostic; a separately frozen independent raw-bit slice
must make it normative before re-closing topology.

## Evidence and decision

Raw reports remain under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d.rhPjQy`.
B4E2D0 passes as observability only. Freeze B4E2D1 to independently bind the
external frame-zero binary64 state and distinguish it from the addition-built
lattice. Do not yet change B4E0/SIRDI roots or rerun B4E2D physics.


# NSR3-B4E2D2 reference-binary64 topology evidence

Date: `2026-08-22`

Status: `PASS / DECODED_TOPOLOGY_RECLOSED / NO_TRAJECTORY`

## Result

Two fresh no-trajectory processes emit byte-identical 1,445-byte reports at
SHA `343f6c424b43bccba57eb59e192fd545c968ab382e6abd539ce748c686e86a3d`.
Both exit zero with empty stderr. The semantic result is
`4805530ed85d6a6b1f05ca855c8661fe036b16e8201fcbf745864de9e78d1be1`.

The exact topologies are:

| Representation | Pair root | Pairs | Directed | Degree |
|---|---|---:|---:|---:|
| addition-built | `c330a0ae...93889` | 335,814 | 596,256 | 117 |
| external/decoded | `fb2b8f8b4c0227cf5d8a7a43ce518ed72b2e5d5cda31fb8e8c17a5727c24ba13` | 342,502 | 611,520 | 120 |

Their set relation is not a simple superset: 315,522 pairs are common,
20,292 are addition-only and 26,980 decoded-only. Every one-sided pair lies
within 5 normalized machine epsilons of the compact-support horizon, below the
frozen limit 64. Both initial evaluations have zero active pressure centres,
zero pressure energy and exactly zero pressure gradient with finite density.

The private one-bit decoded-position mutation changes the pair root to
`4907effc...4dba`, proving the identity binds binary64 input. One static index
serves both workspaces. No solver or trajectory runs.

Raw evidence is under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d2.N106f8`.
The Release executable is 4,506,248 bytes, SHA
`b3909e1f6068b858bd5bf85c9022894db7a4fdbf2a1b64e7f34c529beffb25da`
and Build ID `c2dd41bd5edf226d30df15fcecbef5aee0846a65`.

## Decision

B4E2D2 passes. Preserve old B4E0 as its own addition-built historical
alignment, but use the decoded root/counts for the external-reference pilot.
Freeze B4E2D3 as a new identity with safe empty-prefix failure reporting; only
that contract may authorize a physical rerun.


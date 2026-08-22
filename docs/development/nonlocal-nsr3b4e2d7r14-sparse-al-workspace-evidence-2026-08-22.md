# NSR3-B4E2D7R14 sparse AL workspace evidence

Date: `2026-08-22`

Status: `PASS / SPARSE_AL_WORKSPACE_CANDIDATE`

Implementation commit: `a3ad06cd`.

## Result

The separate sparse augmented-Lagrangian backend reproduces the dense D7R13
oracle exactly. The selected route is:

```text
SPARSE_AL_WORKSPACE_CANDIDATE
```

The candidate uses canonical sorted `JointPair` records, flat CSR adjacency,
an AL-specific coefficient tape and a two-pointer sorted union for divided
current/trial reductions. The existing penalty tape and every parent command
remain unchanged.

## Exact tiny correspondence

| Gate | Result |
|---|---:|
| active AL evaluation/gradient | binary64 exact |
| inactive AL evaluation/gradient | binary64 exact |
| active/inactive HVP | binary64 exact |
| D7R13 divided trials audited | `19`, all exact |
| explicit support add/remove crossing | exact, `1` crossing |
| active dense/sparse transaction root | `4d6b8c58...400ef8`, same |
| active sparse repeat root | same |
| inactive dense/sparse transaction root | `6df37c6a...f1ab1e`, same |

The complete sparse transaction therefore preserves D7R13's accepted states,
work ledger, binary128 audit inputs, confirmation/holdout and rollback. It does
not rely on a tolerance or a widened runtime scalar.

## Nominal frame-zero mapping

The command reconstructs the decoded external Dam frame-zero state, but does
not execute an inner or outer solve:

| Fact | Value |
|---|---:|
| D1 complete reference-frame report SHA | `5a9d2f67550b73169c7405c1c46fb6ee5eeebeb539915d296620e1c407922c07` |
| decoded frame-zero raw-bit root | `0d567ba5512ba237a48e5e0b828a670a398f1bf23a35ac269729cad535f374d7` |
| canonical pair root | `fb2b8f8b4c0227cf5d8a7a43ce518ed72b2e5d5cda31fb8e8c17a5727c24ba13` |
| pairs / directed / maximum degree | `342502 / 611520 / 120` |
| active pressure centres | `0` |
| dense candidate checks per evaluation | `116301000` |
| sparse pair visits | `342502` |
| structural work ratio | `339.562980654128x` |
| all-pair candidate calls | `0` |

This ratio compares enumerated interaction work only. No wall-clock speedup,
real-time or production throughput is claimed.

## Workspace safety

The complete command performs `142` sparse workspace builds and `142`
releases, reaches at most two live workspaces and ends with zero live
workspaces. A forced release-underflow control rejects. Public active and
inactive roots remain unchanged; public commit count and trajectory steps are
zero.

## Reproducibility

Raw evidence:
`/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r14.ixAAFo`.

Two independent clean GCC 15.2 Release builds produce identical
`5,169,848`-byte executables:

```text
SHA-256  8125e3b5ab7103d803b59081338cfcabc6a27879d0c627567e42d0ec3692a762
Build ID 30444805afef3ea1fb7e0b5da93b4909d27ecc78
```

Two fresh processes from each build exit zero with empty stderr. All four emit
the same `1,993`-byte stdout:

```text
stdout SHA-256 88d83b6ec6e659f405595b22e13df363bef2ae4c1f99e4838b54f06a585ad1d3
semantic result b9b37dad69dd48066eec693cb327001efe5121111aaf60b5b8888a78e0559e61
```

Direct parent-command verification is retained in
`/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r14-parent.yscMoD`:

```text
D7R13 stdout 514ea1925a85d398a948a2dcbc319689116114a02335a599e51d6703202c18de
D2 stdout    343f6c424b43bccba57eb59e192fd545c968ab382e6abd539ce748c686e86a3d
```

Both remain byte-identical with empty stderr.

## Decision

D7R14 grants one exact sparse AL structural backend. D7R15 may now freeze and
execute one bounded nominal Dam single-frame shadow transaction using the
selected sparse topology/cache path. It must establish resource/work limits,
failure-before-publication and a reference/error gate before execution.

No trajectory, wall timing, GPU/runtime, public state, physics mutation or
production authority is granted.


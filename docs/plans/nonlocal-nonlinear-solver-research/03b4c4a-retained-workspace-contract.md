# NSR3-B4C4A -- retained accepted-workspace ownership

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / COMPLETE_LANES_BLOCKED`

Parent B4C4M0 passes with JSON-without-final-LF SHA-256
`479dad60d89f3c6cec0555c9ee364f16644fafdbe4ae55966dee6b357d1bb38e`
and semantic SHA-256
`4d7629015292ac8a3798b67175075bc37f32091153b72b179c725a90f74e91c7`.

## Identity

```text
sha256  c0f837c569b6e4eaf362e972ac46b7582b31944f2f96d57668913f9da653badf
text    nextengine.nonlocal.retained-accepted-workspace|v1|scope=private-substep-diagnostic|ownership=solve-to-readonly-diagnostic|expected-builds=p1:264,p2:9
```

## Ownership protocol

On successful `solve_box_kkt_step_joint_query`, the candidate may move the
accepted final workspace to its immediate interval caller instead of releasing
it. Failed solves return no retained workspace and release all local current/
trial workspaces. The caller may use the retained `Evaluation` only for the
existing mechanical-energy and maximum-density-strain diagnostic of the exact
same position state. It then releases the workspace exactly once before
contact/event accounting continues.

The retained workspace is private, reconstructible and non-authoritative. It
cannot cross a substep, macro publication, failure return or canonical commit.

## One-macro correspondence

Run the exact B4C4M0 P1/P2 transactions in candidate and legacy modes. Require
bit-exact adaptive attempts/selection, committed position/velocity, physical
diagnostics, contact/topology state, canonical frame, publication ledger and
trajectory/legacy-ledger/policy-ledger roots.

Require these exact candidate counters:

| Case | Builds | Retained transfers / reads / releases | Removed diagnostic builds |
|---|---:|---:|---:|
| P1 | `264` | `63 / 63 / 63` | `63` |
| P2 | `9` | `3 / 3 / 3` | `3` |

All non-diagnostic category counts remain B4C4M0-exact; the candidate
`SUBSTEP_DIAGNOSTIC` query count is zero. Maximum live workspaces remains at
most two and final live count is zero.

## Receipt and negatives

For every retained read, extend a deterministic policy receipt with sequence,
full workspace state hash and exact binary64 mechanical/strain values. Repeat
runs must reproduce the receipt root. Report the different candidate query root
and the preserved legacy root separately.

One valid consumer-abort path must release its transferred workspace without
publication. A non-finite current input must fail with no retained workspace.
Both controls require final live count zero.

## Gate and authority

Two full reports must be byte-identical and reproduce B4C4M0 at its exact
parent hash. PASS selects only
`ONE_MACRO_RETAINED_ACCEPTED_WORKSPACE_CANDIDATE` and authorizes complete-lane
application. Static support indexing, flat-only CSR, B4D, nominal corpus,
CUDA, runtime/schema and production remain blocked.

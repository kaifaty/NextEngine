# NSR3-B4E2D7R9R1 acceptance-ledger reclosure evidence

Date: `2026-08-22`

Status: `PASS / DIVIDED_DIFFERENCE_REDUCTION_CANDIDATE / PRIVATE_ONLY`

## Reproducibility

Two clean Release builds produce byte-identical 4,956,528-byte executables at
SHA `e6c4743ca226d230c88dce1e27330ce21cd9c1cc537dc8f973d7da9351ce6068`
and Build ID `0e6807d316650483756b1c25bbcb2edd081ef71a`.

Both D7R9R1 processes exit zero with empty stderr and byte-identical
15,880-byte stdout reports at SHA
`2f9a935e4a8f24bc60b8b146424d73c98b5d8dc47756bcf6a70b367f03f47e64`.
The semantic result is
`5866926c59596624bfea7341d2c3d45eef4bc15ecb149f5c7e6ac0bbea5dd0a7`.
Raw evidence is under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d7r9r1.ytAo4c`.

Both builds preserve:

```text
D7R8 PASS stdout  42ce10541b29dd03b589e1e8699436e65cb5a794a45b8644fc040990dd4648b1
D7R9 FAIL stdout  2c45e93d4153edf714b0f490be3f3d760b14a1ead1e8ed8ae9528d2555ec91f0
```

The inherited ledger is exactly `1e-8/trial 7`, `1e-9/trial 5`, no tight-
lane trial and total two. New accepted trials and public commits are zero.
All parent, oracle, branch, work, state and rollback controls pass.

## Candidate result

The frozen oracle ledger remains eleven resolved-positive, zero resolved-
negative and twelve unresolved trials. The candidates score as follows:

| Candidate | Scored passes | Maximum relative error | Result |
|---|---:|---:|---|
| Compensated absolute totals | `2/11` | greater than `1000x` | rejected |
| Propagated divided difference | `11/11` | `0.0399153` | selected |

The selected path's minimum relative error is `0.0000714710`. It passes both
first proposals that cross 192 boundary support/kernel segments and all nine
resolved proposals with stable segment membership. PHR branch crossings
remain zero.

This establishes the mechanism: compensation applied after two independent
absolute objective evaluations cannot remove their input error. Propagating
the representable current-to-trial delta through radius, cubic kernel,
density, PHR square and inertia prevents the common terms from acquiring
independent rounding errors.

## Authority boundary

The PASS selects one binary64 actual-reduction candidate only. No trust trial
was accepted because of it, and it has not been exercised on states reached
after such an acceptance. The twelve unresolved parent trials also remain
unclassified. Therefore the result does not yet authorize solver integration,
an accuracy/cap change, a physical pilot or a performance claim.

## Decision

Select `DIVIDED_DIFFERENCE_REDUCTION_CANDIDATE`. Freeze D7R10 as a bounded
private-inner integration over the three exact pre-failure states. It must use
the candidate only for actual-reduction ratio/acceptance, audit every newly
accepted trial against the independent extended evaluator, preserve rollback
and classify convergence, unresolved accepted signs, resolved contradiction
or remaining inner-policy failure.


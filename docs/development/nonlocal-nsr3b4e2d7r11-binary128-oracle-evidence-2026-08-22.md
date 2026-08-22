# NSR3-B4E2D7R11 binary128 accepted-sign oracle evidence

Date: `2026-08-22`

Status: `PASS / OFFLINE_ACCEPTED_SIGN_CERTIFICATE / OFFLINE_ONLY`

## Reproducibility

Implementation commit:
`80b70a831bbda0f216fc6ab5a41f8df310d25d49`.

Two clean Release builds produce byte-identical 5,029,744-byte executables at
SHA `581327e1b32f605cbd63c3b1e3db8571fe021f0877bf7019c594e5adf73cc47d`
and Build ID `52b0a289edcee3c3ac8688e4c7a69719d2c79e5b`.

Both D7R11 processes exit zero with empty stderr and byte-identical 3,849-byte
stdout reports at SHA
`3178c5cde71e3859fdf763b448c50da4bba98ce73914321ea40b2cf49c7e06b0`.
The semantic result is
`f6810759b34599475d61f3f9e6b2cf8322873ee760209086377ebb2590dd5cab`.
Raw evidence is under
`/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r11.Jmeiwo`.

Both clean executables also preserve the exact D7R10 4,806-byte stdout at
SHA `ee7b1d4eb0b5eb37334415fa38a1ee2c2a716fa9ab5b628985e7fed06c1c710d`.
The frozen identity, three accepted solve roots, work/acceptance ledger,
binary128 profile, formula inputs, two in-process oracle evaluations, pair
membership and forced rollback all pass.

## Accepted-sign result

The independent fixed-order and compensated binary128 evaluators agree on a
positive energy decrease for all three accepted D7R10 pairs:

| Inner request | Divided binary64 reduction | Compensated binary128 reduction | Binary128 ULP ratio | Candidate relative error | Result |
|---|---:|---:|---:|---:|---|
| `1e-8` | `1.7274332726764843e-15` | `1.7274336827583933796e-15` | `2.2961532119842954e21` | `2.3739372e-7` | resolved positive |
| `1e-9` | `1.6759996288654678e-17` | `1.6759902490520310798e-17` | `2.2277731597024933e19` | `5.5965800e-6` | resolved positive |
| `1e-10` | `2.8346132192522592e-19` | `2.8347047276408202073e-19` | `3.7679688837640333e17` | `3.2281453e-5` | resolved positive |

Every magnitude is far above the frozen 4096-binary128-ULP resolution floor.
All binary64/binary128 pair-membership decisions agree. The maximum divided
candidate error is about `0.00323%`, far below the predeclared `5%` gate.

The prior long-double ambiguity was therefore an oracle-resolution limit, not
evidence that the accepted tight step increased energy. Runtime `__float128`
is neither needed nor selected.

## Decision

Select `OFFLINE_ACCEPTED_SIGN_CERTIFICATE`. D7R10's three private
divided-reduction acceptances are certified, so the next bounded stage may
integrate that binary64 reduction into the complete private outer AL solve.

This result is not a general binary64 error bound and grants no runtime,
trajectory, performance, GPU or production authority. D7R12 must remain
private, audit every candidate-created accepted near-floor step with the
offline oracle, preserve the outer pressure-state confirmation gate and roll
all state back.

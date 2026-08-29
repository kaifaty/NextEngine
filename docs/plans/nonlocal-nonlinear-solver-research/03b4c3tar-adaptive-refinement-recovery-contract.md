# NSR3-B4C3TAR -- adaptive canonical refinement recovery

Status: `EXECUTED / FAIL_PRESERVED / LEDGER_RECLOSURE_REQUIRED`

Parent B4C3TA is the preserved negative report with JSON-without-final-LF
SHA-256
`52d9c1c9cce975927c7230ee5f68e7871ab6a345a3b227f8467d6f8ce2d8c377`
and semantic SHA-256
`ed95592f3c493089676ae09ffb7d39a2dc3c735ffcf4a114b57b2b190ca4e3c9`.
The failed-state refinement probe semantic SHA-256 is
`c6767c992f489ec75d022a258c07eb03bd44bb406dcc57cefac5541b8c003c83`;
the P2 physical probe semantic SHA-256 is
`c4d47cd5a84adf6b5e1bee5373d37b4ce1237f7abb87480cb2ac3540b435f665`.

## Identity and unchanged inputs

```text
identity  joint-pressure-canonical-balanced-adaptive-r1-recovery
profile   f57d88c222a8334206a962a72a814bdd530696ed2767c94fa88910281265579c
P1        4128b11190366b45aa6511946cb23b46f254445e5a707ff9cdf7e67a2781aaa6
P2        013a83460fabbded819b7d5d9608747f8bc9548c798d71718603d785c238b3c1
```

All B4C3TA formulae, fixtures, maximum four levels, spectral estimator,
embedded error gate, binary envelope, schedule/contact predicates, ledger and
energy budgets remain unchanged. The repair changes only candidate-failure
classification and restores a bound already present in the frozen B4C3TA
prose.

## Recoverable candidate failure

Each level starts from the same decoded committed macro-frame state and global
accepted-step offset. A failed candidate is refinable if and only if its exact
failure code is:

```text
KKT_SOLVE:REJECT_LIMIT
```

For that code:

1. discard its staged frames, ledger entries, state and roots;
2. record the level as a private failed attempt;
3. add its completed substeps plus the failed KKT substep, outer trials,
   rejects and HVP calls to attempted-work evidence;
4. execute the next finer level, subject to the existing four-level and
   capacity limits.

Every other KKT, canonical publication, range, identity, workspace, topology,
ledger, nonfinite, step-range, capacity or injected failure remains immediately
fatal. In particular, no substring/family match may classify a new error as
recoverable.

## Selection and commit

A passing level alone is insufficient. Select only when levels `l-1` and `l`
both pass, are adjacent in the original refinement sequence, and satisfy the
unchanged embedded gate. Commit only level `l`; every other passing or failed
level remains private. A failed level breaks adjacency. If four levels do not
contain an adjacent passing gated pair, fail the macro frame without changing
committed state, roots, ledger, cumulative totals or global step count.

Report per attempt: frame, level, planned substeps, completed staged substeps,
exact attempted KKT substeps, status/failure, recoverable classification,
outer trials and HVP calls. Report per lane the number of recovered frames,
recoverable failed levels, discarded private frames/ledger entries and exact
attempted/accepted substeps. Retain accepted `<=192` and maximum attempted level
`<=768` gates.

## Restored released-flight bound

The stage probe compares each pressure-inactive precontact transition with an
independently published analytical free-flight transition from the same
canonical input. Aggregate balancing guarantees local component error strictly
below one canonical unit. Require:

```text
maximum precontact position RMS < 1e-6 m + 64 epsilon
maximum precontact velocity RMS < 1e-6 m/s + 64 epsilon
maximum velocity spread <= 2e-6 * precontact_steps + 64 epsilon
```

Pressure violations remain exactly zero and support reaction remains
`<=1e-12`. This repairs the implementation's accidental exact-zero velocity
predicate; it does not derive a tolerance from the observed `1.9246e-7 m/s`.

## Negative controls

Require all of the following:

1. the exact preserved P1 frame-four state still fails at 16, passes at 32/64
   and selects 64 only after the 32/64 gate;
2. a forced non-recoverable solver failure aborts immediately with no commit;
3. a recoverable failure between two passing non-adjacent levels cannot bridge
   the embedded gate;
4. exhaustion without an adjacent passing pair rolls back state/root/ledger and
   global count exactly;
5. the original B4C3TA report remains byte-exact at its frozen FAIL hash.

## Decision boundary

Two complete B4C3TAR reports must be byte-identical and all historical B4C3TA,
B4C3A1, B4C3Q, B4C3A and B4C2T report hashes must remain exact.

PASS selects `CANONICAL_BALANCED_ADAPTIVE_RECOVERY_CANDIDATE` and authorizes
only B4C3TR fixed canonical reference design. FAIL preserves B4C3A1 and the
B4C3TA negative result. No fixed canonical, nominal, CUDA, runtime, schema or
production authority is granted.

## Executed outcome

The r1 controller recovers the exact P1 frame-four and frame-five
`KKT_SOLVE:REJECT_LIMIT` events, and P2 passes completely. It then stops
correctly at P1 frame seven on a non-recoverable publication-ledger gate; see
the
[dated evidence](../../development/nonlocal-nsr3b4c3tar-refinement-recovery-evidence-2026-08-21.md).
The compensated vector has zero closure to the KKT ledger, but its different
normalizer turns an accepted `9.9175e-10` KKT residual into a rejected
`1.1268e-9` publication residual. B4C3TAR remains FAIL. Only a separate ledger
normalization reclosure is authorized; B4C3TR stays blocked.

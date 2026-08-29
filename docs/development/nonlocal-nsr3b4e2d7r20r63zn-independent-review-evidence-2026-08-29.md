# NSR3-B4E2D7R20R63ZN independent review evidence

Status: `REVISION_2_INITIAL_REVIEW_NO_GO / REVISION_3_REPAIR_AUTHOR_PASS / INDEPENDENT_REREVIEW_PENDING / NO_ENDPOINT_AUTHORITY`.

Review date: `2026-08-29` (`Europe/Moscow`). The reviewer received only the
neutral freeze manifest, frozen contract and exact candidate snapshot before
the verdict. Candidate source remained read-only and all reviewer work was in
a detached `/tmp` worktree.

## Reviewed identity

```text
manifest SHA-256  bec5a8b8ba61ae7bf00df88a087bedc1d286c094555c6bb3ec779e4433398500
snapshot          846dbac60285b36f8a162c2cb311eff69e0d7536
parent            d1fbc338df9e136a4a688c48a0789c01efa47b9b
tree              a186fbb3f0622baaa8eee387541475d5bf7ead8d
binary diff       faa4e91905ec50b86f693cd3b1772113584b134268737823cefe7fe22d6612c8
```

Every manifest source, input, binary and evidence identity matched. This
identity closure does not rescue the semantic findings below.

## Load-bearing findings

1. `H*x0` was decoded, rooted and compared before independently solved `x0`
   corresponded to parent role 2. The public `r63zn-x0-mismatch-v1` control
   therefore emitted route 4 with nonzero `HX0_ROOT`, `QuadDecodes=307`,
   `CanonicalRootCalls=8` and `Role2ValueRootComparisons=1`; the checker
   repeated and accepted the same premature consumption. Receipt
   `ab87df21ff8d13d73ef9a46174071f83bd97c1f513722f1cf24394bf8fa3cae2`,
   audit
   `63a565ace5c4f68a0c3b9d04a4b97d72e4a005451516bc792f5b88b7010e9387`.
2. The manifest declared `-std=c++20`, while CMake left extensions enabled.
   Exact standard C++20 rebuilt candidate/checker as `3bbdf381...899c` and
   `1184bd0c...b0f`; only undeclared `-std=gnu++20` reproduced the frozen
   `ea497b33...f9e` and `e3d5a729...355b` binaries.
3. Checker work did not seal actual POSIX read-call/early-stop counts, and its
   audit materialization had no fields-written/serialization counter.
4. Candidate, checker and mutator asserted only the receipt tail; the frozen
   contract requires a complete compile-time offset chain from magic through
   every header/input/semantic/work/event/trace/result boundary.
5. The required reachable nonfinite arithmetic control was absent. The public
   selectors covered mismatch, underflow, nonpositive lower bound, small solve
   and state difference only.

Any one finding was sufficient for `NO-GO`. In particular, finding 1 refutes
the package's claimed first-failure trust boundary; passing author controls
could not admit the initial prefix.

## Reproduced evidence

- Both actual GNU-extension control runs reproduced `116/116` pass,
  `116/116` distinct audits and report
  `745eafba951678bb584e79bf569b554468355c17ff564fdd3e553f10bc264a02`.
- The exact C++20 control corpus was semantically green but its report changed
  to `649f693d8fe3450d214d3d5302c41f325013a86fa9f7610c7a6b2d835489342f`
  because the binary identities differed.
- Baseline receipt `e0015763...f8da`, checker audit `d0a98ac6...2cc3`, empty
  stdout and zero-allocation stderr reproduced.
- No forbidden parent/formula/R63ZJ/K/L/M helper symbol was linked.
- R63ZM independently reproduced audit `fd4bcf00...1f80`, empty stdout and
  zero allocations.

## Required batched repair

The single repair must move role-2 component decode/root/comparison strictly
after all 102 `x0` comparisons, leave every later root/event/work field zero on
route 4 and add a direct early-stop assertion. It must freeze actual standard
C++20, seal checker read calls/stops and audit serialization, assert the whole
layout in candidate/checker/mutator, add a reachable nonfinite public control,
regenerate all binaries/evidence and receive a fresh independent re-review.

## Author-side repair closure

The batched revision-3 repair implements every required item. The direct
`r63zn-x0-mismatch-v1` control now stops at route 4 with zero `H*x0` and later
semantic roots/events, `205` quad decodes, `6` canonical-root calls and zero
role-2 comparisons; receipt
`52455716...4127`, checker audit `36005885...9f04`. The new public
`r63zn-prefix-nonfinite-v1` selector reaches route 5 before publishing the
residual; receipt `699d123b...0134`, audit `2cd6a547...bf54`.

All six R63ZN build targets disable language extensions and two independent
clean `-std=c++20` builds reproduce candidate `583f9558...dffa`, checker
`f899fa1f...b83e0` and mutator `bc2ef0f2...d125`. Candidate, checker and
mutator compile-time assert the complete receipt offset chain. The checker
audit is `700` bytes with `51` work fields and seals actual file read calls,
terminal read stops and the exact `71` audit fields serialized.

The expanded corpus passes `118/118` twice with `118/118` distinct audits and
report `24208010...d2def`. ASan/UBSan passes the same corpus with leak
detection disabled. R63ZM still reproduces exact audit `fd4bcf00...1f80`.
Baseline receipt is `1e42c834...ba1ac`, checker audit
`4f09b539...e2e4eb`, trace `834907ed...e8fe7` and terminal result
`b5907ca5...39d07`. This is repair-author evidence, not a verdict. The one
permitted independent re-review must independently reproduce these closures.

Until that re-review returns `GO`, R63ZN admits no `x0/r0/z0/rho0` prefix.
R63ZM is unchanged; SPEC-38/ADR-076 remain `Proposed` and ProductChecks remain
`NOT_RUN`.

# NSR3-B4EP10SIRDIREI evaluation buffer overwrite evidence -- 2026-08-22

Status: `FAIL / DEFAULT_PATH_REGRESSION / IMPLEMENTATION_REVERTED`

## Result

The implementation probe preserves candidate semantics: identity
`96a22c8e...8f99`, B4EP10SIRDI result `b4f847cb...777e9`, all six frozen role
counts, workspace/retention lifetime and physics roots pass. The unchanged
SIRDI command also remains byte-exact at stdout SHA `539f1ec5...e7e7`.

The implementation nevertheless violates the frozen default-behavior boundary.
Changing the common vector type to a stateful user allocator moves libstdc++
away from its optimized value-initialization path. Its allocator `construct`
hook is dispatched per element, so the supposedly unchanged SIRDI command
takes 10.43 s wall instead of the previously accepted 4.290430589 s median,
a `2.4309914316621053x` regression.

The candidate takes 9.63 s. Its apparent `1.083073727933541x` relative probe
win is invalid because the baseline has been corrupted. Baseline/candidate
total CPU are 62.97/61.19 s (`0.97173257106558675x`), confirming that both
execute the new elementwise allocator path. RSS remains ordinary at
92,168/91,308 KiB and both reports emit no stderr.

## Gate handling

This was an implementation-boundary diagnostic, not the frozen warmup plus
balanced `AB`, `BA`, `AB` measurement. Running the official pairs would compare
two regressed representations and could incorrectly grant speed credit. The
candidate therefore fails before external A/B; no threshold is lowered and no
relative result is promoted.

All implementation changes were reverted before commit. A rebuilt rollback
binary restores exact SIRDI stdout and records a cold verification wall of
4.68 s with empty stderr. Raw failed-probe artifacts remain outside Git under
`/home/kaifaty/.cache/nextengine/external/probe-nonlocal-b4ep10sirdirei-cost.9mKlmQ`;
rollback verification is under
`/home/kaifaty/.cache/nextengine/external/verify-nonlocal-b4ep10sirdirei-rollback.b48Xsg`.

## Decision

Close B4EP10SIRDIREI as FAIL and retain unmodified SIRDI plus the SIRDIREA
structural proof. Do not retry allocator hooks, raw storage, span conversion or
returned-workspace representation in this bounded path. Research only the
independent builder-local density-contribution lane, whose high-water size need
not alter any returned vector size or ownership. No speed credit is granted.
B4E2, broad corpus, runtime/GPU/schema and production remain blocked.

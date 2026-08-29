# NSR3-B4EP10SIRDIREQ default-path drift research -- 2026-08-22

Status: `COMPLETE / TWO-SOURCE_QUALIFICATION_SELECTED`

## Question

Did the selected SIRDI algorithm become slower, or did later opt-in research
instrumentation make its supposedly unchanged default execution materially
more expensive?

The accepted SIRDI build at `f33bf3a` measured 4.290430589 s median wall and
31.58 s median total CPU. After three later instrumentation/audit commits, the
failed SIRDIREP A/B observes 4.893718116 s and 36.06 s for the same SIRDI
command. The post-revert cold control is exact but still takes 4.79 s. Relative
candidate timing is unsafe until this difference is classified.

## Competing hypotheses

1. **Default-path code drift.** The 999-line source delta from accepted SIRDI
   to the reverted current source adds dormant branches, timers, audit state or
   code-layout pressure that costs at least 5% even when no optional command is
   selected.
2. **Host interference.** Current desktop/background scheduling or frequency
   variance slows both source checkpoints; the apparent code trend is not
   attributable.
3. **No material drift.** Fresh independent builds of both source checkpoints
   differ by less than 5%; the prior result was ordinary measurement noise.

The larger current total CPU is evidence for hypothesis 1, while visible host
load and a single 4.79 s rollback control are evidence for hypothesis 2. No
hypothesis is selected from those observations alone.

## Smallest discriminator

Build two independent Release binaries with the same compiler and build
recipe:

- accepted SIRDI source checkpoint `f33bf3aa...27d2`;
- current reverted source checkpoint `b8a1eddf...2f04`.

Run the identical `--nominal-hydro-directed-scratch-reuse-8` command from both
binaries. One warmup per binary precedes serialized `AB`, `BA`, `AB` pairs on
CPUs `0..7`. Require both outputs to remain the accepted SIRDI bytes. Record
monotonic wall, GNU user/system/RSS and non-mutating host observations.

The accepted checkpoint must itself retain median wall at most 4.72 s and
both range ratios at most 1.10. If not, classify the host as unqualified and
do not use this experiment for source attribution. With a healthy host,
median paired wall slowdown and median total-CPU ratio both at least 1.05
select code drift. Both below 1.05 reject material drift; disagreement selects
a narrower mixed wall/CPU discriminator.

## Decision

Freeze B4EP10SIRDIREQ as qualification only. It grants no optimization or
speed credit. A code-drift result routes to source-delta isolation before new
residual work; a healthy no-drift result routes to topology residual research;
an unqualified host stops short-margin A/B work without changing the solver.

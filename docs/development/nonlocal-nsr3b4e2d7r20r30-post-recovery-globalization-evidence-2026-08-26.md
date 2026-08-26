# NSR3-B4E2D7R20R30 post-recovery globalization evidence

Status: `PASS / POST_RECOVERY_SAME_FACE_REJECTION`.

Implementation `16527590` reproduces R29 and emits twice:

```text
e1f807e9ac25ad9317f07bc9e4a4364af76fa653e664ba8da10eaa2e10581ff5
```

The iteration-21 certificate is finite but correctly fails: primal and
stationarity pass, while dual mapping, complementarity and scaled gap remain
`2102.5266x`, `30.8027x` and `84.9261x` above `2^-70`. The gap sign itself is
valid.

Iteration 22 has a resolved 67-row natural face and the same 67-row certified
NNQP support. Its slope is `2.2218562027402492e-34` with bound
`1.3940092248134731e-42`. All 21 inherited trials are exact, mask-stable and
ball-stable; there is one mask regime and no changed scalar at any alpha.
Every R19 Armijo margin is strictly negative, approximately
`-1.0289e-29`, and every rigorous dual-increase lower bound is approximately
`-1.0286e-29`.

Thus the post-recovery failure is not another projector event, insufficient
line depth or an unsigned binary128 comparison. It is a same-face merit
rejection. Notably the rejected `alpha=1` trial reports primal
`1.05e-32`, dual mapping `2.84e-34`, complementarity `1.31e-35` and scaled gap
`4.00e-30`, all numerically below `2^-70`. R30 did not freeze or report the
trial's complete `metrics.certified` predicate, so this observation is not yet
authority to accept it.

R29 remains exact at `3c4f5d51...ba7b`. The next report-only discriminator
checks every terminal certificate predicate and decomposes the Armijo error
budget before any terminal-acceptance policy is considered.

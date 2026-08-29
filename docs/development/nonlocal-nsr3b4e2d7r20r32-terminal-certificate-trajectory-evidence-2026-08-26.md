# NSR3-B4E2D7R20R32 terminal certificate trajectory evidence

Status: `PASS / TERMINAL_CERTIFICATE_TRAJECTORY_CANDIDATE`.

Implementation `86515ea9` emits twice:

```text
e7b9acafd3d7aee53b23bff4c3a286f7cf4578e5e566cd69ab04d11c866f0f73
```

The candidate reconstructs the complete R29 semantic exactly as
`3c4f5d51...ba7b`. All eleven historically certified case roots remain
bit-identical. Shear reproduces the exact 21 accepted steps, including the
single exhausted-line recovery, and reaches the exact R31 rejected line.

After all 21 ordinary Armijo trials reject, exactly one inherited trial is
KKT-certified. The candidate selects power 0 at `alpha=1` as terminal state:

```text
ordinary Armijo accepted   false
Armijo margin              -1.0288940390376748e-29
mask changes               0
ball change                false
terminal trial root        b209fd6f...d3d2
R31 audit root             e36e8d5e...26d7
terminal dual root         c9fc1810...f5f5
terminal metrics root      05b71f6f...9b53
terminal multiplier root   b322759b...a7a9
```

A fresh reconstruction from the returned multiplier state reproduces both
dual and metrics roots and passes the unchanged `2^-70` KKT certificate.
There are 12/12 certified cases, one terminal selection, one terminal state
update, zero new trials, zero ordinary Armijo/tolerance changes and zero
following iterations. Thirteen solver executions include the extra exact R26
shear-prefix control; this count is evidence lineage, not a performance result.

This establishes `SUPPORTED_BOUNDED` correctness of terminal-certificate
precedence over the exact binary128 12-case research corpus. The KKT oracle
still shares implementation lineage, the v3 holdouts are now development data,
and no binary64/runtime/production or performance claim follows.

The next generalization evidence must freeze new source-only holdouts before
building their operators or observing solver behavior.

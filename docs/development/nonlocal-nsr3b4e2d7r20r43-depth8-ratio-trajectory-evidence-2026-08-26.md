# NSR3-B4E2D7R20R43 depth-eight ratio trajectory evidence

Status: `PASS / DEPTH8_RATIO_LATER_BOUNDARY / SECOND INVERSE TARGET`.

Implementation `5b331950` emits reproducible semantic:

```text
7a1e4aaaeb6745361b5bd3f4dcab8a85a5c0c582ceaecf74ef399e2d680f8fa8
```

The practical depth-eight certificate has root `0d3e1914...6849`, radius
`4.5753008642e-14` and uniform error `4.6075931152e-14`. It reproduces the
R41 `24/41/0` signs with minimum separation `1054.3861` and uses exactly
`8x65=520` center-generation dots.

R42's row-61/row-13 ratio ambiguity disappears. The unchanged NNQP performs
two additional principal solves/transitions (`598 -> 600`) before reaching a
new `VERIFIED_INVERSE_AUDIT_REJECTED`. The hook is invoked twice, but matches
and replaces only the original frozen target; its second invocation rejects a
different target without mutation.

The second 65-row system is frozen by:

| object | SHA-256 |
|---|---|
| material tuple | `7e90da896485a868cad7eb2fab1c8f48e7a33611bc3763fe6328d56fe4d5f683` |
| matrix | `f8114cbbe9caec3d2dccb8d1f7988030450e70b218a8efcadabd69ad7b33be97` |
| inverse candidate | `a4fa12c94d1f08efc37ed8371b5ccd0befe105909ebb6ad8735842c83eb479cf` |
| RHS | `53cb4684dd776446547f341f997a0d5a27151859c78d0ff2b2e17adb9470dd32` |
| represented solution | `1e51c3ad6f5498ea9e7d8c85ec8d92d7c26a73c5182d57bca5eda92b35bc523a` |

This is evidence for Q1 and Q3: candidate error alone caused the ratio overlap,
and a distinct later inverse-certificate boundary exists. It is not evidence
that centered refinement automatically generalizes to that boundary. R42 and
R41 remain byte-exact after parameterizing the practical center depth.

R43 applies one target-bound local correction, no second correction, no new
trial and no counterflow or production state. The next discriminator must
audit both `I-AX` and `I-XA` of the second candidate exactly before evaluating
its RHS or permitting another replacement.

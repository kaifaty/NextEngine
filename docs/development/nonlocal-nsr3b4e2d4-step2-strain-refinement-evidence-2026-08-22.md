# NSR3-B4E2D4 step-two strain-refinement evidence

Date: `2026-08-22`

Status: `PASS / FINITE_PENALTY_COMPRESSIBILITY / DIAGNOSTIC_ONLY`

## Reproducibility

Two clean Release builds produce byte-identical 4,541,008-byte executables at
SHA `6846eb7c3746e25984634cbd200980c6aa7fc3e84ce85eae7a8704a164cbf8c2`
and Build ID `063604bb688ef42839ab72281091c41fcac22826`.

Two fresh processes exit zero, emit empty stderr and byte-identical 3,449-byte
reports at SHA
`6dd02ec3b2fde108990e83218f09cca6e5d14fee51b3e716837f8e43e43a16b4`.
The semantic result is
`c2c9ee08717b3c790aef35810122cb8a1bf866f5b13922fcc384d822f5d0fa5f`.
Raw evidence is under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d4.ooMPxT`.

## Result

B4E2D3 steps one and two reproduce exactly, including their frame and
aggregate roots. Separating the private and canonical-publication terms shows
that publication is not the root cause:

- private 80-substep peak strain: `0.0011740875121342143`;
- decoded published strain: `0.0011747197409319732`;
- publication increment: about `6.32e-7`, while the private lane already
  exceeds the `0.001` cap.

Fixed lanes from the exact committed step-one state produce:

| Substeps | Peak private strain | KKT residual | State root |
|---:|---:|---:|---|
| 80 | `0.0011740875121342143` | `4.5875800090895006e-10` | `7f88000d...0a5c` |
| 160 | `0.0011739804862926917` | `7.393935291551327e-12` | `d2d73fdd...b223` |
| 320 | `0.0011739237712489192` | `7.067183862752338e-10` | `a5c9a992...bef2` |

The existing 80/160 and 160/320 adjacent state gates both pass. The 160/320
peak-strain delta is `5.671504377247061e-08`, almost three orders below the
predeclared `5e-5` resolution threshold. Penetration remains exactly zero;
work-only, cache, eight-worker, split-incoming and directed-scratch ownership
certificates pass for both added lanes.

## Conclusion

The step-two violation is a converged property of the current finite-`KAPPA`
compression penalty. It is not repaired by more temporal refinement, and
canonical publication only adds a small secondary increment. Do not modify
the adaptive controller to address this failure.

The next research stage must compare the smallest physically sufficient
penalty increase and its stiffness cost against a constrained or augmented-
Lagrangian incompressibility formulation. No production or performance claim
is admitted by this diagnostic.

# NSR3-B4E2D7R20R63K factor standard-refinement evidence

Status: `PASS / WIDE_FACTOR_STANDARD_REFINEMENT_REJECTED`.

## Reproducible result

Implementation commit: `f3594461`.

Command:

```text
/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v5-factor-standard-refinement
```

The command repeated byte-identically twice:

```text
stdout sha256   7b13fa402665836d36f3442942c34c2c611df942df0fff582034f487224773c8
semantic sha256 61c739a2f7fe417de69cc596d925d6a7ec417360b25118e88875eb8c7b474ade
route           WIDE_FACTOR_STANDARD_REFINEMENT_REJECTED
```

Platform, controls, exact R63J reconstruction, 48 fixed refinement iterations,
15 independent certificates, work and lifecycle gates all pass. No dense
inverse generated an update; `X` appears only in the read-only certificates.

## Retained-wide lane

The retained-wide factor is a useful but slow correction solver. Its certified
error decreases monotonically at every frozen checkpoint:

| Iteration | Error radius | Original residual infinity bound | Signs `+/-/?` |
|---:|---:|---:|---:|
| 0 | `4.14356e15` | `6.86e-2` | `12/24/66` |
| 1 | `2.15568e15` | `2.74e-3` | `12/24/66` |
| 2 | `1.39942e14` | `1.00e-3` | `12/24/66` |
| 4 | `3.15629e12` | `1.60e-5` | `12/24/66` |
| 8 | `1.21412e9` | `4.80e-9` | `12/24/66` |
| 16 | `1.44035e2` | `5.28e-16` | `21/77/4` |

Iteration 16 is close but not admissible: four intervals still contain zero
and the error radius `144` remains above the immutable sign margin `32.4234`.
The frozen 16-iteration budget therefore rejects the lane exactly as specified.

## Export/wide lane

The first update increases the baseline error from `4.11531e15` to
`4.30445e15`, so the full checkpoint sequence is not monotone. It then
contracts strongly:

```text
iteration 2   3.40383e14
iteration 4   1.61800e13
iteration 8   2.59375e10
iteration 16  5.06391e4
```

At iteration 16 all 66 initially unresolved signs remain unresolved. This
lane also has clear late progress but no certificate within the frozen budget.

## Strict-binary64 lane

Strict binary64 factor consumption and update do not exhibit a convergence
trend:

```text
iteration 1   4.86977e15
iteration 2   8.53011e15
iteration 4   1.20943e15
iteration 8   2.03118e16
iteration 16  3.15399e15
```

All frozen checkpoints retain 66 unresolved signs. This is a materially
stronger rejection than merely missing the iteration budget: ordinary
binary64 correction arithmetic is unstable for this captured weak direction.

## Meaning

Standard factor-based refinement is mathematically promising in wide
arithmetic but is not yet a candidate under the frozen contract. In
particular:

- retained/exported factor storage can act as a useful preconditioner;
- 16 stationary correction steps are insufficient for either wide lane;
- strict binary64 substitutions and updates are not a viable correction path;
- R63J remains an oracle recoverability proof, not a production algorithm;
- no state replacement, runtime, GPU, timing or production authority exists.

The next bounded experiment should continue only the two finite wide lanes
from exact iteration 16 through a pre-frozen frontier ending at 32, certifying
every new iteration. This locates the natural convergence boundary before
spending complexity on GMRES-IR/PCG. The strict lane is already rejected and
must not be silently extended or averaged.

## Regression evidence

All parent stdout hashes R63B--R63J remain byte-identical, ending with:

```text
R63I 6aaf5889d5c15734db26a4ac7d6118896419f4d7c4027003790b91517d41bd7a
R63J b4a2908e1cfb724354e16895221df688d27675de48de5da9717674d2fe99191d
```

No timing was admitted. Shared-host duration is not performance evidence.

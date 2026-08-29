# NSR3-B4E2D7R20R51 generic verifier corpus evidence

Status: `PASS / GENERIC_CORPUS_COUNTERFLOW_ONLY_BOUNDARY / 4 OF 5`.

Implementation `21cc6988` emits reproducible semantic:

```text
48df3b2650abb5ff9338c6926c60754494418cfd7037ef4c43851b8c7946adca
```

The root-agnostic policy is replayed independently on all five immutable v4
problems. Torsion reproduces R50 exactly and certifies; the three ordinary
parents remain ordinary and receive zero generic calls. Counterflow also
receives zero calls and preserves its independent direction rejection:

| case | result | generic calls/replacements |
|---|---|---:|
| torsion | `ORDINARY_CERTIFIED`, 887 transitions | `5/5` |
| corner | `ORDINARY_CERTIFIED`, 974 transitions | `0/0` |
| counterflow | `DIRECTION_REJECTED`, 701 transitions | `0/0` |
| helical compression | `ORDINARY_CERTIFIED`, 1180 transitions | `0/0` |
| alternating layer | `ORDINARY_CERTIFIED`, 1033 transitions | `0/0` |

Aggregate generic work remains exactly R50's `27,625` dots and `1,802,450`
input pairs. All five workspace lifecycles are exact. Two R51 runs and R50
regression are bit-stable. This isolates counterflow as the sole remaining v4
boundary; it does not promote the verifier to production.

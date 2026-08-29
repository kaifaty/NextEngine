# NSR3-B4E2D7R20R34 v4 preflight evidence

Status: `PASS / V4_PREFLIGHT_PASS`.

Implementation `ad6191ad` emits twice:

```text
811ac23180956ddf02efae5d52c949bc77619ce33907817d92974e41be4802ab
```

All five immutable R33 sources materialize with exact sparse ownership,
`source_positive=0`, distinct complete problem roots, exact lifecycle and zero
scale-control gaps. Rigorous projected excitation is:

| source | rows | certified positive | certified maximum | problem root |
|---|---:|---:|---:|---|
| torsion | 72 | 47 | `4.230716434701841e-3` | `5584d517...d3ad` |
| corner | 96 | 62 | `4.873602668286030e-3` | `ba002346...b21f` |
| counterflow | 72 | 48 | `3.989286639886989e-3` | `62b1dd7d...7a03` |
| helical compression | 80 | 65 | `7.375593711950711e-3` | `fa59af8d...7364` |
| alternating layer | 84 | 60 | `4.031991536373689e-3` | `192adf04...0e7` |

No solver/candidate iteration or inverse audit runs, and timing remains
inadmissible. The v4 corpus is therefore non-vacuous and eligible for one
predeclared generalization run, but no convergence evidence exists yet.

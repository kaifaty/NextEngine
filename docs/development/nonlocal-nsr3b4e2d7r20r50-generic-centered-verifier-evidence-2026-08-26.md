# NSR3-B4E2D7R20R50 generic centered-verifier evidence

Status: `PASS / GENERIC_VERIFIER_TORSION_CANDIDATE`.

Implementation `8d50c59e` emits reproducible semantic:

```text
190ac441077ac0499e5e9e4a9c419ef0d604bf6aa06017b49d3051ff7e12d86e
```

The hook never inspects tuple roots before deciding. It applies the same
dimension-65, depth-16 practical certificate to every legacy inverse failure.
All five observed certificates pass and are applied; the fifth tuple was not
present in any target-specific parent. Torsion then finishes by the unchanged
ordinary path:

| metric | result |
|---|---:|
| generic calls/replacements | `5/5` |
| accepted iterations | `11` |
| principal solves/transitions | `887/887` |
| final route | `ORDINARY_CERTIFIED` |
| certificate dots | `27,625` |
| dot input pairs | `1,802,450` |
| fraction of frozen 32-certificate cap | `5/32` |

The first new uniform depth-16 certificate has root `1f884821...f737`; the
second/third roots reproduce R49 exactly. The fourth and previously unseen
fifth roots are `843d754f...9409` and `f6987026...fcec`. All five are
no-underflow, left-contractive, fully signed and have positive separation; the
fifth uses `rho=0.07068`, radius `8.49e-15` and signs `19/46/0`.

Two R50 runs and R49/R48 regressions are exact. This proves a root-agnostic
algorithmic mechanism on torsion, not production readiness or corpus-wide
generalization. No timing was executed.

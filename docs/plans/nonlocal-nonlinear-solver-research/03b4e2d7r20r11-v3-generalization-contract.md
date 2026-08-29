# NSR3-B4E2D7R20R11 v3 generalization contract

Status: `FROZEN / ONE-SHOT EXECUTION AUTHORIZED`.

## Parent

- R20R10 implementation `10d2ac7e`, semantic
  `c2328bfb4c6b0ed90a66274a03b5abe51535918c513b3f56693d82ff4c87b577`;
- exact v3 problem roots:

```text
3dc846e59193dd0162a96d9728c313b65b2cf83bcbbe50b3d71d9da5a373dee3
9911ae713962919d0c0f31e1d595783be8c773370d3c87cd052f4fc247cd8c51
08c636f4bf6fbbb55ca97ebada500d387d7071b3f00194b97b3493e11ed68b62
948f5abd99a54feb83d319e0b7982518842680e91c33983581182736aef8ab22
```

- all four strictly excited; solver/inverse iterations zero before R11;
- R8 implementation `301b3a85` and semantic `afca1c77...374a` unchanged.

## Frozen execution

For each v3 problem, call `al_r20_semismooth_case(problem,true)` exactly as R8.
Keep the natural-residual formula, projector generalized derivative,
single-pivot NNQP, cheap and verified inverse enclosures, 21-trial Armijo
sequence, `2^-70` KKT tolerance and 32-accepted-iteration cap unchanged.

Require exact problem-root match before execution and exact workspace release
afterward. Record accepted steps, mask changes, active-set work, inverse audits
and terminal KKT tuple. Do not run v2 solvers in this command.

Routes:

```text
V3_GENERALIZATION_PARENT_REJECTED
V3_GENERALIZATION_SOLVER_REJECTED
V3_GENERALIZATION_UNRESOLVED
V3_GENERALIZATION_CERTIFIED
```

`V3_GENERALIZATION_CERTIFIED` grants solver-generalization evidence only. No
parameter changes, rerun-driven method changes, timing, binary64/runtime/GPU
integration or production authority.

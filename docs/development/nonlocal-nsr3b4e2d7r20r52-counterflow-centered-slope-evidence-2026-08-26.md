# NSR3-B4E2D7R20R52 counterflow centered-slope evidence

Status: `CLOSED FAIL / INVALID CAPTURE PREMISE / NO SCIENTIFIC SOLVER CREDIT`.

## Reproduction

The frozen command returns deterministic `FAIL` /
`COUNTERFLOW_CENTER_CAPTURE_REJECTED` at semantic
`86d2dcd45425b2885a86928bfdd17da3df51da6d47ea4a03538b299f24bb17a6`.
Parent and reproduction gates pass, including the exact R51 counterflow case,
701 transitions and R36 slope root `57b1b93b...f02f`.

## Falsified premise

- verified-inverse observer calls: `0`;
- captured inverse columns: `0`;
- final NNQP inverse-audit records: `0`;
- final support: `66/66`;
- affine-shadow updates: `86`;
- inherited direction error: `8.852949539615971e-3`.

The final direction error comes from ordinary principal solves and affine
active-set propagation, not from a successful verified-inverse audit. Hence no
matrix/inverse/RHS tuple existed for the frozen R52 certificate. The empty
certificate fields are apparatus sentinels and carry no numerical conclusion.

## Regression

The default-null observer preserves R50 semantic `190ac441...d86e` and R51
semantic `48df3b26...adca` exactly. No correction, direction, trial, state,
parameter, timing, runtime or production path changed.

## Consequence

Do not repeat successful-audit capture for counterflow. The smallest valid
successor is a separately frozen, report-only audit of the final direct
66-dimensional principal solve. It must declare one new factor/inverse budget
up front and may only recompose the existing slope bound after independently
certifying the represented solution.

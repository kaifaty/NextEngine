# NSR3-B4E2D7R20R17 dual-refinement candidate contract

Status: `FROZEN / REPORT-ONLY CANDIDATE EXECUTION AUTHORIZED`.

## Parent

- R16 implementation `e7194f05`, semantic
  `e4c5e8d1fa173359010c46c8b44d2921fc462b507657c3a53955c3e04d611f7f`;
- complete direct provenance and dual-refined ratio certificates on both R11
  counterexamples;
- R8 default implementation semantic `afca1c77...374a`.

## Frozen candidate

Add an opt-in `dual_ratio_refinement` flag, default `false`, through NNQP,
semismooth step and case. Default-false arithmetic, branches and roots must not
change.

When and only when the original failure is `RATIO_ORDER_AMBIGUOUS`, the flag
may consume the already defined R16 current/candidate audits. Continue only if
both pass, current provenance corresponds and the dual-refined ratio is exact.
Set current/candidate scalar error enclosures to their verified refined values,
then execute the existing ratio interpolation/removal code unchanged. Otherwise
return the original rejection.

Execute the candidate on all eight original manifest cases and all four v3
cases. Keep 21 line trials, `2^-70` KKT tolerance and 32 accepted-iteration cap.
Require exact input roots, lifecycle, cone/NNQP/globalization/KKT checks and all
twelve certifications. Record fallback count, inverse columns and downstream
route per case.

Separately rerun default R8 and require semantic
`afca1c777053172169e227bcf3625ad7c828892520340db5eb45c98d53b4374a`.

Success grants only `DUAL_REFINEMENT_DEVELOPMENT_CANDIDATE`. No blind
generalization, performance, binary64/runtime/GPU or production authority.


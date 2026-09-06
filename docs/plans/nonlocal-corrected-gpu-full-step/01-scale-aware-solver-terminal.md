# NCGP1 scale-aware projected solver terminal — revision 2

| Field | Value |
| --- | --- |
| Research ID | `NCGP1` revision 2 |
| Status | `FROZEN / IMPLEMENTATION_AUTHORIZED / REPORT_ONLY` |
| Supersedes | Revision 1 only where it inherited no explicit scalable physical terminal from NCGA5 |
| Parent contract | `00-physical-performance-contract.md` at commit `059b9349` |
| Reason | Raw `||g||_2<=1e-10` is an immutable NCGA5 correspondence observable, not a physically scaled terminal for the separately selected NCGP1 trajectory claim |

## Exact terminal

NCGP1 retains every revision-1 profile, objective, trust-region decision,
physical gate, HVP budget and performance rule. It adds one scale-aware
terminal before solver implementation:

```text
R_x = dt^2 / mass * max_i ||g_projected_i||_2 / spacing
success when R_x <= 1e-5.
```

This bounds the unresolved inertia-equivalent displacement by
`spacing*1e-5 = 0.5 micrometres`, one order inside the frozen `5 micrometres`
tiny state gate. Raw `||g||_2` and the old `1e-10` result are still reported,
but they do not select the NCGP1 physical route and cannot relabel NCGA5--7.

## Box-projected gradient

The analytical basin owns center bounds
`[spacing/2, extent-spacing/2]` per axis. At a lower active face, a positive
gradient component would produce infeasible negative descent and is replaced
by zero. At an upper active face, a negative component would produce
infeasible positive descent and is replaced by zero. All other components are
unchanged. Equality is evaluated at the binary32 stored center bound; no
implicit epsilon is permitted.

Steihaug residuals, preconditioning and directions consume this same projected
free-variable space. Trial positions are swept/clamped to the exact bounds.
`pred` is recomputed from the actual projected step, not the unclamped Krylov
proposal. Contact projection counts and face masks are sealed.

## Resolution firewall

- Initial `R_x<=1e-5` is a valid zero-work terminal after one full evaluation.
- Otherwise at least one trial must be accepted before success.
- Reaching the selected total HVP ceiling before the terminal is
  `WORK_BUDGET_EXCEEDED`, not a partial success.
- Minimum radius, nonfinite reduction, missing descent, graph/capacity failure
  or changing a frozen control remains failure.
- No line search, hidden regularization, warm solve, relaxed threshold or
  post-timing terminal change is authorized.

The revision can support only the NCGP1 finite physical corpus. It is not an
Accepted solver tolerance, a product-water decision or proof of convergence.

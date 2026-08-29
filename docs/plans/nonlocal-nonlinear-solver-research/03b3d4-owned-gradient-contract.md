# NSR3-B3D4 -- displacement-owned gradient discriminator contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / B3_RETRY_BLOCKED`

Parent D3 status is `FAIL / ONE_TRIAL_POLICY_REJECTED`, semantic SHA-256
`a550a2e9cd6783bd74a022c612236e47542a1e229f2ec3e4432b7663ef3a0071`.

## Captures

Reproduce D3 and capture its first failed floor state for face/corner at
`96/192/384` substeps per frame. Require exact D3 JSON-without-newline SHA-256
`65737fcabbe8a7d4e926e38bd39b0c4cef20c933b07c19505195641a8a3d1719`.
Publish state hashes and exact substep/failure values.

## Gradient identity

At each captured current state evaluate the same pressure support once, then
construct:

```text
g_legacy = grad Phi + M/h^2*(y-y*)
g_owned  = grad Phi + M/h^2*(delta-delta*)
R_direct = M/h*sum(delta-delta*) + h*sum(grad Phi).
```

Publish norms and vectors for `h*sum(g_legacy)-R_direct` and
`h*sum(g_owned)-R_direct`. The owned identity must lie inside
`displacement_forward_bound + gamma_(N-1)` over the displayed sums. The bound
is computed, not fitted. Legacy mismatch is diagnostic and has no pass/fail
threshold beyond finiteness.

## One-step counterfactual

Use `g_owned` as the only changed input to the existing Steihaug trust step.
Keep the captured trust radius, position-based pressure evaluation, analytic
HVP, negative-curvature handling and all coefficients unchanged. Materialize
the trial from `x+(delta+p)` and measure the owned reaction residual.

Publish HVP calls, step norm, topology, current/trial residual and existing
trial reaction limit. Classify all six states together:

- `OWNED_INERTIA_GRADIENT_CANDIDATE` if the owned identity certifies and every
  new trial retains topology and reaches the unchanged reaction limit;
- `BOUNDED_OWNED_RESIDUAL_ITERATION_REQUIRED` if identity certifies and every
  new trial strictly reduces residual but at least one remains above limit;
- `OWNED_RESIDUAL_LINE_SEARCH_REQUIRED` if identity certifies but any full
  trial fails strict residual decrease;
- `OWNED_GRADIENT_IDENTITY_REJECTED` otherwise.

D4 PASS validates the diagnostic and one classification only. It cannot
continue trajectories or authorize B3R.

## Exit

Two reports are byte-identical. D3, D2, D1, B3D, B3 and B2 remain exact.
Report identity is `displacement-owned-inertia-gradient-r0`.

No physical corpus, CUDA, performance, runtime or production authority is
granted.

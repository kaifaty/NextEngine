# NSR3-B4E2D7R20R26 certified event-side trajectory evidence

Status: `PASS / V3_CERTIFIED_EVENT_SOLVER_REJECTED`.

Implementation `4b0f58ba` preserves all three external regression semantics:

```text
R8   afca1c777053172169e227bcf3625ad7c828892520340db5eb45c98d53b4374a
R17  d224bc8c0f451e1a07aa4cbdf6cc194c85f6341ca3be5a1d74e2a7d97cfe6ab8
R25  d3544d4255d066245d6055b71baa56e67d70e44fce001ba1da83844a4e62d449
```

The opt-in 12-case replay emits:

```text
4203c7ce2b86014a3099eb440bb0d011f8b6e31856ef142c3acbd93ed807a8d2
```

Eleven cases certify. Four event replacements are applied with zero fallback:
three in shear and one in radial compression. Radial remains certified in ten
accepted steps, so the mechanism does not regress that historical case.

Shear no longer reaches the cap. Its three certified crossings reduce the
primal/dual-mapping residual from the R17 value `1.745e-7` to `4.343e-9`, about
40 times smaller, in 19 accepted steps. The following twentieth step cannot
accept any of the existing 21 dyadic trials and returns
`GLOBALIZATION_REJECTED`. Its candidate case root is:

```text
1c9a0bf3457b32e6e81c3aab256dd1ea7ac26cdbd6f5e9c51943e60cd4c9cf91
```

This supports the causal claim that forward-certified face crossing removes
the old pre-event cycle. It does not establish convergence: the trajectory has
reached a new piecewise/globalization boundary. Cap extension and performance
claims remain blocked; the exact rejected step must be diagnosed before
another candidate.


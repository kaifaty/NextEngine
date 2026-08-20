# NSR3-B3D4 displacement-owned gradient evidence -- 2026-08-21

Status: `PASS / BOUNDED_OWNED_RESIDUAL_ITERATION_REQUIRED / NO_B3_RETRY`

The frozen
[D4 discriminator](../plans/nonlocal-nonlinear-solver-research/03b3d4-owned-gradient-contract.md)
replayed the six exact D3 first-failure states twice byte-identically. It
confirms that D1 ownership was incomplete inside the optimizer: reaction used
owned displacement, while the trust gradient still reconstructed inertia from
materialized positions.

## Identity correspondence

In every state, `h*sum(g_owned)` matches the directly measured impulse residual
well inside its computed bound:

| State | direct `||R||` | legacy identity error | owned identity error | owned bound |
|---|---:|---:|---:|---:|
| face / 96 | `5.63e-7` | `2.36e-13` | `9.77e-19` | `3.22e-14` |
| face / 192 | `8.00e-8` | `6.09e-13` | `9.67e-19` | `4.48e-14` |
| face / 384 | `1.51e-12` | `3.17e-12` | `5.18e-21` | `4.46e-14` |
| corner / 96 | `2.01e-7` | `1.53e-13` | `1.42e-19` | `1.97e-14` |
| corner / 192 | `2.94e-7` | `5.37e-14` | `1.33e-18` | `1.91e-14` |
| corner / 384 | `1.24e-12` | `1.48e-12` | `9.11e-21` | `1.97e-14` |

At both fine levels the legacy mismatch exceeds the residual itself. That
fully explains D3's apparent overshoot: the HVP was applied to a gradient from
a different finite-precision state than the reaction gate.

## One-step result

Changing only the inertia-gradient input eliminates overshoot. All trials keep
active/pair topology exact, use one HVP and avoid negative curvature:

| State | legacy trial defect | owned trial defect | limit | owned/current |
|---|---:|---:|---:|---:|
| face / 96 | `4.91e-11` | `4.93e-11` | `1.11e-11` | `8.76e-5` |
| face / 192 | `8.21e-12` | `7.60e-12` | `2.87e-12` | `9.50e-5` |
| face / 384 | `3.17e-12` | `2.66e-17` | `1.02e-12` | `1.76e-5` |
| corner / 96 | `2.38e-11` | `2.38e-11` | `2.59e-12` | `1.19e-4` |
| corner / 192 | `2.69e-11` | `2.69e-11` | `8.47e-12` | `9.15e-5` |
| corner / 384 | `1.48e-12` | `5.55e-17` | `6.06e-13` | `4.48e-5` |

Both fine states now converge in one step. Four coarse/mid states reduce
residual by roughly four orders but remain `1.6x--9.2x` above the gate. This
selects `BOUNDED_OWNED_RESIDUAL_ITERATION_REQUIRED`, not the one-step
`OWNED_INERTIA_GRADIENT_CANDIDATE`.

## Architectural conclusion

Owned displacement must cover the complete inertia state boundary:

```text
energy/gradient: delta-delta*
reaction:        delta-delta*
velocity:        delta/h
position:        x+delta
```

The inertia HVP does not change. A next full-trajectory candidate may enter a
bounded residual phase at the inherited energy floor, rebuild the owned
gradient after each accepted merit step, and stop at the unchanged reaction
gate. It must cap iterations and fail closed on non-decrease/topology change.

## Repeatability

```text
D4 raw SHA-256 (two identical runs):
60a53643f7a587cbfce8fb4d12f7c774b945742e1a66ca04ea4e53a375517946

D4 JSON-without-newline SHA-256:
36f91802d114ba87124d00303c6ac413fbc09d4cd792f4262636e9816a5680ca

D4 semantic SHA-256:
bdf5b7eaa1f215e14ebb05500ffa2495b8cb3fccd19aa4b369514464086aae34

D3 raw preserved:
aa16501e6402c5a4170cec7f58d030506d564362dd928d91ca5d6c36463ca792

D2 raw preserved:
0b9581489e133039c7562f938fdb79d73a8810222341f74aa09f61edf668a1ae
```

No B3 retry, physical corpus, CUDA, performance, runtime or production
authority is granted.

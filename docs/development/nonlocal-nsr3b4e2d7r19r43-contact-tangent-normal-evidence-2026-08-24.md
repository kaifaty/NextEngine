# NSR3-B4E2D7R19R43 contact-tangent normal-step evidence

Date: `2026-08-24`

Status: `PASS / TANGENTIAL_MERIT_STEP_REQUIRED / ROLLBACK EXACT`.

## Outcome

Exact projection of the R40 normal step into all source-active box tangent
half-spaces completely removes contact regression and preserves strong density
progress. It does not make the complete normalized merit decrease. The frozen
route is therefore `TANGENTIAL_MERIT_STEP_REQUIRED`, not step acceptance.

This closes contact ownership for the nominal box case and isolates the next
missing SQP component: a tangential objective-reduction step compatible with
the density linearization and active contact cone.

## Projection and contact

| Quantity | Result |
|---|---:|
| Source-active faces | `1290` |
| Clamped inward components | `1268` |
| New / worsened projected-trial faces | `0 / 0` |
| Source / trial maximum penetration | `2.9985485743705409e-8 / 2.9985485743705409e-8` |

```text
owner root              baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187
projected endpoint root 30885f22768de119065907526f36c225d058c6b97693770be109faf3ddd3b960
projected physical root e73131b4cf059c1c2adc7521f1960d58cae1fcd865685ace0a708cba3d3207b3
projected trial root    0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50
```

Projection is idempotent, norm-nonincreasing and preserves every unconstrained
component bit-for-bit. It is a step-space tangent projection, not a final
position clamp.

## Feasibility, relinearization and merit

| Quantity | Result |
|---|---:|
| Predicted hinge reduction | `2.6064437667433723e-15` |
| Actual hinge reduction | `2.6064437477608628e-15` |
| Ratio `rho` | `0.99999999271708462` |
| Trial hinge `psi` | `6.8540208964482797e-16` |
| Trial violation | `3.7024372773750754e-8` |
| Trial active rows | `464` |
| Fresh trial mapping | `9.7690943252330519e-9` |
| Complete merit reduction | `-1.7608141057937782e-15` |

The density model remains essentially exact, but projection retains less
feasibility improvement than the unprojected R40 trial. Long double resolves
and agrees with the negative complete-merit sign; binary128 is correctly not
executed. No tolerance or coefficient is fitted.

## Reproducibility

Implementation commit: `e6cf3feb`.

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r43-a.M7flL0
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r43-b.Tip5zB
binary SHA-256 fef830989fd02af5100b6110f696ba1014f1262d50c7603b59e42ed7a52b0bfc
size           7652416
ELF build-id   96ca70f34562ce4decd32a6553ce8861045934c2

/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r43-a.JMzRpj
/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r43-b.cZwfQf
stdout bytes   1560
stdout SHA-256 565c70d251612bac17c0b4d20b5ae4f284f9d64c2edd34871db640b1d58bc993
semantic       e1f019dae6e71e40633e56c79ee60e8e81dc3b146fdf55d75542298737f9eea6
stderr bytes   0 / 0
route cases    10 / 10
route root     7d24ebce97b71949eb7da8fbbf3ae794b45d1a0e961ded02b4e21cf207e6a83f
```

No timing was measured or interpreted on the shared host.

## Decision

Preserve the unprojected and projected normal trials as separate immutable
baselines. Research whether a contact-cone-compatible tangential direction
with negative complete-merit directional derivative exists near the projected
trial while remaining approximately in the null space of the fresh density
operator. Do not implement a composite step until that discriminator is
frozen and passes.

# NSR3-B4BK1 contact-face evidence -- 2026-08-21

Status: `PASS / BOX_CONTACT_KKT_CANDIDATE / B4B_R1_DESIGN_AUTHORIZED`

## Reproduction

```text
nonlocal-formula-reclosure --box-contact-kkt-face-self-test
```

Three complete reports are byte-identical:

```text
raw JSON plus LF  0da0758e2e375f403b6cfb061b7c2e6a1e7f499a07d29c299a7a4b8d92542343
JSON without LF   eed7934a81dc451edc2eeaaee81dcb403e2bb94e6799edfdc1646cb7970b4bdd
semantic result   48db28247059f4f61870ee8ff9bc680ebbb5e672039c5c199b11e14dbe980197
```

Parent B4BK r0 remains exact FAIL. B4BK1 changes only the erroneous face
assertion and passes `48/96/192` plus detached P2.

## Exact face result

Every P1 state has counts `[12,12,16,0,12,12]` for
`[x-,x+,y-,y+,z-,z+]`. Representative signed face impulses are:

| fixed | x pair closure, N s | z pair closure, N s | y- fluid impulse, N s | face/aggregate error, N s |
|---:|---:|---:|---:|---:|
| 48 | `5.8144364906034633e-17` | `1.566360854633438e-16` | `1.7031250046244675e-3` | `2.1684050920502838e-19` |
| 96 | `1.0739033106380762e-16` | `5.8144444315373438e-17` | `8.5156250014487098e-4` | `2.1684044701510843e-19` |
| 192 | `8.2767324823863976e-17` | `5.3695096048732356e-17` | `4.2578125000453012e-4` | `4.5273348405515001e-24` |

The lower-y summed multiplier is approximately `19.62 N` at every step size,
which is the weight of the 2 kg bottom layer. Side multiplier sums scale with
the tiny pressure signal and cancel in the signed aggregate.

## Decision

Select `BOX_CONTACT_KKT_CANDIDATE`. Freeze a new full-corpus identity before
continuing P1/P2. No r0 or B4B threshold is amended, and this result gives no
general-mesh, moving-solid, friction, nominal-water or runtime authority.

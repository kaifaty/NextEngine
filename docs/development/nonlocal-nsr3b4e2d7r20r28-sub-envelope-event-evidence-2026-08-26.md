# NSR3-B4E2D7R20R28 sub-envelope event evidence

Status: `PASS / SUB_ENVELOPE_BIDIRECTIONAL_ACCEPTANCE`.

Implementation `36f67e97` reproduces R27 and emits, twice:

```text
a95370b4d4df0159b6cfd569fc85f714766772c702fc35a4f7e2ae617124e52d
```

The fixed iteration-20 face generates 421 component/ball polynomials and 346
algebraic roots. Exactly one root lies in `(0,2^-20]`, and it is admissible:

```text
scalar          168
transition      lower-bound (-1) -> free (0)
bound           zero
alpha_root      1.890816455779798661676982298358834440e-7
dz              9.846749408215789949998378710880995572e-3
```

There is no second bounded component or ball event. The relation residual at
the computed root is `1.79e-43` in binary128.

The unchanged R25 formula sees 12 contributing transpose entries and gives
`Bz=9.879651368079126e-33`. It certifies the two representable sides:

```text
alpha_old  1.890816455779798661676972264944802294e-7
alpha_new  1.890816455779798661676992331772866585e-7
```

The old point retains the complete current mask; the new point changes only
scalar 168. Neither changes ball activity. Both pass rigorous Armijo with the
same displayed margin `7.141838042719974e-19` and reduce the primal residual
slightly from `4.343128927689235e-9` to `4.343128106483270e-9`. Their roots are

```text
old  713db0af86420daba56f7194bd828c6e93c068cdc50d499e2f60dfaff1af76cc
new  d5536891930aead154c3d0f29f10045807d01755759afecdf5c723cb5c2453c1
```

This supports E1 and refutes coupled-event, local-rejection and sampled
precision explanations on this exact direction. It proves that fixed dyadic
depth, not a bad Newton direction, caused the R27 rejection. It does not prove
that committing either point converges; R28 performs zero state updates.

R27 and R26 remain exact at `6ef46e9e...e003` and `4203c7ce...a8d2`.
The next bounded candidate commits only the certified new-side point after the
exact R26 prefix, once, then observes the unchanged remaining trajectory.

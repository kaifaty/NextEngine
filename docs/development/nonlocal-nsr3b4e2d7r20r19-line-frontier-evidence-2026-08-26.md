# NSR3-B4E2D7R20R19 shear line-frontier evidence

Status: `PASS / MASK_CROSSING_FRONTIER`.

Implementation `38b788c8` reproduces the exact R18 shear trajectory and emits:

```text
46f8d84a950e3987609d32858f1c972d60e1cf1d937f1c3fd5b20d1f058ca06f
```

The independently reconstructed binary128 Armijo margin agrees with every
stored accept/reject decision. In steps 25--32:

- seven first mask-stable powers equal the first accepted power;
- no mask-stable existing trial is rejected;
- one crossing trial is accepted, at step 28 and power 16;
- ball activity never changes.

The seven exact coincidences are powers `10,13,14,1,4,8,9`. Before each of
those accepted trials every observed larger alpha changes at least one
projector-mask component and has a strictly negative Armijo margin. The
accepted same-mask margins are small but rigorously positive. Step 28 is the
single transition control: its accepted trial still changes one mask component,
after which the passive support grows from 66 to 67 and the next accepted
power recovers to one.

This rules out the observed dyadic same-face Armijo/model rejection mechanism.
The cap trajectory is instead throttled at a projector-face boundary. It does
not yet prove that the continuous interval contains only one boundary or that
an exact breakpoint step would satisfy Armijo; endpoint mask equality alone
cannot exclude hidden transitions. A fixed-resolution within-bracket geometry
audit is therefore required before implementing a mask-aware path.

No cap, tolerance, line trial, state, runtime or production policy changed.


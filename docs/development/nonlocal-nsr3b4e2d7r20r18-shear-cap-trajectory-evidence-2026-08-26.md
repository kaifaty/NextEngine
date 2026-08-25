# NSR3-B4E2D7R20R18 shear cap-trajectory evidence

Status: `PASS / ACTIVE_SET_CHATTER + GLOBALIZATION_THROTTLED`.

Implementation `251e13e7` reproduces the exact R17 shear case root and emits:

```text
6f019b7c314b09406f68438ec4beadfd839b7664952d346a9c898563036d40aa
```

In frozen steps 25–32:

- all eight accepted powers are positive;
- natural face and ball activity never change;
- the projector mask changes once, at step 28;
- accepted alpha shrinks through `2^-10`, `2^-13`, `2^-14`, `2^-16`, then
  recovers to `1/2` after support grows `66 → 67`, and shrinks again;
- primal/dual ratios range from about `0.5` to `0.9999847`.

The predeclared priority labels this `ACTIVE_SET_CHATTER`, while the independent
T4 flag is also true. It is not a stationary numerical plateau and not uniform
useful contraction. The concrete pattern is a stable natural face with an
Armijo step collapsing near a projector-mask crossing, followed by one mask/
support transition and temporary recovery.

Increasing the iteration cap would merely repeat an unresolved cycle. The next
discriminator must compare the Armijo acceptance frontier to the first
projector-mask-stable trial.


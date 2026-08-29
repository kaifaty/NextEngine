# NSR3-B4E2D7R20R27 post-event globalization evidence

Status: `PASS / POST_EVENT_LINE_ENVELOPE_EXHAUSTED`.

Implementation `5ac52d26` reproduces the complete R26 shear candidate root

```text
1c9a0bf3457b32e6e81c3aab256dd1ea7ac26cdbd6f5e9c51943e60cd4c9cf91
```

and emits the same R27 result in two executions:

```text
6ef46e9ec490bfb40e4b7eecbf50898b3efb4d494fa1028fd501c1f14be6e003
```

The three applied events at iterations `14/17/18` are exact one-component
zero-lower releases for scalars `169/168/189`. Each changes mask code
`-1 -> 0`, changes no other scalar, retains ball activity and produces a
resolved next natural face plus certified NNQP support. This refutes a broken
event-lineage explanation on the frozen trajectory.

At iteration 20, the natural face has 68 rows and the certified NNQP support
has 66. Its slope is `3.7808108286680605e-12` against a bound of
`2.6716768731728212e-33`. Nevertheless all 21 existing dyadic trials from
`alpha=1` through `2^-20` cross the current projector mask: powers `0..7`
change two scalars and powers `8..20` change one. Ball activity never changes,
there is no mask-stable sample and every rigorous R19 Armijo margin is strictly
negative. The smallest absolute margin is `8.407578628753779e-18`, so no
sampled sign is a binary128 precision boundary.

The strongest result is `SUPPORTED_BOUNDED`: the existing line envelope ends
before the first stable current-face point. It does not prove where the nearest
event lies, that a trial below it is acceptable, or that another solver step
would converge. Increasing the dyadic cap, loosening Armijo or promoting R26
to production remains unsupported.

External non-regression roots remain exact:

```text
R8   afca1c777053172169e227bcf3625ad7c828892520340db5eb45c98d53b4374a
R17  d224bc8c0f451e1a07aa4cbdf6cc194c85f6341ca3be5a1d74e2a7d97cfe6ab8
R25  d3544d4255d066245d6055b71baa56e67d70e44fce001ba1da83844a4e62d449
R26  4203c7ce2b86014a3099eb440bb0d011f8b6e31856ef142c3acbd93ed807a8d2
```

The next discriminator is the frozen R28 analytic event bracket inside
`(0, 2^-20]`; it adds no dyadic samples and cannot update solver state.

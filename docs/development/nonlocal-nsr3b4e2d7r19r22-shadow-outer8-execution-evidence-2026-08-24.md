# NSR3-B4E2D7R19R22 shadow outer-8 execution evidence

Date: `2026-08-24`

Status: `PASS / SHADOW_OUTER8_EXECUTION_CANDIDATE / NOT ADMISSIBLE`

## Outcome

The slice candidate and independent unsliced oracle execute outer 8 with
complete bit/work equivalence. Candidate remains within epoch-1 capacity.
The result is finite and accepted but still non-admissible.

```text
update root       e5af5115929a80ce1886615675ed60528a76b2a14cab77e377b70702da2e8b88
work root         b8b9df3b0ff13503fbe8d86f474de94fa33b638f1ab558f9a0ae90a185485dc5
trials            3 accepted / 0 rejected
HVP/workspace/precision delta   80 / 5 / 3
candidate/oracle exact          true
```

## Physical and mechanism result

```text
primal          2.0358387642360753e-08   bits 0x3e55dc122c000000
stationarity    1.924141810385112e-14    bits 0x3d15a9f60fbefbe2
admissible      false
position root   68d0b821be6f1bdd71f70bf570035be7713cd578fe08774644cbd7c53c672b0f
dual root       fbbe1973290706320cbb5f3f3280b6e62b493e3abad4d280ac2985951b7568f3
```

Primal improves only about 4.9% from outer 7, versus about 14.9% from outer 6
to outer 7. This single slowdown observation does not justify changing the AL
or trust policy. All three trials are accepted; increased work is inner-solve
work rather than reject churn. The last trial uses 32 HVP against the unchanged
34-HVP per-trust cap, so the next execution must retain fail-closed structural
classification.

## Canonical successor

```text
history         fa9e2db82287c5f3faac6d8c8031c55bab78230fd708479a569329152c74be7c
consumed owner  08001aecf61cc43ee130187f626264aa093420830244e06293ee7824486c9721
receipt         96975d4dda71d7490e91ccccfa95c51de480c1bcd3191367f527d5b001c2fa8e
state           dc983c93075040fc2f23080072807f8a1b691adade4f3d7a950be3c9f44b36d9
used            9,3,32,1,182,705,46,28,679,26,2,28,0
```

Epoch remains `1`; slice/cumulative HVP become `182/705`, leaving 330 slice
HVP. Recurrence/direct accounting closes at `679 + 26 = 705`.

All `13/13` routes pass at corpus root
`3ca19eaf7ac8bd0eeef827ae76460cd24641efb4b27fc904458fba7f267abe3d`.
Rollback and duplicate replay are exact. No following outer, substep, macro,
trajectory, timing or public/world commit occurs.

## Reproducibility

Contract/research commit: `87a2c764`.

Implementation commit: `d5e3f437`.

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r22-a.x4vSkK`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r22-b.SNPfAj`.

Both binaries are `6,602,008` bytes, have SHA-256
`7efb99868e7ad441c094a83c0e08684b31035686aeacfb6ac76f0e62c6f9ef23`
and GNU build ID `004db5399681e60767ba5e83616b1bc7ab0b744d`.

Fresh one-process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r22-a.ck6s5F`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r22-b.0suXdH`.

Both exit `0`, emit empty stderr and reproduce `1,883` stdout bytes:

```text
stdout SHA-256  32d98ee5549b5321201d9280ddb700967a02de6a83bd8abc283bd2a7d4b18790
semantic        9a5380ef56951213b544b14573e7983d84f1b768e9b16b5aa41a33084fb16296
route           SHADOW_OUTER8_EXECUTION_CANDIDATE
```

## Next action

Research/freeze a zero-work one-use outer-9 grant at exact state/receipt/
history and slice/cumulative `182/705`. Do not change caps or execute outer 9
before that transaction passes.

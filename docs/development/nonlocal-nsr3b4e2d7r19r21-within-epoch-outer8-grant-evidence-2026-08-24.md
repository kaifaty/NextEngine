# NSR3-B4E2D7R19R21 within-epoch outer-8 grant evidence

Date: `2026-08-24`

Status: `PASS / OUTER8_GRANT_CANDIDATE / SHADOW ONLY`

## Result

The frozen zero-work transaction reproduces exact R20, consumes the separate
R20-state owner once and emits an unconsumed outer-8 owner. Epoch and every
resource counter remain unchanged.

```text
route                  OUTER8_GRANT_CANDIDATE
parent stdout SHA-256  df06ac4125082f715d200829b5afef203f0a9ab046f6e9018f97bd8066b1d113
stdout SHA-256         88253c46b9c1763ee4855be5909d1f9b58cf02828efa582b86a1eab61e1cb1a0
semantic result        99d0a756044173aebb640e6d2bc301ac09c993b4ecb92d95bad886b6b4437719
epoch                  1
slice/cumulative HVP   102/625
next outer             8
new solver work        0
```

## Canonical and atomic evidence

All pre-derived roots reproduce exactly:

```text
owner before  138f23969d4cc130e02ddd54cbe1fbbb76dd75b0beee0f53312de8f3116b0c7b
owner after   ca301305414a08ca75cc8acda791d0c4a93152988d184f63316658ac278032d1
grant         6df9d41926767b1cca9df69f576170253e2eb6682435850372bbf4420b7d0b43
receipt       0675f2ab94286aee2b85afe6cf496bcef8fd615efc514143ecf978c51e2bc0f3
active owner  1987820aa99c22ac2d49abbb894994370a2f85231e9b4792bfc518e3d0eb4c54
state         64f90f969b76efcddb331705a653c8d31858e6d502555615d6eea2071c78093f
```

All `14/14` routes pass with corpus root
`2d56e84701dc4bec30ed32549d5fb41336c5b02570cd3638022eb1c74b84ba38`.
Every negative route preserves complete transaction bytes, the sole success
equals the canonical committed state and duplicate replay is idempotent.

R21 adds zero HVP, workspace, precision audit, trial or outer update. It does
not execute outer 8, reset epoch, publish state or create runtime/production
authority.

## Reproducibility

Contract/research commit:
`576e07dc` (`docs: freeze D7R19R21 within-epoch outer8 grant`).

Implementation commit:
`243ec39a` (`research(nonlocal): implement within-epoch outer8 grant`).

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r21-a.EtVozt`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r21-b.kDL8wO`.

Both binaries are `6,567,448` bytes, have SHA-256
`9851684dd97a9136dcc34b79f742f50d063b36a06dfd81a90002d4c2f123701c`
and GNU build ID `827cf15fc0254edae2f0a07008866dc317529229`.

Fresh one-process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r21-a.vjlX4Y`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r21-b.7PNgW3`.

Both exit `0`, emit empty stderr and reproduce the same `1,411` stdout bytes.

## Next action

Research/freeze one shadow outer-8 candidate starting at slice `102/512` and
one independent unsliced oracle starting at cumulative `625/8704`. The
candidate has at most 410 new HVP. Do not execute either lane until the new
contract is frozen.

# Physical sound V37 D0 one-shot protocol

## Authority and question

This protocol opens only the fresh V37 synthetic `train` and `development`
roles frozen by F0/T0. It asks whether QSO-v0 meets the preregistered synthetic
transfer gates. It does not use real audio, validate a material sound, authorize
H0 before D0 `Pass`, cook PCM, enter the demo or change runtime authority.

The only numeric owner is D0R owner `ebbdb386…c50c`. An official capability is
issued only after exact verification of E0 seal `608f9019…1d7f`, T0 seal
`7bae65cc…61573` and provider-ready D0R seal `a2c3c6ce…b18c5` in the frozen
NumPy `2.3.5` / Torch `2.8.0+cu128` CPU environment.

## Predetermined execution

1. Commit the official provider, D0 profile, runner, protocol and pre-access
   tests before evaluating any official target.
2. Run exactly two fresh independent processes, labels A and B, with the same
   checked profile and external empty output parents.
3. Each process verifies all seals, issues one `OFFICIAL_D0` capability, opens
   train before development, evaluates exactly `10,800` target rows / `32,400`
   target values, and keeps method holdout, real and protected access at zero.
4. The sealed owner executes the frozen training, controls, metrics, hard gates,
   resources, terminal precedence and atomic publisher without changed seed,
   capacity, loss, thresholds or role selection.
5. Compare stdout, stderr and the complete output trees byte-for-byte. Empty
   stderr and exact A/B equality are required independently of the scientific
   terminal.

The process command is:

```text
uv run --project lab --with numpy==2.3.5 --with torch==2.8.0 \
  python lab/scripts/physical_sound_v37_d0_fresh_development_v1.py \
  --profile lab/profiles/physical-sound-v37-d0-fresh-development.v1.json \
  --output <fresh-external-output>
```

## Terminal semantics

- `Pass`: publish exactly candidate bundle, candidate freeze, evidence and
  terminal descriptor; only this exact freeze may authorize H0.
- `MetricReject`, `HardGateReject` or `ResourceReject`: publish exactly rejected
  candidate evidence, evidence and terminal descriptor; the QSO-v0 family
  closes and H0 stays unopened.
- `OwnerFault`: post-target fault is terminal, publishes no candidate authority
  and permits no repaired retry.
- `ContractReject`: pre-target failure evaluates zero official values and may be
  repaired before the one-shot target opening.
- Any A/B difference is terminal `DivergentExecution`, grants no candidate
  authority and permits no parameter selection from either run.

No human listening or favorable-result choice can modify the decision.
Generated targets, weights and terminal trees remain outside Git. The checked
result document records only compact hashes, counts, metrics and disposition.

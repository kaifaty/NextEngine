# NSR3-B4C3MAG -- canonical topology reclosure

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / FULL_REPLAY_BLOCKED`

The B4C3MA raw-equality negative control has JSON-without-final-LF SHA-256
`8def13d3b0f54fdffaa451846219d9a842acda416e2270464349aff6831847a1`
and semantic SHA-256
`ee8b80a04e0b251c95697f22a8fc07bd7b4101cf171c3c14a287f3d42e70eb8f`.
The positive B4C3PE1 parent remains exact at
`eb4d82300653d779baf00620cb83a2526d164347c1b97a65b487f1955a3b8d60`.

## Policy identity

```text
sha256  422ae7267170615c23e1757c5b80b14e5811dae7721b170cc9586a957835fa63
text    nextengine.nonlocal.canonical-topology|v1|coordinate=position-um|equality=integer|raw=binary-diagnostic|penetration=zero|parent=macro-adaptive-v1
```

## Exact feature rule

For every sample and box axis, quantize the sample coordinate and low/high
boundary coordinates through `canonical::quantize_position`. Emit lower face
`2*axis` or upper face `2*axis+1` only on exact integer equality. Sort and
compare `(sample_id, face)` pairs exactly.

Require:

1. quantized low `<` quantized high for all axes;
2. every decoded coordinate unit lies within the closed integer interval;
3. private canonical features equal the selected fine KKT terminal features;
4. decoded canonical features equal those same terminal features;
5. no raw decoded penetration; raw binary feature sets and their differences
   remain reported but do not define durable identity.

## Negative controls

- `0.2-0.025` and `0.175` must be raw-unequal and canonical-equal;
- a coordinate exactly one canonical unit away must not match;
- a sample/face id substitution must change the sorted topology set;
- collapsed low/high integer geometry must be rejected;
- out-of-box decoded integer geometry must be rejected.

Reuse all B4C3MA solver, selection, mixed admission, macro-ledger, roots, work,
exact failure grammar and rollback gates unchanged. Reproduce B4C3MA's exact
FAIL report and, for the full command, B4C3PE1's exact PASS report. Two complete
B4C3MAG reports must be byte-identical.

## Decision boundary

PASS selects `CANONICAL_TOPOLOGY_ADAPTIVE_MACRO_TRANSACTION_CANDIDATE` and
authorizes only complete adaptive macro replay design. FAIL preserves B4C3PE1.
Adaptive-versus-fixed comparison, nominal corpus, B4C4/B4D, CUDA, runtime/
schema and production remain blocked.

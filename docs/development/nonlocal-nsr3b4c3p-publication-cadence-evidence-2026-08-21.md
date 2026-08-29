# NSR3-B4C3P publication cadence evidence

Status: `FAIL / MACRO_CADENCE_CONFIRMED / STABILITY_ENVELOPE_UNCLOSED`

Date: `2026-08-21`

## Reproducible result

Command:

```text
nonlocal-formula-reclosure --publication-cadence-probe
```

Two isolated reports are byte-identical:

```text
status                 FAIL
raw JSON + LF          d7603a66d935736127691e49b235e8aabd18ac6017d45b0a35b7ea755fe39d5f
raw JSON without LF    2cbeaafe6b7cdf06a1d982bbdb4134823aecc71247b3fed496bd48e5e32b5b1c
semantic result        e79566a815a2491510318acf1793b693c642b06c84b167f0a4a1e1bd1344965b
wall time              21.76 s / 21.85 s
user CPU                60.07 s / 60.34 s
machine utilization    299% / 299%
maximum RSS            12,136 KiB / 12,908 KiB
```

The isolated stop rule prevents the parent-gated reports. B4C3TAR2 remains the
last selected positive boundary and B4C3TR remains the exact negative control.

## Cadence hypothesis result

All six macro-publication lanes finish. Every private KKT interval, macro
transaction, aggregate macro ledger, legacy/policy root and forced
prepublication rollback passes. Each lane commits only 8 P1 or 16 P2 durable
frames despite executing the same 384--3,072 private substeps as B4C3TR.

Unlike per-substep publication, temporal convergence is restored:

| Case | Position ratio | Velocity ratio | Contact time | Terminal contacts |
|---|---:|---:|---|---|
| P1 | `2.01665` | `2.11554` | exact at all levels | exact at all levels |
| P2 | `1.99217` | `2.16788` | exact at all levels | exact at all levels |

Both fields are classified `OBSERVED_FIRST_ORDER`; no representation-floor
fallback is needed. P2 passes every same-level binary tube and physical gate.
Publication pressure/mechanical budget utilization stays below `0.002` in all
P2 lanes and below `0.0005` in P1.

This directly supports the architectural hypothesis: durable publication
frequency, not the KKT formula or timestep integrator, caused B4C3TR's loss of
refinement convergence and contact-phase drift.

## Sole blocking gate

P1 fixed-192 is the only failed lane gate. At final frame seven:

```text
same-level velocity RMS error   2.8306940518501081e-4 m/s
frozen 32*P*q bound             2.5600000000000000e-4 m/s
bound utilization               1.1057398640039486
```

Position uses only `0.0555` of its bound, first-contact time is exact, all 64
terminal contact features match, density/speed/energy/KKT/ledger gates pass,
and the lane participates in clean first-order convergence. This is not the
B4C3TR failure pattern.

## Interpretation

The reused velocity envelope assumes representation errors add directly as
`32*P*q`. With macro publication, a position perturbation at one boundary also
changes later pressure/contact acceleration before the next publication. That
state-transition gain is absent from the formula. The 10.6% exceedance is
therefore evidence that the envelope is incomplete, not authority to increase
its coefficient after measurement.

## Decision

Preserve B4C3P as FAIL and do not silently select macro cadence yet. The next
allowed research is an independent stability/error-budget discriminator that:

- separates direct publication error from propagated macro-map error;
- evaluates fine-reference contamination relative to the independently
  convergent binary temporal error;
- freezes an a priori acceptance rule before replaying B4C3P;
- retains exact event/contact, physical, transaction and ledger gates.

B4C3TC, adaptive redesign, nominal, CUDA, runtime/schema and production remain
blocked.

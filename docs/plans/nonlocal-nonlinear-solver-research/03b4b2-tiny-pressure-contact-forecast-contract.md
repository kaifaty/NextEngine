# NSR3-B4B2 -- tiny pressure corpus with contact-onset forecast

Status: `PASS / TINY_PRESSURE_CONTACT_FORECAST_CANDIDATE / B4C_DESIGN_AUTHORIZED`

Parent B4BF selects `CONTACT_ONSET_SPECTRAL_FORECAST_CANDIDATE`; semantic
SHA-256 is `c6d53131786bcfacdae1bbb1a4ee2076846019d037b62b167b2a76b4d28e146f`
and JSON-without-final-LF SHA-256 must equal
`dfd6b39d8c12da4e494ff2ed4de5590d7a08b2be598c2d411c076ad6f8f1b550`.
B4B1 remains exact FAIL.

## Identity and sole controller repair

```text
tiny-pressure-water-corpus-r2-contact-forecast
```

Reuse the complete B4B1 KKT substep, P1/P2 fixtures, fixed references,
physical/accuracy/work thresholds and embedded immutable-level transaction.
Only initial substep selection changes:

1. If the frame-start pressure state is active, retain the existing 48-HVP
   start-state spectrum and target `0.15`.
2. If it is inactive, build the uncommitted feasible clamped macro predictor.
3. If that predictor is pressure inactive, select `n=1` with no spectral HVP.
4. If it is active, run the unchanged 48-HVP spectrum there and select
   `ceil(Hf*sqrt(lambda_max/M)/0.15)`.

Report the spectrum source (`START_ACTIVE`, `FORECAST_ACTIVE` or
`INACTIVE_EXACT`) and charge every forecast evaluation/HVP. The forecast may
not mutate frame state, own impulse or serve as an accuracy oracle.

## Inherited execution

At every frame execute `(n,2n)` and at most two further doublings from the
immutable start. Commit the fine state of the first passing embedded pair.
All B4B1 fixed `48/96/192`, per-frame fixed-192 comparisons, contact/KKT
ledger, physical, energy and capacity gates remain literal.

P2 precontact recurrence remains exact. The forecast may become active only
on a frame whose macro predictor reaches contact-created pressure; this does
not itself count as contact. First physical contact remains the first positive
KKT multiplier in an executed accepted interval.

## Repeatability and exit

The first failed fixture/gate stops the corpus. Two reports must be
byte-identical. B4BF, B4B1, B4BK1, B4BK r0, B4B r0, B4A, B3R, D5, original B3
and B2 raw outputs remain exact.

PASS selects `TINY_PRESSURE_CONTACT_FORECAST_CANDIDATE` and authorizes only
B4C joint fluid/support neighborhood plus canonical-runner design. FAIL
preserves the isolated KKT and forecast results and blocks nominal work.

No general mesh, moving solid, friction, equilibrium, internal aperture,
viscosity, surface tension, nominal water, CUDA, runtime or production
integration is authorized.

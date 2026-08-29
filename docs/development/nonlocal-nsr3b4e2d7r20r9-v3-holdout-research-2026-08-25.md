# NSR3-B4E2D7R20R9 v3 blind holdout research

Status: `RESEARCH COMPLETE / SOURCE-ONLY MANIFEST SELECTED`.

## Why a new corpus is mandatory

The R8 solver now certifies every v2 case, but the filled edge/corner ceased to
be blind when they selected global ADMM, projector derivatives, cone-aware
NNQP, relinearization and verified inverse handling. Reusing them would only
measure development fit.

The next evidence must freeze new sources before any sparse operator,
projection, excitation metric or solver result is observed.

## Selected holdout families

Use four closed-box filled lattices with new particle counts, orientations and
velocity fields:

1. `6x3x4` lower-x/upper-z edge with coupled lane shear;
2. `5x5x3` lower-x/lower-y/upper-z corner with asymmetric impact;
3. `7x2x5` upper-y shear layer with longitudinal counter-shear;
4. `5x3x4` radially compressive flow plus weak translation.

The first three stress contact-orientation transfer; the fourth excites a
different continuum mode rather than copying a wall-impact velocity template.
Their total sample count is 277, distinct from every v2 fixture.

## Evidence firewall

R20R9 constructs only positions, velocities, boundaries, ownership metadata
and source hashes. It must not call contact-domain projection, sparse operator
construction, density action, R8 code, KKT code or timing. If a later input-only
operator preflight finds a quiet holdout, that is recorded as a v3 admission
failure; the frozen source is not edited after observation.

The eventual generalization run must use the exact R8 algorithm and constants.
No new thresholds, iteration caps or inverse policy may be selected from v3.

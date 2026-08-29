# NSR3-B4E2D7R20R33 v4 blind holdout research

Status: `RESEARCH COMPLETE / SOURCE-ONLY V4 MANIFEST SELECTED`.

## Why another blind corpus is required

R32 certifies all 12 cases, but every v2/v3 trajectory has now influenced
solver design: ratio refinement, event crossing, exhausted-line recovery and
terminal precedence were selected from their failures. Replaying those cases
can prove non-regression, not generalization.

## Selected source families

Freeze five new closed-box filled lattices before any operator observation:

1. `8x3x3` upper-x/lower-z impact with torsional cross-flow;
2. `4x6x4` upper-x/lower-y/lower-z asymmetric corner impact;
3. `6x4x3` biaxial counterflow with upper-z transport;
4. `4x4x5` anisotropic helical compression;
5. `3x7x4` upper-y transport with alternating transverse layers.

They add new dimensions, face orientations, affine/non-affine velocity fields
and 404 total samples. Equal particle counts in the first and third cases are
intentional controls over different topology dimensions and velocity modes.

## Evidence firewall

R33 may construct only source positions, velocities, closed-box ownership and
hashes. It must report zero operator, projection, preflight, solver and timing
work. The exact velocity equations are frozen in the companion contract.

After the manifest is committed, an input-only R34 may construct operators and
test excitation. A quiet or invalid source is a recorded admission failure;
the R33 sources must not be edited after observing it. Only a passing R34 may
authorize one unchanged R32-policy generalization run.

## Ceiling

Manifest success proves only immutable, finite, distinct source construction.
It provides no solver/generalization, physical-model, runtime, GPU,
performance or production evidence.

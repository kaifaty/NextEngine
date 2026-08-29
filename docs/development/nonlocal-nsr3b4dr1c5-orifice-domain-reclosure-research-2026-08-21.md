# NSR3-B4DR1C5 Orifice domain reclosure research -- 2026-08-21

Status: `COMPLETE / DOMAIN_BOUNDARY_SEPARATION_SELECTED / NO_RERUN_YET`

## Question

How should the adapter represent the intentionally asymmetric Orifice setup
without conflating its analytical world extent with its Akinci source support?

## Finding

The frozen Orifice manifest contains two different extents:

- world/contact box: `[0,2] x [0,1] x [0,1]`;
- Akinci particles: two support layers around only the `[0,1]` source chamber,
  minus the safe opening.

The implementation stored only `boundary_nx=20` and derived `x_max` from it.
That derivation is valid for Hydro and Dam because their boundary support
surrounds the full domain, but invalid for Orifice by construction. Extending
the Akinci lattice to x=2 would also be wrong: it would add receiver-side
support excluded by the frozen comparator design.

## Selection

Give each scenario an explicit analytical `domain_x_max` independent from
`boundary_nx`. Freeze `1.0/20`, `4.0/80` and `2.0/20` for Hydro, Dam and
Orifice. Keep all existing boundary positions and roots unchanged. Add a
manifest-only assertion so this ownership mismatch fails before Simulation in
future revisions.

Because Orifice contact and the global payload profile identity change, issue
a new R1C5 identity and rerun all three pairs. Do not inherit the otherwise
valid R1C4 Hydro/Dam payloads.

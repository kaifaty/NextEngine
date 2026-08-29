# NSR3-B4E2D7R19R48 v1 restoration-certificate diagnostic

Date: `2026-08-25`

Status: `INVALID GEOMETRY ASSUMPTION / NO SOLVER CREDIT`.

V1 reproduced exact R47 bytes and every immutable source/workspace fact, then
stopped at `RESTORATION_CERTIFICATE_GEOMETRY_REJECTED` before one JVP, VJP,
projection or PDAL iteration. The contract assumed that the source-anchored
dimensionless half-skin radius was `0.3`; exact execution reported
`0.059999999999940004`. Thus `normal_radius=0.125` could not be owned by the
frozen R42 superset.

This is a specification error, not evidence against PDAL or next-TRQP
compatibility. The failed result is retained with semantic SHA
`42dc9935b30a75cd17b10b287bbdce959f76587d6fb5f8efc0cd0b4b424274d6`.
No scientific classification or performance credit is assigned.

V2 changes only the independently derived geometry: next radius `1/16` and
half-reserve normal radius `1/32`. Their sum with the exact R43 displacement is
strictly below the inherited half-skin. No nominal solver outcome is used to
choose them.

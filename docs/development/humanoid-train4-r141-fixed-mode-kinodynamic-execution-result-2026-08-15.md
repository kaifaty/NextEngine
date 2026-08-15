# TRAIN-4 R141 fixed-mode kinodynamic execution result — 2026-08-15

| Field | Value |
| --- | --- |
| Scope | Sole clean bounded R141 execution under the frozen R139/R140 method and resource envelope |
| Status | `INVALID` |
| Termination | `SOURCE_OR_RECONSTRUCTION_INVALID` |
| Invalid reason | `R120_ACCEPTED_ARRAY_HASH_CLOSURE_DIFFERS` |
| Gate decision | `STOP_INVALID_R141_WITHOUT_RETRY` |
| Transition | `R141_INVALID_STOP_WITHOUT_RETRY` |
| Claim ceiling | Source-closure failure only; kinodynamic feasibility was not evaluated |

## Immutable result

The sole R141 process ran once from clean commit
`75661a17bc854fd0deb082f5ea731ba726197fbb` with seed zero, one process and
one observed thread. It returned `INVALID` after `0.18046876399967005` seconds
and used at most `262459392` resident bytes. There was no restart, resume,
alternative warm start or manual intervention.

Canonical report/file/profile SHA-256 is
`6933fea7842011f85ff74c0cb2a341e6a91dbd174b4273ca04543c1056d40d86` /
`2a42c1f4cd3a42992d83da4552d7fe1c44a17e8188ad043924734c67c2c020bd` /
`c7df8cef7dac766ea6b5ea93f1466b52dab8318f5819fcfebea741d09851c5e9`.
The canonical hash was independently reproduced exactly. Execution-module and
tool SHA-256 is
`95af0c078c1025465459ca1c0e108dac0668365bad232a477f1ad9abfbf9d15f` /
`4f651fb54e921789a85782ffcaf10d3f1710a0a856c425b21dbe3a597e526de9`.

The external immutable report remains at
`NextEngine-training/generations/humanoid-motor-rebuild-v1/evaluations/TRAIN-4/fixed-mode-kinodynamic-execution-r141/fixed-mode-kinodynamic-execution.json`.
No external report or solver payload is copied into Git.

## Exact stop boundary

R141 reproduced the frozen metadata inventory before it reached any real
kinodynamic work:

- fixed mode sequence SHA-256
  `ab30a912550e06e1219b1fab87e0d93044eba6b0cdf8e7377f7b27a0e1915ca4`;
- graph index SHA-256
  `844ed3a2e727712fedff2c0cd858c30639c8956e054f79f2012d67277535639b`;
- transition replay SHA-256
  `9f289c31ecfe2ef18b4beed4559982c3da25f7244eb6b17c022ac6000edc322a`;
- `4956` force rows, `18` impulse rows and `20` anchor rows.

The next source-closure check reconstructed the six accepted R120 arrays from
the frozen solver-private cache. Five hashes matched exactly: root position,
joint position, root linear velocity, joint velocity and root-yaw velocity.
Only `root_quaternion_q1_30` differed:

| R120 quaternion identity | SHA-256 |
| --- | --- |
| Accepted emitted Q1.30 bytes required by the R120 report | `9b8a83dd33774d0e759237ec7b51997e7404b32d08ee0538ca0bba4abcafb92d` |
| Q1.30 bytes reconstructed from the frozen R120 cache | `501e84bc7b92cd1e4df99e38a9b43ea7c2db36408428ccd5ec9be78896f647a5` |

The failure is representational, not a measured dynamics or cone result. At
R120 acceptance, `reanchor_state` decodes the emitted quantized quaternion to a
rotation matrix and stores only `reference_root_rotation` plus
`root_orientation_delta_rad`. The solver-private cache contains those values,
root/joint positions, velocity and acceleration, but not the accepted Q1.30
quaternion array. Reconstructing a rotation and encoding it back to Q1.30 is a
lossy, non-byte-invertible round trip. An independent diagnostic using both the
R120 rotation exponential and the current frozen kernel produced the same
observed hash, excluding the exponential implementation as the discrepancy.

The frozen pre-solve identity contract required exact byte closure and allowed
no substitute witness or tolerance. R141 therefore failed closed at the right
boundary. It does not prove feasible, infeasible or numerically stopped
kinodynamic optimization; that solve was never reached.

## Counter and validation closure

All real graph/controller/dynamics/Jacobian/residual/system-assembly counters,
QP setups, factorizations, QP solves, exact trial audits, kinodynamic solves,
optimizer steps, cache/candidate construction, PhysX scene runs and training
runs are zero. No accepted anchor or final exact audit exists.

All six execution validations pass: Ruff check/format, `11/11` focused R141
tests, `414/414` complete lab tests, `56/56` motor tests and host-check. The
descriptor, all tracked sources and both external source files also passed
their frozen identity checks. These validations establish implementation and
report integrity only; they do not lift the invalid source-closure result.

## Decision

The sole R141 authority is consumed. The frozen outcome permits no R141 retry,
R142 decision, altered cache, alternative quaternion witness, contact-semantics
change, candidate publication, PhysX, all-17/V19, corpus admission, visual or
exhaustive gate, optimizer or training. R142 was conditional on exact R141
feasible `PASS`, which did not occur.

Any future attempt must start as a separately authorized, independently
hash-closed lineage whose source artifact explicitly preserves the accepted
orientation representation it requires. It cannot be presented as a retry or
continuation of R141.

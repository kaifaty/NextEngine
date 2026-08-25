# NSR3-B4E2D7R19R58 directed-enclosure fixed-point research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / CONTRACT NEXT / REPORT ONLY`.

## Evidence boundary

R57 changes the problem from raw-boundary projection to certificate-aware
projection. Its selected cycle-64 witness has:

```text
raw positive                         0
full binary128 positive              0
full binary128 negative           6000
full binary128 unresolved            0
binary64 directed positive         245
maximum raw              -5.1986625e-22
maximum directed upper    6.3619021e-24
inner model maximum       6.1443737e-28
```

The inner model residual is four orders below the remaining directed upper, so
more cycles within the same R57 subproblem are not the indicated remedy. The
remaining discrepancy is the change in the conservative binary64 enclosure
when the witness moves.

## Fixed-point formulation

Define `U(w)` as the unchanged all-row directed binary64 upper certificate at
witness `w`. One certificate-aware refinement computes

```text
delta(w) = argmin 0.5 ||delta||^2
           subject to U_i(w) + A_i delta <= 0  on the master
                      contact box relative to w

T(w) = w + delta(w)
```

R57 evaluated `T(w55)`. R58 evaluates exactly one more composition
`T(T(w55))`, using the exact retained R57 witness and its exact `U` vector.
This is not an extension of the first inner Dykstra solve: it updates the
certificate source once, then starts a new zero-initialized projection problem.

The map is motivated by iterative refinement: each solve removes the complete
currently proved upper endpoint. R57 reduced the endpoint by `6511.43x` and
the active count from 476 to 245, which is strong empirical contraction. One
additional outer is therefore a bounded discriminator. An open-ended loop is
not authorized.

## Alternatives

### More inner cycles

Rejected. R57's terminal model maximum is already `6.14e-28`; the remaining
certificate upper is `6.36e-24`. Inner depth is no longer the dominant layer.

### Tighten gamma from the binary128 oracle

Rejected for R58. Binary128 proves that the private witness is safe, but the
runtime certificate identity remains binary64. Replacing its forward bound
requires a separate proof and correspondence contract.

### SHQP or extreme-point-corrected semismooth Newton

Retained as the next solver fallback if the fixed-point step leaves a true raw
violation or stalls. Pang's supporting-halfspace QP acceleration and the 2026
degenerate-polyhedral semismooth Newton construction remain the relevant
primary directions
([SHQP](https://arxiv.org/abs/1601.01174),
[degenerate projection SSN](https://arxiv.org/abs/2607.12551)). They are not
needed to test the simpler enclosure-map hypothesis.

## Selected experiment

R58 must replay exact R57 and retain its selected witness, terminal audit and
all transitive R56/R55 roots. It then runs the existing R57 grouped-box core
once with:

- source witness = exact R57 selected cycle-64 witness;
- source halfspaces = exact R57 terminal directed upper;
- zero `delta`, density multipliers and box correction;
- unchanged 494-row master and stable topology;
- unchanged contact box and normal-ball audit;
- exactly 64 cycles and checkpoints 8/16/32/64;
- first certified checkpoint, otherwise cycle 64;
- one selected-witness binary64/binary128 sign decomposition;
- exact rollback.

New work is again fixed at 69 pair passes plus one quad row traversal. Reusing
the existing refinement core is required: R58 changes only the source witness
and upper vector, not the solver algorithm.

## Frozen outcome classes

After structural gates:

1. outside-master positive -> master expansion required;
2. normal-ball violation -> ball integration required;
3. certified selected checkpoint -> restoration-next/TRQP-compatible candidate;
4. full-binary128 raw positive -> raw regression / stronger solver required;
5. unresolved full-binary128 sign -> sign oracle unresolved;
6. remaining directed positives without strict active-count and maximum-upper
   contraction versus R57 -> enclosure fixed-point stalled;
7. remaining directed positives with both measures strictly contracted ->
   enclosure fixed-point contraction candidate;
8. otherwise retain the reference.

No tolerance or ratio threshold is used. Contraction is the exact conjunction
`active_new < 245` and `maximum_upper_new < 6.3619020626182659e-24`.

## Recommendation

Freeze and implement exactly one R58 fixed-point update. If it certifies, stop
numerical refinement and prove the restoration/TRQP transaction boundary. If
it contracts but remains positive, do not automatically add R59 depth: compare
the remaining scale with a separately proved tighter binary64 enclosure and
with SSN/SHQP. If it stalls or regresses, select the SSN/SHQP research branch.

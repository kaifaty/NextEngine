"""Free-body FEM ports on an explicit approximate internet mesh; not pressure.

Same mesh, P1/P2 discretization control. E/rho/nu and Rayleigh parameters from
OF2 ceramic; a declared 1 mN.s impulse, not a learned striker/contact profile.
"""

from __future__ import annotations

import argparse
import json
import time
from pathlib import Path

import numpy as np
import physical_sound_objectfolder2 as source
from scipy.io import wavfile
from scipy.sparse import save_npz
from scipy.sparse.linalg import eigsh


def boundary(nodes, elements):
    faces = np.concatenate(
        [elements[:, f] for f in ([0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3])]
    )
    parents = np.tile(np.arange(len(elements)), 4)
    _, indices, counts = np.unique(
        np.sort(faces, axis=1), axis=0, return_index=True, return_counts=True
    )
    if np.any(counts > 2):
        raise ValueError("nonmanifold volume faces")
    indices = indices[counts == 1]
    faces, parents = faces[indices], parents[indices]
    p = nodes[faces]
    normal = np.cross(p[:, 1] - p[:, 0], p[:, 2] - p[:, 0])
    outward = p.mean(1) - nodes[elements[parents]].mean(1)
    flip = np.einsum("ij,ij->i", normal, outward) < 0
    faces[flip] = faces[flip][:, [0, 2, 1]]
    normal[flip] *= -1
    normals = np.zeros_like(nodes)
    for axis in range(3):
        np.add.at(normals, faces[:, axis], normal)
    ids = np.unique(faces)
    lengths = np.linalg.norm(normals[ids], axis=1)
    if np.any(lengths == 0):
        raise ValueError("zero boundary normal")
    normals[ids] /= lengths[:, None]
    return faces, ids, normals


def port_response(omega, ports, impulse=0.001):
    damping = 0.5 * (6 + 1e-7 * omega**2)
    damped = np.sqrt(omega**2 - damping**2)
    mask = np.isfinite(damped) & (damped > 0) & (damped < 2 * np.pi * 22049)
    if not mask.any():
        raise ValueError("no audible oscillatory modes")
    # Physical input/output residue is invariant to consistent mode sign flips.
    residues = ports[:-1] * ports[-1]
    waves = []
    for r in residues:
        gains = np.vstack([r[mask] / damped[mask], np.zeros((2, mask.sum()))])
        waves.append(
            source.waveform(
                gains, damped[mask] / (2 * np.pi), damping[mask], [impulse, 0, 0]
            )
        )
    return np.array(waves), damping, mask, residues


def solve(args):
    from skfem import (
        Basis,
        BilinearForm,
        ElementTetP1,
        ElementTetP2,
        ElementVector,
        MeshTet,
        asm,
    )
    from skfem.helpers import dot
    from skfem.models.elasticity import lame_parameters, linear_elasticity

    started = time.monotonic()
    report = json.loads((args.mesh / "mesh.json").read_text())
    if source.sha(args.mesh / "mesh.npz") != report["mesh_sha256"]:
        raise ValueError("volume mesh changed")
    with np.load(args.mesh / "mesh.npz", allow_pickle=False) as d:
        nodes, elements = d["nodes"], d["elements"]
    if (
        not np.isfinite(nodes).all()
        or elements.min() < 0
        or elements.max() >= len(nodes)
    ):
        raise ValueError("invalid volume mesh")
    center = (nodes.min(0) + nodes.max(0)) / 2
    length = float(np.ptp(nodes, axis=0).max())
    scaled = (nodes - center) / length
    mesh = MeshTet(scaled.T, elements.T)
    if not np.array_equal(mesh.p.T, scaled):
        raise ValueError("mesh reordered native nodes")
    basis = Basis(
        mesh, ElementVector(ElementTetP1() if args.order == 1 else ElementTetP2())
    )
    if basis.N > args.max_dofs:
        raise ValueError("bounded FEM degrees of freedom exceeded")
    print("assembly", {"order": args.order, "dofs": basis.N}, flush=True)
    lam, mu = lame_parameters(1.0, 0.19)
    k = asm(linear_elasticity(lam, mu), basis).tocsr()
    m = asm(BilinearForm(lambda u, v, w: dot(u, v)), basis).tocsr()
    print(
        "eigensolve", {"nnz": k.nnz, "seconds": time.monotonic() - started}, flush=True
    )
    # Negative shift avoids factorizing singular free-body K. All six rigid
    # modes are retained in the solve and checked, not artificially clamped.
    values, vectors = eigsh(
        k, k=38, M=m, sigma=-1e-4, which="LM", tol=1e-9, v0=np.linspace(1, 2, basis.N)
    )
    order = np.argsort(values)
    values, vectors = values[order], vectors[:, order]
    if np.any(values[6:] <= 0) or np.max(np.abs(values[:6])) > values[6] * 1e-6:
        raise ValueError("six rigid modes followed by positive elastic modes required")
    kv, mv = k @ vectors[:, 6:], m @ vectors[:, 6:]
    residual = np.linalg.norm(kv - mv * values[6:], axis=0) / (
        np.linalg.norm(kv, axis=0) + np.linalg.norm(mv * values[6:], axis=0)
    )
    orthogonal = float(np.max(np.abs(vectors.T @ (m @ vectors) - np.eye(38))))
    if not np.isfinite(residual).all() or residual.max() > 1e-7 or orthogonal > 1e-7:
        raise ValueError("elastic residual or mass orthogonality failed")
    faces, ids, normals = boundary(nodes, elements)
    sorted_ids = ids[np.lexsort(nodes[ids].T[::-1])]
    contacts = sorted_ids[np.linspace(0, len(ids) - 1, 3).astype(int)]
    probe = ids[np.argmax(np.linalg.norm(nodes[ids] - nodes[contacts[0]], axis=1))]
    selected = np.r_[contacts, probe]
    directions = normals[selected].copy()
    directions[:-1] *= -1
    sampled = vectors[basis.nodal_dofs[:, selected], 6:] / np.sqrt(2700 * length**3)
    ports = np.einsum("cpm,pc->pm", sampled, directions)
    omega = np.sqrt(values[6:]) * np.sqrt(7.2e10 / 2700) / length
    waves, damping, audible, residues = port_response(omega, ports)
    signs = np.where(np.arange(len(omega)) % 2, -1.0, 1.0)
    if not np.array_equal(port_response(omega, ports * signs)[0], waves):
        raise ValueError("physical port sign invariance failed")
    if np.any(port_response(omega, ports, 0)[0] != 0):
        raise ValueError("zero impulse response failed")
    args.output.mkdir(parents=True)
    save_npz(args.output / "stiffness.npz", k)
    save_npz(args.output / "mass.npz", m)
    np.savez(
        args.output / "modes.npz",
        eigenvalues=values,
        eigenvectors=vectors,
        omega=omega,
        damping=damping,
        ports=ports,
        residues=residues,
        self_residues=ports**2,
        audible=audible,
        contact_node_ids=selected,
        contact_positions=nodes[selected],
        contact_directions=directions,
        center=center,
        length=length,
        boundary_faces=faces,
    )
    np.save(args.output / "raw-waves.npy", waves)
    gain = min(1.0, 0.5 / abs(waves).max())
    for i, w in enumerate(waves):
        wavfile.write(
            args.output / f"contact-{i}.wav", source.RATE, (w * gain).astype(np.float32)
        )
    wavfile.write(
        args.output / "three-contacts.wav",
        source.RATE,
        (waves.ravel() * gain).astype(np.float32),
    )
    result = {
        "mesh_sha256": report["mesh_sha256"],
        "order": args.order,
        "dofs": int(basis.N),
        "rigid_eigenvalues": values[:6].tolist(),
        "max_elastic_residual": float(residual.max()),
        "mass_orthogonality_error": orthogonal,
        "undamped_hz": (omega / (2 * np.pi)).tolist(),
        "audible_modes": int(audible.sum()),
        "modes": 32,
        "common_gain": float(gain),
        "material": {
            "young": 7.2e10,
            "density": 2700.0,
            "poisson": 0.19,
            "alpha": 6.0,
            "beta": 1e-7,
        },
        "seconds": time.monotonic() - started,
        "script_sha256": source.sha(Path(__file__)),
        "sign_flip_exact": True,
        "zero_impulse_exact": True,
        "scope": "free-body non-rigid displacement at explicit output port, 1mN.s input impulse; approximate mesh, only32elastic modes; not neural, not pressure/realism or full-band convergence",
    }
    (args.output / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2), flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mesh", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--order", type=int, choices=(1, 2), required=True)
    parser.add_argument("--max-dofs", type=int, default=200000)
    args = parser.parse_args()
    if not 1 <= args.max_dofs <= 500000:
        raise ValueError("DOF budget must be in [1,500000]")
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("new external output required")
    solve(args)

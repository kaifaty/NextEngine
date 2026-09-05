#!/usr/bin/env python3
"""Scene look L5b (plan look/05b): a synthetic glTF 2.0 asset for the
reference scene, a riveted steel water tank, written with its `.bin` and
three 512 x 512 PNG maps (albedo sRGB, glTF metallic-roughness linear,
tangent-space normal linear). Deterministic; standard library only. Run
from the repository root:

    python3 tools/gltf/generate.py

The model exercises the importer's paths: a smooth cylinder body, a domed
lid as a child node under a rotation, one leg mesh instanced by four nodes
under translations, a pipe with an elbow, and a painted label on the +x
side whose text reads upright when the UV orientation is right."""
import json, math, os, struct, sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'textures'))
from generate import write_png, value_noise, clamp8, srgb, normal_from_height  # noqa: E402

SIZE = 512
OUT = os.path.join('projects', 'reference-alpha', 'assets', 'models')
NAME = 'water_tank'

# ---------------------------------------------------------------- geometry

class Mesh:
    def __init__(self, name):
        self.name = name
        self.positions = []
        self.normals = []
        self.uvs = []
        self.indices = []

    def vertex(self, p, n, uv):
        self.positions.append(p)
        length = math.sqrt(sum(c * c for c in n))
        self.normals.append(tuple(c / length for c in n))
        self.uvs.append(uv)
        return len(self.positions) - 1

    def triangle(self, a, b, c):
        self.indices.extend((a, b, c))


def ring_u(s, segments):
    """u around the circumference, decreasing with theta so a viewer outside
    reads the map left to right; u = 0.5 at theta = 0 (the +x side)."""
    return 1.0 - s / segments


def cylinder(mesh, radius, y0, y1, segments, v0, v1, caps=True, cap_v=None):
    """A smooth cylinder open along +z... no: closed side, optional caps;
    the seam at theta = pi (the -x side)."""
    rings = []
    for (y, v) in ((y1, v0), (y0, v1)):
        ring = []
        for s in range(segments + 1):
            theta = -math.pi + 2 * math.pi * s / segments
            c, sn = math.cos(theta), math.sin(theta)
            ring.append(mesh.vertex((radius * c, y, radius * sn), (c, 0.0, sn), (ring_u(s, segments), v)))
        rings.append(ring)
    top, bottom = rings
    for s in range(segments):
        # Counter-clockwise seen from outside.
        mesh.triangle(top[s], bottom[s], bottom[s + 1])
        mesh.triangle(top[s], bottom[s + 1], top[s + 1])
    if caps:
        cv = cap_v if cap_v is not None else (v0 + v1) / 2
        for (y, ny, flip) in ((y1, 1.0, False), (y0, -1.0, True)):
            centre = mesh.vertex((0.0, y, 0.0), (0.0, ny, 0.0), (0.5, cv))
            ring = []
            for s in range(segments + 1):
                theta = -math.pi + 2 * math.pi * s / segments
                c, sn = math.cos(theta), math.sin(theta)
                ring.append(mesh.vertex((radius * c, y, radius * sn), (0.0, ny, 0.0),
                                        (0.5 + 0.08 * c, cv + 0.08 * sn)))
            for s in range(segments):
                if flip:
                    mesh.triangle(centre, ring[s], ring[s + 1])
                else:
                    mesh.triangle(centre, ring[s + 1], ring[s])


def dome(mesh, radius, height, rings, segments, v0, v1):
    """A spherical-cap dome from radius at y = 0 to the apex at y = height."""
    # Sphere centre below the base so the cap spans `radius` at y = 0.
    R = (radius * radius + height * height) / (2 * height)
    cy = height - R
    phi0 = math.asin((0 - cy) / R)
    grid = []
    for r in range(rings + 1):
        t = r / rings
        phi = phi0 + (math.pi / 2 - phi0) * t
        rr = R * math.cos(phi)
        y = cy + R * math.sin(phi)
        v = v1 + (v0 - v1) * t
        row = []
        for s in range(segments + 1):
            theta = -math.pi + 2 * math.pi * s / segments
            c, sn = math.cos(theta), math.sin(theta)
            n = (math.cos(phi) * c, math.sin(phi), math.cos(phi) * sn)
            row.append(mesh.vertex((rr * c, y, rr * sn), n, (ring_u(s, segments), v)))
        grid.append(row)
    for r in range(rings):
        for s in range(segments):
            a, b = grid[r][s], grid[r][s + 1]
            c, d = grid[r + 1][s], grid[r + 1][s + 1]
            mesh.triangle(a, b, d)
            mesh.triangle(a, d, c)


def box(mesh, minimum, maximum, uv_min, uv_max):
    (x0, y0, z0), (x1, y1, z1) = minimum, maximum
    faces = [
        ((1, 0, 0), [(x1, y0, z0), (x1, y1, z0), (x1, y1, z1), (x1, y0, z1)]),
        ((-1, 0, 0), [(x0, y0, z1), (x0, y1, z1), (x0, y1, z0), (x0, y0, z0)]),
        ((0, 1, 0), [(x0, y1, z0), (x0, y1, z1), (x1, y1, z1), (x1, y1, z0)]),
        ((0, -1, 0), [(x0, y0, z1), (x0, y0, z0), (x1, y0, z0), (x1, y0, z1)]),
        ((0, 0, 1), [(x1, y0, z1), (x1, y1, z1), (x0, y1, z1), (x0, y0, z1)]),
        ((0, 0, -1), [(x0, y0, z0), (x0, y1, z0), (x1, y1, z0), (x1, y0, z0)]),
    ]
    (u0, v0), (u1, v1) = uv_min, uv_max
    corner_uv = [(u0, v1), (u0, v0), (u1, v0), (u1, v1)]
    for normal, corners in faces:
        ids = [mesh.vertex(p, normal, uv) for p, uv in zip(corners, corner_uv)]
        mesh.triangle(ids[0], ids[2], ids[1])
        mesh.triangle(ids[0], ids[3], ids[2])


def tube(mesh, start, end, radius, segments, v0, v1):
    """A cylinder between two points, smooth, no caps."""
    ax = [e - s for s, e in zip(start, end)]
    length = math.sqrt(sum(c * c for c in ax))
    ax = [c / length for c in ax]
    helper = (0.0, 1.0, 0.0) if abs(ax[1]) < 0.9 else (1.0, 0.0, 0.0)
    side = [ax[1] * helper[2] - ax[2] * helper[1], ax[2] * helper[0] - ax[0] * helper[2],
            ax[0] * helper[1] - ax[1] * helper[0]]
    sl = math.sqrt(sum(c * c for c in side))
    side = [c / sl for c in side]
    up = [ax[1] * side[2] - ax[2] * side[1], ax[2] * side[0] - ax[0] * side[2],
          ax[0] * side[1] - ax[1] * side[0]]
    rings = []
    for (origin, v) in ((start, v0), (end, v1)):
        ring = []
        for s in range(segments + 1):
            theta = 2 * math.pi * s / segments
            c, sn = math.cos(theta), math.sin(theta)
            n = tuple(side[i] * c + up[i] * sn for i in range(3))
            p = tuple(origin[i] + radius * n[i] for i in range(3))
            ring.append(mesh.vertex(p, n, (s / segments, v)))
        rings.append(ring)
    a, b = rings
    for s in range(segments):
        mesh.triangle(a[s], a[s + 1], b[s + 1])
        mesh.triangle(a[s], b[s + 1], b[s])


def sphere(mesh, centre, radius, rings, segments, v0, v1):
    grid = []
    for r in range(rings + 1):
        phi = -math.pi / 2 + math.pi * r / rings
        row = []
        for s in range(segments + 1):
            theta = 2 * math.pi * s / segments
            n = (math.cos(phi) * math.cos(theta), math.sin(phi), math.cos(phi) * math.sin(theta))
            p = tuple(centre[i] + radius * n[i] for i in range(3))
            row.append(mesh.vertex(p, n, (s / segments, v1 + (v0 - v1) * r / rings)))
        grid.append(row)
    for r in range(rings):
        for s in range(segments):
            a, b = grid[r][s], grid[r][s + 1]
            c, d = grid[r + 1][s], grid[r + 1][s + 1]
            mesh.triangle(a, d, b)
            mesh.triangle(a, c, d)


# Texture atlas bands (v from the top of the image):
#   0.00..0.10  lid plates          0.10..0.80  body plates with the label
#   0.80..0.90  plain painted steel (legs, pipe)   0.90..1.00 cap plates
BODY_V0, BODY_V1 = 0.10, 0.80
LID_V0, LID_V1 = 0.0, 0.10
PLAIN_V0, PLAIN_V1 = 0.81, 0.89
CAP_V = 0.95


def build_meshes():
    body = Mesh('tank_body')
    cylinder(body, 0.6, 0.7, 2.1, 32, BODY_V0, BODY_V1, caps=True, cap_v=CAP_V)
    lid = Mesh('tank_lid')
    # A lip ring then the dome, in the lid's own space (its node lifts it).
    cylinder(lid, 0.66, 0.0, 0.08, 32, LID_V0, LID_V1, caps=False)
    dome(lid, 0.66, 0.30, 4, 32, LID_V0, LID_V1)
    leg = Mesh('tank_leg')
    box(leg, (-0.06, 0.0, -0.06), (0.06, 0.72, 0.06), (0.05, PLAIN_V0), (0.25, PLAIN_V1))
    box(leg, (-0.10, 0.0, -0.10), (0.10, 0.03, 0.10), (0.3, PLAIN_V0), (0.5, PLAIN_V1))
    pipe = Mesh('tank_pipe')
    tube(pipe, (0.55, 1.0, 0.0), (1.05, 1.0, 0.0), 0.06, 16, PLAIN_V0, PLAIN_V1)
    sphere(pipe, (1.05, 1.0, 0.0), 0.08, 6, 16, PLAIN_V0, PLAIN_V1)
    tube(pipe, (1.05, 1.0, 0.0), (1.05, 0.0, 0.0), 0.06, 16, PLAIN_V0, PLAIN_V1)
    # A flange where the pipe meets the body.
    tube(pipe, (0.58, 1.0, 0.0), (0.66, 1.0, 0.0), 0.10, 16, PLAIN_V0, PLAIN_V1)
    return [body, lid, leg, pipe]


def write_gltf(meshes):
    blob = bytearray()
    buffer_views = []
    accessors = []
    gltf_meshes = []

    def view(data, target, stride=None):
        while len(blob) % 4:
            blob.append(0)
        offset = len(blob)
        blob.extend(data)
        entry = {'buffer': 0, 'byteOffset': offset, 'byteLength': len(data), 'target': target}
        if stride:
            entry['byteStride'] = stride
        buffer_views.append(entry)
        return len(buffer_views) - 1

    for mesh in meshes:
        pos = b''.join(struct.pack('<fff', *p) for p in mesh.positions)
        nrm = b''.join(struct.pack('<fff', *n) for n in mesh.normals)
        uv = b''.join(struct.pack('<ff', *t) for t in mesh.uvs)
        idx = b''.join(struct.pack('<H', i) for i in mesh.indices)
        mins = [min(p[i] for p in mesh.positions) for i in range(3)]
        maxs = [max(p[i] for p in mesh.positions) for i in range(3)]
        count = len(mesh.positions)
        pv = view(pos, 34962)
        accessors.append({'bufferView': pv, 'componentType': 5126, 'count': count, 'type': 'VEC3',
                          'min': [round(v, 6) for v in mins], 'max': [round(v, 6) for v in maxs]})
        nv = view(nrm, 34962)
        accessors.append({'bufferView': nv, 'componentType': 5126, 'count': count, 'type': 'VEC3'})
        tv = view(uv, 34962)
        accessors.append({'bufferView': tv, 'componentType': 5126, 'count': count, 'type': 'VEC2'})
        iv = view(idx, 34963)
        accessors.append({'bufferView': iv, 'componentType': 5123, 'count': len(mesh.indices), 'type': 'SCALAR'})
        base = len(accessors) - 4
        gltf_meshes.append({'name': mesh.name, 'primitives': [{
            'attributes': {'POSITION': base, 'NORMAL': base + 1, 'TEXCOORD_0': base + 2},
            'indices': base + 3, 'material': 0, 'mode': 4}]})

    half_sqrt2 = math.sqrt(0.5)
    leg_nodes = []
    nodes = [
        {'name': 'tank_body', 'mesh': 0, 'children': [1]},
        # The lid sits on the body's top, turned 30 degrees about y.
        {'name': 'tank_lid', 'mesh': 1, 'translation': [0.0, 2.1, 0.0],
         'rotation': [0.0, round(math.sin(math.radians(15)), 9), 0.0, round(math.cos(math.radians(15)), 9)]},
        {'name': 'tank_pipe', 'mesh': 3},
    ]
    for k in range(4):
        theta = math.pi / 4 + k * math.pi / 2
        nodes.append({'name': f'tank_leg_{k}', 'mesh': 2,
                      'translation': [round(0.46 * math.cos(theta), 6), 0.0, round(0.46 * math.sin(theta), 6)]})
        leg_nodes.append(len(nodes) - 1)
    document = {
        'asset': {'version': '2.0', 'generator': 'tools/gltf/generate.py (Next Engine, plan look/05b)'},
        'scene': 0,
        'scenes': [{'name': 'water_tank', 'nodes': [0, 2] + leg_nodes}],
        'nodes': nodes,
        'meshes': gltf_meshes,
        'materials': [{
            'name': 'painted_steel',
            'pbrMetallicRoughness': {
                'baseColorFactor': [1.0, 1.0, 1.0, 1.0],
                'metallicFactor': 1.0,
                'roughnessFactor': 1.0,
                'baseColorTexture': {'index': 0},
                'metallicRoughnessTexture': {'index': 1},
            },
            'normalTexture': {'index': 2},
            'doubleSided': False,
        }],
        'textures': [{'source': 0, 'sampler': 0}, {'source': 1, 'sampler': 0}, {'source': 2, 'sampler': 0}],
        'images': [
            {'uri': f'{NAME}_albedo.png', 'mimeType': 'image/png'},
            {'uri': f'{NAME}_metallic_roughness.png', 'mimeType': 'image/png'},
            {'uri': f'{NAME}_normal.png', 'mimeType': 'image/png'},
        ],
        'samplers': [{'magFilter': 9729, 'minFilter': 9987, 'wrapS': 10497, 'wrapT': 10497}],
        'accessors': accessors,
        'bufferViews': buffer_views,
        'buffers': [{'byteLength': len(blob), 'uri': f'{NAME}.bin'}],
    }
    with open(os.path.join(OUT, f'{NAME}.bin'), 'wb') as f:
        f.write(bytes(blob))
    with open(os.path.join(OUT, f'{NAME}.gltf'), 'w') as f:
        json.dump(document, f, indent=2)
        f.write('\n')
    return sum(len(m.positions) for m in meshes), sum(len(m.indices) for m in meshes) // 3


# ---------------------------------------------------------------- textures

def rect(x0, y0, x1, y1):
    return (int(x0 * SIZE), int(y0 * SIZE), int(x1 * SIZE), int(y1 * SIZE))


# Block letters "H2O" and an up arrow in a 5-column x 7-row cell grid.
GLYPHS = {
    'H': ["#...#", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"],
    '2': [".###.", "#...#", "....#", "...#.", "..#..", ".#...", "#####"],
    'O': [".###.", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
    '^': ["..#..", ".###.", "#.#.#", "..#..", "..#..", "..#..", "..#.."],
}


def paint_label(mask, x0, y0, cell):
    """Writes H2O and an arrow into `mask` (1 = ink) at pixel origin."""
    for gi, glyph in enumerate('H2O'):
        rows = GLYPHS[glyph]
        for r, row in enumerate(rows):
            for c, ch in enumerate(row):
                if ch == '#':
                    px = x0 + gi * 6 * cell + c * cell
                    py = y0 + r * cell
                    for yy in range(py, py + cell):
                        for xx in range(px, px + cell):
                            mask[yy * SIZE + xx] = 1
    rows = GLYPHS['^']
    ax = x0 + 3 * 6 * cell + cell
    for r, row in enumerate(rows):
        for c, ch in enumerate(row):
            if ch == '#':
                px = ax + c * cell
                py = y0 + r * cell
                for yy in range(py, py + cell):
                    for xx in range(px, px + cell):
                        mask[yy * SIZE + xx] = 1


def textures():
    size = SIZE
    grain = value_noise(51, size, octaves=4, base_period=48.0)
    rust = value_noise(52, size, octaves=5, base_period=96.0)
    dents = value_noise(53, size, octaves=3, base_period=160.0)
    albedo = bytearray(size * size * 4)
    mr = bytearray(size * size * 4)
    height = [0.0] * (size * size)
    label_mask = [0] * (size * size)
    # The label: a white plate on the +x side (u 0.5), in the body band.
    lx0, ly0, lx1, ly1 = rect(0.30, 0.36, 0.70, 0.56)
    cell = 8
    paint_label(label_mask, lx0 + 10, ly0 + 22, cell)
    body_top, body_bottom = int(BODY_V0 * size), int(BODY_V1 * size)
    seam_rows = [body_top + int((body_bottom - body_top) * k / 3) for k in range(1, 3)]
    rivet_rows = [row + 10 for row in seam_rows] + [row - 10 for row in seam_rows] + [body_top + 8, body_bottom - 8]
    lid_top, lid_bottom = int(LID_V0 * size), int(LID_V1 * size)
    for y in range(size):
        for x in range(size):
            i = y * size + x
            paint = (0.30, 0.36, 0.34)  # dull green-grey paint, linear
            h = dents[i] * 0.15 + grain[i] * 0.04
            rough = 0.55 + grain[i] * 0.15
            metal = 0.0
            in_body = body_top <= y < body_bottom
            in_lid = lid_top <= y < lid_bottom
            # Plate seams: a groove every third of the band, and radial
            # seams on the lid.
            seam = False
            if in_body:
                for row in seam_rows:
                    if abs(y - row) <= 3:
                        seam = True
                if x % 128 < 4:
                    seam = True
            if in_lid:
                if x % 64 < 3:
                    seam = True
            if seam:
                h -= 0.5
                rough += 0.2
                paint = tuple(c * 0.6 for c in paint)
            # Rivets: domes along the seam rows every 32 texels.
            rivet = 0.0
            if in_body:
                for row in rivet_rows:
                    dy = y - row
                    if abs(dy) <= 6:
                        dx = (x + 16) % 32 - 16
                        d2 = dx * dx + dy * dy
                        if d2 <= 36:
                            rivet = max(rivet, math.sqrt(max(0.0, 36 - d2)) / 6)
            if rivet > 0:
                h += rivet * 0.9
                rough = 0.45
                paint = (0.26, 0.30, 0.29)
            # Rust streaks below the seams.
            streak = 0.0
            if in_body:
                for row in seam_rows:
                    if 0 <= y - row < 60:
                        streak = max(streak, (1 - (y - row) / 60) * max(0.0, rust[i] - 0.4) * 3.0)
            if streak > 0:
                paint = tuple(paint[k] * (1 - streak) + (0.35, 0.16, 0.08)[k] * streak for k in range(3))
                rough = rough * (1 - streak) + 0.9 * streak
            # The label plate and its ink.
            if lx0 <= x < lx1 and ly0 <= y < ly1 and in_body:
                paint = (0.78, 0.76, 0.70)
                rough = 0.5
                h += 0.3
                if label_mask[i]:
                    paint = (0.04, 0.04, 0.05)
                    rough = 0.6
            # Worn metal on plate edges near the seams.
            if in_body and not seam and (x % 128 < 10) and rust[i] > 0.7:
                metal = 0.9
                rough = 0.35
                paint = (0.5, 0.5, 0.5)
            albedo[i * 4:i * 4 + 4] = bytes([srgb(paint[0]), srgb(paint[1]), srgb(paint[2]), 255])
            mr[i * 4:i * 4 + 4] = bytes([0, clamp8(rough * 255), clamp8(metal * 255), 255])
            height[i] = h
    write_png(os.path.join(OUT, f'{NAME}_albedo.png'), size, size, albedo)
    write_png(os.path.join(OUT, f'{NAME}_metallic_roughness.png'), size, size, mr)
    write_png(os.path.join(OUT, f'{NAME}_normal.png'), size, size, normal_from_height(height, size, 3.0))


if __name__ == '__main__':
    os.makedirs(OUT, exist_ok=True)
    vertices, triangles = write_gltf(build_meshes())
    textures()
    print(f'{NAME}: {vertices} vertices, {triangles} triangles, written to {OUT}')

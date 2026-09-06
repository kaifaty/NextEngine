#!/usr/bin/env python3
"""Scene look L5 (plan look/05): procedural texture sets for the reference
scene, written as 8-bit PNGs (albedo sRGB, glTF metallic-roughness linear,
tangent-space normal linear). Deterministic; no dependencies beyond the
standard library. Run from the repository root:

    python3 tools/textures/generate.py

Sets: ground (grass and dirt), concrete (the works and the dam), pad (the
relay pads' checker with a bevelled edge)."""
import math, struct, zlib, os, random

SIZE = 256
OUT = os.path.join('projects', 'reference-alpha', 'assets', 'textures')


def write_png(path, width, height, rgba):
    raw = bytearray()
    stride = width * 4
    for y in range(height):
        raw.append(0)
        raw.extend(rgba[y * stride:(y + 1) * stride])
    data = zlib.compress(bytes(raw), 9)

    def chunk(kind, body):
        c = struct.pack('>I', len(body)) + kind + body
        return c + struct.pack('>I', zlib.crc32(kind + body) & 0xffffffff)
    png = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0))
    png += chunk(b'IDAT', data) + chunk(b'IEND', b'')
    with open(path, 'wb') as f:
        f.write(png)


def value_noise(seed, size, octaves=5, base_period=64.0):
    """Tileable value noise on a size x size grid, 0..1."""
    rnd = random.Random(seed)
    field = [0.0] * (size * size)
    amplitude, total = 1.0, 0.0
    period = base_period
    for _ in range(octaves):
        cells = max(2, int(round(size / period)))
        lattice = [[rnd.random() for _ in range(cells)] for _ in range(cells)]
        for y in range(size):
            fy = y / size * cells
            iy = int(fy) % cells
            ty = fy - int(fy)
            ty = ty * ty * (3 - 2 * ty)
            row0 = lattice[iy]
            row1 = lattice[(iy + 1) % cells]
            for x in range(size):
                fx = x / size * cells
                ix = int(fx) % cells
                tx = fx - int(fx)
                tx = tx * tx * (3 - 2 * tx)
                a = row0[ix] + (row0[(ix + 1) % cells] - row0[ix]) * tx
                b = row1[ix] + (row1[(ix + 1) % cells] - row1[ix]) * tx
                field[y * size + x] += (a + (b - a) * ty) * amplitude
        total += amplitude
        amplitude *= 0.5
        period /= 2.0
    return [v / total for v in field]


def clamp8(v):
    return max(0, min(255, int(round(v))))


def srgb(v):
    v = max(0.0, min(1.0, v))
    return clamp8(255 * (v * 12.92 if v <= 0.0031308 else 1.055 * v ** (1 / 2.4) - 0.055))


def normal_from_height(height, size, strength):
    """Tangent-space normal map (x right, y up in the image's +v direction)."""
    out = bytearray(size * size * 4)
    for y in range(size):
        for x in range(size):
            hl = height[y * size + (x - 1) % size]
            hr = height[y * size + (x + 1) % size]
            hu = height[((y - 1) % size) * size + x]
            hd = height[((y + 1) % size) * size + x]
            nx = (hl - hr) * strength
            ny = (hu - hd) * strength
            nz = 1.0
            length = math.sqrt(nx * nx + ny * ny + nz * nz)
            i = (y * size + x) * 4
            out[i] = clamp8((nx / length * 0.5 + 0.5) * 255)
            out[i + 1] = clamp8((ny / length * 0.5 + 0.5) * 255)
            out[i + 2] = clamp8((nz / length * 0.5 + 0.5) * 255)
            out[i + 3] = 255
    return out


def write_set(name, albedo_linear, height, roughness, metallic, normal_strength):
    size = SIZE
    albedo = bytearray(size * size * 4)
    mr = bytearray(size * size * 4)
    for i in range(size * size):
        r, g, b = albedo_linear[i]
        albedo[i * 4:i * 4 + 4] = bytes([srgb(r), srgb(g), srgb(b), 255])
        mr[i * 4:i * 4 + 4] = bytes([0, clamp8(roughness[i] * 255), clamp8(metallic * 255), 255])
    write_png(os.path.join(OUT, f'{name}_albedo.png'), size, size, albedo)
    write_png(os.path.join(OUT, f'{name}_metallic_roughness.png'), size, size, mr)
    write_png(os.path.join(OUT, f'{name}_normal.png'), size, size, normal_from_height(height, size, normal_strength))


def ground():
    size = SIZE
    grass = value_noise(11, size, octaves=6, base_period=128.0)
    detail = value_noise(12, size, octaves=4, base_period=16.0)
    albedo, height, rough = [], [], []
    for i in range(size * size):
        patch = grass[i]
        fine = detail[i]
        dirt = (0.22, 0.16, 0.10)
        green = (0.16, 0.24, 0.09)
        t = max(0.0, min(1.0, (patch - 0.35) * 2.2))
        c = [dirt[k] + (green[k] - dirt[k]) * t for k in range(3)]
        shade = 0.75 + 0.5 * fine
        albedo.append((c[0] * shade, c[1] * shade, c[2] * shade))
        height.append(patch * 0.6 + fine * 0.4)
        rough.append(0.85 + 0.15 * (1 - t))
    write_set('ground', albedo, height, rough, 0.0, 6.0)


def concrete():
    size = SIZE
    grain = value_noise(21, size, octaves=6, base_period=32.0)
    stains = value_noise(22, size, octaves=3, base_period=256.0)
    albedo, height, rough = [], [], []
    for y in range(size):
        for x in range(size):
            i = y * size + x
            # panel seams every 128 texels
            seam = 1.0 if (x % 128 < 3 or y % 128 < 3) else 0.0
            base = 0.42 + 0.16 * (grain[i] - 0.5) - 0.10 * stains[i]
            c = base * (1 - 0.35 * seam)
            albedo.append((c * 1.0, c * 0.98, c * 0.94))
            height.append(grain[i] * 0.5 - seam * 0.6)
            rough.append(0.78 + 0.18 * grain[i])
    write_set('concrete', albedo, height, rough, 0.0, 4.0)


def pad():
    size = SIZE
    grain = value_noise(31, size, octaves=4, base_period=32.0)
    albedo, height, rough = [], [], []
    for y in range(size):
        for x in range(size):
            i = y * size + x
            cx = (x // 128) % 2
            cy = (y // 128) % 2
            light = (cx + cy) % 2 == 0
            # a bevel inside each tile
            ex = min(x % 128, 127 - x % 128)
            ey = min(y % 128, 127 - y % 128)
            edge = min(ex, ey)
            bevel = min(1.0, edge / 6.0)
            base = (0.52 if light else 0.12) + 0.06 * (grain[i] - 0.5)
            albedo.append((base, base * 0.97, base * 0.92))
            height.append(bevel * 0.8 + grain[i] * 0.1)
            rough.append(0.45 if light else 0.6)
    write_set('pad', albedo, height, rough, 0.0, 5.0)


def write_png_grey16(path, width, height, samples):
    """Scene look L6a (plan look/06a): a 16-bit grey PNG (height fields)."""
    raw = bytearray()
    for y in range(height):
        raw.append(0)
        for x in range(width):
            raw.extend(struct.pack('>H', samples[y * width + x]))
    data = zlib.compress(bytes(raw), 9)

    def chunk(kind, body):
        c = struct.pack('>I', len(body)) + kind + body
        return c + struct.pack('>I', zlib.crc32(kind + body) & 0xffffffff)
    png = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 16, 0, 0, 0, 0))
    png += chunk(b'IDAT', data) + chunk(b'IEND', b'')
    with open(path, 'wb') as f:
        f.write(png)


def dirt():
    size = SIZE
    clods = value_noise(21, size, octaves=6, base_period=96.0)
    fine = value_noise(22, size, octaves=4, base_period=12.0)
    albedo, height, rough = [], [], []
    for i in range(size * size):
        t = clods[i]
        dark = (0.16, 0.11, 0.07)
        light = (0.30, 0.22, 0.14)
        c = [dark[k] + (light[k] - dark[k]) * t for k in range(3)]
        shade = 0.8 + 0.4 * fine[i]
        albedo.append((c[0] * shade, c[1] * shade, c[2] * shade))
        height.append(t * 0.5 + fine[i] * 0.5)
        rough.append(0.9)
    write_set('dirt', albedo, height, rough, 0.0, 5.0)


def rock():
    size = SIZE
    strata = value_noise(31, size, octaves=5, base_period=160.0)
    cracks = value_noise(32, size, octaves=6, base_period=48.0)
    fine = value_noise(33, size, octaves=3, base_period=8.0)
    albedo, height, rough = [], [], []
    for i in range(size * size):
        y = (i // size) / size
        band = 0.5 + 0.5 * math.sin((y * 7.0 + strata[i] * 1.5) * 2 * math.pi)
        crack = 1.0 if cracks[i] < 0.34 else 0.0
        base = 0.30 + 0.18 * band - 0.15 * crack
        albedo.append((base * 1.0, base * 0.96, base * 0.90))
        height.append(band * 0.4 + cracks[i] * 0.4 + fine[i] * 0.2 - crack * 0.5)
        rough.append(0.7 + 0.2 * crack)
    write_set('rock', albedo, height, rough, 0.0, 7.0)


def sand():
    size = SIZE
    ripples = value_noise(41, size, octaves=3, base_period=64.0)
    grain = value_noise(42, size, octaves=5, base_period=6.0)
    albedo, height, rough = [], [], []
    for i in range(size * size):
        x = (i % size) / size
        ripple = 0.5 + 0.5 * math.sin((x * 9.0 + ripples[i] * 2.0) * 2 * math.pi)
        base = 0.52 + 0.12 * ripple + 0.06 * grain[i]
        albedo.append((base, base * 0.88, base * 0.68))
        height.append(ripple * 0.6 + grain[i] * 0.4)
        rough.append(0.95)
    write_set('sand', albedo, height, rough, 0.0, 3.0)


def terrain():
    """The height field over the floor's 60 x 60 m (0.5 m cells, 121 x 121
    samples, rows along z from -30 m): 0.1 m across |x|, |z| <= 22 m, a
    smooth rise to 2.5 m at the rim; and the 256 x 256 control map (grass,
    dirt, rock, sand weights) over the same extent."""
    columns = rows = 121
    # The flat play area is exactly the floor's 0.1 m (sample 0).
    low, high = 100_000, 2_600_000
    samples = []
    bumps = value_noise(51, 128, octaves=4, base_period=48.0)
    for row in range(rows):
        for column in range(columns):
            x = -30.0 + column * 0.5
            z = -30.0 + row * 0.5
            edge = max(abs(x), abs(z))
            t = max(0.0, min(1.0, (edge - 22.0) / 8.0))
            rise = t * t * (3 - 2 * t)
            bump = bumps[(row % 128) * 128 + (column % 128)]
            height_m = 0.1 + rise * (2.0 + 0.4 * bump)
            samples.append(int(round((height_m * 1_000_000 - low) / (high - low) * 65535)))
    write_png_grey16(os.path.join(OUT, 'terrain_height.png'), columns, rows, samples)

    size = 256
    patches = value_noise(61, size, octaves=5, base_period=64.0)
    control = bytearray(size * size * 4)
    pads = [(-4.0, -12.0), (4.0, -12.0), (0.0, -20.0)]
    for y in range(size):
        for x in range(size):
            wx = -30.0 + (x + 0.5) / size * 60.0
            wz = -30.0 + (y + 0.5) / size * 60.0
            edge = max(abs(wx), abs(wz))
            rock_w = max(0.0, min(1.0, (edge - 21.0) / 5.0))
            # Sand around the pond and along the lake's south shore.
            pond_d = max(abs(wx + 4.0) - 5.0, abs(wz - 7.25) - 2.25)
            lake_d = max(abs(wx + 10.0) - 10.0, abs(wz - 21.0) - 7.0)
            sand_w = max(0.0, min(1.0, 1.0 - max(pond_d, 0.0) / 2.5))
            sand_w = max(sand_w, max(0.0, min(1.0, 1.0 - max(lake_d, 0.0) / 2.0)))
            # Dirt on worn paths between the pads and toward the works,
            # and where the grass patches thin.
            dirt_w = 0.0
            for (px, pz) in pads:
                d = math.hypot(wx - px, wz - pz)
                dirt_w = max(dirt_w, max(0.0, 1.0 - d / 3.0))
            path = abs(wx - (-4.0 + 8.0 * max(0.0, min(1.0, (wz + 20.0) / 32.0))))
            dirt_w = max(dirt_w, max(0.0, 1.0 - path / 1.2) * 0.9)
            dirt_w = max(dirt_w, max(0.0, (0.42 - patches[y * size + x]) * 3.0))
            grass_w = 1.0
            weights = [grass_w, dirt_w, rock_w, sand_w]
            # Later layers override earlier ones.
            out = [0.0, 0.0, 0.0, 0.0]
            remaining = 1.0
            for k in (3, 2, 1, 0):
                w = min(remaining, weights[k])
                out[k] = w
                remaining -= w
            total = sum(out) or 1.0
            i = (y * size + x) * 4
            # The neutral alpha semantics admit no data alpha: the first three
            # weights ride RGB and the fourth is the remainder (alpha 255).
            control[i:i + 4] = bytes([clamp8(out[0] / total * 255), clamp8(out[1] / total * 255),
                                      clamp8(out[2] / total * 255), 255])
    write_png(os.path.join(OUT, 'terrain_control.png'), size, size, control)


if __name__ == '__main__':
    os.makedirs(OUT, exist_ok=True)
    ground()
    concrete()
    pad()
    dirt()
    rock()
    sand()
    terrain()
    print('textures written to', OUT)

"""Splits the merged LAST SERVICE world GLB into separate objects for the editor.

The production exported the whole campus as one mesh. Here it becomes many files under
`world/parts/`: each (zone, material) pair is one object (diner walls, diner floor, yard fence,
road ...), and furniture (chairs, tables) is split into single pieces so the game can place and
turn them itself. `world/parts/index.json` lists every part with its position and kind; the scene
generator (`make_last_service`) turns it into entities grouped by zone.

    python tools/split_world_glb.py [world.glb] [out_dir]
"""
import json
import os
import struct
import sys

import numpy as np

# Engine-space zones (x0, x1, z0, z1), tested in order on a piece's centre.
ZONES = [
    ('Diner', -0.4, 12.4, -15.4, 0.4),
    ('Yard', -9.0, -0.4, -20.0, 0.0),
    ('Road', -40.0, 60.0, 0.4, 14.0),
]
# Doorways of the diner (engine x, z, width, along x?). The production mesh has closed door
# leaves in them; the game has its own hinged doors there, so anything that lies wholly inside a
# doorway (leaf, handle, glass pane) is cut out, or it would stand in the way as a wall.
DOORWAYS = [
    (6.03, 0.0, 1.5, True), (6.5, -8.0, 1.2, True), (9.1, -11.0, 1.6, True), (7.0, -14.05, 1.8, False),
    (0.0, -14.0, 1.75, False), (0.0, -10.0, 1.45, False), (0.0, -6.5, 1.35, False),
]


def in_doorway(lo, hi):
    for x, z, w, along_x in DOORWAYS:
        # Low things right behind a doorway (a bin, a box) block it too.
        half_w, half_d = w * 0.5 + 0.25, (0.75 if hi[1] < 1.3 else 0.4)
        (hx, hz) = (half_w, half_d) if along_x else (half_d, half_w)
        if lo[0] >= x - hx and hi[0] <= x + hx and lo[2] >= z - hz and hi[2] <= z + hz and hi[1] <= 2.6 and lo[1] >= -0.05:
            return True
    return False


# Materials whose pieces are single movable objects (furniture).
PIECES = {'chair', 'table.001'}
# Leftovers from figures and duplicated props (see clean_world_glb.py).
DROP = {
    'Anna_atlas', 'Counter_atlas', 'Pavel_atlas',
    'clean plate', 'dirty plate', 'water', 'glass', 'glass bot', 'bread', 'Chocolate bar',
    'notebook_1', 'cardboard_box_1', 'props',
}
COMPONENT = {5120: np.int8, 5121: np.uint8, 5122: np.int16, 5123: np.uint16, 5125: np.uint32, 5126: np.float32}
WIDTH = {'SCALAR': 1, 'VEC2': 2, 'VEC3': 3, 'VEC4': 4}


def read_glb(path):
    data = open(path, 'rb').read()
    json_len = struct.unpack_from('<I', data, 12)[0]
    doc = json.loads(data[20:20 + json_len])
    rest = data[20 + json_len:]
    bin_len = struct.unpack_from('<I', rest, 0)[0]
    return doc, rest[8:8 + bin_len]


def accessor(doc, binary, index):
    a = doc['accessors'][index]
    view = doc['bufferViews'][a['bufferView']]
    dtype = COMPONENT[a['componentType']]
    width = WIDTH[a['type']]
    start = view.get('byteOffset', 0) + a.get('byteOffset', 0)
    stride = view.get('byteStride', 0)
    count = a['count']
    item = np.dtype(dtype).itemsize * width
    if stride and stride != item:
        raw = np.frombuffer(binary, dtype=np.uint8, count=stride * (count - 1) + item, offset=start)
        rows = np.lib.stride_tricks.as_strided(raw, shape=(count, item), strides=(stride, 1))
        return np.frombuffer(rows.copy().tobytes(), dtype=dtype).reshape(count, width)
    return np.frombuffer(binary, dtype=dtype, count=count * width, offset=start).reshape(count, width).copy()


def components(positions, tris):
    """Connected pieces of a triangle list (vertices welded by position)."""
    keys = np.round(positions * 1000).astype(np.int64)
    _, weld = np.unique(keys, axis=0, return_inverse=True)
    weld = weld.reshape(-1)
    parent = np.arange(weld.max() + 1)

    def find(x):
        root = x
        while parent[root] != root:
            root = parent[root]
        while parent[x] != root:
            parent[x], x = root, parent[x]
        return root

    for a, b, c in weld[tris]:
        ra, rb, rc = find(a), find(b), find(c)
        parent[rb] = ra
        parent[find(rc)] = ra
    roots = np.array([find(weld[t[0]]) for t in tris])
    groups = {}
    for i, r in enumerate(roots):
        groups.setdefault(r, []).append(i)
    return list(groups.values())


def cluster(boxes, gap=0.08):
    """Merges pieces whose bounding boxes touch (a chair's legs, seat and back)."""
    n = len(boxes)
    parent = list(range(n))

    def find(x):
        while parent[x] != x:
            parent[x] = parent[parent[x]]
            x = parent[x]
        return x

    for i in range(n):
        for j in range(i + 1, n):
            (a0, a1), (b0, b1) = boxes[i], boxes[j]
            if np.all(a0 - gap <= b1) and np.all(b0 - gap <= a1):
                parent[find(j)] = find(i)
    out = {}
    for i in range(n):
        out.setdefault(find(i), []).append(i)
    return list(out.values())


def zone_of(centre):
    x, _, z = centre
    for name, x0, x1, z0, z1 in ZONES:
        if x0 <= x <= x1 and z0 <= z <= z1:
            return name
    if abs(x - 6) > 45 or abs(z + 7) > 45:
        return 'Forest'
    return 'Campus'


class Writer:
    """Writes one GLB with a single mesh, copying the material and its textures."""

    def __init__(self, doc, binary):
        self.doc, self.binary = doc, binary

    def write(self, path, material, attrs, tris):
        out = {'asset': {'version': '2.0', 'generator': 'kvas split_world_glb'}, 'buffers': [], 'bufferViews': [], 'accessors': [],
               'scene': 0, 'scenes': [{'nodes': [0]}], 'nodes': [{'mesh': 0}]}
        blob = bytearray()

        def add_view(data, target=None):
            while len(blob) % 4:
                blob.append(0)
            view = {'buffer': 0, 'byteOffset': len(blob), 'byteLength': len(data)}
            if target:
                view['target'] = target
            blob.extend(data)
            out['bufferViews'].append(view)
            return len(out['bufferViews']) - 1

        def add_accessor(arr, kind, ctype):
            view = add_view(arr.tobytes(), 34962 if kind != 'SCALAR' else 34963)
            acc = {'bufferView': view, 'componentType': ctype, 'count': len(arr), 'type': kind}
            if kind == 'VEC3' and ctype == 5126:
                acc['min'] = arr.min(axis=0).tolist()
                acc['max'] = arr.max(axis=0).tolist()
            out['accessors'].append(acc)
            return len(out['accessors']) - 1

        prim = {'attributes': {}, 'mode': 4}
        for name, arr in attrs.items():
            kind = {2: 'VEC2', 3: 'VEC3', 4: 'VEC4'}[arr.shape[1]]
            prim['attributes'][name] = add_accessor(arr.astype(np.float32), kind, 5126)
        prim['indices'] = add_accessor(tris.reshape(-1).astype(np.uint32), 'SCALAR', 5125)
        if material is not None:
            mat = json.loads(json.dumps(self.doc['materials'][material]))
            textures, images, samplers = [], [], []

            def remap(tex_index):
                tex = self.doc['textures'][tex_index]
                new = {}
                if 'source' in tex:
                    img = self.doc['images'][tex['source']]
                    v = self.doc['bufferViews'][img['bufferView']]
                    data = self.binary[v.get('byteOffset', 0):v.get('byteOffset', 0) + v['byteLength']]
                    images.append({'bufferView': add_view(data), 'mimeType': img.get('mimeType', 'image/png')})
                    new['source'] = len(images) - 1
                if 'sampler' in tex:
                    samplers.append(self.doc['samplers'][tex['sampler']])
                    new['sampler'] = len(samplers) - 1
                textures.append(new)
                return len(textures) - 1

            def walk(node):
                if isinstance(node, dict):
                    for k, v in node.items():
                        if k == 'index' and isinstance(v, int):
                            node[k] = remap(v)
                        else:
                            walk(v)

            walk(mat)
            out['materials'] = [mat]
            prim['material'] = 0
            if textures:
                out['textures'] = textures
                out['images'] = images
            if samplers:
                out['samplers'] = samplers
        out['meshes'] = [{'primitives': [prim]}]
        while len(blob) % 4:
            blob.append(0)
        out['buffers'] = [{'byteLength': len(blob)}]
        text = json.dumps(out, separators=(',', ':')).encode('utf-8')
        text += b' ' * ((4 - len(text) % 4) % 4)
        total = 12 + 8 + len(text) + 8 + len(blob)
        with open(path, 'wb') as f:
            f.write(struct.pack('<III', 0x46546C67, 2, total))
            f.write(struct.pack('<II', len(text), 0x4E4F534A))
            f.write(text)
            f.write(struct.pack('<II', len(blob), 0x004E4942))
            f.write(blob)


def slug(text):
    return ''.join(c if c.isalnum() else '_' for c in text).strip('_').lower() or 'part'


def main():
    src = sys.argv[1] if len(sys.argv) > 1 else 'games/last_service/assets/world/last_service_world.glb'
    out_dir = sys.argv[2] if len(sys.argv) > 2 else os.path.join(os.path.dirname(src), 'parts')
    os.makedirs(out_dir, exist_ok=True)
    doc, binary = read_glb(src)
    offset = np.array(doc['nodes'][0].get('translation', [0, 0, 0]), dtype=np.float32)
    names = [m.get('name', '') for m in doc.get('materials', [])]
    writer = Writer(doc, binary)
    index = []
    cut = [0]
    for mesh in doc['meshes']:
        for prim in mesh['primitives']:
            mat = prim.get('material')
            mat_name = names[mat] if mat is not None else 'default'
            if mat_name in DROP:
                continue
            attrs = {k: accessor(doc, binary, v) for k, v in prim['attributes'].items() if k in ('POSITION', 'NORMAL', 'TEXCOORD_0')}
            attrs['POSITION'] = attrs['POSITION'] + offset
            tris = accessor(doc, binary, prim['indices']).reshape(-1, 3) if 'indices' in prim else np.arange(len(attrs['POSITION'])).reshape(-1, 3)
            pos = attrs['POSITION']
            pieces = components(pos, tris)
            boxes = [(pos[tris[p].reshape(-1)].min(axis=0), pos[tris[p].reshape(-1)].max(axis=0)) for p in pieces]
            if mat_name in PIECES:
                groups = [[t for i in g for t in pieces[i]] for g in cluster(boxes)]
                for n, group in enumerate(groups):
                    sub = tris[group]
                    used, local = np.unique(sub.reshape(-1), return_inverse=True)
                    p = pos[used]
                    base = np.array([(p[:, 0].min() + p[:, 0].max()) / 2, p[:, 1].min(), (p[:, 2].min() + p[:, 2].max()) / 2], dtype=np.float32)
                    part = {k: v[used] for k, v in attrs.items()}
                    part['POSITION'] = p - base
                    # Which way the piece faces: away from its tall side (a chair's back).
                    high = p[p[:, 1] > base[1] + 0.6]
                    back = (high.mean(axis=0) - base) if len(high) else np.zeros(3)
                    file = f'{slug(mat_name)}_{n:02d}.glb'
                    writer.write(os.path.join(out_dir, file), mat, part, local.reshape(-1, 3))
                    index.append({'file': f'world/parts/{file}', 'kind': 'piece', 'material': mat_name, 'zone': zone_of(base),
                                  'position': base.tolist(), 'size': (p.max(axis=0) - p.min(axis=0)).tolist(),
                                  'back': [float(back[0]), float(back[2])]})
                continue
            by_zone = {}
            for piece, (lo, hi) in zip(pieces, boxes):
                if in_doorway(lo, hi):
                    cut[0] += 1
                    continue
                by_zone.setdefault(zone_of((lo + hi) / 2), []).extend(piece)
            for zone, group in by_zone.items():
                sub = tris[group]
                used, local = np.unique(sub.reshape(-1), return_inverse=True)
                part = {k: v[used] for k, v in attrs.items()}
                file = f'{slug(zone)}_{slug(mat_name)}.glb'
                writer.write(os.path.join(out_dir, file), mat, part, local.reshape(-1, 3))
                p = part['POSITION']
                index.append({'file': f'world/parts/{file}', 'kind': 'zone', 'material': mat_name, 'zone': zone,
                              'position': [0, 0, 0], 'size': (p.max(axis=0) - p.min(axis=0)).tolist(), 'triangles': int(len(group))})
    json.dump(index, open(os.path.join(out_dir, 'index.json'), 'w'), indent=1)
    print(f'{len(index)} parts -> {out_dir} ({cut[0]} pieces cut out of doorways)')


if __name__ == '__main__':
    main()

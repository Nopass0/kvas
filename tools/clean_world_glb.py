"""Removes staged figures and duplicate props from the LAST SERVICE world GLB.

The production's merged world mesh has Anna, the Counter and Pavel posed in it, plus plates,
food and the horror props that the game places itself as usable things. Left in, they show up
twice and the "monster" never moves. Primitives are dropped by material name; the binary chunk
is kept as it is (unused bytes are harmless).

    python tools/clean_world_glb.py [in.glb] [out.glb] [--drop MATERIAL ...]
"""
import json
import struct
import sys

DROP = {
    'Anna_atlas', 'Counter_atlas', 'Pavel_atlas',
    'clean plate', 'dirty plate', 'water', 'glass', 'glass bot', 'bread', 'Chocolate bar',
    'notebook_1', 'cardboard_box_1', 'props',
}


def main():
    args = [a for a in sys.argv[1:] if not a.startswith('--')]
    src = args[0] if args else 'games/last_service/assets/world/last_service_world.glb'
    dst = args[1] if len(args) > 1 else src
    drop = set(DROP)
    if '--drop' in sys.argv:
        drop |= set(sys.argv[sys.argv.index('--drop') + 1:])
    data = open(src, 'rb').read()
    magic, version, _ = struct.unpack_from('<III', data, 0)
    assert magic == 0x46546C67, 'not a GLB'
    json_len, _ = struct.unpack_from('<II', data, 12)
    doc = json.loads(data[20:20 + json_len])
    rest = data[20 + json_len:]
    names = [m.get('name', '') for m in doc.get('materials', [])]
    removed = 0
    for mesh in doc['meshes']:
        keep = []
        for p in mesh['primitives']:
            if names[p.get('material', 0)] in drop:
                removed += 1
            else:
                keep.append(p)
        mesh['primitives'] = keep
    text = json.dumps(doc, separators=(',', ':')).encode('utf-8')
    text += b' ' * ((4 - len(text) % 4) % 4)
    out = struct.pack('<III', magic, version, 12 + 8 + len(text) + len(rest)) + struct.pack('<II', len(text), 0x4E4F534A) + text + rest
    open(dst, 'wb').write(out)
    print(f'removed {removed} primitives -> {dst}')


if __name__ == '__main__':
    main()

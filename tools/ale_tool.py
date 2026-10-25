#!/usr/bin/env python3
"""ale_tool.py - read Brute Force effect libraries (effects-<level>.ale).

These are Digital Anvil "UTF" containers (the format Freelancer uses): a header, a tree of
44-byte entries (folders and leaves), a names block and a data block. Brute Force's effect files
hold two leaves, laid out like Freelancer's .ale (Librelancer documents those):

  ALEffectLib         f32 version (1.1), i32 count, then per effect: u16-length name (padded to
                      even), 4 floats, i32 n + n x (u32 flag, u32 node, u32 parent, u32 index),
                      i32 m + m x (u32, u32) pairs. `node` is the game's name hash of the node's
                      name (e.g. "laser_mflash_glow.app"), the same hash as BXML names.
  AlchemyNodeLibrary  f32 version (1.2), i32 count, then per node: u32 class hash, then
                      parameters (u16 type, u32 name hash, value) up to type 0. Types: 0x001 bool
                      (value in bit 15), 0x002 int, 0x003 float, 0x103 string, 0x004 u32, 0x104 u32 pair (blend modes),
                      0x105 transform (u32 flags + 9 curves), 0x200 float animation and 0x201
                      colour animation (u8 easing, u8 count; per item f32 sparam, u8 easing, u8 n,
                      n keys of (t, v) / (t, r, g, b)), 0x202 curve. (Freelancer stores the class name as a string;
                      Brute Force hashes it.) Strings are node names ("x.app", "x.emt") and
                      texture names.

usage:
  ale_tool.py tree    FILE.ale|LEVEL.tgz            the UTF tree
  ale_tool.py effects FILE.ale|LEVEL.tgz [TEXT]     effects (with TEXT in the name) and each
                                                    node's string parameters (names, textures)
  ale_tool.py node    FILE.ale|LEVEL.tgz NAME       one node's parameters in full

A .tgz argument reads the effects-*.ale inside it.
"""
import struct, sys, tarfile, os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from xmb_tool import name_hash  # noqa: E402


def load(path):
    if path.endswith('.tgz'):
        t = tarfile.open(path)
        for m in t.getmembers():
            if m.name.endswith('.ale'):
                return t.extractfile(m).read(), m.name
        raise SystemExit(f'{path}: no .ale inside')
    return open(path, 'rb').read(), path


def parse(d):
    """The UTF tree: [(depth, name, data or None for folders)]."""
    if d[:4] != b'UTF ':
        raise ValueError(f'not a UTF file ({d[:4]!r})')
    (_ver, tree_off, _tree_size, _u1, _entry_size, names_off, _names_alloc, names_used,
     data_off) = struct.unpack_from('<9I', d, 4)
    names = d[names_off:names_off + names_used]

    def name_at(o):
        return names[o:names.find(b'\x00', o)].decode('latin-1')

    def walk(o, depth, out):
        while True:
            nxt, name, flags, _z, child, _alloc, size1, _size2 = struct.unpack_from('<8I', d, tree_off + o)
            if flags & 0x10:
                out.append((depth, name_at(name), None))
                if child:
                    walk(child, depth + 1, out)
            else:
                out.append((depth, name_at(name), d[data_off + child:data_off + child + size1]))
            if not nxt:
                return out
            o = nxt

    return walk(0, 0, [])


class Reader:
    def __init__(self, d, o=0):
        self.d, self.o = d, o

    def take(self, fmt):
        v = struct.unpack_from(fmt, self.d, self.o)
        self.o += struct.calcsize(fmt)
        return v[0] if len(v) == 1 else v

    def string(self):
        n = self.take('<H')
        s = self.d[self.o:self.o + n].split(b'\x00')[0].decode('latin-1')
        self.o += n + (n & 1)
        return s


def curve(r):
    """Curve animation: easing, count; each (sparam, value, flags, keyframes (t, v, in, out))."""
    easing, count = r.take('<B'), r.take('<B')
    if count == 0:
        return ('const', r.take('<f'))
    out = []
    for _ in range(count):
        sparam, value, flags, kf = r.take('<ffHH')
        out.append((sparam, value, flags, [r.take('<4f') for _ in range(kf)]))
    return ('curve', easing, out)


def value(r, t):
    base = t & 0x7fff
    if base == 0x001:
        return bool(t & 0x8000)
    if base == 0x002:
        return r.take('<i')
    if base == 0x003:
        return r.take('<f')
    if base == 0x103:
        return r.string()
    if base == 0x004:
        return r.take('<I')
    if base == 0x104:                 # pair (e.g. blend: source, destination)
        return r.take('<II')
    if base == 0x105:
        flags = r.take('<I')
        return ('xform', flags, [curve(r) for _ in range(9)] if flags else [])
    if base in (0x200, 0x201):
        # u8 easing, u8 count; per item f32 sparam, u8 easing, u8 n, n keys (t, v) / (t, r, g, b)
        width = 2 if base == 0x200 else 4
        _easing, count = r.take('<B'), r.take('<B')
        items = []
        for _ in range(count):
            sp, ease, n = r.take('<f'), r.take('<B'), r.take('<B')
            items.append((sp, ease, [r.take(f'<{width}f') for _ in range(n)]))
        return ('floats' if base == 0x200 else 'colors', items)
    if base == 0x202:
        return curve(r)
    raise ValueError(f'unknown ALE value type 0x{t:04x} at 0x{r.o:x}')


def node_library(blob):
    r = Reader(blob)
    version, count = r.take('<f'), r.take('<i')
    nodes = []
    for _ in range(count):
        cls, params = r.take('<I'), []
        while (t := r.take('<H')) != 0:
            crc = r.take('<I')
            params.append((t, crc, value(r, t)))
        nodes.append((cls, params))
    return version, nodes


def effect_library(blob):
    r = Reader(blob)
    version, count = r.take('<f'), r.take('<i')
    effects = []
    for _ in range(count):
        name = r.string()
        extra = [r.take('<f') for _ in range(4)]
        refs = [r.take('<4I') for _ in range(r.take('<i'))]
        pairs = [r.take('<2I') for _ in range(r.take('<i'))]
        effects.append((name, extra, refs, pairs))
    return version, effects


def libraries(d):
    leaf = {n: data for _, n, data in parse(d) if data is not None}
    return effect_library(leaf['ALEffectLib']), node_library(leaf['AlchemyNodeLibrary'])


def nodes_by_name(nodes):
    """name hash -> (class hash, params), keyed by each node's first string (its name)."""
    out = {}
    for cls, params in nodes:
        strs = [v for t, c, v in params if isinstance(v, str)]
        if strs:
            out.setdefault(name_hash(strs[0]), (strs[0], cls, params))
    return out


def short(v):
    s = repr(v)
    return s if len(s) < 140 else s[:137] + '...'


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        return
    cmd, path = sys.argv[1], sys.argv[2]
    d, _ = load(path)
    if cmd == 'tree':
        for depth, n, data in parse(d):
            print('  ' * depth + n + (f'  [{len(data)} B]' if data is not None else '/'))
        return
    (_, effects), (_, nodes) = libraries(d)
    byname = nodes_by_name(nodes)
    if cmd == 'effects':
        text = sys.argv[3].lower() if len(sys.argv) > 3 else ''
        for name, extra, refs, pairs in effects:
            if text and text not in name.lower():
                continue
            print(name)
            for flag, crc, parent, idx in refs:
                node = byname.get(crc)
                strs = [v for t, c, v in node[2] if isinstance(v, str)] if node else ['?']
                print(f'   #{idx} parent {parent:04x} {crc:08x} cls {node[1] if node else 0:08x}: ' + ' | '.join(strs))
    elif cmd == 'node':
        want = sys.argv[3]
        node = byname.get(name_hash(want))
        if not node:
            raise SystemExit(f'no node {want}')
        print(f'{node[0]}  class {node[1]:08x}')
        for t, c, v in node[2]:
            print(f'   type {t:04x} param {c:08x}: {short(v)}')


if __name__ == '__main__':
    main()

#!/usr/bin/env python3
"""
xmb_tool.py - Brute Force BXML (.xmb) / BXSD (.xsb) reader.

Format (reverse-engineered from default.xbe, reader at 0x255330 / 0x254c60 / 0x281d30):

  0x00  'BXML' | 'BXSD'
  0x04  u8 ver[4]               01 01 07 00 (BXML), 01 00 01 00 (BXSD)
  0x08  u32 names_offset        absolute offset of the name table
  0x0C  u32 0
  0x10  u8  ns_count, then ns_count x (u32 prefix_hash, u32 uri_hash)
  names_offset: u8 count (varint?), count x u32 name_hash
  token stream (LZ-compressed u16 words, see TokenStream)
  value stream (raw bytes, layout given by the schema type of each attribute/element)

Names are hashed with a CRC32 variant (game table from the XBE at 0x3bc150) over the
lower-cased name. Recovered names live in xmb_names.txt next to this script.

Tokens (u16, top nibble = kind, low 12 bits = name index, bit 0x800 = continuation):
  0x0nnn  start child element n        0x9nnn  set namespace index
  0xAnnn  attribute n (+1 value)       0xBnnn  empty element n
  0xC000  end element                  0xDnnn  text content, n = item count
"""
import argparse
import os
import struct
import sys

# ---- name hashing -----------------------------------------------------------

# CRC table copied from default.xbe @0x3bc150. Same polynomial as CRC32 (EDB88320) but the top
# byte of most entries differs from the standard table, so zlib.crc32 does NOT match.
_CRC_TABLE = struct.unpack('<256I', bytes.fromhex(
    '00000000963007092c610e12ba51091b19c46dff8ff46af635a563eda39564e43288dbfea4b8dcf71ee9d5ec88d9d2e5'
    '2b4cb601bd7cb108072db813911dbf1a6410b7fdf220b0f44871b9efde41bee67dd4da02ebe4dd0b51b5d410c785d319'
    '56986c03c0a86b0a7af96211ecc965184f5c01fcd96c06f5633d0feef50d08e7c8206efb5e1069f2e44160e9727167e0'
    'd1e4030447d4040dfd850d166bb50a1ffaa8b5056c98b20cd6c9bb1740f9bc1ee36cd8fa755cdff3cf0dd6e8593dd1e1'
    'ac30d9063a00de0f8051d7141661d01db5f4b4f923c4b3f09995baeb0fa5bde29eb802f8088805f1b2d90cea24e90be3'
    '877c6f07114c680eab1d61153d2d661c9041dcf60671dbffbc20d2e42a10d5ed8985b1091fb5b600a5e4bf1b33d4b812'
    'a2c9070834f900018ea8091a18980e13bb0d6af72d3d6dfe976c64e5015c63ecf4516b0b62616c02d83065194e006210'
    'ed9506f47ba501fdc1f408e657c40fefc6d9b0f550e9b7fceab8bee77c88b9eedf1ddd0a492dda03f37cd318654cd411'
    '5861b20dce51b5047400bc1fe230bb1641a5dff2d795d8fb6dc4d1e0fbf4d6e96ae969f3fcd96efa468867e1d0b860e8'
    '732d040ce51d03055f4c0a1ec97c0d173c7105f0aa4102f910100be286200ceb25b5680fb3856f0609d4661d9fe46114'
    '0ef9de0e98c9d9072298d01cb4a8d715173db3f1810db4f83b5cbde3ad6cbaea2083b8edb6b3bfe40ce2b6ff9ad2b1f6'
    '3947d512af77d21b1526db008316dc09120b6313843b641a3e6a6d01a85a6a080bcf0eec9dff09e527ae00feb19e07f7'
    '44930f10d2a3081968f20102fec2060b5d5762efcb6765e671366cfde7066bf4761bd4eee02bd3e75a7adafccc4addf5'
    '6fdfb911f9efbe1843beb703d58eb00ae8a3d6167e93d11fc4c2d80452f2df0df167bbe96757bce0dd06b5fb4b36b2f2'
    'da2b0de84c1b0ae1f64a03fa607a04f3c3ef601755df671eef8e6e0579be690c8cb361eb1a8366e2a0d26ff936e268f0'
    '95770c1403470b1db91602062f26050fbe3bba15280bbd1c925ab407046ab30ea7ffd7ea31cfd0e38b9ed9f81daedef1'
    'b0c2641b26f263129ca36a090a936d00a90609e43f360eed856707f6135700ff824abfe5147ab8ecae2bb1f7381bb6fe'
    '9b8ed21a0dbed513b7efdc0821dfdb01d4d2d3e642e2d4eff8b3ddf46e83dafdcd16be195b26b910e177b00b7747b702'
    'e65a0818706a0f11ca3b060a5c0b0103ff9e65e769ae62eed3ff6bf545cf6cfc78e20ae0eed20de9548304f2c2b303fb'
    '6126671ff71660164d47690ddb776e044a6ad11edc5ad617660bdf0cf03bd80553aebce1c59ebbe87fcfb2f3e9ffb5fa'
    '1cf2bd1d8ac2ba143093b30fa6a3b4060536d0e29306d7eb2957def0bf67d9f92e7a66e3b84a61ea021b68f1942b6ff8'
    '37be0b1ca18e0c151bdf050e8def0207'))


def name_hash(s):
    """Brute Force name hash: CRC32-style over the ASCII-lower-cased name with the game's table."""
    c = 0xFFFFFFFF
    for b in s.lower().encode('latin-1'):
        c = _CRC_TABLE[(b ^ c) & 0xFF] ^ (c >> 8)
    return c ^ 0xFFFFFFFF


_NAMES = None


def names_db():
    global _NAMES
    if _NAMES is None:
        _NAMES = {}
        p = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'xmb_names.txt')
        if os.path.exists(p):
            for line in open(p, encoding='utf-8'):
                line = line.rstrip('\n')
                if line and not line.startswith('#'):
                    _NAMES[name_hash(line)] = line
    return _NAMES


def hname(h):
    if h == 0:
        return ''
    return names_db().get(h, f'h_{h:08x}')



# ---- stream reader (one file cursor shared by token blocks and values) ------------

class Reader:
    def __init__(self, d, pos=0):
        self.d = d
        self.pos = pos

    def bytes(self, n):
        if self.pos + n > len(self.d):
            raise EOFError(f'read of {n} bytes at {self.pos:#x} runs past end ({len(self.d):#x})')
        v = self.d[self.pos:self.pos + n]
        self.pos += n
        return v

    def u8(self):
        return self.bytes(1)[0]

    def u16(self):
        return struct.unpack('<H', self.bytes(2))[0]

    def u32(self):
        return struct.unpack('<I', self.bytes(4))[0]

    def varint(self):
        v = s = 0
        while True:
            b = self.u8()
            v |= (b & 0x7F) << s
            s += 7
            if not b & 0x80:
                return v

    def qname(self):
        """Tagged hash list: 0x00 = none, 0x2N = N hashes (0x21 local name, 0x22 ns-uri + name)."""
        t = self.u8()
        if t == 0:
            return None
        return tuple(self.u32() for _ in range(t & 0x0F))


# ---- token stream decompressor (0x281bf0 / 0x281cc0 / 0x281d30) -------------------

class TokenStream:
    """Lazily decoded, block-structured LZ over u16 words, sharing the file cursor with values.

    Block: u16 nwords (incl. itself), u32 out_count ('wide', BXML >= 1.1) or u16, then words.
    A new block is loaded only when a raw word is needed and the current one is used up, so
    value bytes sit between blocks. Each block starts with a 16-bit flag word (LSB first).
    Flag 0: literal word. Flag 1: match from ring[pos], pos = w & 0x1FF (absolute index in a
    512-word ring), length (w >> 9) + 2. The ring write position resets to 1 per block."""

    def __init__(self, r, wide=True):
        self.r = r
        self.wide = wide
        self.words = ()
        self.idx = 0
        self.ring = [0] * 512
        self.wp = 1
        self.copy = 0
        self.rp = 0
        self.flags, _ = self._raw()
        self.bit = 0

    def _raw(self):
        new = False
        if self.idx >= len(self.words):
            n = self.r.u16()
            if n < 3:
                raise EOFError(f'bad token block header {n} at {self.r.pos - 2:#x}')
            self.words = (n,) + struct.unpack(f'<{n - 1}H', self.r.bytes(2 * (n - 1)))
            self.idx = 3 if self.wide else 2
            self.wp = 1
            new = True
        w = self.words[self.idx]
        self.idx += 1
        return w, new

    def next(self):
        if self.copy:
            v = self.ring[self.rp & 0x1FF]
            self.rp += 1
            self.ring[self.wp] = v
            self.wp = (self.wp + 1) & 0x1FF
            self.copy -= 1
            return v
        if self.bit == 16:
            self.flags, _ = self._raw()
            self.bit = 0
        f = (self.flags >> self.bit) & 1
        self.bit += 1
        w, new = self._raw()
        if new:
            self.flags = w
            f = w & 1
            self.bit = 1
            w, _ = self._raw()
        if not f:
            self.ring[self.wp] = w
            self.wp = (self.wp + 1) & 0x1FF
            return w
        self.rp = w & 0x1FF
        self.copy = (w >> 9) + 2
        return self.next()


# ---- schemas (.xsb) ------------------------------------------------------------------

XS_URI = name_hash('http://www.w3.org/2001/XMLSchema')
DAXNS = name_hash('daxns')
H_STRINGID = name_hash('stringid')     # XBE global 0x46cfc8: values stored as a 4-byte name hash
H_WSTRING = name_hash('wstring')       # XBE global 0x46cfcc
H_INDEXSTR = 0xF548750A                # XBE global 0x46cfd4: values stored as varint (index + 1)
H_PROPS = 0x1B8535F0                   # name of the kind-0x80 property-block element

# xs builtin name hash -> struct format
XS_FIXED = {name_hash(n): f for n, f in {
    'boolean': '?', 'byte': 'b', 'unsignedByte': 'B', 'short': 'h', 'unsignedShort': 'H',
    'int': 'i', 'unsignedInt': 'I', 'float': 'f', 'double': 'd', 'long': 'q', 'unsignedLong': 'Q',
}.items()}

# Schema node kinds
K_COMPLEX = 0x01    # complexType: children are members
K_SIMPLE = 0x02     # simpleType restriction (name : base); also a leaf inside a 0x80 block
K_LIST = 0x06       # list simpleType (name : item type)
K_ELEM = 0x09       # local element with anonymous complex type (children)
K_ATTR = 0x0A       # attribute (name : type)
K_ELEMREF = 0x18    # element (rare)
K_GELEM = 0x19      # element with named type (name : type)
K_PROPS = 0x80      # property block (element H_PROPS); its leaves are read like attributes
K_EXT = 0x101       # complexType extending a base (name : base), children add members


class Node:
    __slots__ = ('kind', 'name', 'type', 'children', 'ns')

    def __init__(self, kind, name, type_, ns):
        self.kind, self.name, self.type, self.ns = kind, name, type_, ns
        self.children = []

    @property
    def local(self):
        return self.name[-1] if self.name else None


class Schema:
    def __init__(self, path, data=None):
        d = data if data is not None else open(path, 'rb').read()
        if d[:4] != b'BXSD':
            raise ValueError(f'{path}: not BXSD')
        r = Reader(d, 0x10)
        r.varint()                                     # always 1
        self.target = r.qname()[-1]
        self.namespaces = [(r.u32(), r.u32()) for _ in range(r.varint())]
        r.pos = struct.unpack_from('<I', d, 8)[0]
        self.nodes = [self._node(r) for _ in range(r.varint())]
        if r.pos != len(d):
            raise ValueError(f'{path}: schema parse ended at {r.pos:#x} of {len(d):#x}')
        self.path = path

    def _node(self, r):
        n = Node(r.u16(), r.qname(), r.qname(), self.target)
        for _ in range(r.varint()):
            n.children.append(self._node(r))
        return n


class SchemaSet:
    def __init__(self, paths=(), blobs=()):
        """paths: .xsb files; blobs: (name, bytes) pairs (e.g. read from common.tgz)."""
        self.schemas = {}
        self.globals = {}                              # (ns, name) -> top-level Node
        for s in [Schema(p) for p in paths] + [Schema(n, b) for n, b in blobs]:
            self.schemas[s.target] = s
            for n in s.nodes:
                self.globals.setdefault((s.target, n.local), n)

    @classmethod
    def from_dir(cls, d):
        return cls(sorted(os.path.join(d, f) for f in os.listdir(d) if f.lower().endswith('.xsb')))

    _cache = {}

    @classmethod
    def find(cls, near):
        """Locate common/schemas/*.xsb walking up from a file in an extracted data tree
        (cached per schema directory, so US and JP trees each use their own schemas)."""
        d = os.path.abspath(os.path.dirname(near))
        for _ in range(8):
            for cand in (os.path.join(d, 'schemas'), os.path.join(d, 'common', 'schemas'),
                         os.path.join(d, 'common', 'common', 'schemas')):
                if os.path.isdir(cand):
                    if cand not in cls._cache:
                        cls._cache[cand] = cls.from_dir(cand)
                    return cls._cache[cand]
            d = os.path.dirname(d)
        raise FileNotFoundError('common/schemas/*.xsb not found; pass --schemas DIR')

    def resolve(self, qn, ns):
        """QName -> Node, or an (ns, name) tuple for xs builtins / unknown types."""
        if qn is None:
            return None
        key = (qn[0], qn[1]) if len(qn) >= 2 else (ns, qn[0])
        return self.globals.get(key, key)

    def members(self, node):
        """Children of a complex type, following K_EXT inheritance to the base."""
        out, seen = [], set()
        while isinstance(node, Node) and id(node) not in seen:
            seen.add(id(node))
            out.extend(node.children)
            node = self.resolve(node.type, node.ns) if node.kind == K_EXT else None
        return out

    def global_elem(self, name, nss):
        """Global element declaration `name`, searched in the given namespaces (in order)."""
        for ns in nss:
            g = self.globals.get((ns, name))
            if g is not None and g.kind in (K_GELEM, K_ELEM, K_ELEMREF, K_SIMPLE):
                return g
        return None

    def find_member(self, ctx, name, attr):
        """Find the declaration of an attribute (attr=True) or child element named `name`."""
        if ctx is None:
            return None
        queue = list(self.members(ctx))
        while queue:
            c = queue.pop(0)
            if c.local == name:
                if attr and c.kind in (K_ATTR, K_SIMPLE, K_GELEM):
                    return c
                if not attr and c.kind in (K_ELEM, K_GELEM, K_PROPS, K_ELEMREF, K_SIMPLE):
                    return c
            if c.kind == K_PROPS and ctx.kind == K_PROPS:
                queue.extend(c.children)
        if not attr:
            # list-typed attributes (vectors etc.) are encoded as a child element + text token
            for c in self.members(ctx):
                if c.local == name and c.kind == K_ATTR:
                    return c
            # <xs:element ref="..."/> to a global declaration
            return self.global_elem(name, [ctx.ns] + [ns for ns in self.schemas if ns != ctx.ns])
        return None

    def content(self, decl):
        """Complex-content context (a Node whose members are the element's members)."""
        if decl is None:
            return None
        if decl.kind in (K_ELEM, K_PROPS):
            return decl
        t = self.resolve(decl.type, decl.ns)
        if isinstance(t, Node) and t.kind in (K_COMPLEX, K_EXT, K_ELEM):
            return t
        return None

    def codec(self, decl):
        """How to read a value of this declaration's type:
        ('fixed', fmt) | ('hash',) | ('index',) | ('str',) | ('wstr',) | ('list', codec) | None"""
        node = decl
        for _ in range(64):
            if node is not decl:
                if node.local == H_STRINGID and node.ns == DAXNS:
                    return ('hash',)
                if node.local == H_INDEXSTR:
                    return ('index',)
                if node.local == H_WSTRING and node.ns == DAXNS:
                    return ('wstr',)
                if node.kind == K_LIST:
                    item = self.resolve(node.type, node.ns)
                    if isinstance(item, Node):
                        return ('list', self.codec(Node(K_SIMPLE, None, node.type, node.ns)))
                    return ('list', self._builtin(item))
            if node.type is None:
                return ('str',) if node.kind in (K_SIMPLE, K_ATTR) else None
            t = self.resolve(node.type, node.ns)
            if not isinstance(t, Node):
                return self._builtin(t)
            if t.kind in (K_COMPLEX, K_EXT, K_ELEM):
                return None
            node = t
        return ('str',)

    @staticmethod
    def _builtin(key):
        ns, name = key
        if ns == XS_URI and name in XS_FIXED:
            return ('fixed', XS_FIXED[name])
        return ('str',)


# ---- BXML document ----------------------------------------------------------------------

class Hash(int):
    """A stringid value (stored as a name hash)."""


class Element:
    __slots__ = ('name', 'attrs', 'children', 'text', 'line')

    def __init__(self, name):
        self.name, self.attrs, self.children, self.text, self.line = name, [], [], None, None


class DecodeError(Exception):
    pass


class BXML:
    def __init__(self, path, schemas=None, data=None):
        d = data if data is not None else open(path, 'rb').read()
        if d[:4] != b'BXML':
            raise ValueError(f'not BXML: {d[:4]!r}')
        self.path = path
        self.d = d
        self.ver = d[4:8]
        self.names_off = struct.unpack_from('<I', d, 8)[0]
        r = Reader(d, 0x10)
        self.namespaces = [(r.u32(), r.u32()) for _ in range(r.varint())]
        r.pos = self.names_off
        self.names = [r.u32() for _ in range(r.varint())]
        self.body_off = r.pos
        self.r = r
        self.S = schemas
        self.ts = TokenStream(r, wide=(d[4], d[5]) >= (1, 1))
        self.root = None
        self.end = None

    def nm(self, idx):
        return self.names[idx] if idx < len(self.names) else 0xFFFFFFFF

    def _count(self, t):
        """Low 12 bits, or 11 bits + 15-bit continuation words when bit 0x800 is set."""
        if not t & 0x800:
            return t & 0xFFF
        n, shift = t & 0x7FF, 11
        while True:
            w = self.ts.next()
            n |= (w & 0x7FFF) << shift
            shift += 15
            if not w & 0x8000:
                return n

    def _value(self, codec, count, where):
        if codec is None:
            raise DecodeError(f'{where}: type unknown (no schema declaration)')
        r = self.r
        k = codec[0]
        if k == 'fixed':
            fmt = codec[1]
            vals = list(struct.unpack(f'<{count}{fmt}', r.bytes(struct.calcsize(fmt) * count)))
            return vals[0] if count == 1 else vals
        if k == 'hash':
            return Hash(r.u32())
        if k == 'index':
            return r.varint() - 1
        if k in ('str', 'wstr'):
            raw = r.bytes(r.varint())
            return raw.decode('utf-16-le' if k == 'wstr' else 'latin-1', 'replace')
        if k == 'list':
            inner = codec[1]
            if inner[0] == 'fixed':
                v = self._value(inner, count, where)
                return v if isinstance(v, list) else [v]
            return [self._value(inner, 1, where) for _ in range(count)]
        raise DecodeError(f'{where}: bad codec {codec}')

    def parse(self):
        t = self.ts.next()
        self.root = Element(self.nm(t & 0xFFF))
        nss = [u for p, u in self.namespaces if p == 0] + [u for p, u in self.namespaces if p != 0]
        decl = self.S.global_elem(self.root.name, nss) if self.S else None
        self._element(self.root, decl)
        self.end = self.r.pos
        return self.root

    def _element(self, el, decl):
        S = self.S
        ctx = S.content(decl) if S else None
        where = lambda: f'{hname(el.name)} @{self.r.pos:#x}'
        while True:
            t = self.ts.next()
            k = t & 0xF000
            if k == 0x9000:
                el.line = self._count(t)
            elif k in (0x0000, 0xB000):
                child = Element(self.nm(t & 0xFFF))
                el.children.append(child)
                if k == 0x0000:
                    cdecl = S.find_member(ctx, child.name, False) if S else None
                    self._element(child, cdecl)
            elif k == 0xA000:
                name = self.nm(t & 0xFFF)
                adecl = S.find_member(ctx, name, True) if S else None
                codec = S.codec(adecl) if adecl else None
                el.attrs.append((name, self._value(codec, 1, f'{where()} attr {hname(name)}')))
            elif k == 0xC000:
                return
            elif k == 0xD000:
                n = self._count(t)
                codec = S.codec(decl) if (S and decl) else None
                el.text = self._value(codec, n, f'{where()} text [{describe(decl, S)}]')
            else:
                raise DecodeError(f'unknown token {t:04x} at {self.r.pos:#x}')


def describe(decl, S):
    if decl is None:
        return 'no decl'
    q = lambda t: ':'.join(hname(h) for h in t) if t else '-'
    s = f'kind {decl.kind:#x} {q(decl.name)} : {q(decl.type)}'
    t = S.resolve(decl.type, decl.ns) if decl.type else None
    if isinstance(t, Node):
        s += f' -> kind {t.kind:#x} : {q(t.type)}'
    return s


# ---- output -------------------------------------------------------------------------------

def fmt_value(v):
    if isinstance(v, Hash):
        return hname(v)
    if isinstance(v, bool):
        return 'true' if v else 'false'
    if isinstance(v, float):
        return f'{v:.7g}'
    if isinstance(v, list):
        return ' '.join(fmt_value(x) for x in v)
    return str(v)


def xml_escape(s):
    return s.replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;').replace('"', '&quot;')


def to_xml(el, out, depth=0):
    pad = '  ' * depth
    tag = hname(el.name)
    merged = {}                                    # repeated attributes = list items, one per token
    for n, v in el.attrs:
        merged.setdefault(n, []).append(fmt_value(v))
    attrs = ''.join(f' {hname(n)}="{xml_escape(" ".join(vs))}"' for n, vs in merged.items())
    if not el.children and el.text is None:
        out.append(f'{pad}<{tag}{attrs}/>')
    elif not el.children:
        out.append(f'{pad}<{tag}{attrs}>{xml_escape(fmt_value(el.text))}</{tag}>')
    else:
        out.append(f'{pad}<{tag}{attrs}>')
        if el.text is not None:
            out.append(f'{pad}  {xml_escape(fmt_value(el.text))}')
        for c in el.children:
            to_xml(c, out, depth + 1)
        out.append(f'{pad}</{tag}>')


def dump_tokens(b):
    print(f'BXML ver {b.ver.hex()} names@{b.names_off:#x} body@{b.body_off:#x}')
    for p, u in b.namespaces:
        print(f'  xmlns:{hname(p) or "(default)"} = {hname(u)}')
    print('  names:', ', '.join(hname(h) for h in b.names))
    print('  (token-only view: value bytes are interleaved, so this stops at the first block)')
    depth = 0
    try:
        while True:
            t = b.ts.next()
            k, n = t >> 12, t & 0xFFF
            nm = hname(b.nm(n))
            if k == 0x0:
                print('  ' * depth + f'<{nm}>'); depth += 1
            elif k == 0xA:
                print('  ' * depth + f'@{nm}')
            elif k == 0xB:
                print('  ' * depth + f'<{nm}/>')
            elif k == 0xC:
                depth -= 1; print('  ' * depth + '</>')
                if depth < 0:
                    break
            elif k == 0xD:
                print('  ' * depth + f'#text x{n}')
            elif k == 0x9:
                print('  ' * depth + f'(line {n})')
            else:
                print('  ' * depth + f'?{t:04x}')
    except EOFError as e:
        print('  <eof>', e)


def convert(path, schemas):
    b = BXML(path, schemas)
    root = b.parse()
    out = ['<?xml version="1.0" encoding="UTF-8"?>']
    to_xml(root, out)
    return b, '\n'.join(out) + '\n'


def iter_xmb(paths):
    for p in paths:
        if os.path.isdir(p):
            for root, _, fs in os.walk(p):
                for f in sorted(fs):
                    if f.lower().endswith('.xmb'):
                        yield os.path.join(root, f)
        else:
            yield p


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--schemas', help='directory with *.xsb (default: search upward for common/schemas)')
    sub = ap.add_subparsers(dest='cmd', required=True)
    p = sub.add_parser('xml', help='decode .xmb files to XML text')
    p.add_argument('files', nargs='+', help='.xmb files or directories')
    p.add_argument('-o', '--out', help='output file (single input) or directory')
    p = sub.add_parser('verify', help='decode files and check each is consumed exactly to EOF')
    p.add_argument('files', nargs='+')
    p.add_argument('-v', action='store_true', help='print each failure')
    p = sub.add_parser('tokens', help='dump the raw token tree of the first block (debug)')
    p.add_argument('file')
    p = sub.add_parser('schema', help='print a .xsb schema tree')
    p.add_argument('file')
    p = sub.add_parser('hash', help='print name hashes')
    p.add_argument('names', nargs='+')
    a = ap.parse_args()

    fixed = SchemaSet.from_dir(a.schemas) if a.schemas else None

    def schemas_for(f):
        return fixed or SchemaSet.find(f)

    if a.cmd == 'xml':
        files = [f for f in iter_xmb(a.files) if os.path.getsize(f)]
        multi = len(files) > 1 or (a.out and os.path.isdir(a.out))
        for f in files:
            try:
                b, text = convert(f, schemas_for(f))
            except (DecodeError, EOFError, ValueError) as e:
                print(f'FAIL {f}: {e}', file=sys.stderr)
                continue
            if b.end != len(b.d):
                print(f'warning: {f}: decoding stopped at {b.end:#x} of {len(b.d):#x}', file=sys.stderr)
            if multi:
                os.makedirs(a.out or '.', exist_ok=True)
                dst = os.path.join(a.out or '.', os.path.splitext(os.path.basename(f))[0] + '.xml')
                open(dst, 'w', encoding='utf-8', newline='\n').write(text)
            elif a.out:
                open(a.out, 'w', encoding='utf-8', newline='\n').write(text)
            else:
                sys.stdout.write(text)
    elif a.cmd == 'verify':
        files = list(iter_xmb(a.files))
        ok, bad, empty = 0, [], 0
        for f in files:
            if not os.path.getsize(f):
                empty += 1
                continue
            try:
                b = BXML(f, schemas_for(f))
                b.parse()
                if b.end == len(b.d):
                    ok += 1
                else:
                    bad.append((f, f'stopped at {b.end:#x} of {len(b.d):#x}'))
            except (DecodeError, EOFError, ValueError, struct.error) as e:
                bad.append((f, str(e)))
        if a.v:
            for f, e in bad:
                print(f'FAIL {f}: {e}')
        print(f'{ok} ok, {len(bad)} failed, {empty} empty (of {len(files)})')
    elif a.cmd == 'tokens':
        dump_tokens(BXML(a.file))
    elif a.cmd == 'schema':
        s = Schema(a.file)
        print(f'targetNamespace {hname(s.target)}')
        for p_, u in s.namespaces:
            print(f'  xmlns:{hname(p_) or "(default)"} = {hname(u)}')

        def qn(q):
            return '-' if q is None else ':'.join(hname(h) for h in q)

        def show(n, depth):
            print('  ' * depth + f'[{n.kind:#x}] {qn(n.name)} : {qn(n.type)}')
            for c in n.children:
                show(c, depth + 1)
        for n in s.nodes:
            show(n, 0)
    elif a.cmd == 'hash':
        for n in a.names:
            print(f'{name_hash(n):08x}  {n}')


if __name__ == '__main__':
    main()

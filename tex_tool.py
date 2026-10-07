#!/usr/bin/env python3
"""
tex_tool.py - Brute Force texture library (.tex + textures-*.xmb) exporter.

Usage:
  python tex_tool.py list   <level.tgz | textures-x.xmb>
  python tex_tool.py export <level.tgz | data-dir | textures-x.xmb> -o OUTDIR [--dds] [--mips]

Inputs can be the level archives straight from the disc (data/e01.tgz, data/common.tgz, ...);
schemas are read from common.tgz next to them. A directory exports every *.tgz in it.

Format notes (see also xmb_tool.py):
  textures-<lvl>.xmb lists every texture: format, mip count, width, height, offset, size in
  textures-<lvl>.tex. Formats are Xbox D3DFORMAT codes; negative codes are cube maps (6 faces,
  each face's mip chain padded to 128 bytes). Uncompressed formats are Morton-swizzled per mip;
  DXT is stored as normal 4x4 blocks.
  Format 11 (P8) textures are bump maps indexing a fixed 256-entry normal palette that the
  game copies from default.xbe (.data, 0x3ba968 in the US build) into a D3D palette. The
  palette is read from your default.xbe at runtime (--xbe), output is a tangent-space normal
  map (R=x, G=y, B=z).
"""
import argparse
import csv
import io
import os
import struct
import sys
import tarfile

import numpy as np
from PIL import Image

import xmb_tool as X

# Xbox D3DFORMAT -> (name, layout, bits per pixel)
#   layout: 'swz' Morton-swizzled, 'lin' linear, 'dxt' 4x4 blocks
FORMATS = {
    0x00: ('L8', 'swz', 8), 0x01: ('AL8', 'swz', 8), 0x02: ('A1R5G5B5', 'swz', 16),
    0x03: ('X1R5G5B5', 'swz', 16), 0x04: ('A4R4G4B4', 'swz', 16), 0x05: ('R5G6B5', 'swz', 16),
    0x06: ('A8R8G8B8', 'swz', 32), 0x07: ('X8R8G8B8', 'swz', 32), 0x0B: ('P8', 'swz', 8),
    0x0C: ('DXT1', 'dxt', 4), 0x0E: ('DXT3', 'dxt', 8), 0x0F: ('DXT5', 'dxt', 8),
    0x10: ('LIN_A1R5G5B5', 'lin', 16), 0x11: ('LIN_R5G6B5', 'lin', 16),
    0x12: ('LIN_A8R8G8B8', 'lin', 32), 0x13: ('LIN_L8', 'lin', 8), 0x19: ('A8', 'swz', 8),
    0x1A: ('A8L8', 'swz', 16), 0x1E: ('LIN_X8R8G8B8', 'lin', 32), 0x1F: ('LIN_A8', 'lin', 8),
    0x2A: ('D24S8', 'swz', 32), 0x2E: ('LIN_D24S8', 'lin', 32),
}
CUBE_ALIGN = 128
PALETTE_SIG = bytes.fromhex('cc197affcc197effcc1980ff')   # first entries of the normal palette

H_FORMAT, H_WIDTH, H_HEIGHT = X.name_hash('format'), X.name_hash('width'), X.name_hash('height')
H_OFFSET, H_SIZE, H_NAME = X.name_hash('offset'), X.name_hash('size'), X.name_hash('name')
H_MIPS = 0x120336EF                                       # mip level count (name not recovered)
H_REF = X.name_hash('reference')
H_DATAFILE = X.name_hash('data-file')


# ---- texture index ------------------------------------------------------------------

class Tex:
    def __init__(self, a):
        self.name = a.get(H_NAME, 0)
        self.code = a[H_FORMAT]
        self.cube = self.code < 0
        base = -self.code if self.cube else self.code
        self.fmt = 0x00 if base == 1000 else base          # -1000: 8-bit intensity cube (light falloff), L8
        self.w, self.h = a[H_WIDTH], a[H_HEIGHT]
        self.mips = max(1, a.get(H_MIPS, 1))
        self.offset, self.size = a[H_OFFSET], a[H_SIZE]
        self.ref = a.get(H_REF, '')

    @property
    def label(self):
        return X.hname(self.name) if self.name else f'tex_{self.offset:08x}'

    @property
    def fname(self):
        return FORMATS.get(self.fmt, (f'fmt{self.fmt}',))[0]

    def mip_dims(self):
        return [(max(self.w >> i, 1), max(self.h >> i, 1)) for i in range(self.mips)]

    def mip_size(self, w, h):
        _, layout, bpp = FORMATS[self.fmt]
        if layout == 'dxt':
            return max(1, (w + 3) // 4) * max(1, (h + 3) // 4) * (8 if self.fmt == 0x0C else 16)
        return w * h * bpp // 8

    def face_size(self):
        return sum(self.mip_size(w, h) for w, h in self.mip_dims())

    def faces(self, blob):
        """Yield per face a list of per-mip byte strings."""
        data = blob[self.offset:self.offset + self.size]
        fs = self.face_size()
        stride = (fs + CUBE_ALIGN - 1) // CUBE_ALIGN * CUBE_ALIGN if self.cube else fs
        for f in range(6 if self.cube else 1):
            p = f * stride
            mips = []
            for w, h in self.mip_dims():
                n = self.mip_size(w, h)
                mips.append(data[p:p + n])
                p += n
            yield mips


def walk_attrs(el):
    if any(k == H_FORMAT for k, _ in el.attrs):
        yield dict(el.attrs)
    for c in el.children:
        yield from walk_attrs(c)


def read_index(xmb_bytes, schemas, name='textures.xmb'):
    root = X.BXML(name, schemas, data=xmb_bytes).parse()
    data_file = None
    for a in walk_attrs_all(root):
        if H_DATAFILE in a:
            data_file = a[H_DATAFILE]
    return [Tex(a) for a in walk_attrs(root)], data_file


def walk_attrs_all(el):
    yield dict(el.attrs)
    for c in el.children:
        yield from walk_attrs_all(c)


# ---- pixel decoding ---------------------------------------------------------------------

_SWZ = {}


def swizzle_map(w, h):
    """Linear index -> swizzled index, Xbox Morton order (x takes the lower bit)."""
    key = (w, h)
    if key not in _SWZ:
        mx = np.zeros(w, np.int64)
        my = np.zeros(h, np.int64)
        xs, ys = np.arange(w), np.arange(h)
        bit, ww, hh, sx, sy = 0, w, h, 0, 0
        while ww > 1 or hh > 1:
            if ww > 1:
                mx |= ((xs >> sx) & 1) << bit; bit += 1; sx += 1; ww >>= 1
            if hh > 1:
                my |= ((ys >> sy) & 1) << bit; bit += 1; sy += 1; hh >>= 1
        _SWZ[key] = (my[:, None] | mx[None, :]).ravel()
    return _SWZ[key]


def to_rgba(tex, raw, w, h, palette):
    """Decode one mip to an RGBA uint8 array (h, w, 4)."""
    name, layout, bpp = FORMATS[tex.fmt]
    if layout == 'dxt':
        return np.asarray(dxt_image(tex.fmt, w, h, raw).convert('RGBA'))
    px = bpp // 8
    a = np.frombuffer(raw, np.uint8)[:w * h * px].reshape(-1, px)
    if layout == 'swz':
        a = a[swizzle_map(w, h)]
    out = np.empty((w * h, 4), np.uint8)
    f = tex.fmt
    if f in (0x00, 0x01, 0x13):                       # luminance
        out[:, :3] = a[:, :1]; out[:, 3] = 255
    elif f in (0x19, 0x1F):                           # alpha only
        out[:, :3] = 255; out[:, 3] = a[:, 0]
    elif f == 0x0B:                                   # palettised (normal map palette)
        if palette is None:
            out[:, :3] = a[:, :1]; out[:, 3] = 255
        else:
            out[:] = palette[a[:, 0]]
    elif f in (0x06, 0x12):                           # A8R8G8B8, stored B G R A
        out[:] = a[:, [2, 1, 0, 3]]
    elif f in (0x07, 0x1E):
        out[:, :3] = a[:, [2, 1, 0]]; out[:, 3] = 255
    elif f == 0x1A:                                   # A8L8: L then A
        out[:, :3] = a[:, :1]; out[:, 3] = a[:, 1]
    elif f in (0x2A, 0x2E):                           # depth/stencil: show depth as gray
        d = a.view('<u4').ravel() >> 8
        out[:, :3] = (d >> 16).astype(np.uint8)[:, None]; out[:, 3] = 255
    elif bpp == 16:
        v = a.view('<u2').ravel().astype(np.uint32)
        if f in (0x05, 0x11):
            r, g, b, al = (v >> 11) & 31, (v >> 5) & 63, v & 31, np.full_like(v, 1)
            out[:, 0], out[:, 1], out[:, 2] = r * 255 // 31, g * 255 // 63, b * 255 // 31
            out[:, 3] = 255
        elif f == 0x04:
            out[:, 0], out[:, 1], out[:, 2], out[:, 3] = [((v >> s) & 15) * 17 for s in (8, 4, 0, 12)]
        else:                                         # 1555
            out[:, 0], out[:, 1], out[:, 2] = [((v >> s) & 31) * 255 // 31 for s in (10, 5, 0)]
            out[:, 3] = np.where((v >> 15) & 1 | (f == 0x03), 255, 0)
    else:
        raise ValueError(f'unsupported format {name}')
    return out.reshape(h, w, 4)


def dds_header(fourcc, w, h, mips, linear, cube=False, rgba=False):
    DDSD = 0x1 | 0x2 | 0x4 | 0x1000 | 0x20000 | (0x80000 if not rgba else 0x8)
    caps = 0x1000 | (0x400008 if mips > 1 else 0) | (0x8 if cube else 0)
    caps2 = 0xFE00 if cube else 0
    if rgba:
        pf = struct.pack('<2I4s5I', 32, 0x41, b'\0\0\0\0', 32, 0xFF0000, 0xFF00, 0xFF, 0xFF000000)
    else:
        pf = struct.pack('<2I4s5I', 32, 0x4, fourcc, 0, 0, 0, 0, 0)
    return (b'DDS ' + struct.pack('<7I', 124, DDSD, h, w, linear, 0, mips) + b'\0' * 44 + pf
            + struct.pack('<4I4x', caps, caps2, 0, 0))


def dxt_image(fmt, w, h, raw):
    four = {0x0C: b'DXT1', 0x0E: b'DXT3', 0x0F: b'DXT5'}[fmt]
    bw, bh = max(1, (w + 3) // 4), max(1, (h + 3) // 4)
    n = bw * bh * (8 if fmt == 0x0C else 16)
    img = Image.open(io.BytesIO(dds_header(four, bw * 4, bh * 4, 1, n) + raw[:n]))
    img.load()
    return img.crop((0, 0, w, h)) if (bw * 4, bh * 4) != (w, h) else img


def write_dds(path, tex, faces, palette):
    """Keep DXT data compressed (all mips); other formats are expanded to 32-bit BGRA."""
    _, layout, _ = FORMATS[tex.fmt]
    with open(path, 'wb') as f:
        if layout == 'dxt':
            four = {0x0C: b'DXT1', 0x0E: b'DXT3', 0x0F: b'DXT5'}[tex.fmt]
            f.write(dds_header(four, tex.w, tex.h, tex.mips, len(faces[0][0]), tex.cube))
            for mips in faces:
                for m in mips:
                    f.write(m)
        else:
            f.write(dds_header(None, tex.w, tex.h, tex.mips, tex.w * 4, tex.cube, rgba=True))
            for mips in faces:
                for (w, h), m in zip(tex.mip_dims(), mips):
                    f.write(to_rgba(tex, m, w, h, palette)[:, :, [2, 1, 0, 3]].tobytes())


# ---- inputs ---------------------------------------------------------------------------------

def load_palette(xbe_path):
    if not xbe_path or not os.path.exists(xbe_path):
        return None
    d = open(xbe_path, 'rb').read()
    i = d.find(PALETTE_SIG)
    if i < 0:
        return None
    return np.frombuffer(d[i:i + 1024], np.uint8).reshape(256, 4)[:, [2, 1, 0, 3]].copy()


def default_xbe(near):
    d = os.path.abspath(near if os.path.isdir(near) else os.path.dirname(near))
    for _ in range(4):
        p = os.path.join(d, 'default.xbe')
        if os.path.exists(p):
            return p
        d = os.path.dirname(d)
    return None


def schemas_from_tgz(common_tgz):
    with tarfile.open(common_tgz) as t:
        blobs = [(m.name, t.extractfile(m).read()) for m in t
                 if m.isfile() and m.name.lower().endswith('.xsb')]
    return X.SchemaSet(blobs=blobs)


_SCHEMA_CACHE = {}


def libraries(path):
    """Yield (library_name, xmb_bytes, tex_bytes_loader, schemas) for an input path."""
    if os.path.isdir(path):
        tgzs = sorted(f for f in os.listdir(path) if f.lower().endswith('.tgz'))
        if tgzs:
            for f in tgzs:
                yield from libraries(os.path.join(path, f))
            return
    if path.lower().endswith('.tgz'):
        common = os.path.join(os.path.dirname(path), 'common.tgz')
        if common not in _SCHEMA_CACHE:
            _SCHEMA_CACHE[common] = schemas_from_tgz(common)
        schemas = _SCHEMA_CACHE[common]
        with tarfile.open(path) as t:
            members = {os.path.basename(m.name): m for m in t if m.isfile()}
            for n, m in sorted(members.items()):
                if n.startswith('textures-') and n.endswith('.xmb') and m.size:
                    texname = n[:-4] + '.tex'
                    if texname in members:
                        xmb = t.extractfile(m).read()
                        blob = t.extractfile(members[texname]).read()
                        yield n[:-4], xmb, blob, schemas
        return
    # loose textures-x.xmb next to its .tex
    schemas = X.SchemaSet.find(path)
    blob = open(path[:-4] + '.tex', 'rb').read()
    yield os.path.basename(path)[:-4], open(path, 'rb').read(), blob, schemas


# ---- commands ---------------------------------------------------------------------------------

def cmd_list(path):
    for lib, xmb, blob, schemas in libraries(path):
        texs, df = read_index(xmb, schemas, lib)
        print(f'== {lib}: {len(texs)} textures in {df} ({len(blob):,} bytes)')
        print(f'  {"name":12} {"format":13} {"size":>9} {"mips":>4} {"offset":>9} {"bytes":>8}')
        for t in texs:
            cube = ' cube' if t.cube else ''
            ref = f'  ref={t.ref}' if t.ref else ''
            print(f'  {t.label:12} {t.fname:13} {t.w:4}x{t.h:<4} {t.mips:4} {t.offset:9} {t.size:8}{cube}{ref}')


def cmd_export(path, out, dds, all_mips, xbe):
    palette = load_palette(xbe or default_xbe(path))
    if palette is None:
        print('warning: normal palette not found (pass --xbe default.xbe); P8 exported as raw indices',
              file=sys.stderr)
    total = errors = 0
    for lib, xmb, blob, schemas in libraries(path):
        texs, _ = read_index(xmb, schemas, lib)
        d = os.path.join(out, lib)
        os.makedirs(d, exist_ok=True)
        rows = []
        for t in texs:
            try:
                if t.fmt not in FORMATS:
                    raise ValueError(f'unknown format code {t.code}')
                faces = list(t.faces(blob))
                if dds:
                    write_dds(os.path.join(d, t.label + '.dds'), t, faces, palette)
                else:
                    levels = range(t.mips) if all_mips else range(1)
                    for lv in levels:
                        w, h = t.mip_dims()[lv]
                        imgs = [to_rgba(t, mips[lv], w, h, palette) for mips in faces]
                        img = np.concatenate(imgs, axis=1)       # cube faces +X -X +Y -Y +Z -Z
                        suffix = f'_mip{lv}' if all_mips else ''
                        Image.fromarray(img, 'RGBA').save(os.path.join(d, f'{t.label}{suffix}.png'))
                total += 1
                status = ''
            except Exception as e:                                 # keep going; report at end
                errors += 1
                status = f'error: {e}'
                print(f'  {lib}/{t.label}: {status}', file=sys.stderr)
            rows.append([t.label, f'{t.name:08x}', t.fname, t.code, t.w, t.h, t.mips,
                         'yes' if t.cube else '', t.offset, t.size, t.ref, status])
        with open(os.path.join(d, 'index.csv'), 'w', newline='', encoding='utf-8') as f:
            w = csv.writer(f)
            w.writerow(['name', 'hash', 'format', 'format_code', 'width', 'height', 'mips', 'cube',
                        'offset', 'size', 'reference', 'status'])
            w.writerows(rows)
        print(f'{lib}: {len(texs)} textures -> {d}')
    print(f'done: {total} exported, {errors} errors')


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest='cmd', required=True)
    p = sub.add_parser('list', help='list textures')
    p.add_argument('path')
    p = sub.add_parser('export', help='export textures to PNG (or DDS)')
    p.add_argument('path')
    p.add_argument('-o', '--out', required=True)
    p.add_argument('--dds', action='store_true', help='write .dds with all mips (DXT kept compressed)')
    p.add_argument('--mips', action='store_true', help='PNG: write every mip level')
    p.add_argument('--xbe', help='default.xbe for the bump-map palette (default: found next to data/)')
    a = ap.parse_args()
    if a.cmd == 'list':
        cmd_list(a.path)
    else:
        cmd_export(a.path, a.out, a.dds, a.mips, a.xbe)


if __name__ == '__main__':
    main()

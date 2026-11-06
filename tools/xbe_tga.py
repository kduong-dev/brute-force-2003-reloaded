"""Finds and extracts the TGA images embedded in default.xbe (the boot screen's sprites).

The boot screen (the "B" emblem with "Loading" at the screen's bottom left while the game data
loads) is blitted by FUN_00068170 from 24-bit TGA files compiled into .data: the emblem's top
(0x3f7f30, 64 x 32) and bottom (0x3f9760, 64 x 32), and the word in five languages
(64 x 20: 0x3faf90 Loading, 0x3fbec0 Cargando, 0x3fcdf0 Laden, 0x3fdd20 Caricamento,
0x3fec50 Chargement; the table at 0x3ffc18 picks one by XGetLanguage). FUN_00068290 draws
them at (48, 366), (48, 398) and (48, 418), every pixel opaque.

usage: python tools/xbe_tga.py scan [file.xbe]
           list every TGA header in the sections (type 2 or 10, 24 or 32 bits)
       python tools/xbe_tga.py extract OUT_DIR [ADDR[=label] ...] [--xbe file.xbe]
           write each TGA (every one found when no address is given) as
           OUT_DIR/xbe_<section>_<addr>[_<label>].png
       python tools/xbe_tga.py boot OUT_DIR [--xbe file.xbe]
           write the seven boot sprites with their names, and the English screen as drawn
           (xbe_boot_screen_64x84_en.png: top, word, bottom stacked)

Needs Pillow. Run from the repository root (the default XBE is Brute Force/default.xbe).
"""
import os
import struct
import sys

from PIL import Image

DEFAULT_XBE = os.path.join('Brute Force', 'default.xbe')

# The boot sprites: address, label (FUN_00068290; the language table at 0x3ffc18).
BOOT = [
    (0x3f7f30, 'emblem_top'),
    (0x3f9760, 'emblem_bottom'),
    (0x3faf90, 'loading_en'),
    (0x3fbec0, 'loading_es'),
    (0x3fcdf0, 'loading_de'),
    (0x3fdd20, 'loading_it'),
    (0x3fec50, 'loading_fr'),
]


class Xbe:
    """The XBE's bytes and its section table, for virtual-address reads."""

    def __init__(self, path):
        self.data = open(path, 'rb').read()
        d = self.data
        base = struct.unpack_from('<I', d, 0x104)[0]
        count = struct.unpack_from('<I', d, 0x11c)[0]
        headers = struct.unpack_from('<I', d, 0x120)[0] - base
        self.sections = []
        for i in range(count):
            _flags, va, vsize, raw, rsize, name_addr = struct.unpack_from('<6I', d, headers + i * 0x38)
            name = d[name_addr - base:d.index(b'\0', name_addr - base)].decode('ascii', 'replace')
            self.sections.append((name, va, vsize, raw, rsize))

    def section(self, va):
        """(name, file offset) of a virtual address, or None outside the file-backed bytes."""
        for name, v, _vsize, raw, rsize in self.sections:
            if v <= va < v + rsize:
                return name, raw + va - v
        return None


def tga_header(d, o):
    """The TGA header at file offset o as a dict, or None if it isn't a plain true-colour one."""
    if o + 18 > len(d):
        return None
    idlen, cmap, kind = d[o], d[o + 1], d[o + 2]
    if cmap != 0 or kind not in (2, 10) or d[o + 3:o + 8] != b'\0' * 5:
        return None
    xo, yo, w, h, bpp, desc = struct.unpack_from('<HHHHBB', d, o + 8)
    if xo or yo or not (4 <= w <= 1024 and 4 <= h <= 1024) or bpp not in (24, 32) or desc & 0xc0:
        return None
    if (desc & 0x0f) != (8 if bpp == 32 else 0):
        return None
    return dict(idlen=idlen, type=kind, w=w, h=h, bpp=bpp, desc=desc, size=18 + idlen + w * h * bpp // 8)


def scan(xbe):
    """Every (section, virtual address, header) that looks like an embedded TGA."""
    found = []
    for name, va, _vsize, raw, rsize in xbe.sections:
        d = xbe.data[raw:raw + rsize]
        for o in range(len(d) - 18):
            h = tga_header(d, o)
            if h and h['type'] == 2 and o + h['size'] <= len(d):
                found.append((name, va + o, h))
    return found


def image(xbe, va):
    """The TGA at a virtual address as a PIL image (RGB or RGBA), or None."""
    sec = xbe.section(va)
    if not sec:
        return None
    _name, o = sec
    h = tga_header(xbe.data, o)
    if not h or h['type'] != 2:
        return None
    start = o + 18 + h['idlen']
    px = xbe.data[start:start + h['w'] * h['h'] * h['bpp'] // 8]
    mode, raw = ('RGBA', 'BGRA') if h['bpp'] == 32 else ('RGB', 'BGR')
    im = Image.frombytes(mode, (h['w'], h['h']), px, 'raw', raw)
    if not h['desc'] & 0x20:  # bottom-left origin: rows are stored bottom first
        im = im.transpose(Image.FLIP_TOP_BOTTOM)
    return im


def main(argv):
    if len(argv) < 2 or argv[1] not in ('scan', 'extract', 'boot'):
        print(__doc__)
        return 1
    args = argv[2:]
    path = DEFAULT_XBE
    if '--xbe' in args:
        i = args.index('--xbe')
        path = args[i + 1]
        del args[i:i + 2]
    if len(args) == 1 and args[0].lower().endswith('.xbe') and argv[1] == 'scan':
        path, args = args[0], []
    xbe = Xbe(path)
    if argv[1] == 'scan':
        for name, va, h in scan(xbe):
            print(f"{name:10} {va:#x}  {h['w']} x {h['h']}  {h['bpp']}-bit  origin {'top' if h['desc'] & 0x20 else 'bottom'}-left  {h['size']} bytes")
        return 0
    out = args[0]
    os.makedirs(out, exist_ok=True)
    if argv[1] == 'boot':
        wanted = BOOT
    elif args[1:]:
        wanted = []
        for a in args[1:]:
            addr, _, label = a.partition('=')
            wanted.append((int(addr, 16), label))
    else:
        wanted = [(va, '') for _name, va, _h in scan(xbe)]
    for va, label in wanted:
        im = image(xbe, va)
        sec = xbe.section(va)
        if im is None or sec is None:
            print(f'{va:#x}: no TGA there')
            continue
        name = f"xbe_{sec[0].strip('.')}_{va:x}" + (f'_{label}' if label else '') + '.png'
        im.save(os.path.join(out, name))
        print(f'{va:#x} -> {name} ({im.width} x {im.height})')
    if argv[1] == 'boot':
        top, bottom, en = (image(xbe, BOOT[i][0]) for i in (0, 1, 2))
        screen = Image.new('RGB', (64, 84))
        screen.paste(top, (0, 0))
        screen.paste(en, (0, 32))
        screen.paste(bottom, (0, 52))
        screen.save(os.path.join(out, 'xbe_boot_screen_64x84_en.png'))
        print('-> xbe_boot_screen_64x84_en.png (as drawn at (48, 366) on the 640 x 480 screen)')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))

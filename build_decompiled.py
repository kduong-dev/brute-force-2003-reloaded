#!/usr/bin/env python3
"""
build_decompiled.py - regenerate decompiled/ from the English (US) disc in "Brute Force/".

  python build_decompiled.py              # everything
  python build_decompiled.py --skip-textures --skip-asm

Output (decompiled/):
  README.md
  xbe/         default.xbe report, hardware scan, strings, kernel imports, .text disassembly
  schemas/     the .xsb schemas as readable trees
  xml/<level>/ every .xmb decoded to .xml (English-only for localised files)
  scripts/<level>/  .gscr level scripts (plain text in the archives)
  textures/<level>/ every texture as PNG + index.csv
"""
import argparse
import contextlib
import io
import os
import re
import shutil
import sys
import tarfile
import time

import tex_tool
import xbe_tool
import xmb_tool as X

ROOT = os.path.dirname(os.path.abspath(__file__))
GAME = os.path.join(ROOT, 'Brute Force')
DATA = os.path.join(GAME, 'data')
OUT = os.path.join(ROOT, 'decompiled')
OTHER_LANG = re.compile(r'-(de|es|fr|it)\.xmb$', re.I)


def capture(fn, *args):
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        fn(*args)
    return buf.getvalue()


def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'w', encoding='utf-8', newline='\n') as f:
        f.write(text)


def level_archives():
    return sorted(os.path.join(DATA, f) for f in os.listdir(DATA) if f.lower().endswith('.tgz'))


def strip_level(name):
    """'e01/characters/x.xmb' -> 'characters/x.xmb' (archives nest everything in <level>/)."""
    parts = name.replace('\\', '/').split('/')
    return '/'.join(parts[1:]) if len(parts) > 1 else parts[0]


# ---- sections ------------------------------------------------------------------------

def build_xbe(asm):
    xbe = os.path.join(GAME, 'default.xbe')
    x = xbe_tool.XBE(xbe)
    d = os.path.join(OUT, 'xbe')
    write(os.path.join(d, 'default.info.txt'), capture(xbe_tool.cmd_info, x, False))
    write(os.path.join(d, 'default.info.json'), capture(xbe_tool.cmd_info, x, True))
    write(os.path.join(d, 'default.hwscan.txt'), capture(xbe_tool.cmd_hwscan, x, 3))
    write(os.path.join(d, 'default.strings.txt'),
          ''.join(f'{va:#010x} {sec:9} {s}\n' for sec, va, s in xbe_tool.iter_strings(x, 5)))
    write(os.path.join(d, 'kernel_imports.txt'),
          ''.join(f'{o:4} {n}\n' for o, n in sorted(x.kimports)))
    if asm:
        import capstone
        md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_32)
        md.skipdata = True
        thunk = {x.kthunk_addr + 4 * i: n for i, (_, n) in enumerate(x.kimports)}
        s = next(s for s in x.sections if s['name'] == '.text')
        with open(os.path.join(d, 'default.text.asm'), 'w', encoding='utf-8', newline='\n') as f:
            f.write(f'; default.xbe .text  VA {s["va"]:#x}  ({s["rsize"]:,} bytes), capstone linear sweep\n'
                    f'; kernel thunk slots are annotated with their xboxkrnl export name\n')
            for a, _, m, o in md.disasm_lite(x.section_bytes(s), s['va']):
                note = ''
                hit = re.search(r'\[(0x[0-9a-f]+)\]', o)
                if hit and int(hit.group(1), 16) in thunk:
                    note = f'    ; {thunk[int(hit.group(1), 16)]}'
                f.write(f'{a:08x}: {m} {o}{note}\n')
    print('  xbe done')


def build_schemas(schema_blobs):
    d = os.path.join(OUT, 'schemas')
    for name, blob in schema_blobs:
        s = X.Schema(name, blob)
        q = lambda t: '-' if t is None else ':'.join(X.hname(h) for h in t)
        lines = [f'targetNamespace {X.hname(s.target)}']
        lines += [f'  xmlns:{X.hname(p) or "(default)"} = {X.hname(u)}' for p, u in s.namespaces]
        lines += ['', '# [kind] name : type    kinds: 0x1 complexType, 0x101 complexType extends base,',
                  '#   0x2 simpleType/simple element, 0x6 list type, 0x9 local element, 0xa attribute,',
                  '#   0x19 element of named type, 0x80 property block', '']

        def show(n, depth):
            lines.append('  ' * depth + f'[{n.kind:#x}] {q(n.name)} : {q(n.type)}')
            for c in n.children:
                show(c, depth + 1)
        for n in s.nodes:
            show(n, 0)
        write(os.path.join(d, os.path.basename(name)[:-4] + '.txt'), '\n'.join(lines) + '\n')
    print(f'  schemas: {len(schema_blobs)}')


def build_xml_and_scripts(schemas):
    n_xml = n_scr = n_skip = 0
    failures = []
    for tgz in level_archives():
        level = os.path.basename(tgz)[:-4]
        with tarfile.open(tgz) as t:
            for m in t:
                if not m.isfile() or not m.size:
                    continue
                rel = strip_level(m.name)
                low = rel.lower()
                if low.endswith('.xmb'):
                    if OTHER_LANG.search(low):
                        n_skip += 1
                        continue
                    data = t.extractfile(m).read()
                    try:
                        b = X.BXML(m.name, schemas, data=data)
                        root = b.parse()
                        out = ['<?xml version="1.0" encoding="UTF-8"?>']
                        X.to_xml(root, out)
                        write(os.path.join(OUT, 'xml', level, rel[:-4] + '.xml'), '\n'.join(out) + '\n')
                        n_xml += 1
                    except Exception as e:
                        failures.append(f'{level}/{rel}: {e}')
                elif low.endswith('.gscr'):
                    write(os.path.join(OUT, 'scripts', level, rel),
                          t.extractfile(m).read().decode('latin-1'))
                    n_scr += 1
    print(f'  xml: {n_xml} files ({n_skip} non-English skipped, {len(failures)} failed); scripts: {n_scr}')
    for f in failures:
        print('   FAIL', f)


def build_characters():
    import char_render
    import gltf_export
    game = char_render.Game()
    d = os.path.join(OUT, 'characters')
    os.makedirs(d, exist_ok=True)
    for name in sorted(game.characters):
        ch = char_render.Character(game, name)
        n, _ = gltf_export.export(ch, os.path.join(d, name + '.glb'))
        v = ch.skin(ch.pose())
        cams = [char_render.frame_camera([v], y) for y in (3.14159, 4.71239, 0.0)]
        tiles = [char_render.render(ch, v, c, 320) for c in cams]
        from PIL import Image
        sheet = Image.new('RGB', (960, 320))
        for i, t in enumerate(tiles):
            sheet.paste(t, (i * 320, 0))
        sheet.save(os.path.join(d, name + '_bind.png'))
        print(f'  character {name}: {len(ch.bones)} bones, {n} animations')


def build_textures():
    with contextlib.redirect_stdout(io.StringIO()):
        tex_tool.cmd_export(DATA, os.path.join(OUT, 'textures'), False, False,
                            os.path.join(GAME, 'default.xbe'))
    # tex_tool names folders textures-<level>; keep just <level>
    d = os.path.join(OUT, 'textures')
    for name in os.listdir(d):
        if name.startswith('textures-'):
            dst = os.path.join(d, name[len('textures-'):])
            if os.path.exists(dst):
                shutil.rmtree(dst)
            os.rename(os.path.join(d, name), dst)
    n = sum(len([f for f in fs if f.endswith('.png')]) for _, _, fs in os.walk(d))
    print(f'  textures: {n} png')


README = '''# Brute Force (Xbox, 2003) - decompiled data, English/US disc

Generated by `build_decompiled.py` from `Brute Force/` (the US disc). Re-run it after the tools
improve; everything here is derived output, the originals are untouched.

| Folder | What | Made by |
|---|---|---|
| `xbe/` | `default.xbe` analysis: header/sections/XDK libraries/kernel imports (`default.info.*`), hardware-access scan, strings, and a full disassembly of the game code (`default.text.asm`, kernel calls annotated) | `xbe_tool.py` + capstone |
| `schemas/` | The 7 BXSD schemas (`common/schemas/*.xsb`) as readable type trees | `xmb_tool.py` |
| `xml/<level>/` | Every BXML `.xmb` file decoded to XML, keeping the archive's folder layout. Localised files: English only | `xmb_tool.py` |
| `scripts/<level>/` | Level `.gscr` scripts (already plain text in the archives) | copied |
| `characters/` | Brutus, Flint, Hawk and Tex as glTF 2.0 (`.glb`): textured skinned mesh, full skeleton and every animation in their motion set (names are hashes), plus a bind-pose preview PNG. Opens in Blender, three.js, Windows 3D Viewer, etc. | `char_render.py` + `gltf_export.py` |
| `textures/<level>/` | Every texture as PNG (top mip) plus `index.csv` (format, size, mips, offset). P8 bump maps are converted to normal maps with the palette from `default.xbe`; cube maps are 6 faces side by side (+X -X +Y -Y +Z -Z) | `tex_tool.py` |

`<level>` is the archive name in `Brute Force/data/` (`common`, `e01`, `m01_a`, `mp1`, `sdm_e01`, `tutorial`, ...).

## Reading the XML

* Names are stored as hashes in the game files. Recovered names print normally; the rest print as
  `h_xxxxxxxx` (the hash). Add guesses to `xmb_names.txt` (one per line) and re-run to fill them in.
* `stringid` values (object, texture and material names) are hashes too, so materials refer to
  textures as e.g. `texture-name="h_021fd89f"`, which is the file `textures/<level>/h_021fd89f.png`.
* Vectors and other lists are space-separated (`value="1 1 1"`).
* `<h_1b8535f0 ...>` is the game's generic property block; its attributes are the object's fields.

## Not here yet

* Static level meshes / props (`objects-*.ivd`; only characters are exported), collision
  (`.ipn`), effects (`.ale`), sound banks
  (`.xwb`/`.mem`, see `xwb_tool.py`) and movies (`.bik`) are not converted yet.
'''


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--skip-textures', action='store_true')
    ap.add_argument('--skip-asm', action='store_true', help='skip the (large) .text disassembly')
    ap.add_argument('--skip-characters', action='store_true')
    a = ap.parse_args()
    os.makedirs(OUT, exist_ok=True)
    t0 = time.time()
    with tarfile.open(os.path.join(DATA, 'common.tgz')) as t:
        blobs = [(m.name, t.extractfile(m).read()) for m in t
                 if m.isfile() and m.name.lower().endswith('.xsb')]
    schemas = X.SchemaSet(blobs=blobs)
    write(os.path.join(OUT, 'README.md'), README)
    build_xbe(not a.skip_asm)
    build_schemas(blobs)
    build_xml_and_scripts(schemas)
    if not a.skip_textures:
        build_textures()
    if not a.skip_characters:
        build_characters()
    print(f'done in {time.time() - t0:.0f}s -> {OUT}')


if __name__ == '__main__':
    main()

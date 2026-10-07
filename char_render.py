#!/usr/bin/env python3
"""
char_render.py - render a Brute Force character (skinned mesh + textures) playing an animation.

  python char_render.py list                                   # characters in common.tgz
  python char_render.py anims brutus                           # that character's animations
  python char_render.py render brutus --anim 5 -o brutus.gif   # animated GIF
  python char_render.py render brutus --anim all -o out/       # every animation, one GIF each
  python char_render.py render brutus --pose bind -o bind.png  # bind (T) pose, front/side/back
  python char_render.py uv-check flint --texture h_e6ee3065    # overlay UVs on a texture

Characters are looked up in data/common.tgz (Brutus, Flint, Hawk, Tex) and, with --level,
in a level archive (e.g. --level e01 for the militia / mutants there).

How it works (all reverse-engineered, see decompiled/README.md and the tools):
  objecttypes   character -> motion set (ms_<name>) + compound archetype
  objects       compound archetype: PART list (bones), joints (parent-part, parent-point),
                per-part inverse bind matrix (local = R @ v + t), LOD skin meshes
  objects.ivd   skinned vertices, 32 bytes: pos f32x3, packed normal, uv f32x2,
                4 weights u8 (sum 255), 4 bone indices u8 (= 1 + 5*palette slot);
                indices as raw u16 strips/lists or NV2A push buffers
  animations    archetype-set <name>: animations -> target bone -> channel in .chnl
  .chnl         quaternion channels: 3 x int16 (/32767), w = sqrt(1-x^2-y^2-z^2);
                keyed channels prefix each frame with a f32 time; root channels also
                carry a f32x3 position. Rotations are full local rotations (child rows =
                q * parent rows), child origin = parent origin + parent-point in parent frame.
Skin textures: a skinned mesh's material (type h_eb58ab52) has no textures itself; the next
material in the library holds them (verified against UV layouts with `uv-check`).
"""
import argparse
import os
import struct
import sys
import tarfile

import numpy as np
from PIL import Image, ImageDraw

import tex_tool
import xmb_tool as X

ROOT = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(ROOT, 'Brute Force', 'data')
H = X.name_hash

# Skinned-mesh materials (type h_eb58ab52) carry no textures; each is followed in the material
# library by the material that does (constant h_edca4b16 = how many follow, always 1).
SKIN_WRAPPER = 0xEB58AB52
H_WRAP_COUNT = 0xEDCA4B16
SKINS = {}                       # optional manual material -> texture overrides

DT_QUAT, DT_VEC, DT_XFORM = 0x06677BCB, 0xF36D8810, 0x06901441
FT_KEYED = 0x00AD833C
LIST_PRIM = 0x0E7AB726                         # raw index buffer type hash meaning "triangle list"
NEUTRAL_FACE = 0xEFF8AC8D                      # first clip of every <name>face set: closed, relaxed mouth


# ---- archive access ---------------------------------------------------------------------

class Archive:
    """Members of a level .tgz, by base name."""

    def __init__(self, path):
        self.path = path
        self.t = tarfile.open(path)
        self.members = {os.path.basename(m.name): m for m in self.t if m.isfile() and m.size}
        self.cache = {}

    def get(self, name):
        if name not in self.cache:
            m = self.members.get(name)
            self.cache[name] = self.t.extractfile(m).read() if m else None
        return self.cache[name]

    def find(self, prefix, ext):
        return [n for n in self.members if n.startswith(prefix) and n.endswith(ext)]


def A(e):
    return {X.hname(k): v for k, v in e.attrs}


def walk(e):
    yield e
    for c in e.children:
        yield from walk(c)


def tagged(e, name):
    return [c for c in e.children if X.hname(c.name) == name]


def qmat(q):
    x, y, z, w = q
    return np.array([[1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
                     [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
                     [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)]])


# ---- game data ----------------------------------------------------------------------------

class Game:
    def __init__(self, level=None, xbe=None):
        self.archives = [Archive(os.path.join(DATA, 'common.tgz'))]
        if level:
            p = level if level.endswith('.tgz') else os.path.join(DATA, level + '.tgz')
            self.archives.insert(0, Archive(p))
        common = self.archives[-1]
        self.schemas = X.SchemaSet(blobs=[(n, common.get(n)) for n in common.members if n.endswith('.xsb')])
        self.palette = tex_tool.load_palette(xbe or os.path.join(ROOT, 'Brute Force', 'default.xbe'))
        self.objects = []        # (root element, ivd bytes)
        self.materials = {}
        self.textures = {}       # hash -> (Tex, blob)
        self.channels = {}       # name -> (attrs, chnl bytes)
        self.anim_sets = {}      # name hash -> archetype-set element
        self.characters = {}     # name -> (compound hash, motion set hash)
        for ar in self.archives:
            self._load(ar)

    def parse(self, ar, name):
        data = ar.get(name)
        return X.BXML(name, self.schemas, data=data).parse() if data else None

    def _load(self, ar):
        for n in ar.find('objects-', '.xmb'):
            root = self.parse(ar, n)
            self.objects.append((root, ar.get(n[:-4] + '.ivd')))
        for n in ar.find('materials-', '.xmb'):
            seq = []
            for m in walk(self.parse(ar, n)):
                if X.hname(m.name) == 'Material':
                    a = dict(m.attrs)
                    tex = {A(t)['param-id']: A(t)['texture-name'] for t in walk(m) if X.hname(t.name) == 'texture'}
                    consts = {A(c).get('param-id'): A(c).get('value') for c in walk(m) if X.hname(c.name) == 'constant'}
                    seq.append((a[H('name')], a.get(H('Type')), tex, consts))
            for i, (name, typ, tex, consts) in enumerate(seq):
                if typ == SKIN_WRAPPER and not tex:
                    # skinned-mesh wrapper: its render material(s) are the next N entries in the
                    # library (N = constant h_edca4b16, always 1 on the disc)
                    count = int(consts.get(H_WRAP_COUNT) or 1)
                    for nxt in seq[i + 1:i + 1 + count]:
                        if nxt[2]:
                            tex = nxt[2]
                            break
                self.materials.setdefault(name, tex)
        for n in ar.find('textures-', '.xmb'):
            blob = ar.get(n[:-4] + '.tex')
            texs, _ = tex_tool.read_index(ar.get(n), self.schemas, n)
            for t in texs:
                self.textures.setdefault(t.name, (t, blob))
        for n in ar.find('animations-', '.xmb'):
            root = self.parse(ar, n)
            chnl = ar.get(n[:-4] + '.chnl')
            for e in walk(root):
                a = A(e)
                if 'num-frames' in a:
                    self.channels.setdefault(a['name'], (a, chnl))
                if X.hname(e.name) == 'archetype-set':
                    self.anim_sets.setdefault(A(e.children[0])['name'], e)
        for n in ar.find('objecttypes', '.xmb'):
            for e in walk(self.parse(ar, n)):
                a = A(e)
                if 'motion-archetype' in a:
                    base = [c for c in e.children if X.hname(c.name) == 'base']
                    if base:
                        mesh = A(base[0]).get('mesh-name')
                        ms = a['motion-archetype']
                        key = ms[3:] if isinstance(ms, str) and ms.startswith('ms_') else str(ms)
                        self.characters.setdefault(key, (mesh, H(key)))

    # -- lookups
    def archetype(self, h):
        for root, ivd in self.objects:
            for lib in tagged(root, 'archetype-library'):
                for a in tagged(lib, 'archetype'):
                    if dict(a.attrs).get(H('name')) == h:
                        return a, root, ivd
        raise KeyError(f'archetype {X.hname(h)} not found')

    def mesh(self, h):
        for root, ivd in self.objects:
            for lib in tagged(root, 'mesh-library'):
                for m in tagged(lib, 'mesh'):
                    if dict(m.attrs).get(H('name')) == h:
                        return m, ivd
        raise KeyError(f'mesh {X.hname(h)} not found')

    def _find_texture(self, h):
        """Character skins live in every level's texture library rather than in common:
        open other level archives (textures only) until the texture turns up."""
        if h in self.textures:
            return
        if not hasattr(self, '_tex_fallback'):
            loaded = {os.path.abspath(a.path) for a in self.archives}
            self._tex_fallback = [os.path.join(DATA, f) for f in sorted(os.listdir(DATA))
                                  if f.endswith('.tgz') and os.path.abspath(os.path.join(DATA, f)) not in loaded]
        while self._tex_fallback and h not in self.textures:
            ar = Archive(self._tex_fallback.pop(0))
            for n in ar.find('textures-', '.xmb'):
                texs, _ = tex_tool.read_index(ar.get(n), self.schemas, n)
                blob = ar.get(n[:-4] + '.tex')
                for t in texs:
                    self.textures.setdefault(t.name, (t, blob))

    def texture_rgba(self, h, cache={}):
        if h not in cache:
            self._find_texture(h)
            ent = self.textures.get(h)
            if ent is None:
                cache[h] = None
            else:
                t, blob = ent
                cache[h] = tex_tool.to_rgba(t, next(t.faces(blob))[0], t.w, t.h, self.palette)
        return cache[h]


# ---- character model ---------------------------------------------------------------------------

class Character:
    def __init__(self, game, name, lod=0, overrides=None):
        self.game = game
        if name not in game.characters:
            raise SystemExit(f'unknown character {name!r}; try: {", ".join(sorted(game.characters))}')
        self.name = name
        compound, self.set_hash = game.characters[name]
        arch, root, _ = game.archetype(compound)
        self._skeleton(arch)
        meshes = [A(m)['mesh-name'] for m in walk(arch) if X.hname(m.name) == 'mesh' and 'mesh-name' in A(m)]
        mesh_el, self.ivd = game.mesh(meshes[min(lod, len(meshes) - 1)])
        self.palettes = [list(p.children[0].text) for p in tagged(mesh_el, 'h_f1330520')]
        self.geosets = [self._geoset(g) for g in walk(mesh_el) if X.hname(g.name) == 'geoset']
        self._attach_rigid()
        self.overrides = overrides or {}

    def _skeleton(self, arch):
        g = self.game
        self.bones = [A(p)['part-name'] for p in walk(arch) if X.hname(p.name) == 'PART']
        part_arch = {A(p)['part-name']: A(p)['archetype-name'] for p in walk(arch) if X.hname(p.name) == 'PART'}
        self.bind = {}
        for b in self.bones:
            pa, _, _ = g.archetype(part_arch[b])
            v = np.array(tagged(pa, 'h_e4b822af')[0].text, float)
            R, t = v[:9].reshape(3, 3), v[9:12]          # inverse bind: local = R @ v + t
            self.bind[b] = (R, -t @ R)                  # bone -> model (row vectors)
        self.parent, self.ppoint, self.cpoint = {}, {}, {}
        for j in walk(arch):
            if X.hname(j.name) == 'h_151ace78':
                a = A(j)
                c = a['child-part']
                self.parent[c] = a['parent-part']
                vals = {X.hname(k.name): k.text for k in j.children[0].children}
                self.ppoint[c] = np.array(vals.get('parent-point', [0, 0, 0]), float)
                self.cpoint[c] = np.array(vals.get('child-point', [0, 0, 0]), float)
        # free face joints store both points as zero: take the offset from the bind matrices
        # (otherwise they collapse onto the head bone and cave in the forehead)
        for c, p in self.parent.items():
            if not self.ppoint[c].any() and not self.cpoint[c].any():
                self.ppoint[c] = (self.bind[c][1] - self.bind[p][1]) @ self.bind[p][0].T

    def _pushbuffer(self, o):
        d = self.ivd
        idx, prim = [], None
        while True:
            v = struct.unpack_from('<I', d, o)[0]; o += 4
            cnt, m = (v >> 18) & 0x7FF, v & 0x1FFC
            args = struct.unpack_from(f'<{cnt}I', d, o); o += 4 * cnt
            if m == 0x17FC:
                if args[0] == 0:
                    return prim, idx
                prim = args[0]
            elif m == 0x1800:
                for a in args:
                    idx += (a & 0xFFFF, a >> 16)
            elif m == 0x1808:
                idx += args

    @staticmethod
    def _triangles(is_list, idx):
        idx = np.asarray(idx)
        if is_list:
            return idx[:len(idx) // 3 * 3].reshape(-1, 3)
        out = []
        for i in range(len(idx) - 2):
            a, b, c = idx[i], idx[i + 1], idx[i + 2]
            if a != b and b != c and a != c:
                out.append((a, b, c) if i % 2 == 0 else (b, a, c))
        return np.array(out)

    def _geoset(self, g):
        rec = A(next(e for e in walk(g) if X.hname(e.name) == 'h_fde50e9f'))
        cnt = rec['h_0fdf830e']
        if cnt >> 16:                                  # NV2A push buffer: size << 16 | index count
            prim, idx = self._pushbuffer(rec['h_fbb881da'])
            is_list = prim == 5
        else:
            idx = struct.unpack_from(f'<{cnt & 0xFFFF}H', self.ivd, rec['h_fbb881da'])
            is_list = rec['h_1a7eec9f'] == LIST_PRIM
        nv, vb = rec['h_0c368bc2'], rec['h_0cdffa25']
        raw = np.frombuffer(self.ivd, np.uint8, nv * 32, vb).reshape(nv, 32)
        skinned = rec.get('h_e80dae68', 0) > 0         # max weights per vertex; 0 = rigid geoset
        return dict(pos=raw[:, :12].copy().view('<f4').reshape(nv, 3),
                    uv=raw[:, 16:24].copy().view('<f4').reshape(nv, 2),
                    w=raw[:, 24:28] / 255.0, slot=(raw[:, 28:32].astype(int) - 1) // 5,
                    tris=self._triangles(is_list, idx), pal=rec['h_0bdc6e74'], skinned=skinned, vb=vb,
                    rigid_bone=None, mat=dict(g.attrs)[H('material-name')])

    def _attach_rigid(self):
        """Rigid geosets (hair, helmets) carry no bone data here; attach each to the bone that
        dominates the nearest skinned vertices."""
        pts, bones = [], []
        for g in self.geosets:
            if not g['skinned']:
                continue
            pal = self.palettes[g['pal']]
            k = g['w'].argmax(1)
            slots = g['slot'][np.arange(len(k)), k]
            pts.append(g['pos']); bones.append([pal[s] if 0 <= s < len(pal) else 0 for s in slots])
        if not pts:
            return
        pts = np.concatenate(pts); bones = np.concatenate(bones)
        for g in self.geosets:
            if g['skinned']:
                continue
            votes = []
            for chunk in np.array_split(g['pos'], max(1, len(g['pos']) // 256)):
                d = ((chunk[:, None, :] - pts[None, :, :]) ** 2).sum(2)
                votes += list(bones[d.argmin(1)])
            g['rigid_bone'] = int(np.bincount(votes).argmax())

    def texture_for(self, gi, g):
        h = self.overrides.get(gi)
        if h is None:
            h = self.game.materials.get(g['mat'], {}).get(H('color'))
        if h is None:
            h = SKINS.get(g['mat'])
        return self.game.texture_rgba(h) if h is not None else None

    # -- animation
    def animations(self):
        s = self.game.anim_sets.get(self.set_hash)
        return tagged(s, 'archetype') if s is not None else []

    def face_animations(self):
        """The '<name>face' set: lip-sync / expression poses for the face bones. The bind pose
        of human faces is modelled mid-speech, so the game always layers one of these."""
        s = self.game.anim_sets.get(H(self.name + 'face'))
        return tagged(s, 'archetype') if s is not None else []

    def default_face(self):
        faces = self.face_animations()
        names = [A(f.children[0]).get('name') for f in faces]
        return names.index(NEUTRAL_FACE) if NEUTRAL_FACE in names else (0 if faces else None)

    def _sample(self, ch, t):
        a, chnl = self.game.channels[ch]
        n = a['num-frames']; per = a['size'] // n
        raw = chnl[a['offset']:a['offset'] + a['size']]
        if a['frame-type'] == FT_KEYED:
            times = [struct.unpack_from('<f', raw, i * per)[0] for i in range(n)]
            i = max(0, min(int(np.searchsorted(times, t, 'right')) - 1, n - 1)); j = min(i + 1, n - 1)
            f = 0.0 if j == i else float(np.clip((t - times[i]) / max(times[j] - times[i], 1e-9), 0, 1))
            off = 4
        else:
            x = t / a.get('h_f63f0f49', 1 / 30)
            i = max(0, min(int(x), n - 1)); j = min(i + 1, n - 1); f = float(np.clip(x - i, 0, 1))
            off = 0
        fa, fb = raw[i * per + off:(i + 1) * per], raw[j * per + off:(j + 1) * per]

        def quat(b):
            q = np.array(struct.unpack('<3h', b)) / 32767.0
            return np.array([*q, np.sqrt(max(0.0, 1 - q @ q))])

        def nlerp(p, q):
            if p @ q < 0:
                q = -q
            r = p * (1 - f) + q * f
            return r / np.linalg.norm(r)
        dt = a['data-type']
        if dt == DT_QUAT:
            return 'q', nlerp(quat(fa), quat(fb)), None
        if dt == DT_VEC:
            return 'v', None, np.array(struct.unpack('<3f', fa)) * (1 - f) + np.array(struct.unpack('<3f', fb)) * f
        if dt == DT_XFORM:
            p = np.array(struct.unpack('<3f', fa[:12])) * (1 - f) + np.array(struct.unpack('<3f', fb[:12])) * f
            return 'qv', nlerp(quat(fa[12:18]), quat(fb[12:18])), p
        return None, None, None

    def pose(self, anim=None, t=0.0, root_motion=False, face='default', face_t=0.0):
        """face: 'default' (neutral expression), None (raw bind face) or an index into
        face_animations(); the face clip loops on its own clock face_t."""
        targets = {}                                    # bone -> (channel, time)
        if anim is not None:
            for tg in tagged(anim, 'target'):
                a = A(tg.children[0])
                if a.get('data-channel') in self.game.channels:
                    targets[a['name']] = (a['data-channel'], t)
        if face == 'default':
            face = self.default_face()
        faces = self.face_animations()
        if face is not None and 0 <= face < len(faces):
            fa = faces[face]
            dur = max(A(fa.children[0]).get('duration', 0) or 0, 1e-6)
            for tg in tagged(fa, 'target'):
                a = A(tg.children[0])
                if a.get('data-channel') in self.game.channels and a['name'] in self.parent:
                    # the neutral face is a two-key 0.03 s pose: hold its first key
                    targets[a['name']] = (a['data-channel'], 0.0 if dur < 0.1 else face_t % dur)
        world = {}
        for b in self.bones:                            # PART order is parent-first
            Rb, pb = self.bind[b]
            kind, q, v = self._sample(*targets[b]) if b in targets else (None, None, None)
            if b not in self.parent:
                R, p = Rb, pb
                if kind == 'qv':
                    R = qmat(q).T
                    p = v if root_motion else np.array([0.0, v[1], 0.0])
                world[b] = (R, p)
                continue
                # noqa
            Rp, pp = world[self.parent[b]]
            L = Rb @ self.bind[self.parent[b]][0].T
            off = self.ppoint[b]
            if kind == 'q':
                L = qmat(q).T
            elif kind == 'v':
                off = v
            R = L @ Rp
            world[b] = (R, pp + off @ Rp - self.cpoint[b] @ R)
        return world

    def skin(self, world):
        mats = []
        for b in self.bones:
            Rb, pb = self.bind[b]; Rw, pw = world[b]
            M = Rb.T @ Rw
            mats.append((M, pw - pb @ M))
        out = []
        for g in self.geosets:
            if not g['skinned']:
                M, T = mats[g['rigid_bone'] or 0]
                out.append(g['pos'] @ M + T)
                continue
            pal = self.palettes[g['pal']] if 0 <= g['pal'] < len(self.palettes) else None
            v = np.zeros_like(g['pos'])
            for k in range(4):
                w = g['w'][:, k:k + 1]
                if not w.any():
                    continue
                bones = [pal[s] if pal and 0 <= s < len(pal) else 0 for s in g['slot'][:, k]]
                M = np.stack([mats[b][0] for b in bones]); T = np.stack([mats[b][1] for b in bones])
                v += w * (np.einsum('ni,nij->nj', g['pos'], M) + T)
            out.append(v)
        return out


# ---- software rasterizer ------------------------------------------------------------------------

def frame_camera(vert_sets, yaw):
    pts = np.concatenate([np.concatenate(v) for v in vert_sets])
    c = np.cos(yaw); s = np.sin(yaw)
    R = np.array([[c, 0, s], [0, 1, 0], [-s, 0, c]])
    p = pts @ R.T
    lo, hi = p.min(0), p.max(0)
    return R, (lo + hi) / 2, (hi - lo)[:2].max() * 1.08


def render(char, verts, cam, size=384, bg=(0.17, 0.19, 0.23)):
    R, center, ext = cam
    img = np.empty((size, size, 3), np.float32); img[:] = bg
    zb = np.full((size, size), np.inf, np.float32)
    light = np.array([0.35, 0.75, -0.55]); light /= np.linalg.norm(light)
    for gi, (g, v) in enumerate(zip(char.geosets, verts)):
        p = v @ R.T - center
        sx = (p[:, 0] / ext + 0.5) * size
        sy = (0.5 - p[:, 1] / ext) * size
        z = -p[:, 2]
        tex = char.texture_for(gi, g)
        uv = g['uv']
        for a, b, c in g['tris']:
            x = (sx[a], sx[b], sx[c]); y = (sy[a], sy[b], sy[c])
            area = (x[1] - x[0]) * (y[2] - y[0]) - (x[2] - x[0]) * (y[1] - y[0])
            if abs(area) < 1e-9:
                continue
            x0, x1 = max(int(min(x)), 0), min(int(max(x)) + 1, size)
            y0, y1 = max(int(min(y)), 0), min(int(max(y)) + 1, size)
            if x0 >= x1 or y0 >= y1:
                continue
            X_, Y_ = np.meshgrid(np.arange(x0, x1) + 0.5, np.arange(y0, y1) + 0.5)
            w0 = ((x[1] - X_) * (y[2] - Y_) - (x[2] - X_) * (y[1] - Y_)) / area
            w1 = ((x[2] - X_) * (y[0] - Y_) - (x[0] - X_) * (y[2] - Y_)) / area
            w2 = 1 - w0 - w1
            m = (w0 >= 0) & (w1 >= 0) & (w2 >= 0)
            if not m.any():
                continue
            zz = w0 * z[a] + w1 * z[b] + w2 * z[c]
            sub = zb[y0:y1, x0:x1]
            m &= zz < sub
            if not m.any():
                continue
            sub[m] = zz[m]
            n = np.cross(p[b] - p[a], p[c] - p[a]); n /= np.linalg.norm(n) + 1e-12
            shade = 0.35 + 0.65 * abs(n @ light)
            if tex is not None:
                u = (w0 * uv[a, 0] + w1 * uv[b, 0] + w2 * uv[c, 0])[m]
                vv = (w0 * uv[a, 1] + w1 * uv[b, 1] + w2 * uv[c, 1])[m]
                th, tw = tex.shape[:2]
                col = tex[np.floor(vv * th).astype(int) % th, np.floor(u * tw).astype(int) % tw, :3] / 255.0
            else:
                col = np.array([0.72, 0.72, 0.72])
            img[y0:y1, x0:x1][m] = col * shade
    return Image.fromarray((np.clip(img, 0, 1) * 255).astype(np.uint8))


# ---- commands ------------------------------------------------------------------------------------

def anim_label(i, anim):
    a = A(anim.children[0])
    return f'{i:3}  {X.hname(a["name"]):12} {a.get("duration", 0):6.2f}s'


def render_anim(char, anim, out, size, fps, yaw, root_motion, face='default'):
    dur = A(anim.children[0]).get('duration', 1.0) if anim is not None else 0
    n = max(1, int(round(dur * fps)))
    poses = [char.skin(char.pose(anim, dur * k / n, root_motion, face, dur * k / n)) for k in range(n)]
    cam = frame_camera(poses, yaw)                    # fixed camera for the whole clip
    frames = [render(char, v, cam, size) for v in poses]
    frames[0].save(out, save_all=True, append_images=frames[1:], duration=int(1000 / fps), loop=0)
    return n


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--level', help='also load a level archive (e.g. e01) for its characters')
    ap.add_argument('--xbe', help='default.xbe (bump palette); default: Brute Force/default.xbe')
    sub = ap.add_subparsers(dest='cmd', required=True)
    sub.add_parser('list', help='list characters')
    p = sub.add_parser('anims', help="list a character's animations"); p.add_argument('character')
    p = sub.add_parser('render', help='render an animation (GIF) or the bind pose (PNG)')
    p.add_argument('character')
    p.add_argument('--anim', default='0', help="animation index, hash name, or 'all'")
    p.add_argument('--pose', choices=['anim', 'bind'], default='anim')
    p.add_argument('-o', '--out', required=True, help='output .gif/.png, or a directory with --anim all')
    p.add_argument('--size', type=int, default=384)
    p.add_argument('--fps', type=int, default=15)
    p.add_argument('--yaw', type=float, default=200, help='camera angle in degrees (180 = front)')
    p.add_argument('--lod', type=int, default=0)
    p.add_argument('--root-motion', action='store_true', help='let the root translate (default: in place)')
    p.add_argument('--face', default='default',
                   help="facial pose: 'default' (neutral), 'none' (raw bind face), or a face clip index (see anims)")
    p.add_argument('--texture', action='append', default=[], metavar='GEOSET=HASH',
                   help='force a texture on a geoset, e.g. 3=h_e6ee3065 (repeatable)')
    p = sub.add_parser('gltf', help='export mesh + skeleton + textures + animations to .glb')
    p.add_argument('character')
    p.add_argument('-o', '--out', required=True, help='output .glb')
    p.add_argument('--anim', default='all', help="'all' (default), 'none', or comma-separated indices")
    p.add_argument('--lod', type=int, default=0)
    p.add_argument('--mirror', action='store_true', help='flip Z (left/right) for the opposite handedness')
    p.add_argument('--texture', action='append', default=[], metavar='GEOSET=HASH')
    p = sub.add_parser('uv-check', help="draw a character's UV layout over a texture")
    p.add_argument('character')
    p.add_argument('--texture', required=True, help='texture hash, e.g. h_e6ee3065')
    p.add_argument('-o', '--out', default='uv-check.png')
    p.add_argument('--lod', type=int, default=0)
    a = ap.parse_args()

    game = Game(a.level, a.xbe)
    if a.cmd == 'list':
        for name, (compound, _) in sorted(game.characters.items()):
            ok = 'animated' if game.anim_sets.get(H(name)) is not None else 'no animation set'
            print(f'{name:16} skeleton {X.hname(compound):12} {ok}')
        return

    parse_hash = lambda s: int(s[2:] if s.startswith('h_') else s, 16)
    overrides = {}
    for spec in getattr(a, 'texture', []) if a.cmd in ('render', 'gltf') else []:
        k, v = spec.split('=')
        overrides[int(k)] = parse_hash(v)
    char = Character(game, a.character, getattr(a, 'lod', 0), overrides)
    anims = char.animations()

    if a.cmd == 'gltf':
        import gltf_export
        sel = None if a.anim == 'all' else [] if a.anim == 'none' else [int(x) for x in a.anim.split(',')]
        n, size = gltf_export.export(char, a.out, sel, a.mirror)
        print(f'wrote {a.out}: {len(char.bones)} joints, {len(char.geosets)} primitives, {n} animations, '
              f'{size / 1e6:.1f} MB')
    elif a.cmd == 'anims':
        print(f'{a.character}: {len(anims)} animations, {len(char.bones)} bones, '
              f'{sum(len(g["tris"]) for g in char.geosets)} triangles')
        for i, an in enumerate(anims):
            print(anim_label(i, an))
        faces = char.face_animations()
        if faces:
            print(f'face clips ({a.character}face, layered on the face bones; default {char.default_face()}):')
            for i, an in enumerate(faces):
                print(anim_label(i, an))
    elif a.cmd == 'uv-check':
        tex = game.texture_rgba(parse_hash(a.texture))
        if tex is None:
            raise SystemExit('texture not found')
        S = 512
        base = Image.fromarray(tex).convert('RGB').resize((S, S))
        tiles = []
        for gi, g in enumerate(char.geosets):
            ov = base.copy(); dr = ImageDraw.Draw(ov)
            uv = g['uv'] % 1.0
            for x, y, z in g['tris']:
                dr.polygon([(uv[i, 0] * S, uv[i, 1] * S) for i in (x, y, z)], outline=(255, 0, 255))
            dr.text((6, 6), f'geoset {gi}', fill=(255, 255, 0))
            tiles.append(Image.blend(base, ov, 0.6))
        sheet = Image.new('RGB', (S * len(tiles), S))
        for i, t in enumerate(tiles):
            sheet.paste(t, (i * S, 0))
        sheet.save(a.out)
        print(f'wrote {a.out}: the right texture for a geoset has its UV wireframe sitting on painted islands')
    elif a.cmd == 'render':
        yaw = np.radians(a.yaw)
        face = None if a.face == 'none' else 'default' if a.face == 'default' else int(a.face)
        if a.pose == 'bind':
            v = char.skin(char.pose(face=face))
            cams = [frame_camera([v], y) for y in (np.pi, np.pi / 2 + np.pi, 0)]
            tiles = [render(char, v, c, a.size) for c in cams]
            sheet = Image.new('RGB', (a.size * 3, a.size))
            for i, t in enumerate(tiles):
                sheet.paste(t, (i * a.size, 0))
            sheet.save(a.out)
            print(f'wrote {a.out}')
            return
        if not anims:
            raise SystemExit(f'{a.character} has no animation set loaded (try --level)')
        if a.anim == 'all':
            os.makedirs(a.out, exist_ok=True)
            for i, an in enumerate(anims):
                dst = os.path.join(a.out, f'{a.character}_{i:03}_{X.hname(A(an.children[0])["name"])}.gif')
                n = render_anim(char, an, dst, a.size, a.fps, yaw, a.root_motion, face)
                print(f'{anim_label(i, an)}  {n} frames -> {dst}')
            return
        if a.anim.isdigit():
            an = anims[int(a.anim)]
        else:
            h = parse_hash(a.anim) if a.anim.startswith('h_') else H(a.anim)
            an = next((x for x in anims if A(x.children[0])['name'] == h), None)
            if an is None:
                raise SystemExit(f'animation {a.anim} not found')
        n = render_anim(char, an, a.out, a.size, a.fps, yaw, a.root_motion, face)
        print(f'{anim_label(anims.index(an), an)}  {n} frames -> {a.out}')


if __name__ == '__main__':
    main()

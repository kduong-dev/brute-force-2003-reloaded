"""
gltf_export.py - write a char_render.Character (skinned mesh, skeleton, textures, animations)
to a binary glTF 2.0 file (.glb).

Mapping from the game data:
  * bones become nodes; local rotation = the game's quaternion as-is (x, y, z, w), local
    translation = joint parent-point (minus child-point rotated), root from its bind/channel
  * inverse bind matrices are the stored part matrices (local = R @ v + t), written 1:1
  * skinned vertices: palette slot -> bone index; rigid geosets get weight 1 on their bone
  * every animation in the character's set becomes a glTF animation (LINEAR samplers); the
    '<name>face' clips are added as face_NN_* animations, and the neutral face is baked into
    the rest pose (the bind face is modelled mid-speech)
  * --mirror flips Z (and winding) in case you want the left/right-mirrored convention
"""
import io
import json
import struct

import numpy as np
from PIL import Image

import char_render as C
import xmb_tool as X


class _Buf:
    def __init__(self):
        self.data = bytearray()
        self.views = []
        self.accessors = []

    def add(self, arr, comp, typ, target=None, minmax=False):
        arr = np.ascontiguousarray(arr)
        while len(self.data) % 4:
            self.data.append(0)
        off = len(self.data)
        self.data += arr.tobytes()
        view = {'buffer': 0, 'byteOffset': off, 'byteLength': arr.nbytes}
        if target:
            view['target'] = target
        self.views.append(view)
        count = arr.shape[0]
        acc = {'bufferView': len(self.views) - 1, 'componentType': comp, 'count': int(count), 'type': typ}
        if minmax:
            a2 = arr.reshape(count, -1)
            acc['min'] = [float(x) for x in a2.min(0)]
            acc['max'] = [float(x) for x in a2.max(0)]
        self.accessors.append(acc)
        return len(self.accessors) - 1

    def add_image(self, png):
        while len(self.data) % 4:
            self.data.append(0)
        off = len(self.data)
        self.data += png
        self.views.append({'buffer': 0, 'byteOffset': off, 'byteLength': len(png)})
        return len(self.views) - 1


F32, U16, U32, U8 = 5126, 5123, 5125, 5121


def _normals(char, g):
    raw = np.frombuffer(char.ivd, np.uint8, len(g['pos']) * 32, g['vb']).reshape(-1, 32)
    v = raw[:, 12:16].copy().view('<u4').ravel().astype(np.int64)

    def sx(x, b):
        return np.where(x >= 1 << (b - 1), x - (1 << b), x)
    n = np.stack([sx(v & 0x7FF, 11) / 1023.0, sx((v >> 11) & 0x7FF, 11) / 1023.0,
                  sx((v >> 22) & 0x3FF, 10) / 511.0], 1)
    return (n / np.maximum(np.linalg.norm(n, axis=1, keepdims=True), 1e-9)).astype(np.float32)


def _channel_keys(char, ch):
    """All (time, kind, quat, vec) keys of a channel, decoded."""
    a, chnl = char.game.channels[ch]
    n = a['num-frames']; per = a['size'] // n
    raw = chnl[a['offset']:a['offset'] + a['size']]
    keyed = a['frame-type'] == C.FT_KEYED
    dt = a.get('h_f63f0f49', 1 / 30)
    out = []
    for i in range(n):
        f = raw[i * per:(i + 1) * per]
        t = struct.unpack_from('<f', f)[0] if keyed else i * dt
        if keyed:
            f = f[4:]
        if a['data-type'] == C.DT_QUAT:
            q = np.array(struct.unpack('<3h', f[:6])) / 32767.0
            out.append((t, np.array([*q, np.sqrt(max(0.0, 1 - q @ q))]), None))
        elif a['data-type'] == C.DT_VEC:
            out.append((t, None, np.array(struct.unpack('<3f', f[:12]))))
        elif a['data-type'] == C.DT_XFORM:
            q = np.array(struct.unpack('<3h', f[12:18])) / 32767.0
            out.append((t, np.array([*q, np.sqrt(max(0.0, 1 - q @ q))]), np.array(struct.unpack('<3f', f[:12]))))
    return out


def export(char, path, anims=None, mirror=False, textures=True):
    S = np.diag([1.0, 1.0, -1.0]) if mirror else np.eye(3)
    qm = np.array([-1.0, -1.0, 1.0, 1.0]) if mirror else np.ones(4)   # quaternion under Z-mirror
    buf = _Buf()
    gl = {'asset': {'version': '2.0', 'generator': 'XBE Mod gltf_export.py (Brute Force)'},
          'scene': 0, 'scenes': [{'nodes': []}], 'nodes': [], 'meshes': [], 'skins': [],
          'materials': [], 'textures': [], 'images': [], 'samplers': [{'magFilter': 9729, 'minFilter': 9987}],
          'animations': [], 'buffers': [], 'bufferViews': [], 'accessors': []}

    # ---- skeleton ----------------------------------------------------------------------
    bones = char.bones
    bidx = {b: i for i, b in enumerate(bones)}
    world = char.pose(face=None)                             # bind pose
    # human faces are modelled mid-speech; bake the neutral face clip into the rest pose
    neutral = {}
    faces = char.face_animations()
    fi = char.default_face()
    if fi is not None:
        for tg in C.tagged(faces[fi], 'target'):
            a = C.A(tg.children[0])
            if a.get('data-channel') in char.game.channels and a.get('name') in char.parent:
                neutral[a['name']] = char._sample(a['data-channel'], 0.0)
    for b in bones:
        node = {'name': X.hname(b)}
        if b in char.parent:
            L = char.bind[b][0] @ char.bind[char.parent[b]][0].T
            q = _quat_from_rows(L)
            t = char.ppoint[b] - char.cpoint[b] @ L
            kind, nq, nv = neutral.get(b, (None, None, None))
            if kind == 'q':
                q = nq
            elif kind == 'v':
                t = nv
        else:
            R, p = world[b]
            q = _quat_from_rows(R)
            t = p
        node['rotation'] = [float(x) for x in q * qm]
        node['translation'] = [float(x) for x in t @ S]
        gl['nodes'].append(node)
    for b in bones:
        kids = [bidx[c] for c in bones if char.parent.get(c) == b]
        if kids:
            gl['nodes'][bidx[b]]['children'] = kids
    roots = [bidx[b] for b in bones if b not in char.parent]

    ibm = []
    for b in bones:
        Rb, pb = char.bind[b]                                # bone->model rows; inverse = local = R v + t
        R = Rb                                               # column form of inverse bind rotation
        t = -(pb @ Rb.T)
        M = np.eye(4)
        M[:3, :3] = S @ R @ S
        M[:3, 3] = S @ t
        ibm.append(M.T.ravel())                              # glTF is column-major
    ibm_acc = buf.add(np.array(ibm, np.float32), F32, 'MAT4')
    gl['skins'].append({'joints': list(range(len(bones))), 'inverseBindMatrices': ibm_acc,
                        'skeleton': roots[0]})

    # ---- mesh -------------------------------------------------------------------------------
    tex_index = {}
    prims = []
    for gi, g in enumerate(char.geosets):
        pos = (g['pos'] @ S).astype(np.float32)
        nrm = (_normals(char, g) @ S).astype(np.float32)
        uv = g['uv'].astype(np.float32)
        if g['skinned']:
            pal = char.palettes[g['pal']]
            joints = np.array([[pal[s] if 0 <= s < len(pal) else 0 for s in row] for row in g['slot']], np.uint16)
            w = g['w'].astype(np.float32)
            w[w.sum(1) == 0, 0] = 1
            w /= w.sum(1, keepdims=True)
            joints[w == 0] = 0
        else:
            joints = np.zeros((len(pos), 4), np.uint16); joints[:, 0] = g['rigid_bone'] or 0
            w = np.zeros((len(pos), 4), np.float32); w[:, 0] = 1
        tris = g['tris'].astype(np.uint32)
        if mirror:
            tris = tris[:, [0, 2, 1]]
        attrs = {'POSITION': buf.add(pos, F32, 'VEC3', 34962, minmax=True),
                 'NORMAL': buf.add(nrm, F32, 'VEC3', 34962),
                 'TEXCOORD_0': buf.add(uv, F32, 'VEC2', 34962),
                 'JOINTS_0': buf.add(joints, U16, 'VEC4', 34962),
                 'WEIGHTS_0': buf.add(w, F32, 'VEC4', 34962)}
        prim = {'attributes': attrs, 'indices': buf.add(tris.ravel(), U32, 'SCALAR', 34963), 'mode': 4}
        mat = {'name': X.hname(g['mat']), 'pbrMetallicRoughness': {'metallicFactor': 0.0, 'roughnessFactor': 0.9},
               'doubleSided': False}
        h = char.overrides.get(gi) or char.game.materials.get(g['mat'], {}).get(C.H('color')) or C.SKINS.get(g['mat'])
        if textures and h is not None and char.game.texture_rgba(h) is not None:
            if h not in tex_index:
                png = io.BytesIO()
                Image.fromarray(char.game.texture_rgba(h)).save(png, 'PNG')
                gl['images'].append({'name': X.hname(h), 'mimeType': 'image/png', 'bufferView': buf.add_image(png.getvalue())})
                gl['textures'].append({'source': len(gl['images']) - 1, 'sampler': 0})
                tex_index[h] = len(gl['textures']) - 1
            # alpha in these skins is a specular mask (BF_CS_rt = Color-Specular), not
            # transparency, so materials stay OPAQUE (the glTF default)
            mat['pbrMetallicRoughness']['baseColorTexture'] = {'index': tex_index[h]}
        else:
            mat['pbrMetallicRoughness']['baseColorFactor'] = [0.7, 0.7, 0.7, 1.0]
        gl['materials'].append(mat)
        prim['material'] = len(gl['materials']) - 1
        prims.append(prim)
    gl['meshes'].append({'name': char.name, 'primitives': prims})
    mesh_node = len(gl['nodes'])
    gl['nodes'].append({'name': char.name + '_mesh', 'mesh': 0, 'skin': 0})
    gl['scenes'][0]['nodes'] = roots + [mesh_node]

    # ---- animations ---------------------------------------------------------------------------
    all_anims = char.animations()
    clips = [(f'{ai:03d}_', all_anims[ai]) for ai in (anims if anims is not None else range(len(all_anims)))]
    if anims is None:
        clips += [(f'face_{fi_:02d}_', f) for fi_, f in enumerate(faces)]
    for label, an in clips:
        head = C.A(an.children[0])
        samplers, channels = [], []
        for tg in C.tagged(an, 'target'):
            a = C.A(tg.children[0])
            b, ch = a.get('name'), a.get('data-channel')
            if b not in bidx or ch not in char.game.channels:
                continue
            keys = _channel_keys(char, ch)
            if not keys:
                continue
            times = np.array([k[0] for k in keys], np.float32)
            if np.any(np.diff(times) <= 0):                 # glTF needs strictly increasing times
                keep = np.concatenate([[True], np.diff(times) > 0])
                keys = [k for k, ok in zip(keys, keep) if ok]; times = times[keep]
            t_acc = buf.add(times, F32, 'SCALAR', minmax=True)
            if keys[0][1] is not None:
                q = np.array([k[1] for k in keys]) * qm
                if b not in char.parent:                    # root rotation is a world rotation
                    pass
                samplers.append({'input': t_acc, 'output': buf.add(q.astype(np.float32), F32, 'VEC4'),
                                 'interpolation': 'LINEAR'})
                channels.append({'sampler': len(samplers) - 1, 'target': {'node': bidx[b], 'path': 'rotation'}})
            if keys[0][2] is not None:
                v = np.array([k[2] for k in keys])
                if b in char.parent:
                    v = v - 0                                # vector channels replace the joint offset
                samplers.append({'input': t_acc, 'output': buf.add((v @ S).astype(np.float32), F32, 'VEC3'),
                                 'interpolation': 'LINEAR'})
                channels.append({'sampler': len(samplers) - 1, 'target': {'node': bidx[b], 'path': 'translation'}})
        if channels:
            gl['animations'].append({'name': f'{label}{X.hname(head["name"])}', 'samplers': samplers,
                                     'channels': channels})

    # ---- write GLB ------------------------------------------------------------------------------
    while len(buf.data) % 4:
        buf.data.append(0)
    gl['buffers'] = [{'byteLength': len(buf.data)}]
    gl['bufferViews'] = buf.views
    gl['accessors'] = buf.accessors
    for k in ('textures', 'images', 'animations'):
        if not gl[k]:
            del gl[k]
    if 'textures' not in gl:
        del gl['samplers']
    js = json.dumps(gl, separators=(',', ':')).encode()
    js += b' ' * (-len(js) % 4)
    with open(path, 'wb') as f:
        f.write(struct.pack('<III', 0x46546C67, 2, 12 + 8 + len(js) + 8 + len(buf.data)))
        f.write(struct.pack('<II', len(js), 0x4E4F534A) + js)
        f.write(struct.pack('<II', len(buf.data), 0x004E4942) + bytes(buf.data))
    return len(gl.get('animations', [])), len(buf.data)


def _quat_from_rows(L):
    """Quaternion q with qmat(q).T == L (rows convention), i.e. q of the column matrix L.T."""
    M = L.T
    t = np.trace(M)
    if t > 0:
        s = np.sqrt(t + 1.0) * 2
        q = [(M[2, 1] - M[1, 2]) / s, (M[0, 2] - M[2, 0]) / s, (M[1, 0] - M[0, 1]) / s, 0.25 * s]
    elif M[0, 0] > M[1, 1] and M[0, 0] > M[2, 2]:
        s = np.sqrt(1.0 + M[0, 0] - M[1, 1] - M[2, 2]) * 2
        q = [0.25 * s, (M[0, 1] + M[1, 0]) / s, (M[0, 2] + M[2, 0]) / s, (M[2, 1] - M[1, 2]) / s]
    elif M[1, 1] > M[2, 2]:
        s = np.sqrt(1.0 + M[1, 1] - M[0, 0] - M[2, 2]) * 2
        q = [(M[0, 1] + M[1, 0]) / s, 0.25 * s, (M[1, 2] + M[2, 1]) / s, (M[0, 2] - M[2, 0]) / s]
    else:
        s = np.sqrt(1.0 + M[2, 2] - M[0, 0] - M[1, 1]) * 2
        q = [(M[0, 2] + M[2, 0]) / s, (M[1, 2] + M[2, 1]) / s, 0.25 * s, (M[1, 0] - M[0, 1]) / s]
    q = np.array(q)
    return q / np.linalg.norm(q)

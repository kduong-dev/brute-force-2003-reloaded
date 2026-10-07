#!/usr/bin/env python3
"""
xwb_tool.py - Read / extract / replace / rebuild XACT wave banks (.xwb) from
Brute Force (2003, Xbox).  Handles the early XDK "WBND" v2/v3 format.

BANK arguments may be .xwb files, .tgz archives containing .xwb files
(data/ml-sounds/<lang>/*.tgz = voice/dialogue banks), or directories
(searched recursively for both).

Commands:
  info     BANK...                              list entries
  find     BANK... PATTERN                      search entry names (glob, e.g. "dx*_hawk_*")
  extract  BANK... [-o DIR] [-e PAT] [--raw]    decode entries to .wav
  replace  BANK... -e NAME -w WAV               swap one entry's audio (all banks that contain it)
  verify   BANK...                              parse -> rebuild -> byte-compare

Needs numpy (for the ADPCM codec).
"""
import argparse
import fnmatch
import glob
import gzip
import os
import re
import shutil
import struct
import sys
import wave
import zlib

import numpy as np

MAGIC = b'WBND'

# Bank flags
BANK_STREAMING = 0x00000001
BANK_ENTRYNAMES = 0x00010000

# MINIWAVEFORMAT format tags (low 2 bits)
TAG_PCM = 0
TAG_XBOX_ADPCM = 1
TAG_WMA = 2
TAG_NAMES = {TAG_PCM: 'PCM', TAG_XBOX_ADPCM: 'XboxADPCM', TAG_WMA: 'WMA'}

ADPCM_BLOCK = 36          # bytes per channel per block
ADPCM_SAMPLES = 65        # header sample + 64 nibbles


# ---------------------------------------------------------------------------
# Wave bank model
# ---------------------------------------------------------------------------

def align_up(v, a):
    return (v + a - 1) // a * a if a > 1 else v


class Entry:
    def __init__(self, flags, fmt, data, loop_start=0, loop_len=0, name=''):
        self.flags = flags        # per-entry flags; 0x10000 on most music, 0 on a few
        self.fmt = fmt            # packed MINIWAVEFORMAT
        self.data = data          # raw (encoded) audio bytes
        self.loop_start = loop_start
        self.loop_len = loop_len
        self.name = name

    @property
    def tag(self):
        return self.fmt & 3

    @property
    def channels(self):
        return (self.fmt >> 2) & 7

    @property
    def rate(self):
        return (self.fmt >> 5) & 0x3FFFFFF

    @property
    def bits(self):
        return 16 if self.fmt >> 31 else 8

    @property
    def num_samples(self):
        if self.tag == TAG_XBOX_ADPCM:
            return len(self.data) // (ADPCM_BLOCK * self.channels) * ADPCM_SAMPLES
        if self.tag == TAG_PCM:
            return len(self.data) // (self.channels * self.bits // 8)
        return 0

    @property
    def duration(self):
        return self.num_samples / self.rate if self.rate else 0.0


class WaveBank:
    """
    File layout (all little-endian):
      0x00  'WBND'
      0x04  u32 version (2 for data/sounds, 3 for media/Wave.xwb)
      0x08  4 x {u32 offset, u32 length} segments:
              [0] bank data   [1] entry metadata   [2] entry names   [3] wave data
      bank data:   u32 flags, u32 entry_count, char name[16],
                   u32 meta_elem_size (24), u32 name_elem_size (64), u32 alignment,
                   (v3: + 4 extra bytes, preserved)
      metadata:    per entry 6 x u32: flags, MINIWAVEFORMAT, play_off, play_len,
                   loop_start, loop_len   (play_off is relative to the wave segment)
      names:       per entry char[name_elem_size]  (only when BANK_ENTRYNAMES)
      wave data:   starts aligned; each entry aligned to `alignment`, zero padded
    """

    def __init__(self):
        self.version = 2
        self.flags = 0
        self.name = ''
        self.meta_size = 24
        self.name_size = 64
        self.alignment = 4
        self.bank_extra = b''     # trailing bank-data bytes beyond the known fields
        self.entries = []

    # -- parsing ------------------------------------------------------------
    @classmethod
    def load(cls, path):
        with open(path, 'rb') as f:
            return cls.parse(f.read())

    @classmethod
    def parse(cls, d):
        if d[:4] != MAGIC:
            raise ValueError(f'not a WBND wave bank (magic {d[:4]!r})')
        wb = cls()
        wb.version = struct.unpack_from('<I', d, 4)[0]
        segs = struct.unpack_from('<8I', d, 8)
        bd_off, bd_len = segs[0], segs[1]
        wb.flags, count = struct.unpack_from('<II', d, bd_off)
        wb.name = d[bd_off + 8:bd_off + 24].split(b'\0')[0].decode('latin-1')
        wb.meta_size, wb.name_size, wb.alignment = struct.unpack_from('<3I', d, bd_off + 24)
        wb.bank_extra = d[bd_off + 36:bd_off + bd_len]
        if wb.meta_size < 24:
            raise ValueError(f'unsupported metadata element size {wb.meta_size}')

        meta_off, (names_off, names_len), wave_off = segs[2], segs[4:6], segs[6]
        for i in range(count):
            flags, fmt, off, length, ls, ll = struct.unpack_from('<6I', d, meta_off + i * wb.meta_size)
            name = ''
            if names_len:
                p = names_off + i * wb.name_size
                name = d[p:p + wb.name_size].split(b'\0')[0].decode('latin-1')
            data = d[wave_off + off:wave_off + off + length]
            wb.entries.append(Entry(flags, fmt, data, ls, ll, name))
        return wb

    # -- writing ------------------------------------------------------------
    def build(self):
        n = len(self.entries)
        has_names = bool(self.flags & BANK_ENTRYNAMES)
        bank = struct.pack('<II16s3I', self.flags, n,
                           self.name.encode('latin-1')[:16],
                           self.meta_size, self.name_size, self.alignment) + self.bank_extra
        bd_off = 8 + 4 * 8
        segs = [bd_off, len(bank), 0, 0, 0, 0, 0, 0]
        body = bytearray(bank)

        if n:
            meta_off = bd_off + len(bank)
            names_off = meta_off + n * self.meta_size
            names_len = n * self.name_size if has_names else 0
            wave_off = align_up(names_off + names_len, self.alignment)

            # lay out wave data
            offsets, cur = [], 0
            for e in self.entries:
                cur = align_up(cur, self.alignment)
                offsets.append(cur)
                cur += len(e.data)
            wave_len = align_up(cur, self.alignment)

            meta = bytearray()
            for e, off in zip(self.entries, offsets):
                rec = struct.pack('<6I', e.flags, e.fmt, off, len(e.data), e.loop_start, e.loop_len)
                meta += rec.ljust(self.meta_size, b'\0')
            names = bytearray()
            if has_names:
                for e in self.entries:
                    nb = e.name.encode('latin-1')
                    if len(nb) >= self.name_size:
                        raise ValueError(f'entry name too long: {e.name}')
                    names += nb.ljust(self.name_size, b'\0')

            segs[2:8] = [meta_off, len(meta),
                         names_off if has_names else 0, names_len,
                         wave_off, wave_len]
            body += meta + names
            body += b'\0' * (wave_off - (bd_off + len(body)))
            wave = bytearray(wave_len)
            for e, off in zip(self.entries, offsets):
                wave[off:off + len(e.data)] = e.data
            body += wave

        return MAGIC + struct.pack('<I8I', self.version, *segs) + bytes(body)

    def find(self, key):
        """Look up an entry by name or index."""
        for i, e in enumerate(self.entries):
            if e.name == key or str(i) == str(key):
                return i
        return None


# ---------------------------------------------------------------------------
# Xbox ADPCM (IMA ADPCM, 36-byte blocks per channel, 65 samples per block)
#
# Block per channel: s16 predictor, u8 step_index, u8 0, then 32 bytes of
# 4-bit codes (low nibble first).  For multichannel, the 4-byte headers come
# first (ch0, ch1, ...), then the data interleaves 4-byte words per channel.
# Blocks are independent, so the codec is vectorised across all blocks.
# ---------------------------------------------------------------------------

STEP_TABLE = np.array([
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45,
    50, 55, 60, 66, 73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230,
    253, 279, 307, 337, 371, 408, 449, 494, 544, 598, 658, 724, 796, 876, 963,
    1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272, 2499, 2749, 3024, 3327,
    3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630, 9493, 10442,
    11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794,
    32767], dtype=np.int32)
INDEX_TABLE = np.array([-1, -1, -1, -1, 2, 4, 6, 8] * 2, dtype=np.int32)


def adpcm_decode(data, channels):
    """Xbox ADPCM bytes -> int16 array of shape (samples, channels)."""
    bs = ADPCM_BLOCK * channels
    nb = len(data) // bs
    if nb == 0:
        return np.zeros((0, channels), np.int16)
    blk = np.frombuffer(data, np.uint8, nb * bs).reshape(nb, bs)
    hdr = blk[:, :4 * channels].reshape(nb, channels, 4)
    pred = hdr[:, :, 0].astype(np.int32) | (hdr[:, :, 1].astype(np.int32) << 8)
    pred = np.where(pred >= 0x8000, pred - 0x10000, pred)
    idx = np.clip(hdr[:, :, 2].astype(np.int32), 0, 88)

    body = blk[:, 4 * channels:].reshape(nb, 8, channels, 4).transpose(0, 2, 1, 3).reshape(nb, channels, 32)
    nib = np.empty((nb, channels, 64), np.int32)
    nib[:, :, 0::2] = body & 0x0F
    nib[:, :, 1::2] = body >> 4

    out = np.empty((nb, ADPCM_SAMPLES, channels), np.int16)
    out[:, 0, :] = pred
    for i in range(64):
        n = nib[:, :, i]
        step = STEP_TABLE[idx]
        diff = step >> 3
        diff = diff + np.where(n & 4, step, 0) + np.where(n & 2, step >> 1, 0) + np.where(n & 1, step >> 2, 0)
        pred = np.clip(np.where(n & 8, pred - diff, pred + diff), -32768, 32767)
        idx = np.clip(idx + INDEX_TABLE[n], 0, 88)
        out[:, i + 1, :] = pred
    return out.reshape(nb * ADPCM_SAMPLES, channels)


def _adpcm_encode_pass(blocks, idx0):
    """blocks: (nb, 65, ch) int32. Returns (nibbles (nb,ch,64), final idx (nb,ch))."""
    nb, _, ch = blocks.shape
    pred = blocks[:, 0, :].copy()
    idx = idx0.copy()
    nib = np.empty((nb, ch, 64), np.int32)
    for i in range(64):
        target = blocks[:, i + 1, :]
        step = STEP_TABLE[idx]
        diff = target - pred
        sign = (diff < 0).astype(np.int32) * 8
        diff = np.abs(diff)
        code = np.zeros_like(diff)
        vp = step >> 3
        b = diff >= step
        code |= b * 4; diff = diff - b * step; vp = vp + b * step
        s = step >> 1
        b = diff >= s
        code |= b * 2; diff = diff - b * s; vp = vp + b * s
        s = step >> 2
        b = diff >= s
        code |= b * 1; vp = vp + b * s
        pred = np.clip(np.where(sign, pred - vp, pred + vp), -32768, 32767)
        code |= sign
        idx = np.clip(idx + INDEX_TABLE[code], 0, 88)
        nib[:, :, i] = code
    return nib, idx


def adpcm_encode(pcm):
    """int16 array (samples, channels) -> Xbox ADPCM bytes (padded to whole blocks)."""
    n, ch = pcm.shape
    nb = max(1, -(-n // ADPCM_SAMPLES))
    padded = np.zeros((nb * ADPCM_SAMPLES, ch), np.int32)
    padded[:n] = pcm
    blocks = padded.reshape(nb, ADPCM_SAMPLES, ch)

    # Pass 1 estimates each block's ending step index; pass 2 seeds every block
    # with the previous block's index, matching what a sequential encoder does.
    _, end_idx = _adpcm_encode_pass(blocks, np.zeros((nb, ch), np.int32))
    idx0 = np.zeros((nb, ch), np.int32)
    idx0[1:] = end_idx[:-1]
    nib, _ = _adpcm_encode_pass(blocks, idx0)

    out = np.zeros((nb, ADPCM_BLOCK * ch), np.uint8)
    hdr = out[:, :4 * ch].reshape(nb, ch, 4)
    p = blocks[:, 0, :].astype(np.int32) & 0xFFFF
    hdr[:, :, 0] = p & 0xFF
    hdr[:, :, 1] = p >> 8
    hdr[:, :, 2] = idx0
    body = (nib[:, :, 0::2] | (nib[:, :, 1::2] << 4)).astype(np.uint8)        # (nb, ch, 32)
    out[:, 4 * ch:] = body.reshape(nb, ch, 8, 4).transpose(0, 2, 1, 3).reshape(nb, 32 * ch)
    return out.tobytes()


# ---------------------------------------------------------------------------
# WAV I/O
# ---------------------------------------------------------------------------

def entry_to_pcm(e):
    if e.tag == TAG_XBOX_ADPCM:
        return adpcm_decode(e.data, e.channels)
    if e.tag == TAG_PCM:
        if e.bits == 16:
            a = np.frombuffer(e.data[:len(e.data) // (2 * e.channels) * 2 * e.channels], '<i2')
        else:
            a = (np.frombuffer(e.data, np.uint8).astype(np.int16) - 128) << 8
            a = a[:len(a) // e.channels * e.channels]
        return a.reshape(-1, e.channels)
    raise NotImplementedError(f'cannot decode format {TAG_NAMES.get(e.tag, e.tag)}')


def write_pcm_wav(path, pcm, rate):
    with wave.open(path, 'wb') as w:
        w.setnchannels(pcm.shape[1])
        w.setsampwidth(2)
        w.setframerate(rate)
        w.writeframes(pcm.astype('<i2').tobytes())


def write_raw_wav(path, e):
    """Write the entry's encoded bytes untouched inside a RIFF header."""
    ch, rate = e.channels, e.rate
    if e.tag == TAG_XBOX_ADPCM:
        ba = ADPCM_BLOCK * ch
        fmt = struct.pack('<HHIIHHHH', 0x0069, ch, rate, rate * ba // ADPCM_SAMPLES, ba, 4, 2, ADPCM_SAMPLES - 1)
    elif e.tag == TAG_PCM:
        ba = ch * e.bits // 8
        fmt = struct.pack('<HHIIHH', 1, ch, rate, rate * ba, ba, e.bits)
    else:
        raise NotImplementedError('raw export only for PCM / Xbox ADPCM')
    riff = b'WAVE' + b'fmt ' + struct.pack('<I', len(fmt)) + fmt + b'data' + struct.pack('<I', len(e.data)) + e.data
    with open(path, 'wb') as f:
        f.write(b'RIFF' + struct.pack('<I', len(riff)) + riff)


def read_wav(path):
    """Read a PCM .wav -> (int16 array (samples, channels), rate)."""
    with wave.open(path, 'rb') as w:
        ch, sw, rate, n = w.getnchannels(), w.getsampwidth(), w.getframerate(), w.getnframes()
        raw = w.readframes(n)
    if sw == 2:
        a = np.frombuffer(raw, '<i2').astype(np.int32)
    elif sw == 1:
        a = (np.frombuffer(raw, np.uint8).astype(np.int32) - 128) << 8
    elif sw == 3:
        b = np.frombuffer(raw, np.uint8).reshape(-1, 3).astype(np.int32)
        a = (b[:, 0] | (b[:, 1] << 8) | (b[:, 2] << 16))
        a = np.where(a >= 0x800000, a - 0x1000000, a) >> 8
    elif sw == 4:
        a = np.frombuffer(raw, '<i4').astype(np.int64) >> 16
    else:
        raise ValueError(f'unsupported sample width {sw}')
    return a.reshape(-1, ch).astype(np.int32), rate


def convert_pcm(pcm, rate, want_ch, want_rate):
    """Channel-mix and (linearly) resample to the target layout."""
    if pcm.shape[1] != want_ch:
        mono = pcm.mean(axis=1, keepdims=True)
        pcm = np.repeat(mono, want_ch, axis=1) if want_ch > 1 else mono
    if rate != want_rate and len(pcm):
        n_out = int(round(len(pcm) * want_rate / rate))
        x = np.linspace(0, len(pcm) - 1, n_out)
        pcm = np.stack([np.interp(x, np.arange(len(pcm)), pcm[:, c]) for c in range(pcm.shape[1])], axis=1)
    return np.clip(np.round(pcm), -32768, 32767).astype(np.int16)


def encode_for_entry(e, pcm):
    if e.tag == TAG_XBOX_ADPCM:
        return adpcm_encode(pcm)
    if e.tag == TAG_PCM and e.bits == 16:
        return pcm.astype('<i2').tobytes()
    if e.tag == TAG_PCM:
        return ((pcm.astype(np.int32) >> 8) + 128).astype(np.uint8).tobytes()
    raise NotImplementedError(f'cannot encode format {TAG_NAMES.get(e.tag, e.tag)}')


# ---------------------------------------------------------------------------
# Containers: a bare .xwb, or a .tgz (gzip'd tar) holding .xwb members
#
# The game's archives were made with old GNU tar (space-padded octal fields),
# which Python's tarfile can't reproduce.  So we never regenerate the tar:
# members are spliced over the original bytes and only a replaced member's
# size + checksum fields are rewritten, in the same field style.
# ---------------------------------------------------------------------------

TAR_BLOCK = 512
TAR_RECORD = 10240


def _tar_members(tar):
    """Yield (header_offset, name, size) for every member of a raw tar."""
    off = 0
    while off + TAR_BLOCK <= len(tar):
        hdr = tar[off:off + TAR_BLOCK]
        if hdr == bytes(TAR_BLOCK):
            break
        name = hdr[:100].split(b'\0')[0].decode('latin-1')
        size = int(hdr[124:136].strip(b' \0') or b'0', 8)
        yield off, name, size
        off += TAR_BLOCK + align_up(size, TAR_BLOCK)


def _octal_field(old, value):
    """Format `value` as octal in the same padding/terminator style as `old`."""
    m = re.match(rb'^([ 0]*)([0-7]*)([ \0]*)$', old)
    trail = m.group(3) if m else b'\0'
    pad = b'0' if old[:1] == b'0' else b' '
    digits = b'%o' % value
    width = len(old) - len(trail)
    if len(digits) > width:
        raise ValueError('value too large for tar field')
    return digits.rjust(width, pad) + trail


def _tar_header(hdr, size):
    h = bytearray(hdr)
    h[124:136] = _octal_field(bytes(h[124:136]), size)
    old_chk = bytes(h[148:156])
    h[148:156] = b' ' * 8
    h[148:156] = _octal_field(old_chk, sum(h))
    return bytes(h)


def tar_replace(tar, new_data):
    """Rebuild a raw tar with {member_name: bytes} substituted."""
    out = bytearray()
    for off, name, size in _tar_members(tar):
        hdr = tar[off:off + TAR_BLOCK]
        if name in new_data:
            data = new_data[name]
            out += _tar_header(hdr, len(data)) + data + bytes(align_up(len(data), TAR_BLOCK) - len(data))
        else:
            out += tar[off:off + TAR_BLOCK + align_up(size, TAR_BLOCK)]
    out += bytes(2 * TAR_BLOCK)
    out += bytes(align_up(len(out), TAR_RECORD) - len(out))
    return bytes(out)


def gzip_like(orig_gz, payload):
    """gzip `payload` reusing the original header's filename / mtime / OS byte."""
    flg, mtime, os_byte = orig_gz[3], orig_gz[4:8], orig_gz[9]
    hdr = bytearray(b'\x1f\x8b\x08' + bytes([flg & 0x08]) + mtime + b'\x02' + bytes([os_byte]))
    if flg & 0x08:                          # FNAME
        p = 10 + (2 + int.from_bytes(orig_gz[10:12], 'little') if flg & 0x04 else 0)
        hdr += orig_gz[p:orig_gz.index(b'\0', p) + 1]
    c = zlib.compressobj(9, zlib.DEFLATED, -15, 9)
    body = c.compress(payload) + c.flush()
    return bytes(hdr) + body + struct.pack('<II', zlib.crc32(payload), len(payload) & 0xFFFFFFFF)


def is_archive(path):
    return path.lower().endswith(('.tgz', '.tar.gz'))


class Container:
    """A file on disk holding one or more wave banks."""

    def __init__(self, path):
        self.path = path
        with open(path, 'rb') as f:
            self.raw = f.read()
        self.banks = {}                     # member name ('' for a bare .xwb) -> WaveBank
        self.orig = {}                      # member name -> original bank bytes
        if is_archive(path):
            self.tar = gzip.decompress(self.raw)
            for off, name, size in _tar_members(self.tar):
                if name.lower().endswith('.xwb'):
                    d = self.tar[off + TAR_BLOCK:off + TAR_BLOCK + size]
                    self.orig[name] = d
                    self.banks[name] = WaveBank.parse(d)
        else:
            self.tar = None
            self.orig[''] = self.raw
            self.banks[''] = WaveBank.parse(self.raw)

    def label(self, member):
        return f'{self.path}:{member}' if member else self.path

    def build(self, members=None):
        """Serialise; `members` limits which banks get rebuilt (others keep original bytes)."""
        if self.tar is None:
            return self.banks[''].build()
        new = {m: self.banks[m].build() for m in (members if members is not None else self.banks)}
        return gzip_like(self.raw, tar_replace(self.tar, new))


def archive_has_xwb(path):
    """Cheap check: does the first tar member end in .xwb? (ml-sounds archives do,
    level archives under data/ don't - avoids decompressing those when scanning dirs)."""
    try:
        with gzip.open(path, 'rb') as f:
            hdr = f.read(TAR_BLOCK)
        return hdr[:100].split(b'\0')[0].lower().endswith(b'.xwb')
    except OSError:
        return False


def expand(paths):
    out = []
    for p in paths:
        if os.path.isdir(p):
            found = []
            for root, _, files in os.walk(p):
                for fn in files:
                    full = os.path.join(root, fn)
                    if fn.lower().endswith('.xwb') or (is_archive(fn) and archive_has_xwb(full)):
                        found.append(full)
            out += sorted(found)
        else:
            out += sorted(glob.glob(p)) or [p]
    return out


def iter_containers(paths):
    for path in expand(paths):
        try:
            c = Container(path)
        except (ValueError, OSError) as ex:
            print(f'skip {path}: {ex}', file=sys.stderr)
            continue
        if c.banks:
            yield c


def common_root(paths):
    dirs = [os.path.dirname(os.path.abspath(p)) for p in expand(paths)]
    return os.path.commonpath(dirs) if dirs else '.'


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def fmt_desc(e):
    return f'{TAG_NAMES.get(e.tag, e.tag)} {e.channels}ch {e.rate}Hz' + (f' {e.bits}bit' if e.tag == TAG_PCM else '')


def name_match(e, i, patterns):
    return not patterns or any(fnmatch.fnmatchcase(e.name, p) or str(i) == p for p in patterns)


def cmd_info(args):
    for c in iter_containers(args.banks):
        for member, wb in c.banks.items():
            kind = 'streaming' if wb.flags & BANK_STREAMING else 'in-memory'
            print(f'{c.label(member)}: "{wb.name}" v{wb.version} {kind}, {len(wb.entries)} entries, align {wb.alignment:#x}')
            for i, e in enumerate(wb.entries):
                loop = f' loop {e.loop_start}+{e.loop_len}' if e.loop_len else ''
                print(f'  [{i:3}] {e.name or "-":36} {fmt_desc(e):24} {len(e.data):>9} B '
                      f'{e.duration:7.2f}s flags={e.flags:#x}{loop}')


def cmd_find(args):
    hits = 0
    for c in iter_containers(args.banks):
        for member, wb in c.banks.items():
            for i, e in enumerate(wb.entries):
                if fnmatch.fnmatchcase(e.name, args.pattern):
                    print(f'{c.label(member)} [{i:3}] {e.name:36} {e.duration:6.2f}s')
                    hits += 1
    print(f'{hits} matches')


def cmd_extract(args):
    root = common_root(args.banks)
    for c in iter_containers(args.banks):
        rel = os.path.splitext(os.path.relpath(os.path.abspath(c.path), root))[0]
        for member, wb in c.banks.items():
            outdir = os.path.join(args.out, rel)
            if len(c.banks) > 1:
                outdir = os.path.join(outdir, os.path.splitext(os.path.basename(member))[0])
            picked = [(i, e) for i, e in enumerate(wb.entries) if name_match(e, i, args.entry)]
            if picked:
                os.makedirs(outdir, exist_ok=True)
            for i, e in picked:
                fn = os.path.join(outdir, f'{i:03d}_{e.name or "entry"}.wav')
                if args.raw:
                    write_raw_wav(fn, e)
                else:
                    write_pcm_wav(fn, entry_to_pcm(e), e.rate)
                print(f'{fn}  ({fmt_desc(e)}, {e.duration:.2f}s)')


def cmd_replace(args):
    pcm_in, rate_in = read_wav(args.wav)
    root = common_root(args.banks)
    encoded = {}                            # (fmt) -> bytes, so each target format is encoded once
    hits = 0
    for c in iter_containers(args.banks):
        changed = []
        for member, wb in c.banks.items():
            i = wb.find(args.entry)
            if i is None:
                continue
            e = wb.entries[i]
            if e.fmt not in encoded:
                if (pcm_in.shape[1], rate_in) != (e.channels, e.rate):
                    print(f'note: converting {pcm_in.shape[1]}ch {rate_in}Hz -> {e.channels}ch {e.rate}Hz '
                          f'(pre-convert in an audio editor for best quality)')
                encoded[e.fmt] = encode_for_entry(e, convert_pcm(pcm_in, rate_in, e.channels, e.rate))
            old = e.duration
            e.data = encoded[e.fmt]
            changed.append(member)
            print(f'{c.label(member)} [{i}] {e.name}: {old:.2f}s -> {e.duration:.2f}s')
        if not changed:
            continue
        if args.in_place:
            dst = c.path
            if not os.path.exists(dst + '.bak'):
                shutil.copy2(dst, dst + '.bak')
        else:
            dst = os.path.join(args.out, os.path.relpath(os.path.abspath(c.path), root))
            os.makedirs(os.path.dirname(dst) or '.', exist_ok=True)
        with open(dst, 'wb') as f:
            f.write(c.build(changed))
        print(f'  => {dst}')
        hits += len(changed)
    if not hits:
        sys.exit(f'entry "{args.entry}" not found in any bank')
    print(f'{hits} bank(s) patched')


def cmd_verify(args):
    bad = 0
    for c in iter_containers(args.banks):
        problems = [m or 'xwb' for m, wb in c.banks.items() if wb.build() != c.orig[m]]
        if c.tar is not None:
            # splice every bank back in through the header rewriter; tar must come out identical
            if tar_replace(c.tar, {m: wb.build() for m, wb in c.banks.items()}) != c.tar:
                problems.append('tar')
            if gzip.decompress(c.build()) != c.tar:
                problems.append('gzip')
        bad += bool(problems)
        print(f'{"FAIL" if problems else "OK  "} {c.path}' + (f'  ({", ".join(problems)})' if problems else ''))
    print(f'{bad} failures')
    sys.exit(1 if bad else 0)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest='cmd', required=True)

    p = sub.add_parser('info', help='list bank contents')
    p.add_argument('banks', nargs='+')
    p.set_defaults(func=cmd_info)

    p = sub.add_parser('find', help='search entry names across banks')
    p.add_argument('banks', nargs='+')
    p.add_argument('pattern', help='glob pattern, e.g. "dx*_hawk_*"')
    p.set_defaults(func=cmd_find)

    p = sub.add_parser('extract', help='extract entries to .wav')
    p.add_argument('banks', nargs='+')
    p.add_argument('-o', '--out', default='xwb_out')
    p.add_argument('-e', '--entry', action='append', help='only entries matching name/glob/index (repeatable)')
    p.add_argument('--raw', action='store_true', help='keep Xbox ADPCM (fmt 0x0069) instead of decoding')
    p.set_defaults(func=cmd_extract)

    p = sub.add_parser('replace', help='replace an entry with a .wav (in every given bank that has it)')
    p.add_argument('banks', nargs='+')
    p.add_argument('-e', '--entry', required=True, help='entry name or index')
    p.add_argument('-w', '--wav', required=True, help='PCM .wav (8/16/24/32-bit, any rate)')
    g = p.add_mutually_exclusive_group()
    g.add_argument('-o', '--out', default='xwb_mod', help='output directory, mirrors input layout (default xwb_mod)')
    g.add_argument('--in-place', action='store_true', help='overwrite banks/archives (keeps a .bak)')
    p.set_defaults(func=cmd_replace)

    p = sub.add_parser('verify', help='byte-identical round-trip check')
    p.add_argument('banks', nargs='+')
    p.set_defaults(func=cmd_verify)

    args = ap.parse_args()
    args.func(args)


if __name__ == '__main__':
    main()

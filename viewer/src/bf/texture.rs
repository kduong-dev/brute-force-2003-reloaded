//! Texture libraries: textures-<lvl>.xmb (index) + textures-<lvl>.tex (data). Port of tex_tool.py.
//! Formats are Xbox D3DFORMAT codes; uncompressed formats are Morton-swizzled, DXT is plain blocks.
//! A texture's mip levels follow each other in its data, largest first, each swizzled (or cut
//! into blocks) on its own; a cube map's faces each hold a whole chain, and only face 0 is read.

use super::bxml::Element;
use super::hash::h;

/// A texture entry's mip level count (textures-*.xmb; its name not recovered, tex_tool.py's
/// `H_MIPS`): the tutorial's terrain colours have 7 (DXT1 256 x 256 down to 4 x 4), their masks
/// 8, the bump maps 9; most HUD and effect textures in common.tgz have 1.
const H_MIPS: u32 = 0x1203_36EF;

/// D3DFMT_P8 (0x0B): the bump maps' format, an index into the game's normal palette per texel
/// (tex_tool.py; `Game::bump_mips`).
pub const P8: u32 = 0x0B;

#[derive(Clone, Debug)]
pub struct TexInfo {
    pub name: u32,
    pub fmt: u32,
    pub w: u32,
    pub h: u32,
    /// how many mip levels the data holds (`H_MIPS`, at least 1)
    pub mips: u32,
    pub offset: usize,
    pub size: usize,
}

/// One level of a texture's mip chain, decoded to RGBA8, rows top to bottom.
#[derive(Clone, Debug)]
pub struct MipLevel {
    pub w: u32,
    pub h: u32,
    pub px: Vec<u8>,
}

/// A texture's mip chain as its archive holds it (`decode_mips`), largest level first: level n
/// is the top's size halved n times (at least 1). The console samples these levels (MIPFILTER
/// LINEAR, see `scene::world_sampler`); the demo used to upload the top one alone.
#[derive(Clone, Debug)]
pub struct Mips {
    pub levels: Vec<MipLevel>,
}

impl Mips {
    /// A one-level chain of an RGBA8 picture (one the demo makes, or a texture's top level).
    pub fn single(w: u32, h: u32, px: Vec<u8>) -> Self {
        Mips { levels: vec![MipLevel { w, h, px }] }
    }

    /// The largest level.
    pub fn top(&self) -> &MipLevel {
        &self.levels[0]
    }

    /// Every texel of every level through `f` (RGBA8, four bytes).
    pub fn map_texels(&mut self, mut f: impl FnMut(&mut [u8])) {
        for l in &mut self.levels {
            l.px.chunks_exact_mut(4).for_each(&mut f);
        }
    }

    /// Each level's picture through `f` (its width, height and RGBA8 texels in, new texels out),
    /// for a change that looks at more than one texel at a time.
    pub fn map_levels(&mut self, mut f: impl FnMut(u32, u32, &[u8]) -> Vec<u8>) {
        for l in &mut self.levels {
            l.px = f(l.w, l.h, &l.px);
        }
    }

    /// Each level's alpha taken from `alpha`'s level that a sampler would pick for it (the
    /// smallest one still as large, else its largest), nearest texel: a material's separate
    /// "alpha" texture, sampled with the colour at the same uv, has its own chain.
    pub fn take_alpha(&mut self, alpha: &Mips) {
        for l in &mut self.levels {
            let a = alpha.levels.iter().rev().find(|a| a.w >= l.w && a.h >= l.h).unwrap_or(&alpha.levels[0]);
            for y in 0..l.h {
                for x in 0..l.w {
                    let (sx, sy) = (x * a.w / l.w, y * a.h / l.h);
                    l.px[((y * l.w + x) * 4 + 3) as usize] = a.px[((sy * a.w + sx) * 4 + 3) as usize];
                }
            }
        }
    }
}

pub fn index(root: &Element) -> Vec<TexInfo> {
    let mut out = vec![];
    for e in root.walk() {
        let Some(code) = e.attr(h("format")).and_then(|v| v.as_i64()) else { continue };
        let get = |n: &str| e.attr(h(n)).and_then(|v| v.as_i64()).unwrap_or(0);
        let base = code.unsigned_abs() as u32;   // negative = cube map; we use face 0
        out.push(TexInfo {
            name: e.attr(h("name")).and_then(|v| v.as_hash()).unwrap_or(0),
            fmt: if base == 1000 { 0 } else { base },
            w: get("width") as u32,
            h: get("height") as u32,
            mips: e.attr(H_MIPS).and_then(|v| v.as_i64()).unwrap_or(1).max(1) as u32,
            offset: get("offset") as usize,
            size: get("size") as usize,
        });
    }
    out
}

fn swizzle_map(w: u32, h: u32) -> Vec<usize> {
    let (mut mx, mut my) = (vec![0usize; w as usize], vec![0usize; h as usize]);
    let (mut bit, mut ww, mut hh, mut sx, mut sy) = (0, w, h, 0, 0);
    while ww > 1 || hh > 1 {
        if ww > 1 {
            for (x, m) in mx.iter_mut().enumerate() {
                *m |= ((x >> sx) & 1) << bit;
            }
            bit += 1; sx += 1; ww >>= 1;
        }
        if hh > 1 {
            for (y, m) in my.iter_mut().enumerate() {
                *m |= ((y >> sy) & 1) << bit;
            }
            bit += 1; sy += 1; hh >>= 1;
        }
    }
    let mut out = Vec::with_capacity((w * h) as usize);
    for y in 0..h as usize {
        for x in 0..w as usize {
            out.push(my[y] | mx[x]);
        }
    }
    out
}

fn rgb565(c: u16) -> [u8; 3] {
    let r = ((c >> 11) & 31) as u32;
    let g = ((c >> 5) & 63) as u32;
    let b = (c & 31) as u32;
    [(r * 255 / 31) as u8, (g * 255 / 63) as u8, (b * 255 / 31) as u8]
}

/// Decode one 8-byte DXT colour block into 16 RGBA pixels.
fn color_block(b: &[u8], dxt1: bool, out: &mut [[u8; 4]; 16]) {
    let c0 = u16::from_le_bytes([b[0], b[1]]);
    let c1 = u16::from_le_bytes([b[2], b[3]]);
    let (p0, p1) = (rgb565(c0), rgb565(c1));
    let mix = |a: u8, b: u8, wa: u32, wb: u32| ((a as u32 * wa + b as u32 * wb) / (wa + wb)) as u8;
    let mut pal = [[0u8; 4]; 4];
    pal[0] = [p0[0], p0[1], p0[2], 255];
    pal[1] = [p1[0], p1[1], p1[2], 255];
    if c0 > c1 || !dxt1 {
        for i in 0..3 {
            pal[2][i] = mix(p0[i], p1[i], 2, 1);
            pal[3][i] = mix(p0[i], p1[i], 1, 2);
        }
        pal[2][3] = 255;
        pal[3][3] = 255;
    } else {
        for i in 0..3 {
            pal[2][i] = mix(p0[i], p1[i], 1, 1);
        }
        pal[2][3] = 255;
        pal[3] = [0, 0, 0, 0];
    }
    let idx = u32::from_le_bytes([b[4], b[5], b[6], b[7]]);
    for (i, px) in out.iter_mut().enumerate() {
        *px = pal[((idx >> (2 * i)) & 3) as usize];
    }
}

fn dxt(fmt: u32, w: u32, h: u32, data: &[u8]) -> Option<Vec<u8>> {
    let (bw, bh) = (w.div_ceil(4).max(1), h.div_ceil(4).max(1));
    let bsize = if fmt == 0x0C { 8 } else { 16 };
    if data.len() < (bw * bh * bsize) as usize {
        return None;
    }
    let mut out = vec![0u8; (w * h * 4) as usize];
    let mut px = [[0u8; 4]; 16];
    for by in 0..bh {
        for bx in 0..bw {
            let b = &data[((by * bw + bx) * bsize) as usize..][..bsize as usize];
            match fmt {
                0x0C => color_block(b, true, &mut px),
                0x0E => {
                    color_block(&b[8..], false, &mut px);
                    for (i, p) in px.iter_mut().enumerate() {
                        let a = (b[i / 2] >> (4 * (i % 2))) & 15;
                        p[3] = a * 17;
                    }
                }
                _ => {
                    color_block(&b[8..], false, &mut px);
                    let (a0, a1) = (b[0] as u32, b[1] as u32);
                    let bits = u64::from_le_bytes([b[2], b[3], b[4], b[5], b[6], b[7], 0, 0]);
                    for (i, p) in px.iter_mut().enumerate() {
                        let k = ((bits >> (3 * i)) & 7) as u32;
                        p[3] = match k {
                            0 => a0,
                            1 => a1,
                            _ if a0 > a1 => ((8 - k) * a0 + (k - 1) * a1) / 7,
                            6 => 0,
                            7 => 255,
                            _ => ((6 - k) * a0 + (k - 1) * a1) / 5,
                        } as u8;
                    }
                }
            }
            for (i, p) in px.iter().enumerate() {
                let (x, y) = (bx * 4 + (i as u32 % 4), by * 4 + (i as u32 / 4));
                if x < w && y < h {
                    out[((y * w + x) * 4) as usize..][..4].copy_from_slice(p);
                }
            }
        }
    }
    Some(out)
}

/// Bytes per texel and whether the texels are swizzled, for the uncompressed formats read here
/// (tex_tool.py FORMATS); None for the DXT ones and the formats not handled.
fn texel_layout(fmt: u32) -> Option<(usize, bool)> {
    match fmt {
        0x00 | 0x0B | 0x19 => Some((1, true)),
        0x06 | 0x07 => Some((4, true)),
        0x12 | 0x1E => Some((4, false)),
        _ => None,
    }
}

/// How many bytes one `w` x `h` mip level takes in format `fmt` (tex_tool.py `mip_size`: 4 x 4
/// blocks of 8 bytes for DXT1, 16 for DXT3 / DXT5, each at least one block); None for a format
/// not handled here.
fn level_bytes(fmt: u32, w: u32, h: u32) -> Option<usize> {
    let blocks = (w.div_ceil(4).max(1) * h.div_ceil(4).max(1)) as usize;
    match fmt {
        0x0C => Some(blocks * 8),
        0x0E | 0x0F => Some(blocks * 16),
        _ => texel_layout(fmt).map(|(bpp, _)| (w * h) as usize * bpp),
    }
}

/// Decode one `w` x `h` level of format `fmt` from its bytes to RGBA8.
fn decode_level(fmt: u32, w: u32, h: u32, data: &[u8]) -> Option<Vec<u8>> {
    if matches!(fmt, 0x0C | 0x0E | 0x0F) {
        return dxt(fmt, w, h, data);
    }
    let (bpp, swz) = texel_layout(fmt)?;
    let n = (w * h) as usize;
    if data.len() < n * bpp {
        return None;
    }
    let map: Vec<usize> = if swz { swizzle_map(w, h) } else { (0..n).collect() };
    let mut out = vec![0u8; n * 4];
    for (i, &s) in map.iter().enumerate() {
        let p = &data[s * bpp..s * bpp + bpp];
        out[i * 4..i * 4 + 4].copy_from_slice(&match fmt {
            0x00 | 0x0B => [p[0], p[0], p[0], 255],      // P8 bump maps: indices shown as grey
            0x19 => [255, 255, 255, p[0]],
            0x06 | 0x12 => [p[2], p[1], p[0], p[3]],
            _ => [p[2], p[1], p[0], 255],
        });
    }
    Some(out)
}

/// Decode the top mip (first face) to RGBA8. Returns None for formats not handled here.
pub fn decode(t: &TexInfo, blob: &[u8]) -> Option<Vec<u8>> {
    let data = blob.get(t.offset..t.offset + t.size)?;
    decode_level(t.fmt, t.w, t.h, data)
}

/// Decode every mip level the texture's data holds (first face), largest first. The chain stops
/// early where its data runs out; None for formats not handled here.
pub fn decode_mips(t: &TexInfo, blob: &[u8]) -> Option<Mips> {
    let data = blob.get(t.offset..t.offset + t.size)?;
    let mut levels = vec![];
    let mut at = 0;
    for i in 0..t.mips {
        let (w, h) = ((t.w >> i).max(1), (t.h >> i).max(1));
        let n = level_bytes(t.fmt, w, h)?;
        let Some(px) = data.get(at..at + n).and_then(|d| decode_level(t.fmt, w, h, d)) else { break };
        levels.push(MipLevel { w, h, px });
        at += n;
    }
    (!levels.is_empty()).then_some(Mips { levels })
}

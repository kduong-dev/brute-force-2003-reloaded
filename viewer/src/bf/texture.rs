//! Texture libraries: textures-<lvl>.xmb (index) + textures-<lvl>.tex (data). Port of tex_tool.py.
//! Formats are Xbox D3DFORMAT codes; uncompressed formats are Morton-swizzled, DXT is plain blocks.

use super::bxml::Element;
use super::hash::h;

#[derive(Clone, Debug)]
pub struct TexInfo {
    pub name: u32,
    pub fmt: u32,
    pub w: u32,
    pub h: u32,
    pub offset: usize,
    pub size: usize,
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

/// Decode the top mip (first face) to RGBA8. Returns None for formats not handled here.
pub fn decode(t: &TexInfo, blob: &[u8]) -> Option<Vec<u8>> {
    let data = blob.get(t.offset..t.offset + t.size)?;
    let (w, h) = (t.w, t.h);
    if matches!(t.fmt, 0x0C | 0x0E | 0x0F) {
        return dxt(t.fmt, w, h, data);
    }
    let (bpp, swz) = match t.fmt {
        0x00 | 0x0B | 0x19 => (1, true),
        0x06 | 0x07 => (4, true),
        0x12 | 0x1E => (4, false),
        _ => return None,
    };
    let n = (w * h) as usize;
    if data.len() < n * bpp {
        return None;
    }
    let map: Vec<usize> = if swz { swizzle_map(w, h) } else { (0..n).collect() };
    let mut out = vec![0u8; n * 4];
    for (i, &s) in map.iter().enumerate() {
        let p = &data[s * bpp..s * bpp + bpp];
        out[i * 4..i * 4 + 4].copy_from_slice(&match t.fmt {
            0x00 | 0x0B => [p[0], p[0], p[0], 255],      // P8 bump maps: indices shown as grey
            0x19 => [255, 255, 255, p[0]],
            0x06 | 0x12 => [p[2], p[1], p[0], p[3]],
            _ => [p[2], p[1], p[0], 255],
        });
    }
    Some(out)
}

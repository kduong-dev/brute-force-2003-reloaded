//! Sound banks: sounds-<lvl>.xmb (index) + sounds-<lvl>.mem (in-memory Xbox ADPCM data).
//!
//!   <Sound h_1d70db35=FILE ...><h_1b8535f0 iD=SOUND_ID Type=.../></Sound>   sound id -> file
//!   <h_0f6056c9 file-name=FILE resource-type=0|1 offset=.. size=.. format=..>  file -> data
//! resource-type 0 lives in the .mem blob (one offset); type 1 are streamed from the level's
//! language wave bank (ml-sounds/<lang>/<lvl>-<lang>.xwb, absolute offsets, one per language
//! slot, all equal), see `SoundBank::add_stream`. format = sample_rate * 32 + codec (5 = Xbox ADPCM).
//! Xbox ADPCM: 36-byte blocks per channel = 4-byte header (i16 sample, u8 step index, pad) plus
//! 64 nibbles -> 65 samples (port of xwb_tool.adpcm_decode).

use std::collections::HashMap;
use std::sync::Arc;

use super::bxml::Element;
use super::hash::h;

const STEP: [i32; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66, 73, 80, 88, 97,
    107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408, 449, 494, 544, 598, 658, 724, 796,
    876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272, 2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871,
    5358, 5894, 6484, 7132, 7845, 8630, 9493, 10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623,
    27086, 29794, 32767,
];
const INDEX: [i32; 16] = [-1, -1, -1, -1, 2, 4, 6, 8, -1, -1, -1, -1, 2, 4, 6, 8];

/// Decode mono Xbox ADPCM to i16 samples.
pub fn adpcm_decode_mono(data: &[u8]) -> Vec<i16> {
    let mut out = Vec::with_capacity(data.len() / 36 * 65);
    for blk in data.chunks_exact(36) {
        let mut pred = i16::from_le_bytes([blk[0], blk[1]]) as i32;
        let mut idx = (blk[2] as i32).clamp(0, 88);
        out.push(pred as i16);
        for &byte in &blk[4..] {
            for n in [byte & 0x0F, byte >> 4] {
                let n = n as i32;
                let step = STEP[idx as usize];
                let mut diff = step >> 3;
                if n & 4 != 0 { diff += step; }
                if n & 2 != 0 { diff += step >> 1; }
                if n & 1 != 0 { diff += step >> 2; }
                pred = if n & 8 != 0 { pred - diff } else { pred + diff }.clamp(-32768, 32767);
                idx = (idx + INDEX[n as usize]).clamp(0, 88);
                out.push(pred as i16);
            }
        }
    }
    out
}

/// Decode Xbox ADPCM with any number of channels to interleaved i16 samples. A block is 36
/// bytes per channel: every channel's 4-byte header, then 4-byte runs of 8 nibbles taking turns
/// between the channels (port of xwb_tool.adpcm_decode).
pub fn adpcm_decode(data: &[u8], channels: usize) -> Vec<i16> {
    let ch = channels.max(1);
    if ch == 1 {
        return adpcm_decode_mono(data);
    }
    let mut out = Vec::with_capacity(data.len() / (36 * ch) * 65 * ch);
    for blk in data.chunks_exact(36 * ch) {
        let mut chans: Vec<Vec<i16>> = vec![];
        for c in 0..ch {
            // this channel's block as a mono one: header, then its 8 runs of 4 bytes
            let mut mono = blk[c * 4..c * 4 + 4].to_vec();
            for run in 0..8 {
                let at = 4 * ch + (run * ch + c) * 4;
                mono.extend_from_slice(&blk[at..at + 4]);
            }
            chans.push(adpcm_decode_mono(&mono));
        }
        for i in 0..65 {
            out.extend(chans.iter().map(|c| c[i]));
        }
    }
    out
}

/// 16-bit mono PCM WAV file bytes.
pub fn wav_bytes(rate: u32, samples: &[i16]) -> Vec<u8> {
    wav_bytes_channels(rate, 1, samples)
}

/// 16-bit PCM WAV file bytes of interleaved samples.
pub fn wav_bytes_channels(rate: u32, channels: u16, samples: &[i16]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut v = Vec::with_capacity(44 + data_len as usize);
    v.extend_from_slice(b"RIFF");
    v.extend_from_slice(&(36 + data_len).to_le_bytes());
    v.extend_from_slice(b"WAVEfmt ");
    v.extend_from_slice(&16u32.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes());          // PCM
    v.extend_from_slice(&channels.to_le_bytes());
    v.extend_from_slice(&rate.to_le_bytes());
    v.extend_from_slice(&(rate * 2 * channels as u32).to_le_bytes());
    v.extend_from_slice(&(2 * channels).to_le_bytes());
    v.extend_from_slice(&16u16.to_le_bytes());
    v.extend_from_slice(b"data");
    v.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        v.extend_from_slice(&s.to_le_bytes());
    }
    v
}

struct FileRec {
    offset: usize,
    size: usize,
    rate: u32,
    codec: u32,
    blob: Arc<Vec<u8>>,
}

/// Sound ids -> decodable in-memory sample data, merged over several banks.
#[derive(Default)]
pub struct SoundBank {
    /// sound id -> file name
    sounds: HashMap<u32, u32>,
    /// file name -> data (in-memory files only)
    files: HashMap<u32, FileRec>,
    /// streamed voice lines from wave banks: name hash -> (format, bank, data range)
    voices: HashMap<u32, (u32, Arc<Vec<u8>>, std::ops::Range<usize>)>,
    /// streamed files (resource-type 1) not yet attached to their wave bank: (offset, size, format)
    streamed: HashMap<u32, (usize, usize, u32)>,
    /// sound id -> its Type (MUSIC_TYPE, h("ambient"), ...)
    types: HashMap<u32, u32>,
}

/// A sound's Type for a music track (a level's sound-triggers play them); ambience beds are
/// Type "ambient".
pub const MUSIC_TYPE: u32 = 0xF92F_B2B9;

/// Entries of an XACT wave bank (WBND v2/v3, port of xwb_tool.WaveBank.parse): name, packed
/// MINIWAVEFORMAT (tag = fmt & 3: 0 PCM, 1 Xbox ADPCM; channels = fmt >> 2 & 7; rate = fmt >> 5
/// & 0x3ffffff) and the data's byte range.
pub fn xwb_entries(d: &[u8]) -> Vec<(String, u32, std::ops::Range<usize>)> {
    let u32_at = |o: usize| d.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()) as usize);
    if d.get(..4) != Some(b"WBND".as_slice()) {
        return vec![];
    }
    let seg = |i: usize| u32_at(8 + i * 4).unwrap_or(0);
    let bank = seg(0);
    let (Some(count), Some(meta_size), Some(name_size)) = (u32_at(bank + 4), u32_at(bank + 24), u32_at(bank + 28)) else { return vec![] };
    let (meta, names, names_len, wave) = (seg(2), seg(4), seg(5), seg(6));
    (0..count).filter_map(|i| {
        let m = meta + i * meta_size;
        let (fmt, off, len) = (u32_at(m + 4)? as u32, u32_at(m + 8)?, u32_at(m + 12)?);
        let name = if names_len > 0 {
            let raw = d.get(names + i * name_size..names + (i + 1) * name_size)?;
            String::from_utf8_lossy(&raw[..raw.iter().position(|&b| b == 0).unwrap_or(raw.len())]).into_owned()
        } else {
            String::new()
        };
        let range = wave + off..wave + off + len;
        (range.end <= d.len()).then_some((name, fmt, range))
    }).collect()
}

impl SoundBank {
    /// Add a bank: decoded sounds-<lvl>.xmb and its .mem blob.
    pub fn add(&mut self, index: &Element, mem: Arc<Vec<u8>>) {
        let first = |v: &super::bxml::Value| v.ints().first().copied().unwrap_or(0);
        // streamed offsets are into this bank's own wave bank (`add_stream` after it)
        self.streamed.clear();
        for e in index.walk() {
            if e.name == h("Sound") {
                let file = e.attr(0x1D70_DB35).and_then(|v| v.as_hash());
                let id = e.child(0x1B85_35F0).and_then(|p| p.attr(h("iD"))).and_then(|v| v.as_hash());
                if let (Some(f), Some(id)) = (file, id) {
                    self.sounds.entry(id).or_insert(f);
                }
                let kind = e.child(0x1B85_35F0).and_then(|p| p.attr(h("Type"))).and_then(|v| v.as_hash());
                if let (Some(id), Some(k)) = (id, kind) {
                    self.types.entry(id).or_insert(k);
                }
            }
            if e.name == 0x0F60_56C9 {
                let Some(name) = e.attr(h("file-name")).and_then(|v| v.as_hash()) else { continue };
                let get = |n: &str| e.attr(h(n)).map(first).unwrap_or(0);
                let format = get("format") as u32;
                if e.attr(h("resource-type")).map(first) != Some(0) {
                    // streamed from the language wave bank
                    self.streamed.entry(name).or_insert((get("offset") as usize, get("size") as usize, format));
                    continue;
                }
                self.files.entry(name).or_insert(FileRec {
                    offset: get("offset") as usize,
                    size: get("size") as usize,
                    rate: format >> 5,
                    codec: format & 31,
                    blob: mem.clone(),
                });
            }
        }
    }

    /// A sound's Type (see MUSIC_TYPE), if the loaded banks define it.
    pub fn sound_type(&self, id: u32) -> Option<u32> {
        self.types.get(&id).copied()
    }

    /// Every sound id that has in-memory sample data.
    pub fn ids(&self) -> Vec<u32> {
        let mut v: Vec<u32> = self.sounds.keys().copied().filter(|&id| self.has(id)).collect();
        v.sort_unstable();
        v
    }

    /// Add a voice wave bank (.xwb bytes); its lines are found by the hash of their names (as
    /// chatter files list them). Returns how many were added.
    pub fn add_voices(&mut self, xwb: Arc<Vec<u8>>) -> usize {
        let entries = xwb_entries(&xwb);
        let n = entries.len();
        for (name, fmt, range) in entries {
            self.voices.entry(h(&name)).or_insert((fmt, xwb.clone(), range));
        }
        n
    }

    /// Attach a level's language wave bank (.xwb bytes) to the streamed files of its sound bank
    /// (resource-type 1: door and gate sounds, music stings). Returns how many were attached.
    pub fn add_stream(&mut self, xwb: Arc<Vec<u8>>) -> usize {
        let mut n = 0;
        for (name, (offset, size, format)) in std::mem::take(&mut self.streamed) {
            if offset + size <= xwb.len() {
                n += 1;
                self.files.entry(name).or_insert(FileRec { offset, size, rate: format >> 5, codec: format & 31, blob: xwb.clone() });
            }
        }
        n
    }

    pub fn has(&self, id: u32) -> bool {
        self.sounds.get(&id).is_some_and(|f| self.files.contains_key(f)) || self.voices.contains_key(&id)
    }

    /// (sample rate, mono samples) of a sound id (in-memory sounds, then voice lines).
    pub fn pcm(&self, id: u32) -> Option<(u32, Vec<i16>)> {
        let Some(f) = self.sounds.get(&id).and_then(|f| self.files.get(f)) else {
            let (fmt, bank, range) = self.voices.get(&id)?;
            let (tag, channels, rate) = (fmt & 3, (fmt >> 2) & 7, (fmt >> 5) & 0x3ff_ffff);
            let data = bank.get(range.clone())?;
            return match (tag, channels) {
                (1, 1) => Some((rate.max(8000), adpcm_decode_mono(data))),
                (0, 1) => Some((rate.max(8000), data.chunks_exact(2).map(|b| i16::from_le_bytes([b[0], b[1]])).collect())),
                _ => None,
            };
        };
        let data = f.blob.get(f.offset..f.offset + f.size)?;
        match f.codec {
            5 => Some((f.rate.max(8000), adpcm_decode_mono(data))),
            _ => None,
        }
    }

    pub fn wav(&self, id: u32) -> Option<Vec<u8>> {
        self.pcm(id).map(|(rate, s)| wav_bytes(rate, &s))
    }
}

/// Footstep / landing / slide sound sets of one world material (4 random variants each).
#[derive(Clone, Debug, Default)]
pub struct Surface {
    pub id: i64,
    /// footstep type (character's h_072ab4af) -> variants
    pub footsteps: HashMap<i64, Vec<u32>>,
    pub jump_land: Vec<u32>,
    /// bullets hitting it: ammo type -> hit sounds / hit effect type; its debris effect (for
    /// flesh: the blood bundle)
    pub hit_sounds: HashMap<i64, Vec<u32>>,
    pub hit_effect: HashMap<i64, u32>,
    pub debris_effect: Option<u32>,
    pub slide: Vec<u32>,
}

/// world-materials.xmb -> surfaces.
pub fn surfaces(root: &Element) -> Vec<Surface> {
    // list values are stored as repeated attributes (one per item), sometimes as a List value
    let ids = |e: &Element| -> Vec<u32> { list(e, h("sound-effect")) };
    root.children.iter().filter(|e| e.name == h("world-material")).map(|m| {
        let mut s = Surface { id: m.attr(0x00DC_198B).and_then(|v| v.as_i64()).unwrap_or(-1), ..Default::default() };
        for e in m.walk() {
            if e.name == 0xEB5C_BA39 {
                let ty = e.attr(0x030C_AC28).and_then(|v| v.as_i64()).unwrap_or(-1);
                s.footsteps.insert(ty, ids(e));
            } else if e.name == h("jump-land") {
                s.jump_land = ids(e);
            } else if e.name == h("slide") {
                s.slide = ids(e);
            } else if e.name == 0xE8B4_97F1 {
                // a bullet hitting this material: per ammo type, the hit effect and four sounds
                let ammo = e.attr(h("ammo-type")).and_then(|v| v.as_i64()).unwrap_or(-1);
                s.hit_sounds.insert(ammo, list(e, h("hit-sound")));
                if let Some(fx) = e.attr(0xF6F0_8598).and_then(|v| v.as_hash()) {
                    s.hit_effect.insert(ammo, fx);
                }
            } else if e.name == 0xF3E8_8948 {
                s.debris_effect = e.attr(h("debris-effect")).and_then(|v| v.as_hash());
            }
        }
        s
    }).collect()
}

/// A hash list attribute: stored as repeated attributes (one per item) or as one List value;
/// empty entries dropped.
fn list(e: &Element, name: u32) -> Vec<u32> {
    {
        e.attrs.iter().filter(|(k, _)| *k == name).flat_map(|(_, v)| match v {
            super::bxml::Value::List(l) => l.iter().filter_map(|x| x.as_hash()).collect::<Vec<_>>(),
            other => other.as_hash().into_iter().collect(),
        }).filter(|&x| x != 0 && x != super::hash::h("")).collect()
    }
}

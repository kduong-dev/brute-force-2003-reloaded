//! Sound banks: sounds-<lvl>.xmb (index) + sounds-<lvl>.mem (in-memory Xbox ADPCM data).
//!
//!   <Sound h_1d70db35=FILE falloff=.. h_fd40e332=.. h_eff0deab=..><h_1b8535f0 iD=SOUND_ID Type=.../></Sound>
//!   <h_0f6056c9 file-name=FILE resource-type=0|1|2 offset=.. size=.. time=.. format=..>  file -> data
//! resource-type 0 lives in the .mem blob (one offset); type 1 are streamed from the level's
//! language wave bank (ml-sounds/<lang>/<lvl>-<lang>.xwb, absolute offsets, one per language
//! slot, all equal), see `SoundBank::add_stream`; type 2 are the tracks of the level's music bank
//! (data/sounds/<lvl>.xwb: its music and ambience beds, whose entry names hash to the sound ids:
//! "tutorial_01" is h_f5862d3f), kept apart (`SoundBank::is_music`). format = sample_rate * 32 +
//! codec (5 = Xbox ADPCM mono, 9 stereo). A sound id is the hash of its wave's name in the wave
//! banks (the tutorial's h_e759c574 is dxmi_bratton_m00049).
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
    /// the music bank's files (resource-type 2, data/sounds/<lvl>.xwb): file name -> (size,
    /// format), for their length; their data is the music bank's (play_sound_triggers.rs)
    music: HashMap<u32, (usize, u32)>,
    /// sound id -> its Type (MUSIC_TYPE, h("ambient"), ...)
    types: HashMap<u32, u32>,
    /// sound id -> its library entry's distances and volume (`SoundParams`)
    params: HashMap<u32, SoundParams>,
    /// sound id -> its wave's name in the wave banks added (`add_voices`, `add_stream`), for logs
    names: HashMap<u32, String>,
}

/// A sound library entry's distances and volume: its `<Sound>` element's attributes, which the
/// runtime entry FUN_0017beb0 finds keeps at +8 (the min distance), +0xc (the max) and +0x10 (the
/// volume). A sound-trigger uses them where its own are -1 (FUN_002060c0). Which attribute lands
/// in which field is inferred from their use: FUN_0017c170 makes a voice 3D only when +8 isn't
/// -1, and `falloff` is -1 on exactly the 2D sounds (every voice line, the music).
#[derive(Clone, Copy, Debug)]
pub struct SoundParams {
    /// `falloff` (+8): the distance it's heard at full volume within (DirectSound's min
    /// distance); -1 a 2D sound
    pub min_distance: f32,
    /// h_fd40e332 (+0xc): the distance it no longer quietens beyond (DirectSound's max distance;
    /// every entry has one)
    pub max_distance: f32,
    /// h_eff0deab (+0x10): its volume, 0-1; 1 for the 28% of entries without one (the voice
    /// lines among them): inferred
    pub volume: f32,
}

/// A library entry's h_fd40e332, its max distance (see `SoundParams`).
const H_MAX_DISTANCE: u32 = 0xFD40_E332;
/// A library entry's h_eff0deab (`volume`, recovered by hashing), its volume 0-1.
const H_VOLUME: u32 = 0xEFF0_DEAB;

/// A format's bytes a second, FUN_00183c30's nAvgBytesPerSec: `format` packs the codec (bits 0-1:
/// 1 Xbox ADPCM, else PCM), the channels (bits 2-4), the sample rate (bits 5-30) and, for PCM,
/// 16-bit samples (bit 31; else 8). Xbox ADPCM counts 36 bytes a channel for each whole 64 of the
/// rate ((rate >> 6) x 36 x channels: 12384 at 22050 Hz mono, 24768 stereo); PCM rate x channels x
/// bytes a sample.
pub fn bytes_per_second(format: u32) -> f32 {
    let (tag, channels, rate) = (format & 3, (format >> 2) & 7, (format >> 5) & 0x3ff_ffff);
    let n = if tag == 1 { (rate >> 6) * 36 * channels } else { rate * channels * if format & 0x8000_0000 != 0 { 2 } else { 1 } };
    n.max(1) as f32
}

/// A sound's Type for a music track (a level's sound-triggers play them); ambience beds are
/// Type "ambient".
pub const MUSIC_TYPE: u32 = 0xF92F_B2B9;

/// The entry names of an XACT wave bank from its start alone (`head`: as much of the file as
/// holds its header, metadata and names; see `xwb_entries`), without its wave data. Empty for
/// a stub bank (splash_screen.xwb, mp2.xwb: 76 bytes, no entries) or one without names.
pub fn xwb_names(head: &[u8]) -> Vec<String> {
    let u32_at = |o: usize| head.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()) as usize);
    if head.get(..4) != Some(b"WBND".as_slice()) {
        return vec![];
    }
    let seg = |i: usize| u32_at(8 + i * 4).unwrap_or(0);
    let bank = seg(0);
    let (Some(count), Some(name_size)) = (u32_at(bank + 4), u32_at(bank + 28)) else { return vec![] };
    let (names, names_len) = (seg(4), seg(5));
    if names_len == 0 || name_size == 0 {
        return vec![];
    }
    (0..count).filter_map(|i| {
        let raw = head.get(names + i * name_size..names + (i + 1) * name_size)?;
        Some(String::from_utf8_lossy(&raw[..raw.iter().position(|&b| b == 0).unwrap_or(raw.len())]).into_owned())
    }).collect()
}

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

/// One entry of a wave bank (`d` the .xwb's bytes; its MINIWAVEFORMAT `fmt` and data `range` from
/// `xwb_entries`) as WAV file bytes: Xbox ADPCM decoded, any channel count (the music banks'
/// tracks are stereo), or PCM taken as 16-bit.
pub fn xwb_wav(d: &[u8], fmt: u32, range: std::ops::Range<usize>) -> Option<Vec<u8>> {
    let (tag, channels, rate) = (fmt & 3, (fmt >> 2) & 7, (fmt >> 5) & 0x3ff_ffff);
    let data = d.get(range)?;
    let pcm = match tag {
        1 => adpcm_decode(data, channels as usize),
        0 => data.chunks_exact(2).map(|b| i16::from_le_bytes([b[0], b[1]])).collect(),
        _ => return None,
    };
    Some(wav_bytes_channels(rate.max(8000), channels.max(1) as u16, &pcm))
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
                let float = |k: u32| e.attr(k).and_then(|v| v.as_f32());
                if let Some(id) = id {
                    self.params.entry(id).or_insert(SoundParams {
                        min_distance: float(h("falloff")).unwrap_or(-1.0),
                        max_distance: float(H_MAX_DISTANCE).unwrap_or(-1.0),
                        volume: float(H_VOLUME).unwrap_or(1.0),
                    });
                }
            }
            if e.name == 0x0F60_56C9 {
                let Some(name) = e.attr(h("file-name")).and_then(|v| v.as_hash()) else { continue };
                let get = |n: &str| e.attr(h(n)).map(first).unwrap_or(0);
                let format = get("format") as u32;
                match e.attr(h("resource-type")).map(first) {
                    Some(0) => {}
                    // a track of the level's music bank (its offsets are into that file, not the
                    // language wave bank's)
                    Some(2) => {
                        self.music.entry(name).or_insert((get("size") as usize, format));
                        continue;
                    }
                    // streamed from the language wave bank
                    _ => {
                        self.streamed.entry(name).or_insert((get("offset") as usize, get("size") as usize, format));
                        continue;
                    }
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

    /// A sound's library entry (`SoundParams`), if the loaded banks define it.
    pub fn params(&self, id: u32) -> Option<SoundParams> {
        self.params.get(&id).copied()
    }

    /// Whether a sound is a track of the level's music bank (its file's resource-type 2: the
    /// music cues and ambience beds the level's sound-triggers play, data/sounds/<lvl>.xwb).
    pub fn is_music(&self, id: u32) -> bool {
        self.sounds.get(&id).is_some_and(|f| self.music.contains_key(f))
    }

    /// How long a sound plays as the game reckons it, the sound system's vtable +0x64
    /// (0x17f620): its file's size over its format's bytes a second (`bytes_per_second`), for the
    /// slot the banks keep (the first: English). The file records' own `time` agrees (the
    /// tutorial's terminal_03: 12852 / 12384 = 1.0378 s, time="1.037791"). A sound-trigger's
    /// FINISHED comes that long after it starts (FUN_002062a0). None for a sound the loaded banks
    /// don't have.
    pub fn length(&self, id: u32) -> Option<f32> {
        if let Some(f) = self.sounds.get(&id).and_then(|f| self.files.get(f)) {
            return Some(f.size as f32 / bytes_per_second((f.rate << 5) | f.codec));
        }
        if let Some(&(size, format)) = self.sounds.get(&id).and_then(|f| self.music.get(f)) {
            return Some(size as f32 / bytes_per_second(format));
        }
        let (fmt, _, range) = self.voices.get(&id)?;
        Some(range.len() as f32 / bytes_per_second(*fmt))
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
            self.names.entry(h(&name)).or_insert(name);
        }
        n
    }

    /// The name of a sound's wave in the wave banks added (dxmi_bratton_m00049 for the
    /// tutorial's h_e759c574), if one had it.
    pub fn name(&self, id: u32) -> Option<&str> {
        self.names.get(&id).map(String::as_str)
    }

    /// Attach a level's language wave bank (.xwb bytes) to the streamed files of its sound bank
    /// (resource-type 1: door and gate sounds, music stings). Returns how many were attached.
    pub fn add_stream(&mut self, xwb: Arc<Vec<u8>>) -> usize {
        for (name, _, _) in xwb_entries(&xwb) {
            self.names.entry(h(&name)).or_insert(name);
        }
        let mut n = 0;
        for (name, (offset, size, format)) in std::mem::take(&mut self.streamed) {
            if offset + size <= xwb.len() {
                n += 1;
                self.files.entry(name).or_insert(FileRec { offset, size, rate: format >> 5, codec: format & 31, blob: xwb.clone() });
            }
        }
        n
    }

    /// A bank of only the sounds `ids` this one can play from its files (in-memory or attached
    /// streamed ones), each with its own copy of its bytes: what's kept of a level's language
    /// wave bank for a few voice lines, without the whole bank (type_assets.rs, #180).
    pub fn subset(&self, ids: &std::collections::HashSet<u32>) -> SoundBank {
        let mut out = SoundBank::default();
        for &id in ids {
            let Some((&file, rec)) = self.sounds.get(&id).and_then(|f| self.files.get_key_value(f)) else { continue };
            let Some(data) = rec.blob.get(rec.offset..rec.offset + rec.size) else { continue };
            out.sounds.insert(id, file);
            if let Some(&t) = self.types.get(&id) {
                out.types.insert(id, t);
            }
            if let Some(&p) = self.params.get(&id) {
                out.params.insert(id, p);
            }
            out.files.entry(file).or_insert(FileRec { offset: 0, size: rec.size, rate: rec.rate, codec: rec.codec, blob: Arc::new(data.to_vec()) });
        }
        out
    }

    /// Add `other`'s sounds `ids` where this bank can't play them yet: its file for each (a sound
    /// already named here keeps its own file name, which gets `other`'s data when it has none).
    /// How many became playable.
    pub fn merge(&mut self, other: &SoundBank, ids: &std::collections::HashSet<u32>) -> usize {
        let mut n = 0;
        for &id in ids {
            if self.has(id) {
                continue;
            }
            let Some((&file, rec)) = other.sounds.get(&id).and_then(|f| other.files.get_key_value(f)) else { continue };
            let name = *self.sounds.entry(id).or_insert(file);
            if let Some(&t) = other.types.get(&id) {
                self.types.entry(id).or_insert(t);
            }
            if let Some(&p) = other.params.get(&id) {
                self.params.entry(id).or_insert(p);
            }
            if !self.files.contains_key(&name) {
                self.files.insert(name, FileRec { offset: rec.offset, size: rec.size, rate: rec.rate, codec: rec.codec, blob: rec.blob.clone() });
                n += 1;
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

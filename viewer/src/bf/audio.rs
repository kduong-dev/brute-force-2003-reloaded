//! Sound banks: sounds-<lvl>.xmb (index) + sounds-<lvl>.mem (in-memory Xbox ADPCM data).
//!
//!   <Sound h_1d70db35=FILE falloff=.. h_fd40e332=.. h_eff0deab=..><h_1b8535f0 iD=SOUND_ID Type=.../></Sound>
//!   <h_0f6056c9 file-name=FILE resource-type=0|1|2 offset=.. size=.. time=.. format=..>  file -> data
//! resource-type 0 lives in the .mem blob (one offset); type 1 are streamed from the level's
//! language wave bank (ml-sounds/<lang>/<lvl>-<lang>.xwb, absolute offsets, one per language
//! slot, all equal), see `SoundBank::add_stream`; type 2 are the tracks of the level's music bank
//! (data/sounds/<lvl>.xwb: its music and ambience beds), kept apart (`SoundBank::is_music`,
//! `MusicFile`). format = sample_rate * 32 + codec (5 = Xbox ADPCM mono, 9 stereo).
//!
//! **A sound reaches its wave through the file record, never through a name** (#398: the game's
//! FUN_0017bf40 finds the record by the sound id, FUN_00183710 seeks the bank file to the record's
//! `offset` and streams its `size` bytes; the bank's own entry table and names are never read).
//! The sound id is the hash of the sound *entry's* name, which is the wave's name for most (the
//! tutorial's h_f5862d3f is "tutorial_01", its h_e759c574 dxmi_bratton_m00049) and not
//! for 24 of the 187 entries of the 38 level banks (sdm_e40's h_10fb2011 is
//! creepy_estuary_mutant_01, sdm_m07's h_e70fb918 ambiencehellish-2ch): looking a wave up by the
//! hash of its name loses those. The wave banks' names are for the logs and the test tools'
//! lists (`xwb_directory`, `SoundBank::add_music_names`).
//! Xbox ADPCM: 36-byte blocks per channel = 4-byte header (i16 sample, u8 step index, pad) plus
//! 64 nibbles -> 65 samples (port of xwb_tool.adpcm_decode).
//!
//! A speech sound (Type h_012ed199) also carries its lip-sync (#255, #325): a speech block of
//! viseme tracks, one per mouth shape, each a curve of `<frame time weight>` keys (seconds into
//! the line, percent), which the face controller plays as the line is said (`Visemes`):
//!
//!   <Sound ..><h_1b8535f0 iD=.. Type="h_012ed199"/>
//!     <h_012ed199>                      the speech block
//!       <h_1b8535f0 h_11b737e0=../>     a stringid on a few (not read by the game's face code)
//!       <h_0128a038>
//!         <h_1b8535f0 h_f63f0f49="30"/> always 30 (kept by the game at the block's +4, never read)
//!         <h_f3280a84>                  a track
//!           <h_1b8535f0 h_fcf7be63=VISEME/>   the viseme's name hash (`face::viseme_by_hash`)
//!           <frame><h_1b8535f0 time="0.128" weight="0"/></frame> ...

use std::collections::HashMap;
use std::sync::Arc;

use super::bxml::Element;
use super::face::{viseme_by_hash, VISEMES};
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

/// Where a track of a level's music bank is, as its sound library's file record has it (a
/// resource-type 2 `<h_0f6056c9>`, FUN_0017adb0 copies it into the runtime record: +0x1c the
/// `offset`, +0x30 the `size`, +0x58 the `format`, +0x6c the hash of the library's name): the
/// bytes at `offset` for `size` of `data/sounds/<library>.xwb`, in `format`. FUN_00183710 seeks the
/// bank file there and streams the sound; the bank's entry table is not consulted (#398).
#[derive(Clone, Debug, PartialEq)]
pub struct MusicFile {
    /// the sound library the record belongs to (`sounds-<library>.xmb`, `library_of`), whose
    /// bank file `data/sounds/<library>.xwb` the bytes are in (FUN_00183570's path slot 2,
    /// `<data dir>\sounds\<library>.xwb`)
    pub library: Arc<str>,
    /// the absolute byte offset in that file (`offset`, the first of its five language slots)
    pub offset: usize,
    /// the length in bytes (`size`)
    pub size: usize,
    /// the packed format (`format`): codec in bits 0-1 (1 Xbox ADPCM), channels in bits 2-4, sample
    /// rate from bit 5; 705609 is 22050 Hz stereo ADPCM, every track's
    pub format: u32,
}

impl MusicFile {
    /// The bytes' place in the bank file.
    pub fn range(&self) -> std::ops::Range<usize> {
        self.offset..self.offset + self.size
    }

    /// How long the track plays as the game reckons it (`bytes_per_second`; the record's own
    /// `time` is the same figure).
    pub fn secs(&self) -> f32 {
        self.size as f32 / bytes_per_second(self.format)
    }

    /// The track's bytes, read from its bank file in `data_dir` (`<data_dir>/sounds/<library>.xwb`):
    /// a seek to the offset and a read of the size, as the stream does, not the whole bank.
    pub fn read(&self, data_dir: &std::path::Path) -> Result<Vec<u8>, String> {
        use std::io::{Read, Seek, SeekFrom};
        let path = data_dir.join("sounds").join(format!("{}.xwb", self.library));
        let mut f = std::fs::File::open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        f.seek(SeekFrom::Start(self.offset as u64)).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut bytes = vec![0u8; self.size];
        f.read_exact(&mut bytes).map_err(|e| format!("{}: {} bytes at {} aren't all there: {e}", path.display(), self.size, self.offset))?;
        Ok(bytes)
    }

    /// The track as a WAV file, from the bytes `read` gave (`xwb_wav`: Xbox ADPCM of any channel
    /// count, or PCM); None for a format the decoder doesn't read.
    pub fn wav(&self, bytes: &[u8]) -> Option<Vec<u8>> {
        xwb_wav(bytes, self.format, 0..bytes.len())
    }
}

/// The library a sound bank's archive member holds: `e05/sounds-e05.xmb` is "e05" (the folder
/// and the `sounds-` and `.xmb` taken off), the name of its wave bank `data/sounds/e05.xwb`
/// (FUN_0017adb0 builds the path from the library's name).
pub fn library_of(member: &str) -> &str {
    member.rsplit('/').next().unwrap_or(member).trim_start_matches("sounds-").trim_end_matches(".xmb")
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
    /// the music banks' files (resource-type 2, data/sounds/<library>.xwb): file name -> where
    /// their bytes are (`MusicFile`); the data is read from the bank file when a track plays
    /// (play_sound_triggers.rs `MusicBank`), the first record of a file name kept
    music: HashMap<u32, MusicFile>,
    /// sound id -> its Type (MUSIC_TYPE, h("ambient"), ...)
    types: HashMap<u32, u32>,
    /// sound id -> its library entry's distances and volume (`SoundParams`)
    params: HashMap<u32, SoundParams>,
    /// sound id -> its wave's name in the wave banks added (`add_voices`, `add_stream`), for logs
    names: HashMap<u32, String>,
    /// sound id -> its speech block's viseme tracks (the runtime entry's +0x1c, FUN_0017b800),
    /// for the speech sounds that have one (`Visemes`)
    visemes: HashMap<u32, Arc<Visemes>>,
}

/// A `<Sound>`'s speech block (h_012ed199, the speech Type's name as its element too).
const H_SPEECH: u32 = 0x012E_D199;
/// The speech block's viseme block (h_0128a038), which holds the tracks.
const H_VISEME_BLOCK: u32 = 0x0128_A038;
/// One viseme track (h_f3280a84; the parser makes a new one at 0x1a2577).
const H_TRACK: u32 = 0xF328_0A84;
/// A track's viseme name (h_fcf7be63; FUN_001a24b0 keeps its hash, or the hash of a string).
const H_VISEME_NAME: u32 = 0xFCF7_BE63;
/// The element that holds an element's own attributes in the library's documents (h_1b8535f0:
/// a `<Sound>`'s `iD` and `Type`, a track's name, a frame's `time` and `weight`).
const H_FIELDS: u32 = 0x1B85_35F0;

/// One key of a viseme track: FUN_0017b800 makes each 16 bytes {time, weight, 0, 1.0} from the
/// parsed frame's {time, weight}, then FUN_0017a990 sets the last field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VisemeKey {
    /// seconds into the line (a key may lie before 0, which the line, read from 0 on, only
    /// meets as the start of the span that holds 0)
    pub time: f32,
    /// percent, as the data has it (85 the usual peak; up to 140 in the data, which the blend
    /// holds at 1)
    pub weight: f32,
    /// 1 / (the next key's time - this one's) (FUN_0017a990), what FUN_0017aa70 multiplies the
    /// time since this key by; 1.0, the value the key is made with, on a track of two keys or
    /// fewer, which FUN_0017a990 leaves alone (none in the data: every track has 3 keys or more)
    pub inv: f32,
}

/// One viseme's track of a speech sound (FUN_0017b800's 0x18-byte record: +0 the name hash, +8
/// / +0xc the keys, +0x14 the viseme index).
#[derive(Clone, Debug, PartialEq)]
pub struct VisemeTrack {
    /// the viseme it drives, 0-21 (`face::VISEME_NAMES`; the data's tracks name only the 15
    /// mouth shapes)
    pub viseme: usize,
    pub keys: Vec<VisemeKey>,
}

impl VisemeTrack {
    /// A track of `viseme` from its frames' (time, weight) in the data's order (FUN_0017b800 keeps
    /// it; the data's are all in time order, none at the same time): each key's `inv` the inverse
    /// of the gap to the next one when the track has more than two keys (FUN_0017a990, the last
    /// key's against nothing: 1.0 kept), else 1.0 throughout.
    pub fn new(viseme: usize, frames: &[(f32, f32)]) -> VisemeTrack {
        let mut keys: Vec<VisemeKey> = frames.iter().map(|&(time, weight)| VisemeKey { time, weight, inv: 1.0 }).collect();
        if keys.len() > 2 {
            for k in 0..keys.len() - 1 {
                keys[k].inv = 1.0 / (keys[k + 1].time - keys[k].time);
            }
        }
        VisemeTrack { viseme, keys }
    }

    /// The track's weight (0-1, x 0.01 of the data's percent) `t` seconds into the line:
    /// FUN_0017aa70. Before the first key, its weight; after the last, the last's; else the key
    /// whose span holds t (the first k with key k's time <= t < key k+1's, or the last span) and
    /// the next, mixed by (t - key k's time) x key k's `inv`. 0 for a track without keys.
    pub fn sample(&self, t: f32) -> f32 {
        let keys = &self.keys;
        let (Some(first), Some(last)) = (keys.first(), keys.last()) else { return 0.0 };
        if t < first.time {
            return first.weight * 0.01;
        }
        if last.time < t {
            return last.weight * 0.01;
        }
        // (the search stops at the last span: with t at the last key's time it is that one)
        let n = keys.len();
        let k = (0..n.saturating_sub(2)).find(|&k| keys[k].time <= t && t < keys[k + 1].time).unwrap_or(n.saturating_sub(2));
        let Some(next) = keys.get(k + 1) else { return keys[k].weight * 0.01 };
        let f = (t - keys[k].time) * keys[k].inv;
        (f * next.weight + (1.0 - f) * keys[k].weight) * 0.01
    }
}

/// A speech sound's viseme tracks: the runtime block FUN_0017b800 builds into its library entry's
/// +0x1c from the `<h_012ed199>` speech block (0 for a sound without one: None here). The block
/// has a slot for each of the 22 visemes, filled from the tracks by their name's place
/// (`face::viseme_by_hash`): a track naming another is dropped, and a later track of the same
/// viseme takes its slot (none in the data does).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Visemes {
    /// the tracks, by viseme
    pub tracks: Vec<VisemeTrack>,
}

impl Visemes {
    /// FUN_0017aa30: each track's weight at `t` s into the line (`VisemeTrack::sample`) into
    /// `weights` at its viseme. The visemes no track drives keep what they had.
    pub fn write(&self, t: f32, weights: &mut [f32]) {
        for track in &self.tracks {
            if let Some(w) = weights.get_mut(track.viseme) {
                *w = track.sample(t);
            }
        }
    }

    /// A `<Sound>`'s viseme tracks, if it has a speech block (FUN_0017b800; the parse's
    /// FUN_001a24b0 for a track's name, 0x1a2577 for a new track). A block with no tracks still
    /// makes one, with none: the face's blend then shows the base through the line.
    fn read(sound: &Element) -> Option<Visemes> {
        let block = sound.child(H_SPEECH)?;
        let mut slots: Vec<Option<VisemeTrack>> = vec![None; VISEMES];
        for track in block.children_named(H_VISEME_BLOCK).flat_map(|b| b.children_named(H_TRACK)) {
            let Some(viseme) = track.child(H_FIELDS).and_then(|f| f.attr(H_VISEME_NAME)).and_then(|v| v.as_hash()).and_then(viseme_by_hash) else {
                continue;
            };
            let frames: Vec<(f32, f32)> = track.children_named(h("frame")).filter_map(|f| {
                let fields = f.child(H_FIELDS)?;
                Some((fields.attr(h("time"))?.as_f32()?, fields.attr(h("weight"))?.as_f32()?))
            }).collect();
            slots[viseme] = Some(VisemeTrack::new(viseme, &frames));
        }
        Some(Visemes { tracks: slots.into_iter().flatten().collect() })
    }
}

/// A sound library entry's distances and volume: its `<Sound>` element's attributes, which the
/// runtime entry FUN_0017beb0 finds keeps at +8 (the min distance), +0xc (the max) and +0x10 (the
/// volume). A voice's play parameters left unset come from them (FUN_0017c0b0: its min distance
/// +0x30 from +8, its max +0x38 from +0xc, its volume +0x48 from +0x10); a sound-trigger's own
/// are used where they're over 0 (FUN_002060c0). FUN_0017c170 makes a voice 3D only when +8
/// isn't -1. The attributes' order fits: in the tutorial's library `falloff` is below
/// h_fd40e332 in 618 of the 619 entries that have both (equal in the other, 50 and 50), and -1 in
/// 100 entries, the 2D sounds (the voice lines, the music, three ambiences).
#[derive(Clone, Copy, Debug)]
pub struct SoundParams {
    /// `falloff` (+8): the distance it's heard at full volume within (DirectSound's min
    /// distance); -1 a 2D sound
    pub min_distance: f32,
    /// h_fd40e332 (+0xc): the distance its rolloff curve reaches nothing at (DirectSound's max
    /// distance; every entry has one)
    pub max_distance: f32,
    /// h_eff0deab (+0x10): its volume, 0-1; 1 for the 28% of entries without one (the voice
    /// lines among them; the parser's default isn't traced: the 1 is a guess)
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

/// One wave of a level's music bank as the test tools list it (`SoundBank::bank_tracks`, #398).
#[derive(Clone, Debug, PartialEq)]
pub struct BankTrack {
    /// the wave's name in the bank's entry table (the game never reads it; the wave is found by
    /// the file record's place)
    pub name: String,
    /// where its bytes are: the file record of the entry that plays it, else the bank entry's own
    /// place (every wave of the game's banks has a record)
    pub file: MusicFile,
    /// the Type of the sound entry that plays it (`MUSIC_TYPE`, or "ambient" for a bed); None when
    /// no entry does
    pub kind: Option<u32>,
    /// the sound ids whose entries play it
    pub sounds: Vec<u32>,
}

/// A wave of a music bank decoded for a mix (play.rs `music_mix`): its name, its WAV and the Type
/// of the sound entry that plays it (`BankTrack::kind`), which says whether it's the level's music
/// or a bed; None when no entry does, and the mix falls back to the name (`amb...` is a bed).
pub struct MixTrack {
    pub name: String,
    pub wav: Vec<u8>,
    pub kind: Option<u32>,
}

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

/// How much of a wave bank file holds its header, entry table and names, for `xwb_directory`: in
/// data/sounds the names end by byte 868 at most (m09_b.xwb's, 9 entries), so 16 KB is more than
/// enough, and none of the wave data is read.
pub const XWB_HEAD: u64 = 16 * 1024;

/// The entries of an XACT wave bank (WBND v2/v3, port of xwb_tool.WaveBank.parse) from its start
/// alone (`head`: `XWB_HEAD` bytes of the file will do): name, packed MINIWAVEFORMAT (tag = fmt &
/// 3: 0 PCM, 1 Xbox ADPCM; channels = fmt >> 2 & 7; rate = fmt >> 5 & 0x3ffffff) and the data's
/// byte range **in the file**, which may reach past `head` (`xwb_entries` drops what reaches past
/// the bytes it is given). Empty names for a bank without any.
pub fn xwb_directory(head: &[u8]) -> Vec<(String, u32, std::ops::Range<usize>)> {
    let d = head;
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
        Some((name, fmt, wave + off..wave + off + len))
    }).collect()
}

/// The entries of a whole XACT wave bank file `d` (`xwb_directory`), those whose data lie in it.
pub fn xwb_entries(d: &[u8]) -> Vec<(String, u32, std::ops::Range<usize>)> {
    xwb_directory(d).into_iter().filter(|e| e.2.end <= d.len()).collect()
}

/// How long a WAV made by `wav_bytes_channels` plays (s): its data over its byte rate (the
/// header's bytes 28-31 and 40-43); 0 for a stub or no header.
pub fn wav_secs(wav: &[u8]) -> f32 {
    let u32_at = |o: usize| wav.get(o..o + 4).map_or(0, |b| u32::from_le_bytes(b.try_into().unwrap()));
    let rate = u32_at(28);
    if rate == 0 { 0.0 } else { u32_at(40) as f32 / rate as f32 }
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
    /// Add a bank: decoded sounds-<library>.xmb (`library`, see `library_of`) and its .mem blob.
    pub fn add(&mut self, library: &str, index: &Element, mem: Arc<Vec<u8>>) {
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
                    // (the first entry of an id keeps its tracks, as its other fields)
                    if !self.visemes.contains_key(&id) {
                        if let Some(v) = Visemes::read(e) {
                            self.visemes.insert(id, Arc::new(v));
                        }
                    }
                }
            }
            if e.name == 0x0F60_56C9 {
                let Some(name) = e.attr(h("file-name")).and_then(|v| v.as_hash()) else { continue };
                let get = |n: &str| e.attr(h(n)).map(first).unwrap_or(0);
                let format = get("format") as u32;
                match e.attr(h("resource-type")).map(first) {
                    Some(0) => {}
                    // a track of the level's music bank (its offset is into that library's own
                    // bank file, not the language wave bank's): where its bytes are, whatever the
                    // wave in the bank is called (#398)
                    Some(2) => {
                        self.music.entry(name).or_insert_with(|| MusicFile {
                            library: library.into(),
                            offset: get("offset") as usize,
                            size: get("size") as usize,
                            format,
                        });
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

    /// A speech sound's viseme tracks (its library entry's +0x1c), if the loaded banks define it
    /// with a speech block: what a face controller keeps as it says the line (FUN_001435d0's +4).
    pub fn visemes(&self, id: u32) -> Option<Arc<Visemes>> {
        self.visemes.get(&id).cloned()
    }

    /// Whether a sound is a track of the level's music bank (its file's resource-type 2: the
    /// music cues and ambience beds the level's sound-triggers play, data/sounds/<lvl>.xwb).
    pub fn is_music(&self, id: u32) -> bool {
        self.sounds.get(&id).is_some_and(|f| self.music.contains_key(f))
    }

    /// Where a music-bank sound's bytes are: sound id -> its file name -> that file's record
    /// (offset, size, format) and the library whose bank file holds it (FUN_0017bf40's lookup, then
    /// FUN_00183710's seek; #398). None for a sound that isn't a music-bank track.
    pub fn music_file(&self, id: u32) -> Option<&MusicFile> {
        self.sounds.get(&id).and_then(|f| self.music.get(f))
    }

    /// The file name of the record that has `library`'s bank bytes at `offset`, if one does.
    fn music_file_at(&self, library: &str, offset: usize) -> Option<u32> {
        self.music.iter().filter(|(_, m)| &*m.library == library && m.offset == offset).map(|(f, _)| *f).min()
    }

    /// The sounds (ids, in order) whose entries play the wave at `offset` of `library`'s music bank:
    /// the entries that name the file record with that place. Empty for a wave no entry plays (none
    /// in the game's 38 banks: every wave of every bank is some entry's).
    pub fn track_sounds(&self, library: &str, offset: usize) -> Vec<u32> {
        let Some(file) = self.music_file_at(library, offset) else { return vec![] };
        let mut ids: Vec<u32> = self.sounds.iter().filter(|(_, f)| **f == file).map(|(id, _)| *id).collect();
        ids.sort_unstable();
        ids
    }

    /// The Type of the sound entry that plays the wave at `offset` of `library`'s music bank
    /// (`MUSIC_TYPE`, or "ambient" for a bed; the lowest sound id's when several entries share the
    /// wave, which in the data have one Type between them), if an entry does: how a track is told
    /// to be the level's music or its bed without going by the wave's name (#398).
    pub fn track_kind(&self, library: &str, offset: usize) -> Option<u32> {
        self.track_sounds(library, offset).into_iter().find_map(|id| self.types.get(&id).copied())
    }

    /// Name the sounds that play the waves of `library`'s music bank, for the logs: `head` is the
    /// first `XWB_HEAD` bytes of `data/sounds/<library>.xwb`, whose entry at a file record's offset
    /// gives its name to the sounds that name that record (the game itself never reads the names).
    /// A name already known stays. Returns how many sounds got one.
    pub fn add_music_names(&mut self, library: &str, head: &[u8]) -> usize {
        let mut named = vec![];
        for (name, _, range) in xwb_directory(head) {
            named.extend(self.track_sounds(library, range.start).into_iter().map(|id| (id, name.clone())));
        }
        let before = self.names.len();
        for (id, name) in named {
            self.names.entry(id).or_insert(name);
        }
        self.names.len() - before
    }

    /// The waves of `library`'s music bank (`head`: the first `XWB_HEAD` bytes of its file) in the
    /// bank's order, each with where its bytes are, the sounds that play it and their Type: the
    /// list the test tools' track menu shows (#398).
    pub fn bank_tracks(&self, library: &str, head: &[u8]) -> Vec<BankTrack> {
        xwb_directory(head).into_iter().map(|(name, fmt, range)| {
            let file = self.music_file_at(library, range.start).and_then(|f| self.music.get(&f)).cloned()
                .unwrap_or(MusicFile { library: library.into(), offset: range.start, size: range.len(), format: fmt });
            BankTrack { name, kind: self.track_kind(library, range.start), sounds: self.track_sounds(library, range.start), file }
        }).collect()
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
        if let Some(m) = self.music_file(id) {
            return Some(m.secs());
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
            if let Some(v) = self.visemes.get(&id) {
                out.visemes.insert(id, v.clone());
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
            if let Some(v) = other.visemes.get(&id) {
                self.visemes.entry(id).or_insert_with(|| v.clone());
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

#[cfg(test)]
mod tests {
    use super::super::bxml::Value;
    use super::*;

    /// The first keys of the tutorial's "yeah yeah" line's `if` track (h_ec50fb88,
    /// sounds-tutorial.xml), in the data's (seconds, percent).
    const YEAH_IF: [(f32, f32); 7] = [(-0.264, 0.0), (0.0, 20.0), (0.608, 0.0), (0.72, 85.0), (0.944, 0.0), (1.104, 85.0), (1.28, 20.0)];

    /// FUN_0017aa70 against the game: todo/255 take07's face log has Tex's weights as he says
    /// h_ec50fb88, read at the controller's clock less the line's start (+0x40 - +0x44). Its `if`
    /// at those times is the track's, to the log's three decimals: 0.179 at 0.0631 s, 0.114 at
    /// 0.2617, 0.349 at 0.6540, 0.848 at 0.7197, 0.578 at 0.7917, 0.248 at 0.9906, 0.591 at 1.0552.
    #[test]
    fn a_track_gives_the_games_weights() {
        let track = VisemeTrack::new(6, &YEAH_IF);
        for (t, logged) in [(0.0631, 0.179), (0.2617, 0.114), (0.6540, 0.349), (0.7197, 0.848), (0.7917, 0.578), (0.9906, 0.248), (1.0552, 0.591)] {
            let w = track.sample(t);
            assert!((w - logged).abs() < 0.0006, "at {t}: {w} against the log's {logged}");
        }
        // before the first key its weight, after the last the last's, at a key its own
        assert_eq!(track.sample(-1.0), 0.0);
        assert!((track.sample(5.0) - 0.2).abs() < 1e-6);
        assert!((track.sample(0.72) - 0.85).abs() < 1e-6);
        assert!((track.sample(0.0) - 0.2).abs() < 1e-6);
    }

    /// FUN_0017a990 sets the keys' inverse spans only on a track of three keys or more: one of two
    /// keeps the 1.0 its keys are made with, so its fraction is the bare seconds since its first key
    /// (none in the data has two). A key-less track gives 0, a one-key track its weight.
    #[test]
    fn a_short_track_keeps_the_unscaled_fraction() {
        let two = VisemeTrack::new(0, &[(0.0, 0.0), (0.5, 100.0)]);
        assert_eq!(two.keys[0].inv, 1.0);
        assert!((two.sample(0.25) - 0.25).abs() < 1e-6, "{}", two.sample(0.25));
        assert!((two.sample(0.5) - 0.5).abs() < 1e-6);
        assert!((two.sample(0.6) - 1.0).abs() < 1e-6);
        let three = VisemeTrack::new(0, &[(0.0, 0.0), (0.5, 100.0), (1.0, 0.0)]);
        assert!((three.keys[0].inv - 2.0).abs() < 1e-6);
        assert!((three.sample(0.25) - 0.5).abs() < 1e-6);
        assert_eq!(VisemeTrack::new(0, &[]).sample(1.0), 0.0);
        assert!((VisemeTrack::new(0, &[(0.3, 40.0)]).sample(0.3) - 0.4).abs() < 1e-6);
    }

    /// FUN_0017aa30 writes only its tracks' visemes: the others keep their weights.
    #[test]
    fn the_tracks_write_their_own_visemes() {
        let v = Visemes { tracks: vec![VisemeTrack::new(6, &YEAH_IF), VisemeTrack::new(14, &[(0.0, 50.0), (1.0, 50.0), (2.0, 0.0)])] };
        let mut w = [0.3; VISEMES];
        v.write(0.0631, &mut w);
        assert!((w[6] - 0.179).abs() < 0.0006);
        assert!((w[14] - 0.5).abs() < 1e-6);
        assert_eq!(w[0], 0.3);
        assert_eq!(w[15], 0.3);
    }

    /// A `<Sound>`'s speech block as the library has it: a track named by its viseme's hash (or a
    /// string the decoder knows, `size`), its frames in order; a track naming none of the 22 is
    /// dropped; a sound without a block has no tracks, one with an empty block an empty set.
    #[test]
    fn a_speech_block_reads_into_tracks() {
        let el = |name: u32, attrs: Vec<(u32, Value)>, children: Vec<Element>| Element { name, attrs, children, text: None };
        let frame = |t: f64, w: f64| el(h("frame"), vec![], vec![el(H_FIELDS, vec![(h("time"), Value::Float(t)), (h("weight"), Value::Float(w))], vec![])]);
        let track = |name: Value, frames: Vec<Element>| {
            let mut c = vec![el(H_FIELDS, vec![(H_VISEME_NAME, name)], vec![])];
            c.extend(frames);
            el(H_TRACK, vec![], c)
        };
        let block = el(H_SPEECH, vec![], vec![el(H_FIELDS, vec![], vec![]), el(H_VISEME_BLOCK, vec![], vec![
            el(H_FIELDS, vec![(0xF63F_0F49, Value::Int(30))], vec![]),
            track(Value::Hash(0xE1D3_067C), vec![frame(-0.264, 0.0), frame(0.0, 20.0), frame(0.608, 0.0)]),
            track(Value::Str("size".into()), vec![frame(0.1, 0.0), frame(0.2, 60.0), frame(0.3, 0.0)]),
            track(Value::Hash(0x1234_5678), vec![frame(0.0, 50.0)]),
        ])]);
        let sound = el(h("Sound"), vec![], vec![el(H_FIELDS, vec![(h("iD"), Value::Hash(0xEC50_FB88))], vec![]), block]);
        let v = Visemes::read(&sound).expect("a speech block");
        assert_eq!(v.tracks.iter().map(|t| t.viseme).collect::<Vec<_>>(), vec![6, 11]);
        assert_eq!(v.tracks[0].keys.len(), 3);
        assert!((v.tracks[0].sample(0.0631) - 0.179).abs() < 0.0006);
        assert!((v.tracks[1].sample(0.2) - 0.6).abs() < 1e-6);
        assert!(Visemes::read(&el(h("Sound"), vec![], vec![])).is_none());
        let empty = el(h("Sound"), vec![], vec![el(H_SPEECH, vec![], vec![])]);
        assert_eq!(Visemes::read(&empty), Some(Visemes::default()));
    }

    /// 22050 Hz stereo Xbox ADPCM, the format of every track of the music banks (`format` 705609).
    const STEREO_22050: u32 = 22050 * 32 + 9;

    /// A wave bank file as `xwb_directory` reads one (WBND: the segment table at 8, the bank's
    /// count, entry size and name size, the entries' format / offset / length, their names, then the
    /// wave data from 0x800, each wave on a 0x800 boundary as the game's banks have them): `waves` are
    /// (name, data).
    fn wave_bank(waves: &[(&str, Vec<u8>)]) -> Vec<u8> {
        let (bank, meta, names, wave) = (0x40usize, 0x80usize, 0x400usize, 0x800usize);
        let (meta_size, name_size) = (24usize, 64usize);
        let mut d = vec![0u8; wave];
        d[..4].copy_from_slice(b"WBND");
        let put = |d: &mut Vec<u8>, at: usize, v: usize| d[at..at + 4].copy_from_slice(&(v as u32).to_le_bytes());
        for (i, v) in [(0, bank), (2, meta), (4, names), (5, waves.len() * name_size), (6, wave)] {
            put(&mut d, 8 + i * 4, v);
        }
        put(&mut d, bank + 4, waves.len());
        put(&mut d, bank + 24, meta_size);
        put(&mut d, bank + 28, name_size);
        let mut at = 0;
        for (i, (name, data)) in waves.iter().enumerate() {
            put(&mut d, meta + i * meta_size + 4, STEREO_22050 as usize);
            put(&mut d, meta + i * meta_size + 8, at);
            put(&mut d, meta + i * meta_size + 12, data.len());
            d[names + i * name_size..names + i * name_size + name.len()].copy_from_slice(name.as_bytes());
            d.resize(wave + at, 0);
            d.extend_from_slice(data);
            at = (at + data.len()).div_ceil(0x800) * 0x800;
        }
        d
    }

    /// A sound library holding sounds (wave-file, id, Type) and music file records (file, offset,
    /// size, format) as the decoded `sounds-<level>.xml` has them.
    fn sound_library(sounds: &[(u32, u32, u32)], files: &[(u32, usize, usize, u32)]) -> Element {
        let el = |name: u32, attrs: Vec<(u32, Value)>, children: Vec<Element>| Element { name, attrs, children, text: None };
        let mut kids: Vec<Element> = sounds.iter().map(|&(file, id, kind)| {
            el(h("Sound"), vec![(0x1D70_DB35, Value::Hash(file))], vec![
                el(H_FIELDS, vec![(h("iD"), Value::Hash(id)), (h("Type"), Value::Hash(kind))], vec![])])
        }).collect();
        kids.extend(files.iter().map(|&(file, offset, size, format)| {
            // (a value per language slot, five of them, as the data has them)
            let slots = |v: usize| Value::List(vec![Value::Int(v as i64), Value::Int(0), Value::Int(0), Value::Int(0), Value::Int(0)]);
            el(0x0F60_56C9, vec![(h("file-name"), Value::Hash(file)), (h("resource-type"), Value::Int(2)), (h("offset"), slots(offset)),
                                 (h("size"), slots(size)), (h("format"), slots(format as usize))], vec![])
        }));
        el(h("sound-library"), vec![], kids)
    }

    /// The lookup the game does (#398): a music sound's id leads to its file, the file's record gives
    /// the offset, size and format of its bytes in its library's bank file, and the wave's name
    /// plays no part. Here the entry for the second wave is named otherwise ("renamed_entry", as
    /// sdm_m07's `ambiencehellish_2` is for the wave `ambiencehellish-2ch`): the id is the hash
    /// of the entry's name, not the wave's, and a lookup by the wave's name finds nothing.
    #[test]
    fn a_music_sound_finds_its_bytes_by_its_file_record() {
        let waves = [("first_wave", vec![1u8; 144]), ("second_wave", vec![2u8; 216])];
        let bank = wave_bank(&waves);
        let entries = xwb_entries(&bank);
        assert_eq!(entries.iter().map(|e| (e.0.as_str(), e.1, e.2.start, e.2.len())).collect::<Vec<_>>(),
                   vec![("first_wave", STEREO_22050, 0x800, 144), ("second_wave", STEREO_22050, 0x1000, 216)]);
        let (first_id, second_id) = (h("first_wave"), h("renamed_entry"));
        assert_ne!(second_id, h("second_wave"));
        let library = sound_library(&[(h("music\\first_wave.wav"), first_id, MUSIC_TYPE), (h("music\\second_wave.wav"), second_id, h("ambient"))],
                                    &[(h("music\\first_wave.wav"), 0x800, 144, STEREO_22050), (h("music\\second_wave.wav"), 0x1000, 216, STEREO_22050)]);
        let mut sounds = SoundBank::default();
        sounds.add("lvl", &library, Arc::new(vec![]));
        // the record of the renamed entry: its bytes' place, though the wave's name hashes to no sound
        let file = sounds.music_file(second_id).expect("a file record");
        assert_eq!((&*file.library, file.offset, file.size, file.format), ("lvl", 0x1000, 216, STEREO_22050));
        assert!(sounds.music_file(h("second_wave")).is_none(), "the wave's own name is no sound's id");
        assert!(sounds.is_music(second_id) && !sounds.has(second_id));
        assert_eq!(sounds.length(second_id), Some(216.0 / bytes_per_second(STEREO_22050)));
        // the wave at a place: the sounds that play it and their Type, in that library only
        assert_eq!(sounds.track_sounds("lvl", 0x1000), vec![second_id]);
        assert_eq!(sounds.track_kind("lvl", 0x1000), Some(h("ambient")));
        assert_eq!(sounds.track_kind("lvl", 0x800), Some(MUSIC_TYPE));
        assert_eq!(sounds.track_kind("other", 0x1000), None);
        assert_eq!(sounds.track_kind("lvl", 0x1800), None);
        // the bank's list: each wave by its place, a wave no entry plays with its own entry's place
        let listed = sounds.bank_tracks("lvl", &bank);
        assert_eq!(listed.iter().map(|t| (t.name.as_str(), t.file.offset, t.kind, t.sounds.clone())).collect::<Vec<_>>(),
                   vec![("first_wave", 0x800, Some(MUSIC_TYPE), vec![first_id]), ("second_wave", 0x1000, Some(h("ambient")), vec![second_id])]);
        let unplayed = SoundBank::default().bank_tracks("lvl", &bank);
        assert_eq!(unplayed[1].file, MusicFile { library: "lvl".into(), offset: 0x1000, size: 216, format: STEREO_22050 });
        assert_eq!((unplayed[1].kind, unplayed[1].sounds.is_empty()), (None, true));
        // the names, for the logs: the wave at the record's place names the sound
        assert_eq!(sounds.add_music_names("lvl", &bank), 2);
        assert_eq!((sounds.name(first_id), sounds.name(second_id)), (Some("first_wave"), Some("second_wave")));
        assert_eq!(sounds.add_music_names("lvl", &bank), 0);
    }

    /// The bytes come from the bank file by the record's offset and size alone (`MusicFile::read`: a
    /// seek and a read, not the whole bank), whatever the entry table says; a file too short for the
    /// record is an error. They decode by the record's format: three blocks of stereo ADPCM make 3 x
    /// 65 frames of 22050 Hz.
    #[test]
    fn the_bytes_are_read_at_the_records_offset_and_decoded_by_its_format() {
        let bank = wave_bank(&[("first_wave", vec![1u8; 144]), ("second_wave", vec![2u8; 216])]);
        let dir = std::env::temp_dir().join(format!("bf_music_bank_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("sounds")).unwrap();
        std::fs::write(dir.join("sounds").join("lvl.xwb"), &bank).unwrap();
        let file = MusicFile { library: "lvl".into(), offset: 0x1000, size: 216, format: STEREO_22050 };
        let bytes = file.read(&dir);
        let short = MusicFile { size: 217 + 0x10000, ..file.clone() }.read(&dir);
        let missing = MusicFile { library: "nowhere".into(), ..file.clone() }.read(&dir);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(bytes.as_deref(), Ok(&[2u8; 216][..]));
        assert!(short.is_err() && missing.is_err());
        let wav = file.wav(&bytes.unwrap()).expect("decodes");
        assert!((wav_secs(&wav) - 3.0 * 65.0 / 22050.0).abs() < 1e-5, "{}", wav_secs(&wav));
        assert_eq!(file.range(), 0x1000..0x1000 + 216);
        // (a format the decoder doesn't read: none)
        assert!(MusicFile { format: 22050 * 32 + 2, ..file }.wav(&[0u8; 16]).is_none());
    }

    /// A library's bank is named by its archive member: `e05/sounds-e05.xmb` is "e05",
    /// `mp_common/sounds-sdm_bf.xmb` "sdm_bf", a bare name stays; and a WAV's length is its data over
    /// its byte rate (three seconds of 22050 Hz stereo; none for a stub).
    #[test]
    fn a_library_is_named_by_its_member_and_a_wav_knows_its_length() {
        assert_eq!(library_of("e05/sounds-e05.xmb"), "e05");
        assert_eq!(library_of("mp_common/sounds-sdm_bf.xmb"), "sdm_bf");
        assert_eq!(library_of("sounds-common.xmb"), "common");
        assert_eq!(library_of("tutorial"), "tutorial");
        let wav = wav_bytes_channels(22050, 2, &vec![0i16; 22050 * 2 * 3]);
        assert!((wav_secs(&wav) - 3.0).abs() < 1e-4);
        assert_eq!(wav_secs(&[]), 0.0);
        assert_eq!(wav_secs(&[0u8; 44]), 0.0);
    }

    /// The game's data directory, if it's there: $BF_DATA_DIR, else the one beside this crate. The
    /// tests that read the real banks skip without it (the data isn't in the repository).
    fn game_data() -> Option<std::path::PathBuf> {
        let dir = std::env::var("BF_DATA_DIR").map(std::path::PathBuf::from).ok()
            .or_else(|| std::env::var("CARGO_MANIFEST_DIR").ok().map(|m| std::path::PathBuf::from(m).join("../Brute Force/data")))?;
        dir.join("common.tgz").exists().then_some(dir)
    }

    /// The real records (#398, todo/398-level-music/answers.md section 1): in five of the game's
    /// banks every wave of the bank file has exactly one file record at its place, with its size and
    /// format, and a sound entry that plays it with a Type; the five tracks the demo lost by the name
    /// lookup are found at the offsets the record gives (sdm_e40's music h_10fb2011 at 2048, e05's
    /// at 2072576, sdm_m07's h_e70fb918 at 2195456, whose entry is `ambiencehellish_2`), and the
    /// bytes at them decode to the waves' lengths. Skipped without the game's data.
    #[test]
    fn the_real_records_land_on_the_waves_of_their_banks() {
        let Some(dir) = game_data() else { return };
        let mut schemas = super::super::bxml::SchemaSet::default();
        let common = super::super::archive::Archive::open(&dir.join("common.tgz"), |n| n.ends_with(".xsb")).unwrap();
        for n in common.find("", ".xsb") {
            schemas.add(common.get(&n).unwrap()).unwrap();
        }
        let sounds_of = |level: &str| super::super::character::Game::sound_bank_of(&schemas, &dir, level).unwrap();
        for level in ["sdm_e40", "e05", "sdm_m07", "m04_a", "tutorial"] {
            let bank = std::fs::read(dir.join("sounds").join(format!("{level}.xwb"))).unwrap();
            let sounds = sounds_of(level);
            let listed = sounds.bank_tracks(level, &bank[..XWB_HEAD as usize]);
            assert_eq!(listed.len(), xwb_entries(&bank).len(), "{level}");
            for (t, (name, fmt, range)) in listed.iter().zip(xwb_entries(&bank)) {
                assert_eq!(t.name, name);
                assert_eq!((t.file.offset, t.file.size, t.file.format), (range.start, range.len(), fmt), "{level}: {name}'s record");
                assert!(!t.sounds.is_empty() && t.kind.is_some(), "{level}: no sound entry plays {name}");
            }
        }
        let e40 = sounds_of("sdm_e40");
        let creepy = e40.music_file(0x10FB_2011).expect("sdm_e40's music");
        assert_eq!((&*creepy.library, creepy.offset, creepy.size), ("sdm_e40", 2048, 2541744));
        assert!(e40.music_file(h("creepy_estuary_mutant_01")).is_none(), "no sound is named as the wave");
        assert_eq!(e40.sound_type(0x10FB_2011), Some(MUSIC_TYPE));
        let wav = creepy.wav(&creepy.read(&dir).unwrap()).unwrap();
        assert!((wav_secs(&wav) - 104.06).abs() < 0.05, "{}", wav_secs(&wav));
        let e05 = sounds_of("e05");
        assert_eq!(e05.music_file(0x10FB_2011).map(|f| (f.offset, f.size)), Some((2072576, 2541744)));
        // (and its stings: the victory and defeat cues the name lookup missed)
        assert_eq!(e05.music_file(0xF934_A4BE).map(|f| (f.offset, f.size)), Some((8833024, 538560)));
        assert_eq!(e05.music_file(0xEA61_DFAE).map(|f| (f.offset, f.size)), Some((8187904, 644616)));
        let m07 = sounds_of("sdm_m07");
        assert_eq!(h("ambiencehellish_2"), 0xE70F_B918);
        assert_eq!(m07.music_file(0xE70F_B918).map(|f| (f.offset, f.size)), Some((2195456, 3839328)));
        assert_eq!(m07.sound_type(0xE70F_B918), Some(MUSIC_TYPE));
    }
}

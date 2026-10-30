//! The level's sound- and dialog-triggers (#223, the tutorial's step 4 of #28;
//! viewer/docs/triggers.md, "Sound- and dialog-triggers"), the world's dialog queue, and the voices
//! they play. A child of play_triggers.rs: its two classes are the runtime's (their handlers and
//! updates are `Triggers`' and dispatched from there), and its system plays what they ask for.
//! Written from default.xbe's code (the addresses are in the comments and in
//! viewer/docs/code-map.md, "Sound"); the takes in todo/28-tutorial confirm the timing.
//!
//! The sound-trigger (vtable 0x395d50; its fields from +0x40, its constructor 0x1fce50):
//!  * PLAY_SOUND or OPERATE (its handler 0x206450) turns it on (+0x8d bit 1). With h_15bc1769
//!    that's all: its update starts the voice while a player's character is within its
//!    max-distance and stops it beyond (FUN_00206b50). Otherwise it starts at once
//!    (FUN_002062a0): a one-shot still playing isn't restarted, a looping one playing goes on; the
//!    time left (+0xa4) becomes the sound's length (the sound system's vtable +0x64: the file's
//!    size over its bytes a second) and the voice starts (FUN_002060c0): alike everywhere for
//!    Types 0, 2 and 3 (and for any sound whose library entry's falloff is -1), else at its place
//!    (its first object's, else its transform's) with its min and max distance (its own, else the
//!    library entry's: full within the min, then the rolloff curve every 3D voice gets, down to
//!    nothing at the max, `distance_gain`), at its volume / 100 (else the entry's), with its
//!    fade-in and fade-out (the voice's envelope: in linearly, out linearly in dB), looping or
//!    once.
//!  * Its update (0x206690), run while +0x3d bit 6 is set: FIRST_UPDATE to itself, enabled or
//!    not; while it runs, the time left goes down by dt, and once it's gone (a one-shot)
//!    FINISHED (25) goes to its targets (Types 0 and 1: FUN_001fcf00), it's off and stopped.
//!  * STOP_SOUND, STOP, STOP_DIALOG (FUN_002063c0 with 0): stopped (the voice fading out over its
//!    fade-out) and off. PAUSE_SOUND / RESUME_SOUND pause and resume the voice and the clock.
//!  * PLAY_DIALOG queues its sound, once, on the world's dialog queue (world+0x208, FUN_00179d60),
//!    which plays one line at a time (FUN_00179ec0, the world update's last step); FINISHED once
//!    its line has played (FUN_0017a0f0).
//!  * Its DISABLE is its own (FUN_00206610): the voice plays out, its time and queued line are
//!    dropped (so no FINISHED), and its update stops once the voice ends.
//!  * The Type (the SND_IS_ names at 0x3bf200): 0 GLOBAL, 1 POSITIONAL, 2 AMBIENT_OUTDOOR,
//!    3 AMBIENT_INDOOR, 4 POSITIONAL_INDOOR, 5 POSITIONAL_OUTDOOR. Types 2-5 run all this but send
//!    no FINISHED, and play at a level (+0xa0) that follows where the listener stands
//!    (FUN_00206880, moving there at 4 a second, FUN_00206a20): full when it's indoors for 3 and
//!    4, outdoors for 2 and 5, else their volume / 100. The listener is indoors when the
//!    character its camera follows stands on an indoor floor (`indoors`). The levels' ambience
//!    beds are these: amb_ferix_06 (m01_a, m01_b, m05_b, m05_y), amb_shrike_01 and _02 (m09_a,
//!    m09_x, m09_b), light_bird_bed_1 (sdm_e40) and sdm_e34's wind are Type 2 at 15-35;
//!    amb_shrike_int_01 (m09_c) and m01_a's mechanical_amb_int_1 Type 3. The tutorial has none.
//!
//! The dialog-trigger (vtable 0x3952a0): PLAY_DIALOG or OPERATE on its first object, a character
//! (a spawn-trigger's copy): a dead one sends FINISHED at once, a living one says the line
//! (FUN_001267b0, through its voice) and the trigger's update sends FINISHED once it's said
//! (FUN_00126810). Here the line is a plain sound at the character (no mouth, no radio filter).
//!
//! The music: a level's music bank (data/sounds/<level>.xwb) holds its music cues and ambience
//! beds, and its sound-triggers play them (resource-type 2: the tutorial's three cues with their
//! stop timers, its two beds); the demo plays them through the music volume. The sound system
//! keeps one music voice (class 1, a sound of Type h_f92fb2b9; 0x17eff0, `MusicSlot`): a music
//! track that starts while another still plays fades that one out over its fade-out, and its own
//! trigger loses its handle (its later stop does nothing). The beds (Type "ambient") aren't
//! music and play under it.
//!
//! Test hooks: BF_TRIGGER_LOG=1 (play_triggers.rs) logs what each sound- and dialog-trigger does,
//! the queue's lines and, at the start, every one's sound, length and how it plays; BF_SOUND_LOG=1
//! the voices as they start and stop (volume, where, distances); either, the listener going
//! indoors or out. BF_MUTE=1: the triggers run, no voice plays. BF_NO_MUSIC=1: no music-bank
//! track.

use std::collections::VecDeque;

use bevy::audio::Volume as AudioVolume;
use bf_viewer::bf::audio::{self, SoundBank, SoundParams};
use bf_viewer::bf::character::OBJECT_GAME;
use bf_viewer::bf::collision::MATERIAL_INDOOR;
use bf_viewer::bf::level::{LU_GEOSET, LU_INDOOR};
use bf_viewer::bf::triggers::SoundTrigger;

use super::*;

/// A sound-trigger's +0x50 while it runs (its time going down, or its line queued).
const RUNNING: u8 = 0;
/// +0x50 stopped (its constructor's value, FUN_002063c0's).
const STOPPED: u8 = 1;
/// +0x50 paused (PAUSE_SOUND).
const PAUSED: u8 = 2;
/// How fast a Type 2-5 trigger's level moves to where it should be (FUN_00206a20: dt x 4.0 a
/// frame).
const ZONE_RAMP: f32 = 4.0;
/// A character's line heard from another character: as play.rs `play_sounds` hears a squad
/// member's sounds, quieter with the distance from the player, 1 - d / SPEAKER_HEARD (at least
/// SPEAKER_FLOOR), x SPEAKER_SHARE (the demo's mixing, not the game's: a stub). The chatter's
/// voices too (play_chatter.rs `voice_start`).
pub(crate) const SPEAKER_HEARD: f32 = 35.0;
/// The least share of a line said by another character (play.rs `play_sounds`' 0.15).
pub(crate) const SPEAKER_FLOOR: f32 = 0.15;
/// Another character's line at most this share (play.rs `play_sounds`' 0.8).
pub(crate) const SPEAKER_SHARE: f32 = 0.8;
/// The rolloff curve every 3D voice gets as it's made (FUN_00181770 for a buffer, 0x1836eb for a
/// stream: IDirectSound*_SetRolloffCurve with the five gains at 0x38dae8), spread evenly from
/// the min distance to the max (see `distance_gain`).
const ROLLOFF: [f32; 5] = [1.0, 0.375, 0.125, 0.0625, 0.0];
/// How far a voice's fade-out takes it down by its end, in dB (it's then cut). The fades are the
/// voice's envelope (FUN_0017f330 -> 0x181c90, IDirectSoundBuffer_SetEG: the fade-in its attack,
/// the fade-out its release, x 48000 / 512) and their shape the sound chip's, not the game's
/// code: take04's tutorial_01 stop (a 3 s fade-out) fits a fall linear in dB, about -60 dB over
/// it (its music level at 0.25 s steps from 199.0 s: 0.65, 0.30, 0.25, 0.16, 0.09, 0.03; the
/// tester's measure). The fade-in fits a linear rise (take09, take11).
const FADE_OUT_DB: f32 = 60.0;
/// OT_PLATFORM (the OT_ name table at 0x3be878; OT_STATIC_GEOMETRY is `OBJECT_GAME`, 2): the
/// other kind of object whose floor sets a character's indoor bit (FUN_0012dab0).
const OT_PLATFORM: i64 = 26;

/// The class of a music voice (FUN_0017aa00's 1): the one class the sound system keeps a single
/// voice of (0x17eff0, see `MusicSlot`).
const CLASS_MUSIC: u32 = 1;

/// A sound's class, as the library loader works it out from its `Type` (0x17bbe9: FUN_0017aa00
/// into the runtime entry's +0x18, which FUN_0017c170 hands the new voice through the sound
/// system's +0x4c, 0x17f300, into its +0x2c): h_f92fb2b9 (`MUSIC_TYPE`) 1, h_e34c5f66 2,
/// h_012ed199 3, any other 0 (the ambience beds' "ambient" among them). None for a sound the
/// loaded banks don't define.
fn voice_class(sounds: &SoundBank, id: u32) -> u32 {
    match sounds.sound_type(id) {
        Some(audio::MUSIC_TYPE) => CLASS_MUSIC,
        Some(0xE34C_5F66) => 2,
        Some(0x012E_D199) => 3,
        _ => 0,
    }
}

/// Types 2-5, the indoor and outdoor ones: their level follows whether the listener is indoors
/// (FUN_00206880), and they send no FINISHED (FUN_001fcf00).
fn zone(kind: i64) -> bool {
    (2..=5).contains(&kind)
}

/// Whether the listener's character stands indoors, as FUN_0012dab0 keeps it in its ground
/// record (char+0x43c bit 0) and FUN_00178c10 hands it to the sound system for the listener whose
/// camera follows it (+0xc8): from the floor under its feet (`at`), on the terrain the
/// triangle's indoor bit (`MATERIAL_INDOOR`), on static geometry (OBJECT_GAME) or a platform
/// (OT_PLATFORM) its light-usage: LU_INDOOR indoors, LU_GEOSET the triangle's bit, any other
/// outdoors. On anything else, or with no floor below, it stays as it `was`.
fn indoors(arena: &world::Arena, level: &Level, game: &Game, at: Vec3, was: bool) -> bool {
    let Some((material, terrain, object)) = arena.floor_kind(at.x, at.z, at.y + STEP_UP) else { return was };
    if terrain {
        return material & MATERIAL_INDOOR != 0;
    }
    let Some(o) = object.and_then(|i| level.objects.get(i as usize)) else { return was };
    if !game.object_types.get(&o.kind).is_some_and(|t| matches!(t.object_type, OBJECT_GAME | OT_PLATFORM)) {
        return was;
    }
    match o.light_usage {
        LU_GEOSET => material & MATERIAL_INDOOR != 0,
        usage => usage == LU_INDOOR,
    }
}

/// Types 1, 4 and 5 play at a place (FUN_002060c0 sets the position for them; the update moves
/// the voice with the first object, 0x2067f4).
fn positional(kind: i64) -> bool {
    matches!(kind, 1 | 4 | 5)
}

/// A sound's name for the logs: its wave's (`SoundBank::name`), else its hash.
fn sound_name(sounds: &SoundBank, id: u32) -> String {
    sounds.name(id).map_or(format!("h_{id:08x}"), |n| format!("{n} (h_{id:08x})"))
}

/// A sound-trigger as the game keeps it (vtable 0x395d50; its fields from +0x40, its constructor
/// 0x1fce50), with what the runtime looks up once: its sound's length, library entry and kind.
pub(crate) struct Sound {
    /// its attributes (`bf::triggers::SoundTrigger`)
    data: SoundTrigger,
    /// its sound's name, for the logs
    name: String,
    /// its sound's length (s) as the sound system reckons it (`SoundBank::length`); 0 for a
    /// sound the banks lack, which ends at once (FUN_002062a0 keeps the 0.0 it loaded)
    length: f32,
    /// its sound's library entry; None when the banks lack it (FUN_0017c170 makes no voice)
    library: Option<SoundParams>,
    /// a track of the level's music bank (resource-type 2)
    music: bool,
    /// its first object (h_0b6b92ba), where a positional voice plays and which it follows
    /// (FUN_00205fb0 through FUN_001fb020: for a spawn-trigger, its copy); 0 none
    object: u32,
    /// that object's place when it's a placed one (a character's is looked up as it moves)
    object_at: Option<Vec3>,
    /// its transform's position (+0x80-0x88), where it plays without an object
    at: Vec3,
    /// its voice (+0x44), 0 none
    voice: u32,
    /// when that voice's sound ends (the sim clock; never for a looping one): the sound system's
    /// "still playing" (vtable +0x2c), kept on the clock so a muted run times the same
    voice_end: f32,
    /// its line on the world's dialog queue (+0x48)
    ticket: u32,
    /// +0x50: RUNNING, STOPPED or PAUSED
    state: u8,
    /// +0x8d bit 1: on (a PLAY until it ends, a STOP, or a zero volume)
    on: bool,
    /// +0x8d bit 2: playing (its time going down)
    playing: bool,
    /// +0x8d bit 3: its voice playing out after a DISABLE (FUN_00206610)
    play_out: bool,
    /// +0x8d bit 4: its line queued (PLAY_DIALOG); FUN_002063c0 leaves it set
    queued: bool,
    /// +0xa0: a Type 2-5 trigger's level (1 for Types 0 and 1 as they start)
    level: f32,
    /// the level a Type 2-5 trigger last headed to (`zone_ramp`), for the log
    heading: f32,
    /// +0xa4: the time left of its sound (s)
    left: f32,
    /// +0x3d bit 6: in the world's update loop (FUN_000d99d0 calls a trigger's update only with it):
    /// set by the constructor and every enable, cleared by its disable unless its voice plays out
    pub(super) listed: bool,
}

impl Sound {
    /// Trigger `t`'s state as the level starts: its sound looked up in the loaded banks, its
    /// first object placed if it's a placed one.
    pub(super) fn new(t: &Trigger, data: &SoundTrigger, level: &Level, game: &Game) -> Sound {
        let object = t.objects.first().copied().unwrap_or(0);
        Sound {
            name: sound_name(&game.sounds, data.sound),
            length: game.sounds.length(data.sound).unwrap_or(0.0),
            library: game.sounds.params(data.sound),
            music: game.sounds.is_music(data.sound),
            object,
            object_at: level.objects.iter().find(|o| object != 0 && o.name == object).map(|o| o.transform.w_axis.truncate()),
            at: t.transform.map_or(Vec3::ZERO, |m| m.w_axis.truncate()),
            data: data.clone(),
            voice: 0,
            voice_end: 0.0,
            ticket: 0,
            state: STOPPED,
            on: false,
            playing: false,
            play_out: false,
            queued: false,
            level: 0.0,
            heading: 0.0,
            left: 0.0,
            listed: true,
        }
    }

    /// Its max distance: its own, else its library entry's (FUN_00206b50, FUN_00206880); 0 when
    /// neither has one.
    fn max_distance(&self) -> f32 {
        if self.data.max_distance > 0.0 { self.data.max_distance } else { self.library.map_or(0.0, |l| l.max_distance) }
    }
}

/// A dialog-trigger as the game keeps it (vtable 0x3952a0; its constructor 0x1fd430: +0x44 and
/// +0x48 1).
pub(crate) struct Dialog {
    /// `dialog` (+0x40): the line, a sound library entry
    dialog: u32,
    /// its name, for the logs
    name: String,
    /// h_e725a1a5 (+0x48), handed to the character's line (FUN_001267b0's EDX): the squad's
    /// chatter hushed while it plays (play_chatter.rs `Chatter::hush`)
    hush: bool,
    /// the line's length (`SoundBank::length`): how long its character speaks
    length: f32,
    /// +0x44: its line is done, or none was said; false while its character says it
    done: bool,
    /// when the line ends (the sim clock): its character's voice speaking (FUN_00126810)
    ends: f32,
    /// the line's voice, 0 none
    voice: u32,
}

impl Dialog {
    /// A dialog-trigger's state as the level starts, its line looked up in the loaded banks.
    pub(super) fn new(dialog: u32, hush: bool, game: &Game) -> Dialog {
        Dialog { dialog, name: sound_name(&game.sounds, dialog), hush, length: game.sounds.length(dialog).unwrap_or(0.0), done: true, ends: 0.0, voice: 0 }
    }
}

/// Where a voice plays from: a 3D voice's place and its min and max distance (the DirectSound
/// buffer's, FUN_0017eb90), or a character who says a line.
#[derive(Clone, Copy, Debug)]
enum Heard {
    /// heard alike everywhere
    Everywhere,
    /// a 3D voice: at a place, full within the min distance, then quieter along the rolloff
    /// curve to nothing at the max (`distance_gain`)
    At { at: Vec3, min: f32, max: f32 },
    /// a character's line (its trigger name)
    Speaker(u32),
}

/// What the triggers ask of the sound system (DAT_00469940) this frame: `voices` does it.
enum VoiceAct {
    /// a new voice (FUN_0017c170: vtable +0x1c; FUN_002060c0: +0x50 its fades, +0x28 its play):
    /// `volume` None plays it at its library entry's volume (FUN_002060c0 sets its own only up
    /// to 1); `music` a music-bank track; `by` who, for the log
    Start { voice: u32, sound: u32, looping: bool, volume: Option<f32>, from: Heard, fade_in: f32, fade_out: f32, music: bool, by: String },
    /// vtable +0x20: stopped, at once (1) or with its fade-out (0): the voice's StopEx, for a
    /// stream 0x182f30's IDirectSoundStream_FlushEx with ASYNC alone (1) or ASYNC | ENVELOPE
    /// (0: CMcpxStream_Stop runs the envelope's release for flag 4), as take04's start music
    /// fades over its 3 s when its stop timer runs out
    Stop { voice: u32, at_once: bool },
    /// vtable +0x34: its volume set (a Type 2-5 trigger's level as it moves, FUN_00206a20)
    Volume { voice: u32, volume: f32 },
    /// vtable +0x54: paused or resumed
    Pause { voice: u32, on: bool },
    /// vtable +0x40: a positional voice moved with its object (0x20682f)
    Move { voice: u32, at: Vec3 },
    /// the sound system's own stop of the music voice a new one replaces (0x17eff0: StopEx(0),
    /// its fade-out), which goes by its pointer, not its handle (`MusicSlot`)
    Replaced { voice: u32 },
}

impl VoiceAct {
    /// The handle an act on a voice looks its voice up by (the sound system's map, 0x17ee90);
    /// None for a start, which makes one, and for the system's own stop.
    fn handle(&self) -> Option<u32> {
        match *self {
            VoiceAct::Stop { voice, .. } | VoiceAct::Volume { voice, .. } | VoiceAct::Pause { voice, .. } | VoiceAct::Move { voice, .. } => Some(voice),
            VoiceAct::Start { .. } | VoiceAct::Replaced { .. } => None,
        }
    }

    /// What the act does, for the log of one dropped.
    fn what(&self) -> &'static str {
        match self {
            VoiceAct::Start { .. } => "start",
            VoiceAct::Stop { .. } => "stop",
            VoiceAct::Volume { .. } => "volume",
            VoiceAct::Pause { on: true, .. } => "pause",
            VoiceAct::Pause { .. } => "resume",
            VoiceAct::Move { .. } => "move",
            VoiceAct::Replaced { .. } => "replace",
        }
    }
}

/// A character in the world, as of the end of the last frame (`voices` notes them): its trigger
/// name (0 for one no trigger names, which name lookups skip), feet, whether it's dead, and
/// whether a human plays it.
#[derive(Clone, Copy)]
struct Seen {
    name: u32,
    feet: Vec3,
    dead: bool,
    human: bool,
}

/// One line on the world's dialog queue (FUN_00179d60's 0x18-byte entry: +8 the sound, +0xc the
/// ticket, +0x10 its length, +0x14 the hush flag).
#[derive(Clone, Copy)]
struct Line {
    ticket: u32,
    sound: u32,
    left: f32,
    hush: bool,
}

/// The world's dialog queue (world+0x208; its update FUN_00179ec0, the last step of the world's
/// update): the PLAY_DIALOG lines play one after another, each for its length, alike
/// everywhere at its library entry's volume (FUN_0017c110 with the default parameters).
#[derive(Default)]
struct Radio {
    /// the lines waiting, in order (its list +0x28 / +0x2c, count +0x30)
    lines: VecDeque<Line>,
    /// the line playing, or the last played (+0x38 the sound, +0x3c the ticket, +0x40 the time
    /// left, +0x44 the hush flag)
    current: Option<Line>,
    /// +4 bit 3: a line playing
    playing: bool,
    /// its voice (+0x34), 0 none, and when its sound ends
    voice: u32,
    voice_end: f32,
    /// tickets handed out (+0x48, counting from 0)
    tickets: u32,
}

impl Radio {
    /// FUN_00179d60: `sound` queued, with its length (the sound system's +0x64, as it's queued);
    /// its ticket.
    fn queue(&mut self, sound: u32, length: f32, hush: bool) -> u32 {
        let ticket = self.tickets;
        self.tickets += 1;
        self.lines.push_back(Line { ticket, sound, left: length, hush });
        ticket
    }

    /// FUN_0017a0f0: whether a ticket's line is done: the current one once its time is out, any
    /// other once it's no longer waiting.
    fn done(&self, ticket: u32) -> bool {
        match self.current {
            Some(c) if c.ticket == ticket => c.left <= 0.0,
            _ => !self.lines.iter().any(|l| l.ticket == ticket),
        }
    }

    /// FUN_0017a130: a ticket's line taken back: the current one's time ended and its voice
    /// stopped at once, a waiting one dropped.
    fn cancel(&mut self, ticket: u32, acts: &mut Vec<VoiceAct>) {
        match &mut self.current {
            Some(c) if c.ticket == ticket => {
                c.left = 0.0;
                if self.voice != 0 {
                    acts.push(VoiceAct::Stop { voice: self.voice, at_once: true });
                    self.voice = 0;
                }
            }
            _ => self.lines.retain(|l| l.ticket != ticket),
        }
    }

    /// FUN_00179ec0 over `dt`: the playing line's time goes down (its voice let go once it has
    /// ended), and with none playing the next waiting one starts, hushing the chatter for its
    /// length when it has the flag (0x17a046-0x17a064: FUN_00141100 keyed by its sound, into
    /// `hushes`). `voices_made` hands out voice handles; `log` names what it plays.
    #[allow(clippy::too_many_arguments)]
    fn update(&mut self, dt: f32, now: f32, acts: &mut Vec<VoiceAct>, hushes: &mut Vec<(u32, Option<f32>)>, voices_made: &mut u32, sounds: &SoundBank,
              log: bool) {
        if self.playing {
            let left = match &mut self.current {
                Some(c) => {
                    c.left = (c.left - dt).max(0.0);
                    c.left
                }
                None => 0.0,
            };
            // (the game stops its voice at once then, which has played its data: the demo's
            // runs a little longer, see `sound_update`, and is left to run out)
            if self.voice != 0 && (now >= self.voice_end || left <= 0.0) {
                self.voice = 0;
            }
            if left <= 0.0 {
                self.playing = false;
            }
        }
        if self.playing {
            return;
        }
        let Some(line) = self.lines.pop_front() else { return };
        self.current = Some(line);
        // (its voice or none, the hush comes: 0x17a024 jumps to the test at 0x17a046)
        if line.hush {
            hushes.push((line.sound, Some(line.left)));
        }
        if line.sound == 0 || line.left <= 0.0 {
            if let Some(c) = &mut self.current {
                c.left = 0.0;
            }
            return;
        }
        *voices_made += 1;
        self.voice = *voices_made;
        self.voice_end = now + line.left;
        self.playing = true;
        if log {
            println!("t {now:.2}: the dialog queue plays {} (ticket {}, {:.2} s){}", sound_name(sounds, line.sound), line.ticket, line.left,
                     if line.hush { "; it hushes the squad's chatter (FUN_00141100)" } else { "" });
        }
        acts.push(VoiceAct::Start { voice: self.voice, sound: line.sound, looping: false, volume: None, from: Heard::Everywhere, fade_in: 0.0, fade_out: 0.0,
                                    music: false, by: "the dialog queue".into() });
    }
}

/// The triggers' sound side, one per level (`Triggers::sound`): the world's dialog queue, the
/// voices the triggers asked for this frame, where the listener is and whether it's indoors (the
/// `listener` system's, before the triggers' frame), and where the characters were at the end of
/// the last (`voices` keeps them; the handlers between frames read them).
#[derive(Default)]
pub(crate) struct SoundWorld {
    radio: Radio,
    acts: Vec<VoiceAct>,
    /// the chatter mutes asked for this frame (the lines with the hush flag h_e725a1a5): a key and
    /// its length (FUN_00141100), or None to take the key's off (FUN_00141180); `voices` hands
    /// them to the chatter manager (play_chatter.rs `Chatter::hush`)
    hushes: Vec<(u32, Option<f32>)>,
    /// voice handles handed out so far (the sound system's, DAT_00469988 counting up)
    voices_made: u32,
    /// the listener: the character the camera follows (FUN_00178c10, `listener`); None before the
    /// first frame's look
    listener: Option<Vec3>,
    /// whether the listener is indoors (the sound system's +0xc8 for its one listener, `indoors`);
    /// None before the first frame's look
    indoors: Option<bool>,
    /// the characters in the world
    seen: Vec<Seen>,
    /// the sound system's one music voice and the handles it took away (`MusicSlot`)
    music: MusicSlot,
}

/// The sound system's music voice (DAT_00469940 +0x1a4, 0 as FUN_0017e7c0 makes it): the last
/// class-1 voice (`CLASS_MUSIC`) played. As a voice plays (the system's +0x28, 0x17f070),
/// 0x17eff0 runs before the voice's own Play: for a class-1 one, when the current music voice
/// is still sounding (0x17efb0: there's one, its +0x38 set, its status playing, its class 1),
/// that one is stopped with StopEx(0) (a stream's 0x182f30: IDirectSoundStream_FlushEx with
/// ASYNC | ENVELOPE, so it fades out through its envelope's release, its own fade-out) and its
/// +0x1c bit 0 cleared; then the node of the handle map (+0xcc) whose voice is the *new* one is
/// erased, so the new voice's handle finds nothing from then on: its trigger's stop (+0x20),
/// volume (+0x34), pause (+0x54), position (+0x40) and "still playing" (+0x2c) do nothing. Then
/// the new voice is the current one. With none sounding the new one just becomes current, its
/// handle kept. The tracks themselves have a stream each (8, FUN_00182120(8) at 0x17f999;
/// FUN_001822d0 picks a free one), so this is the only thing that keeps two from playing.
/// Code only: no take has a track starting while another plays.
///
/// Not modelled: the old voice's handle stays in the map (only its bit 0 is cleared), so its
/// trigger's later stop reaches whatever that stream plays by then, should it have been reused
/// (tutorial_01's timer525 at 168 s after a replace).
#[derive(Default)]
struct MusicSlot {
    /// the current music voice
    current: Option<MusicVoice>,
    /// the handles 0x17eff0 erased: every act on them is dropped
    orphans: HashSet<u32>,
}

/// The current music voice (`MusicSlot`): its handle, and when it stops sounding.
#[derive(Clone, Copy)]
struct MusicVoice {
    voice: u32,
    /// when it's no longer playing (the sim clock): never for a looping one until it's stopped,
    /// a one-shot's end, a stop's end of its release. The stream's "playing" (its +0xa8, set by
    /// its Play 0x182f00) isn't cleared by an envelope stop (0x182f30 with 0), so it's taken to
    /// last until the release has run (a guess: in take09 mellow, which started long after
    /// tutorial_01's stop, still ends on its own timer, so by then it's cleared)
    until: f32,
    /// its fade-out (s), what a stop and a replace fade it over
    fade_out: f32,
}

/// Whether a level's sound-triggers play tracks of its music bank (resource-type 2): every level
/// with a bank has a FIRST_UPDATE one (sdm_e34's h_1844a2b7 its caspian_action20, the tutorial's
/// h_0378b249 its tutorial_01). Then they play its music, cue by cue, and play.rs `setup`
/// doesn't (`music_mix`).
pub(crate) fn plays_music(level: &Level, sounds: &SoundBank) -> bool {
    level.triggers.iter().any(|t| matches!(&t.class, Class::Sound(s) if sounds.is_music(s.sound)))
}

/// Where the listener stands, before the triggers' frame (play_triggers.rs `run`); the voices,
/// after it; and the start's log.
pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Playing), log_sounds.after(super::start))
        .add_systems(Update, (listener.before(super::run), voices.after(super::run)).run_if(in_state(AppState::Playing)));
}

/// Before the triggers' frame, the sound system's listener as FUN_00178c10 sets it each frame for
/// a camera that follows a character (the demo's follow camera always does): at the character
/// (the player's, its feet) and indoors when it stands indoors (`indoors`). So the indoor and
/// outdoor sound-triggers' first update finds it as the game's does: a character's ground record
/// is filled as it's placed (FUN_001154b0 -> FUN_0012dc00). For any other camera, a cutscene's
/// (play_script.rs, #222), FUN_00178c10 gives the listener the camera's own transform and leaves
/// its indoors as it was: the cutscene's 3D sounds are heard from its camera (the opening's lava
/// pit from its lava shot). BF_TRIGGER_LOG or BF_SOUND_LOG: each change indoors or out.
fn listener(mut tr: ResMut<Triggers>, map: Res<MapLevel>, game: Res<GameData>, player: Res<Player>, cutscenes: Res<script::Cutscenes>) {
    if tr.live.is_empty() {
        return;
    }
    if let Some(at) = cutscenes.camera_at() {
        tr.sound.listener = Some(at);
        return;
    }
    let feet = player.position + Vec3::Y * (GROUND + player.height.max(0.0));
    tr.sound.listener = Some(feet);
    let (Some(a), Some(level)) = (world::arena(), map.0.as_ref()) else { return };
    let was = tr.sound.indoors;
    let now = indoors(&a, level, &game.0, feet, was.unwrap_or(false));
    if was != Some(now) && (tr.log || std::env::var("BF_SOUND_LOG").is_ok()) {
        println!("t {:.2}: the listener's character is {} (its feet at {feet:.1})", tr.clock, if now { "indoors" } else { "outdoors" });
    }
    tr.sound.indoors = Some(now);
}

/// BF_TRIGGER_LOG: at the start, each sound- and dialog-trigger's sound, its length and how it
/// plays.
fn log_sounds(tr: Res<Triggers>) {
    if !tr.log {
        return;
    }
    for l in &tr.live {
        match &l.state {
            ClassState::Sound(s) => {
                let d = &s.data;
                let place = match s.library {
                    Some(lib) if positional(d.kind) && lib.min_distance != -1.0 => {
                        let min = if d.min_distance > 0.0 { d.min_distance } else { lib.min_distance };
                        format!("at {:.1}, {min} to {} m", s.object_at.unwrap_or(s.at), s.max_distance())
                    }
                    _ => "alike everywhere".to_string(),
                };
                println!("sound-trigger {}: {} {:.2} s, Type {}, {place}{}{}{}{}{}{}", tr.label(l.t.name), s.name, s.length, d.kind,
                         if d.looping { ", looping" } else { "" }, if d.near { format!(", within {} m of a player", s.max_distance()) } else { String::new() },
                         if s.music { ", a music-bank track" } else { "" }, if d.hush { ", hushes the chatter (not modelled)" } else { "" },
                         if s.library.is_none() { ", not in the banks" } else { "" }, if l.enabled { "" } else { " (disabled)" });
            }
            ClassState::Dialog(d) => {
                println!("dialog-trigger {}: {} {:.2} s, said by {}{}", tr.label(l.t.name), d.name, d.length,
                         l.t.objects.first().map_or("no one".to_string(), |&o| tr.label(o)), if d.hush { ", hushes the chatter (not modelled)" } else { "" });
            }
            _ => {}
        }
    }
}

impl Triggers {
    /// Trigger `i`'s sound-trigger fields.
    fn sound_of(&mut self, i: usize) -> Option<&mut Sound> {
        match &mut self.live[i].state {
            ClassState::Sound(s) => Some(s),
            _ => None,
        }
    }

    /// An object as the triggers hold it (FUN_001fb020): a spawn-trigger's current copy (0
    /// before its first), else the object itself. A squad member the demo starts at its
    /// spawn-trigger's place counts as that trigger's copy (`copy_of`, as for its death): the
    /// tutorial's dialog-triggers name trigger_spawn8, whose copy tex_1 is the player.
    fn copy_or(&self, o: u32) -> u32 {
        match self.by_name.get(&o).map(|&k| (k, &self.live[k].state)) {
            Some((_, ClassState::Spawn { copy, .. })) if *copy != 0 => *copy,
            Some((k, ClassState::Spawn { .. })) => self.copy_of.iter().find(|(_, &t)| t == k).map_or(0, |(&c, _)| c),
            _ => o,
        }
    }

    /// A character in the world by its trigger name, as of the last frame.
    fn seen(&self, name: u32) -> Option<Seen> {
        self.sound.seen.iter().find(|c| name != 0 && c.name == name).copied()
    }

    /// FUN_00205fb0: where sound-trigger `i` plays: its first object's place (a character as it
    /// stands, a placed object), else its own.
    fn sound_place(&self, i: usize) -> Vec3 {
        let ClassState::Sound(s) = &self.live[i].state else { return Vec3::ZERO };
        if s.object != 0 {
            if let Some(c) = self.seen(self.copy_or(s.object)) {
                return c.feet;
            }
            if let Some(p) = s.object_at {
                return p;
            }
        }
        s.at
    }

    /// 0x206450: a sound-trigger's handler (see the module notes).
    pub(super) fn sound(&mut self, i: usize, ev: Ev) {
        if !self.live[i].enabled {
            return self.base_and_after(i, &ev);
        }
        let mut k = 0;
        while let Some(j) = self.next_entry(i, &ev, k) {
            k = j + 1;
            let action = self.live[i].t.entries[j].action;
            let (clock, log, who) = (self.clock, self.log, self.who(i));
            let Some(s) = self.sound_of(i) else { return };
            match action {
                act::OPERATE | act::PLAY_SOUND => {
                    s.on = true;
                    if s.data.near {
                        if log {
                            println!("t {clock:.2}: {who} {} on {}: on, {} plays while a player is within {} m", act::name(action), sig::name(ev.signal), s.name,
                                     s.max_distance());
                        }
                    } else {
                        // (+0xa0: a Type 2-5 one's level from FUN_00206880, else 1.0)
                        let zoned = zone(s.data.kind);
                        let level = if zoned { self.zone_level(i) } else { 1.0 };
                        if let Some(s) = self.sound_of(i) {
                            s.level = level;
                            s.heading = level;
                        }
                        if log {
                            let at = if zoned {
                                format!(" (level {level:.2}: the listener is {})", if self.sound.indoors == Some(true) { "indoors" } else { "outdoors" })
                            } else {
                                String::new()
                            };
                            println!("t {clock:.2}: {who} {} on {}{at}", act::name(action), sig::name(ev.signal));
                        }
                        self.sound_start(i, true);
                    }
                }
                act::STOP | act::STOP_SOUND | act::STOP_DIALOG => {
                    if log {
                        println!("t {clock:.2}: {who} {} on {}: {} stops{}", act::name(action), sig::name(ev.signal), s.name,
                                 if s.voice != 0 && s.data.fade_out > 0.0 { format!(" (fading out over {} s)", s.data.fade_out) } else { String::new() });
                    }
                    self.sound_stop(i, false);
                    if let Some(s) = self.sound_of(i) {
                        s.on = false;
                    }
                }
                act::PAUSE_SOUND | act::RESUME_SOUND => {
                    let on = action == act::PAUSE_SOUND;
                    if s.voice != 0 {
                        let voice = s.voice;
                        self.sound.acts.push(VoiceAct::Pause { voice, on });
                    }
                    if let Some(s) = self.sound_of(i) {
                        s.state = if on { PAUSED } else { RUNNING };
                    }
                }
                act::PLAY_DIALOG => {
                    if !s.queued {
                        s.queued = true;
                        let (sound, length, hush) = (s.data.sound, s.length, s.data.hush);
                        let ticket = self.sound.radio.queue(sound, length, hush);
                        if let Some(s) = self.sound_of(i) {
                            s.ticket = ticket;
                            s.state = RUNNING;
                            if log {
                                println!("t {clock:.2}: {who} PLAY_DIALOG on {}: queues {} (ticket {ticket}, {:.2} s)", sig::name(ev.signal), s.name, s.length);
                            }
                        }
                    }
                }
                _ => {
                    self.base_and_after(i, &ev);
                    continue;
                }
            }
            self.entry_used(i, j, &ev);
        }
    }

    /// FUN_00179d60 for another class: `sound` queued on the world's dialog queue with its
    /// `length` and no hush (a radar-trigger's cue, 0x20736a pushes 0; play_radar.rs), its ticket
    /// kept by nobody. `by` names the sender for the log.
    pub(super) fn queue_line(&mut self, sound: u32, length: f32, by: &str) {
        let ticket = self.sound.radio.queue(sound, length, false);
        if self.log {
            println!("t {:.2}: {by} queues h_{sound:08x} on the dialog queue (ticket {ticket}, {length:.2} s)", self.clock);
        }
    }

    /// FUN_00206250: the volume sound-trigger `i` plays at: a Type 2-5 one's level, else its
    /// `volume` / 100, else 2.0, which FUN_002060c0 reads as its library entry's (it sets a
    /// volume only up to 1).
    fn sound_volume(&self, i: usize) -> f32 {
        let ClassState::Sound(s) = &self.live[i].state else { return 0.0 };
        if zone(s.data.kind) {
            s.level
        } else if s.data.volume >= 0.0 {
            s.data.volume * 0.01
        } else {
            2.0
        }
    }

    /// FUN_00206880: where a Type 2-5 trigger's level should be: its volume / 100 (0 without),
    /// raised toward 1 by the share of the listeners indoors (the sound system's +0xc8 bytes;
    /// Types 3 and 4, the indoor ones) or outdoors (2 and 5); 0 with h_15bc1769 and the listener
    /// beyond its max distance. The demo has the one listener (`SoundWorld::indoors`): a share of
    /// 0 or 1.
    fn zone_level(&self, i: usize) -> f32 {
        let ClassState::Sound(s) = &self.live[i].state else { return 0.0 };
        let vol = if s.data.volume >= 0.0 { s.data.volume * 0.01 } else { 0.0 };
        let flagged = if self.sound.indoors == Some(true) { 1.0 } else { 0.0 };
        let share = if matches!(s.data.kind, 3 | 4) { flagged } else { 1.0 - flagged };
        let mut level = share * (1.0 - vol) + vol;
        let max = s.max_distance();
        if s.on && s.data.near && max > 0.0 {
            let place = self.sound_place(i);
            if self.sound.listener.is_some_and(|l| l.distance(place) > max) {
                level = 0.0;
            }
        }
        level.clamp(0.0, 1.0)
    }

    /// FUN_002062a0: sound-trigger `i` starts its sound, and with `voice` its voice too: a
    /// one-shot still playing or a looping one isn't restarted; otherwise it's stopped at once,
    /// and at a volume over 0 its time is its sound's length and it plays (FUN_002060c0), the
    /// squad's chatter hushed for that time when it has h_e725a1a5 (0x206343-0x206360:
    /// FUN_00141100 keyed by the trigger's +4, an object's name); at 0 a one-shot is off.
    fn sound_start(&mut self, i: usize, voice: bool) {
        let (clock, log, who) = (self.clock, self.log, self.who(i));
        let Some(s) = self.sound_of(i) else { return };
        if s.playing && (s.data.looping || s.left > 0.0) {
            if log {
                println!("t {clock:.2}: {who}: {} is still playing ({:.2} s left): not restarted", s.name, s.left);
            }
            return;
        }
        self.sound_stop(i, true);
        let vol = self.sound_volume(i);
        let Some(s) = self.sound_of(i) else { return };
        if vol <= 0.0 {
            if !s.data.looping {
                s.on = false;
                return;
            }
        } else {
            s.left = s.length;
            s.playing = true;
            if voice {
                self.sound_voice(i, vol);
            }
            let key = self.live[i].t.name;
            if let Some(s) = self.sound_of(i).filter(|s| s.data.hush) {
                let left = s.left;
                self.sound.hushes.push((key, Some(left)));
            }
        }
        if let Some(s) = self.sound_of(i) {
            s.state = RUNNING;
        }
    }

    /// FUN_002060c0: sound-trigger `i`'s voice starts at `vol` (over 1: its library entry's):
    /// none when its sound isn't in the banks (FUN_0017c170); 3D at its place for Types 1, 4 and
    /// 5 when its library entry's falloff isn't -1 (a 2D voice takes no place), with its min and
    /// max distance where they're over 0, else the entry's; its fades; looping or once.
    fn sound_voice(&mut self, i: usize, vol: f32) {
        let place = self.sound_place(i);
        let (clock, log, who) = (self.clock, self.log, self.who(i));
        let ClassState::Sound(s) = &mut self.live[i].state else { return };
        let Some(lib) = s.library else {
            s.voice = 0;
            if log {
                println!("t {clock:.2}: {who}: {} isn't in the loaded banks: no voice", s.name);
            }
            return;
        };
        self.sound.voices_made += 1;
        s.voice = self.sound.voices_made;
        s.voice_end = if s.data.looping { f32::INFINITY } else { clock + s.length };
        let d = &s.data;
        let from = if positional(d.kind) && lib.min_distance != -1.0 {
            Heard::At { at: place, min: if d.min_distance > 0.0 { d.min_distance } else { lib.min_distance },
                       max: if d.max_distance > 0.0 { d.max_distance } else { lib.max_distance } }
        } else {
            Heard::Everywhere
        };
        if log {
            let place = match from { Heard::At { at, min, max } => format!(" at {at:.1}, {min} to {max} m"), _ => " alike everywhere".to_string() };
            println!("t {clock:.2}: {who}: plays {} ({:.2} s){place}{}", s.name, s.length, if d.looping { ", looping" } else { "" });
        }
        self.sound.acts.push(VoiceAct::Start { voice: s.voice, sound: d.sound, looping: d.looping, volume: (vol <= 1.0).then_some(vol), from,
                                               fade_in: d.fade_in, fade_out: d.fade_out, music: s.music, by: who });
    }

    /// FUN_002063c0: sound-trigger `i` stopped: its voice (`at_once`, or fading out) unless it's
    /// playing out, its queued line taken back (a ticket 0 stays: the game checks +0x48 against
    /// 0), its chatter mute taken off when it has h_e725a1a5 (0x20640a-0x206422: FUN_00141180), its
    /// time ended; its "queued" bit stays.
    fn sound_stop(&mut self, i: usize, at_once: bool) {
        let key = self.live[i].t.name;
        let ClassState::Sound(s) = &mut self.live[i].state else { return };
        let world = &mut self.sound;
        if s.voice != 0 && !s.play_out {
            world.acts.push(VoiceAct::Stop { voice: s.voice, at_once });
            s.voice = 0;
        }
        if s.ticket != 0 {
            world.radio.cancel(s.ticket, &mut world.acts);
            s.ticket = 0;
        }
        if s.data.hush {
            world.hushes.push((key, None));
        }
        s.state = STOPPED;
        s.left = 0.0;
        s.playing = false;
    }

    /// FUN_00206610, a sound-trigger's disable (its vtable +0x48, after the base's bits 0 and 6):
    /// a voice still playing plays out (bit 3, and bit 6 kept for the update that lets it go),
    /// and a sound playing is stopped without its voice (FUN_002063c0: no FINISHED).
    pub(super) fn sound_disabled(&mut self, i: usize) {
        let clock = self.clock;
        let Some(s) = self.sound_of(i) else { return };
        s.listed = false;
        if s.voice != 0 && clock < s.voice_end {
            s.play_out = true;
            s.listed = true;
        }
        if s.playing {
            self.sound_stop(i, false);
        }
    }

    /// 0x206690: a sound-trigger's update (see the module notes); the world calls it only while
    /// +0x3d bit 6 is set (`listed`).
    pub(super) fn sound_update(&mut self, i: usize, dt: f32) {
        if !self.sound_of(i).is_some_and(|s| s.listed) {
            return;
        }
        self.base_update(i);
        // (0x2066a4: FUN_001fe660 again, so it gets its FIRST_UPDATE disabled too)
        self.first_update(i);
        let (clock, log, id) = (self.clock, self.log, self.live[i].t.id);
        // (its name for the log, made only when logging: this runs every frame)
        let who = if log { self.who(i) } else { String::new() };
        let Some(s) = self.sound_of(i) else { return };
        // a voice playing out after a DISABLE: let go once it has ended (FUN_00206390 with 1,
        // on a voice that's done: the demo's is left to run out, see below)
        if s.play_out {
            if clock < s.voice_end {
                return;
            }
            s.play_out = false;
            s.listed = false;
            s.voice = 0;
            return;
        }
        let finished = Ev { signal: sig::FINISHED, id, subject: 0, data: -1 };
        if s.state == RUNNING {
            if s.queued {
                let ticket = s.ticket;
                if self.sound.radio.done(ticket) {
                    let Some(s) = self.sound_of(i) else { return };
                    s.queued = false;
                    s.ticket = 0;
                    let send = !zone(s.data.kind);
                    if log {
                        println!("t {clock:.2}: {who}: its line {} has played", s.name);
                    }
                    if send {
                        self.send(i, finished);
                    }
                    // (stopped after the send, as 0x20676a has it)
                    if let Some(s) = self.sound_of(i) {
                        s.state = STOPPED;
                    }
                }
            } else {
                s.left -= dt;
                if !s.data.looping && s.left <= 0.0 {
                    if log {
                        println!("t {clock:.2}: {who}: {} has played ({:.2} s)", s.name, s.length);
                    }
                    if !zone(s.data.kind) {
                        self.send(i, finished);
                    }
                    if let Some(s) = self.sound_of(i) {
                        s.on = false;
                        // (a voice that started with its time has played its data by now, within
                        // the frame, and the stop that follows finds it done. The demo's decoder
                        // makes 65 samples of a 36-byte block where the game reckons 64, so its
                        // voice runs a little longer: it's left to run out, not cut)
                        if s.voice_end - clock <= dt + 1e-4 {
                            s.voice = 0;
                        }
                    }
                    self.sound_stop(i, false);
                }
            }
        }
        let Some(s) = self.sound_of(i) else { return };
        if s.queued {
            return;
        }
        // a positional voice follows its object as it moves (a character's: a placed one stays)
        if s.voice != 0 && positional(s.data.kind) && s.object != 0 {
            let (voice, object) = (s.voice, s.object);
            if let Some(c) = self.seen(self.copy_or(object)) {
                self.sound.acts.push(VoiceAct::Move { voice, at: c.feet });
            }
        }
        let Some(s) = self.sound_of(i) else { return };
        if !s.on {
            return;
        }
        if zone(s.data.kind) {
            self.zone_ramp(i, dt);
        } else if s.data.near {
            self.sound_near(i);
        }
    }

    /// FUN_00206a20: a Type 2-5 trigger's level moves to where it should be (`zone_level`) at
    /// ZONE_RAMP a second; at 0 it stops, and above it starts again when it isn't playing; its
    /// voice's volume follows (the sound system's +0x34).
    fn zone_ramp(&mut self, i: usize, dt: f32) {
        let target = self.zone_level(i);
        let (clock, log) = (self.clock, self.log);
        let who = if log { self.who(i) } else { String::new() };
        let Some(s) = self.sound_of(i) else { return };
        if target == s.level {
            return;
        }
        if log && target != s.heading {
            println!("t {clock:.2}: {who}: {} heads to level {target:.2} (from {:.2})", s.name, s.level);
        }
        s.heading = target;
        s.level = if target > s.level { (s.level + dt * ZONE_RAMP).min(target) } else { (s.level - dt * ZONE_RAMP).max(target) };
        if s.level <= 0.0 {
            self.sound_stop(i, false);
            return;
        }
        if !s.playing || (!s.data.looping && s.left <= 0.0) {
            let level = s.level;
            self.sound_start(i, false);
            self.sound_voice(i, level);
        }
        if let Some(s) = self.sound_of(i) {
            if s.voice != 0 {
                let (voice, volume) = (s.voice, s.level);
                self.sound.acts.push(VoiceAct::Volume { voice, volume });
            }
        }
    }

    /// FUN_00206b50: h_15bc1769's start and stop by distance. Playing, it stops (fading out) once
    /// no player's character is within its max distance, its voice alone stops once the listener
    /// isn't; not playing, it starts (FUN_002062a0) once one is, its voice once the listener is
    /// too.
    fn sound_near(&mut self, i: usize) {
        let place = self.sound_place(i);
        let Some(s) = self.sound_of(i) else { return };
        let max = s.max_distance();
        if max <= 0.0 {
            return;
        }
        let (playing, looping, left, voice) = (s.playing, s.data.looping, s.left, s.voice);
        // (the world's +0xbec vtable +0x24, 0x8d660: the nearest player-controlled character)
        let d = self.sound.seen.iter().filter(|c| c.human && !c.dead).map(|c| c.feet.distance(place)).fold(f32::INFINITY, f32::min);
        let heard = self.sound.listener.map_or(d, |l| l.distance(place));
        if playing && (looping || left > 0.0) {
            if d > max {
                if self.log {
                    println!("t {:.2}: {}: no player within {max} m ({d:.1} m): it stops", self.clock, self.who(i));
                }
                self.sound_stop(i, false);
                return;
            }
            if heard > max {
                // FUN_00206390 with 0: the voice alone
                if let Some(s) = self.sound_of(i) {
                    if s.voice != 0 && !s.play_out {
                        let voice = s.voice;
                        s.voice = 0;
                        self.sound.acts.push(VoiceAct::Stop { voice, at_once: false });
                    }
                }
                return;
            }
            if voice != 0 {
                return;
            }
        } else {
            if d >= max {
                return;
            }
            self.sound_start(i, false);
            if heard >= max {
                return;
            }
        }
        if self.log {
            println!("t {:.2}: {}: a player within {max} m ({d:.1} m)", self.clock, self.who(i));
        }
        let vol = self.sound_volume(i);
        self.sound_voice(i, vol);
    }

    /// A dialog-trigger's speaker (0x206d80: its first object, a spawn-trigger's copy for a
    /// spawn-trigger), a character in the world: its name and whether it's dead. None when there's
    /// none (the handler then skips the entry without counting it).
    fn dialog_speaker(&self, i: usize) -> Option<(u32, bool)> {
        let o = *self.live[i].t.objects.first()?;
        let c = self.seen(self.copy_or(o))?;
        Some((c.name, c.dead))
    }

    /// 0x206d20: a dialog-trigger's handler: PLAY_DIALOG and OPERATE make its character say the
    /// line (a dead one: FINISHED at once), STOP and STOP_DIALOG stop it, PAUSE_ and
    /// RESUME_DIALOG are counted and do nothing, the rest are the base's; with no character in
    /// the world an entry does nothing at all.
    pub(super) fn dialog(&mut self, i: usize, ev: Ev) {
        if !self.live[i].enabled {
            return self.base_and_after(i, &ev);
        }
        let mut k = 0;
        while let Some(j) = self.next_entry(i, &ev, k) {
            k = j + 1;
            let Some((speaker, dead)) = self.dialog_speaker(i) else { continue };
            let action = self.live[i].t.entries[j].action;
            let (clock, log, who, id) = (self.clock, self.log, self.who(i), self.live[i].t.id);
            let speaker_name = self.label(speaker);
            match action {
                act::OPERATE | act::PLAY_DIALOG => {
                    if dead {
                        if log {
                            println!("t {clock:.2}: {who} {} on {}: {speaker_name} is dead: FINISHED at once", act::name(action), sig::name(ev.signal));
                        }
                        self.send(i, Ev { signal: sig::FINISHED, id, subject: 0, data: -1 });
                        if let ClassState::Dialog(d) = &mut self.live[i].state {
                            d.done = true;
                        }
                    } else if let ClassState::Dialog(d) = &mut self.live[i].state {
                        d.done = false;
                        d.ends = clock + d.length;
                        self.sound.voices_made += 1;
                        d.voice = self.sound.voices_made;
                        if log {
                            println!("t {clock:.2}: {who} {} on {}: {speaker_name} says {} ({:.2} s)", act::name(action), sig::name(ev.signal), d.name, d.length);
                        }
                        self.sound.acts.push(VoiceAct::Start { voice: d.voice, sound: d.dialog, looping: false, volume: None, from: Heard::Speaker(speaker),
                                                               fade_in: 0.0, fade_out: 0.0, music: false, by: who });
                        // its hush flag: the chatter hushed for the line's length (FUN_001267b0 ->
                        // FUN_001435d0's FUN_00141100 at 0x14379d, keyed by the character's +4,
                        // its name, xor the line); its stop takes nothing off (FUN_00143570)
                        if d.hush && d.length > 0.0 {
                            self.sound.hushes.push((speaker ^ d.dialog, Some(d.length)));
                        }
                    }
                }
                act::STOP | act::STOP_DIALOG => {
                    // FUN_001267f0: the character's line stopped
                    if let ClassState::Dialog(d) = &mut self.live[i].state {
                        if d.voice != 0 {
                            self.sound.acts.push(VoiceAct::Stop { voice: d.voice, at_once: true });
                            d.voice = 0;
                        }
                        d.ends = clock;
                        d.done = true;
                    }
                }
                act::PAUSE_DIALOG | act::RESUME_DIALOG => {}
                _ => {
                    self.base_and_after(i, &ev);
                    continue;
                }
            }
            self.entry_used(i, j, &ev);
        }
    }

    /// 0x206ee0: a dialog-trigger's update: the base's, then, enabled, its FIRST_UPDATE and, once
    /// its character has said its line (FUN_00126810 false), FINISHED to its targets.
    pub(super) fn dialog_update(&mut self, i: usize) {
        self.base_update(i);
        if !self.live[i].enabled {
            return;
        }
        self.first_update(i);
        let ClassState::Dialog(d) = &self.live[i].state else { return };
        if d.done || self.dialog_speaker(i).is_none() || self.clock < d.ends {
            return;
        }
        if self.log {
            println!("t {:.2}: {}: its line {} has been said", self.clock, self.who(i), d.name);
        }
        let id = self.live[i].t.id;
        self.send(i, Ev { signal: sig::FINISHED, id, subject: 0, data: -1 });
        if let ClassState::Dialog(d) = &mut self.live[i].state {
            d.done = true;
            d.voice = 0;
        }
    }
}

/// A voice playing for the triggers (an entity with its AudioPlayer): what `voices` needs to keep
/// its volume (distance, fades) and to stop or pause it.
#[derive(Component)]
struct TriggerVoice {
    /// the triggers' handle for it
    id: u32,
    /// its volume before distance and fades (x `scale`)
    volume: f32,
    /// the music volume for a music-bank track (`music_volume`), else 1: what its volume is
    /// scaled by, also when it's set again (`VoiceAct::Volume`)
    scale: f32,
    /// where it plays from
    from: Heard,
    /// its fades (s): in as it starts, out as a stop with them ends it
    fade_in: f32,
    fade_out: f32,
    /// how long it has played (s)
    age: f32,
    /// stopped with its fade-out: the seconds left of it
    stopping: Option<f32>,
    /// paused (PAUSE_SOUND)
    paused: bool,
}

/// A 3D voice's gain `d` m from the listener, as the dsound library's distance attenuation
/// (FUN_00314e64) works it out with the game's rolloff curve (ROLLOFF): full within `min`; from
/// there to `max` five equal stretches, each linear from the curve's last point (1.0 at `min`) to
/// its next: 1.0, 0.375, 0.125, 0.0625 and 0 at `max`. The distance is held at `max`, so it's
/// silent there and beyond (the buffers aren't made with DSBCAPS_MUTE3DATMAXDISTANCE, 0x1817c8:
/// their flags are DSBCAPS_CTRL3D alone, but the curve ends at 0). With `max` short of `min`
/// (m07_c's h_133f6a6c: 100 and 50, the one of the levels' 504 positional sound-triggers) the
/// library's sum gives 0 past `min`; with the two equal it divides 0 by 0, taken the same.
pub(crate) fn distance_gain(d: f32, min: f32, max: f32) -> f32 {
    if d <= min {
        return 1.0;
    }
    if max <= min {
        return 0.0;
    }
    let step = (max - min) / ROLLOFF.len() as f32;
    let x = (d.min(max) - min) / step;
    let k = (x as usize).min(ROLLOFF.len() - 1);
    let from = if k == 0 { 1.0 } else { ROLLOFF[k - 1] };
    from + (ROLLOFF[k] - from) * (x - k as f32)
}

impl TriggerVoice {
    /// Its volume now: its own, x its distance gain from the listener (`distance_gain`) or for a
    /// line its speaker's share (SPEAKER_HEARD: 1 for the player's character), x its fades: in
    /// over `fade_in` from its start, linearly; out over `fade_out` once stopped, linearly in dB
    /// down FADE_OUT_DB (the envelope's attack and release, see FADE_OUT_DB).
    fn level(&self, listener: Vec3, player: Vec3, seen: &[Seen]) -> f32 {
        let place = match self.from {
            Heard::Everywhere => 1.0,
            Heard::At { at, min, max } => distance_gain(listener.distance(at), min, max),
            Heard::Speaker(name) => seen.iter().find(|c| c.name == name).map_or(1.0, |c| {
                if c.human { 1.0 } else { (1.0 - c.feet.distance(player) / SPEAKER_HEARD).clamp(SPEAKER_FLOOR, 1.0) * SPEAKER_SHARE }
            }),
        };
        let fade_in = if self.fade_in > 0.0 { (self.age / self.fade_in).clamp(0.0, 1.0) } else { 1.0 };
        let fade_out = match self.stopping {
            Some(left) if self.fade_out > 0.0 => {
                let gone = ((self.fade_out - left) / self.fade_out).clamp(0.0, 1.0);
                10f32.powf(-FADE_OUT_DB / 20.0 * gone)
            }
            _ => 1.0,
        };
        self.volume * place * fade_in * fade_out
    }
}

/// The level's music bank, data/sounds/<level>.xwb, for the tracks its sound-triggers play: read
/// when the first one starts (its entries' names hash to the sound ids: "tutorial_01" is
/// h_f5862d3f), each track decoded the first time it plays.
#[derive(Default)]
struct MusicBank {
    level: String,
    bytes: Option<Arc<Vec<u8>>>,
    entries: HashMap<u32, (u32, std::ops::Range<usize>)>,
    decoded: HashMap<u32, Option<Handle<AudioSource>>>,
}

impl MusicBank {
    /// Sound `id`'s track of `level`'s music bank, decoded (Xbox ADPCM, stereo), if it has one.
    fn track(&mut self, level: &str, id: u32, sources: &mut Assets<AudioSource>) -> Option<Handle<AudioSource>> {
        if self.level != level {
            *self = MusicBank { level: level.to_string(), ..Default::default() };
            let path = data_dir().join("sounds").join(format!("{level}.xwb"));
            match std::fs::read(&path) {
                Ok(d) => {
                    self.entries = audio::xwb_entries(&d).into_iter().map(|(name, fmt, range)| (h(&name), (fmt, range))).collect();
                    self.bytes = Some(Arc::new(d));
                }
                Err(e) => eprintln!("{}: {e}", path.display()),
            }
        }
        let MusicBank { bytes, entries, decoded, .. } = self;
        decoded.entry(id).or_insert_with(|| {
            let (fmt, range) = entries.get(&id)?.clone();
            let wav = audio::xwb_wav(bytes.as_ref()?, fmt, range)?;
            Some(sources.add(AudioSource { bytes: Arc::from(wav.into_boxed_slice()) }))
        }).clone()
    }
}

/// Each frame after the triggers' (play_triggers.rs `run`): the world's dialog queue (FUN_00179ec0,
/// the world update's last step), then what the triggers asked of the sound system: voices
/// started (the sound banks' sounds, the music bank's tracks through the music volume, marked
/// LevelMusic), stopped, their volume set, paused, moved; then each voice's volume (its distance
/// from the listener, as `listener` found it; its fades). Where the characters are
/// is kept for the triggers' next frame. The chatter mutes the lines asked for go to the chatter
/// manager (play_chatter.rs). BF_MUTE: nothing plays; BF_NO_MUSIC and deathmatch (which plays
/// without music, play.rs `setup`): no music-bank track. BF_SOUND_LOG: each voice as it starts
/// and stops.
#[allow(clippy::too_many_arguments)]
fn voices(mut commands: Commands, time: Res<Time>, mut tr: ResMut<Triggers>, game: Res<GameData>, current: Res<CurrentMap>, mode: Res<Mode>,
          player: Res<Player>, squad: Res<Squad>, mut cache: ResMut<SoundCache>, mut sources: ResMut<Assets<AudioSource>>, mut bank: Local<MusicBank>,
          mut chatter: ResMut<chatter::Chatter>, mut playing: Query<(Entity, &mut TriggerVoice, Option<&mut AudioSink>)>) {
    if tr.live.is_empty() {
        return;
    }
    let dt = frame_dt(&time);
    let tr = &mut *tr;
    let now = tr.clock;
    // the dialog queue
    let world = &mut tr.sound;
    world.radio.update(dt, now, &mut world.acts, &mut world.hushes, &mut world.voices_made, &game.0.sounds, tr.log);
    // the chatter mutes
    for (key, length) in world.hushes.drain(..) {
        chatter.hush(key, length);
    }
    // where the listener and the characters are
    let here = player.position + Vec3::Y * GROUND;
    let listener = tr.sound.listener.unwrap_or(here);
    // (the unnamed ones too: the player on a map whose squad no spawn-trigger names is still the
    // character h_15bc1769 measures from)
    let seen: Vec<Seen> = std::iter::once((&*player, true)).chain(squad.0.iter().map(|m| (m, false))).filter(|(u, _)| !u.stand_in)
        .map(|(u, human)| Seen { name: u.trigger_name, feet: u.position + Vec3::Y * GROUND, dead: u.dead, human }).collect();
    tr.sound.seen = seen.clone();
    let (mute, no_music, log) = (std::env::var("BF_MUTE").is_ok(), std::env::var("BF_NO_MUSIC").is_ok() || mode.deathmatch, std::env::var("BF_SOUND_LOG").is_ok());
    // the voices started in this batch, not in the query yet (a stop in the same frame ends them)
    let mut started: HashMap<u32, Entity> = HashMap::new();
    // (the music voice's replaces and the acts they leave without a voice: either log)
    let music_log = log || tr.log;
    let mut acts: VecDeque<VoiceAct> = std::mem::take(&mut tr.sound.acts).into();
    while let Some(a) = acts.pop_front() {
        // a handle 0x17eff0 took away finds no voice (0x17ee90): the act does nothing
        if let Some(voice) = a.handle().filter(|v| tr.sound.music.orphans.contains(v)) {
            if music_log {
                println!("t={now:6.2} trigger voice {voice}: {} dropped: its handle was erased as it replaced the music voice (0x17eff0)", a.what());
            }
            continue;
        }
        match a {
            VoiceAct::Start { voice, sound, looping, volume, from, fade_in, fade_out, music, by } => {
                // the one music voice (0x17eff0, `MusicSlot`), before the voice plays
                if voice_class(&game.0.sounds, sound) == CLASS_MUSIC {
                    let slot = &mut tr.sound.music;
                    if let Some(old) = slot.current.filter(|c| now < c.until) {
                        if music_log {
                            println!("t={now:6.2} trigger voice {voice}: {} for {by} replaces the music voice {}{}; its own handle is erased (0x17eff0): \
                                      its trigger can no longer stop, set or move it", sound_name(&game.0.sounds, sound), old.voice,
                                     if old.fade_out > 0.0 { format!(", which fades out over {} s", old.fade_out) } else { ", which stops".to_string() });
                        }
                        slot.orphans.insert(voice);
                        acts.push_front(VoiceAct::Replaced { voice: old.voice });
                        // (its "still playing", +0x2c, finds nothing either)
                        for l in &mut tr.live {
                            if let ClassState::Sound(s) = &mut l.state {
                                if s.voice == voice {
                                    s.voice_end = now;
                                }
                            }
                        }
                    }
                    let length = game.0.sounds.length(sound).unwrap_or(0.0);
                    tr.sound.music.current = Some(MusicVoice { voice, until: if looping { f32::INFINITY } else { now + length }, fade_out });
                }
                let own = volume.unwrap_or_else(|| game.0.sounds.params(sound).map_or(1.0, |p| p.volume));
                let scale = if music { music_volume() } else { 1.0 };
                let v = TriggerVoice { id: voice, volume: own * scale, scale, from, fade_in, fade_out, age: 0.0, stopping: None, paused: false };
                let start = v.level(listener, here, &seen);
                let what = if log {
                    let place = match from {
                        Heard::At { at, min, max } => format!(" at {at:.1} ({min} to {max} m, {:.1} m away)", listener.distance(at)),
                        Heard::Speaker(name) => format!(" said by {}", tr.label(name)),
                        Heard::Everywhere => String::new(),
                    };
                    format!("t={now:6.2} trigger voice {voice}: {} for {by}: vol {:.2} (now {start:.2}){place}{}{}{}", sound_name(&game.0.sounds, sound), v.volume,
                            if looping { ", looping" } else { "" }, if fade_in > 0.0 { format!(", fading in over {fade_in} s") } else { String::new() },
                            if music { ", music" } else { "" })
                } else {
                    String::new()
                };
                if mute || (music && no_music) {
                    if log {
                        println!("{what}: not played ({})", if mute { "BF_MUTE" } else { "no music here" });
                    }
                    continue;
                }
                let handle = if music {
                    bank.track(&current.0, sound, &mut sources)
                } else {
                    cache.0.entry(sound).or_insert_with(|| game.0.sounds.wav(sound).map(|w| sources.add(AudioSource { bytes: Arc::from(w.into_boxed_slice()) }))).clone()
                };
                let Some(handle) = handle else {
                    if log {
                        println!("{what}: no sample data in the loaded banks");
                    }
                    continue;
                };
                if log {
                    println!("{what}");
                }
                let settings = if looping { PlaybackSettings::LOOP } else { PlaybackSettings::DESPAWN };
                let mut e = commands.spawn((AudioPlayer::new(handle), PlaybackSettings { volume: AudioVolume::Linear(start), ..settings }, v,
                                            Name::new(format!("trigger voice {voice} h_{sound:08x}"))));
                if music {
                    e.insert(LevelMusic);
                }
                started.insert(voice, e.id());
            }
            VoiceAct::Stop { .. } | VoiceAct::Replaced { .. } => {
                let (voice, at_once) = match a {
                    VoiceAct::Stop { voice, at_once } => (voice, at_once),
                    VoiceAct::Replaced { voice } => (voice, false),
                    _ => unreachable!(),
                };
                // the music voice stops sounding once its release has run
                if let Some(c) = tr.sound.music.current.as_mut().filter(|c| c.voice == voice) {
                    c.until = c.until.min(now + if at_once { 0.0 } else { c.fade_out });
                }
                if let Some(e) = started.remove(&voice) {
                    commands.entity(e).try_despawn();
                    if log {
                        println!("t={now:6.2} trigger voice {voice} stops as it starts");
                    }
                    continue;
                }
                for (e, mut v, _) in &mut playing {
                    if v.id != voice {
                        continue;
                    }
                    if at_once || v.fade_out <= 0.0 {
                        commands.entity(e).try_despawn();
                    } else if v.stopping.is_none() {
                        v.stopping = Some(v.fade_out);
                    }
                    if log {
                        println!("t={now:6.2} trigger voice {voice} stops{}", if at_once || v.fade_out <= 0.0 { String::new() } else { format!(", fading out over {} s", v.fade_out) });
                    }
                }
            }
            VoiceAct::Volume { voice, volume } => {
                // (a voice started in this batch has it already)
                for (_, mut v, _) in &mut playing {
                    if v.id == voice {
                        v.volume = volume * v.scale;
                    }
                }
            }
            VoiceAct::Pause { voice, on } => {
                for (_, mut v, sink) in &mut playing {
                    if v.id == voice {
                        v.paused = on;
                        if let Some(s) = sink {
                            if on { s.pause() } else { s.play() }
                        }
                    }
                }
            }
            VoiceAct::Move { voice, at } => {
                for (_, mut v, _) in &mut playing {
                    if let (true, Heard::At { at: place, .. }) = (v.id == voice, &mut v.from) {
                        *place = at;
                    }
                }
            }
        }
    }
    // each voice's volume now
    for (e, mut v, sink) in &mut playing {
        if !v.paused {
            v.age += dt;
        }
        if let Some(left) = v.stopping {
            let left = left - dt;
            if left <= 0.0 {
                commands.entity(e).try_despawn();
                continue;
            }
            v.stopping = Some(left);
        }
        let level = v.level(listener, here, &seen);
        if let Some(mut s) = sink {
            s.set_volume(AudioVolume::Linear(level));
        }
    }
}

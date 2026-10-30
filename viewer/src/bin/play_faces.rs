//! Faces (issue #255): what a character's face shows in play, as default.xbe's face controller
//! (char+0x5b8, FUN_00143380) drives it. The game plays no face clip while a character talks: the
//! controller writes the face bones' joints every frame from the 22 viseme weights over the base
//! pose (`Sc_base`), and only the blink is a clip. The first slice made the controller and the
//! blink; the second (#325) the lip-sync: a line's viseme tracks drive the mouth's weights while
//! it is said. The expressions' weights fade as the game's do, but nothing sets one yet (#326).
//!
//!  - **The table** (`bf/face.rs`, FUN_00142cb0 and FUN_00142df0): per model, the face bones from
//!    `Sc_base`, its pose, and the 22 `Sc_<viseme>` poses with the type's scales.
//!  - **The face bones** belong to the controller (FUN_00143380 flags each node, +0x44 bit 5,
//!    which the animation binder FUN_00234190 skips): a body clip never moves them, so the demo
//!    lays no face clip over its poses either (it did, `Sc_bump`, the b/m/p mouth, before this),
//!    and `step` puts the controller's pose on them after every layer, overlay and aim turn of
//!    the body.
//!  - **At rest** the face shows the base pose, written at the controller's construction
//!    (FUN_00143380's last call). The blend (FUN_001439b0) writes the face only while something
//!    shows: a line being said with its tracks, or an expression that isn't BASE. Otherwise it
//!    stops the line (FUN_00143570: the mouth's weights zeroed) and writes nothing, so the bones
//!    keep the last pose written: the same here (`Face::output`). So a line that ends with its
//!    mouth half open leaves it so until the next line (take07's ends with `if` at 0.44).
//!  - **The blink** (FUN_00143d10): the clock `since_blink` counts up each tick, starting at 10 s
//!    (0x41200000, set by the constructor) so the first blink comes at the first tick. Once the
//!    clip's length (0.333 s) is behind it, a time over the type's blink interval (h_09109ae2:
//!    4 s on the squad) plays `Sc_blink` once and zeroes the clock. Between the clip's length and
//!    the interval the game tests a random number against the type's chance (0.05), but its
//!    number runs 0 to 131072 (unscaled, over dt x 30), so the test passes about once in 2.6
//!    million frames: here it never does, and the blink is a regular cycle of the interval plus
//!    up to a frame (the takes' face logs, todo/255 take01-03, confirm it: every controller's
//!    +0x4c resets every 4.0 s). The clip drives its lids (the bones of `Sc_blink` that aren't
//!    face bones); after it ends they stay at its last key, as the nodes keep what the clip last
//!    wrote. The clip is eased in over 0.1 s (a transition from the lids' pose, the weight
//!    smoothstep(t / 0.1): the request's +0x10, read at 0x2536f0, FUN_00280180 and 0x280330): the
//!    lids show the clip from 0.1 s on and lag it before (`bf/face.rs` `write_blink`), from the
//!    model's own lids at the first blink and from the clip's last key, where the last one left
//!    them, at the others.
//!  - **The phase is when the character was made**: the first blink is the first tick and the
//!    cycle runs on from it, so characters made apart blink apart (the squad's copies, 0.5 s
//!    apart in e34, as the takes' controllers are) and ones made together blink together. A
//!    character not in the world yet (`Player::absent`) isn't ticked (`step`).
//!  - **Alive only, once a frame**: the game ticks the controller from the character's update
//!    while it lives (FUN_0011caa0, 0x11dc7b), so `step` runs in `step_player`: a dead
//!    character's face keeps the pose it died with (the ragdoll starts from `last_world`, which
//!    has it), a cutscene's characters blink like the rest. A knocked-down living character has
//!    no step while its ragdoll lies, but its controller ticks all the same (`tick`): its line and
//!    its blink run on their time, not drawn (the ragdoll holds the face it fell with), and the
//!    get-up draws the face where they are by then; the get-up's second step of a frame draws it
//!    without a second tick.
//!
//! **The lip-sync** (#325): the weights, 22 floats in the VE_ order (`face::VISEME_NAMES`: 0-14
//! the mouth shapes, 15-21 the expressions), come from the line being said. Every speech path
//! with a speaker ends in the controller's speak, FUN_001435d0 (`speak`, `Face::speak`): it stops
//! what the controller was saying (FUN_00143570, `stop`: the mouth's weights zeroed), keeps the
//! sound library entry's viseme tracks (+4: the entry's +0x1c, `bf/audio.rs` `Visemes`), speaks
//! for the sound's length (+0x54, +0x50) and notes its clock (+0x44 = +0x40). Each tick
//! (FUN_001437d0) the clock goes on and the time left down; while it is over 0 the blend writes
//! the tracks at the time since the line began into the mouth's weights (FUN_0017aa30), the rest
//! keeping theirs, and blends the poses by them. Nothing is read from the audio. The paths:
//!  - **chatter** (`play_chatter.rs` `Chatter::voice_start` / `voice_stop`, the sound event's
//!    start FUN_0013fab0 -> 0x13fb31 and its stop 0x13fbde): every squad and enemy line;
//!  - **a dialog-trigger's line** (`play_sound_triggers.rs` `speakers`: FUN_001267b0 -> 0x1267d1,
//!    its STOP FUN_001267f0 -> FUN_00143570), a cutscene's on-camera lines among them;
//!  - **not** the world's dialog queue (the radio: Bratton's lines), which has no speaker.
//!
//!  The two paths come at different places in the game's world update (FUN_000d9a20): the
//!  chatter manager's update (0xd9ac7) before the objects' ticks (0xd9d0b), so a chatter line's
//!  mouth moves in the frame its sound starts; the scripts' update (0xd9d18, whose 103 signals a
//!  dialog-trigger) and the triggers' loop (0xd9d20) after them, so a dialog line's moves from the
//!  next (take07's log: the controller read with the line just begun, +0x44 = +0x40, between two
//!  frames). The demo's chatter runs after the characters' step, so it backs its line's start and
//!  time left up by the frame (`ago`); the dialog lines are put on the faces after the step too
//!  (`speakers`), as the game's are.
//!
//! Test hooks:
//!  - `BF_FACE_LOG=1` prints each face made (its bones, its blink, its stretch list), each blink
//!    as it starts, each line said or stopped, and the weights each tick while a line speaks
//!    (`t=12.07 tex: h_ec50fb88 at 0.0667 s, 2.980 s left: if=0.179`: the time since its start
//!    and its time left, the controller's +0x40 - +0x44 and +0x50).
//!  - `BF_FACE_CHECK=1` prints, for each face made, how closely its table reproduces the clips
//!    it was read from (`check`).
//!  - `BF_FACE_COST=1` prints, each second, the faces' share of the frame (their steps' time).
//!  - `BF_TEST_SPEAK` (play_chatter.rs) has a character say a sound at a time.
//!  - `BF_TEST_FACE=<s>,<who>,<viseme>,<weight>[;...]`: at `s` seconds set a viseme's weight (0
//!    to 1) on `who`: `me` (the controlled character), a squad name or 0-3 (`tex`, `2`), or
//!    `all`. A test pose (`Face::test`): while one of its weights is over 0 the blend shows it in
//!    place of the controller's weights, and it stays until another entry changes it. The mouth
//!    at a weight of 1 shows the viseme's clip, which bf_viewer's Q / E shows too (a check of the
//!    blend). With every weight back at 0 the face keeps the last pose blended (the game's idle
//!    controller writes nothing): set `base` to 1 to go back to the rest face.
//!  - `BF_TEST_FACE_CAM=<s>,<who>[,<m>[,<rise>]][;...]`: at `s` seconds the free camera goes `m`
//!    metres (0.8) in front of `who`'s face, `rise` m (0) higher (negative: lower), looking at it
//!    (`who` as above, the first it names), and stays there as the head moves: the close-ups for
//!    a captured face, whatever height the head is and however the idle turns it. Brutus's head
//!    hangs forward and low: `1.1,-0.3` frames it.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::OnceLock;

use bf_viewer::bf::audio::{SoundBank, Visemes};
use bf_viewer::bf::face::{self, FaceTable, Kind, MOUTH_VISEMES, VISEMES};

use super::*;

/// The blink clock at a controller's construction (FUN_00143380: `+0x4c = 0x41200000`, 10.0): far
/// over any type's interval, so the first tick blinks.
const FIRST_BLINK_CLOCK: f32 = 10.0;
/// How fast an expression's weight moves (FUN_001438d0: 2.0 a second, [0x40c934]): 0.5 s from
/// nothing to full, and back. todo/255 take04's log has Tex's angry weight rising 0.12-0.13 a 15 Hz
/// row and full in 0.5-0.6 s.
const FADE_RATE: f32 = 2.0;
/// EFE_BASE (the EFE_ table 0x3be030: 1), the expression a controller starts with (FUN_00143380:
/// +0x58 = +0x5c = 1): no weight of its own (`Sc_base` is the base pose), so a face showing it
/// shows no expression (FUN_001438d0 returns "shown isn't BASE"). An expression e's weight is
/// 15 + e (the map 0x3c0340: EFE_ANGRY 0 is `angry`, 15).
const BASE: usize = 1;

/// A character's face controller (the game's object at char+0x5b8, 0x64 bytes): the 22 weights,
/// the pose the face bones show, the line it says and the blink's clock. The model's table
/// (`Character::face`) holds the data; this holds what changes.
#[derive(Clone, Debug)]
pub struct Face {
    /// the viseme weights (the controller's +0x34 array, 22 floats), in `face::VISEME_NAMES`'
    /// order, 0 to 1 (one above 1 counts as 1, one below 0 as 0): the line's tracks write the
    /// mouth's (0-14) and the expression fade the last seven
    weights: [f32; VISEMES],
    /// the pose the face bones show now, one entry per face bone (the output pose, +0x24): the
    /// base at the start, the last blend's after
    output: Vec<[f32; 4]>,
    /// seconds since the last blink (+0x4c), 10 at the start
    since_blink: f32,
    /// a blink has started: the lids are the clip's, held at its last key once it ends
    blinked: bool,
    /// how many blinks have started (the log's count)
    blinks: u32,
    /// the controller's clock (+0x40): every tick's dt, from 0 at its construction
    clock: f32,
    /// the sound it says (the game keeps its voice at +0: here the sound, for the log), None once
    /// stopped
    sound: Option<u32>,
    /// the line's viseme tracks (+4: its sound library entry's +0x1c), None for a sound without
    /// a speech block or once stopped
    tracks: Option<Arc<Visemes>>,
    /// the clock at the line's start (+0x44)
    start: f32,
    /// the line's time left (+0x50: the sound's length, run down by the ticks)
    left: f32,
    /// speaking (+0x54): set by the speak, cleared by the tick that takes `left` to 0 or under, or
    /// by a stop
    speaking: bool,
    /// the expression it heads to (+0x58) and the one it shows (+0x5c), EFE_ values: BASE at
    /// the start. Nothing sets them yet (#326: Tex's berserk, the expression-trigger, the
    /// chatter's events), so the fade (`fade`) is idle
    target: usize,
    shown: usize,
    /// BF_TEST_FACE's pose: while one of its weights is over 0, the blend shows it in place of
    /// the controller's (a test hook's, not the game's)
    test: Option<[f32; VISEMES]>,
}

impl Default for Face {
    /// A character with no face: nothing to show.
    fn default() -> Self {
        Face { weights: [0.0; VISEMES], output: vec![], since_blink: FIRST_BLINK_CLOCK, blinked: false, blinks: 0, clock: 0.0, sound: None, tracks: None,
               start: 0.0, left: 0.0, speaking: false, target: BASE, shown: BASE, test: None }
    }
}

impl Face {
    /// FUN_00143380: a controller for a model with face table `table`: every weight 0, the base
    /// pose as the output (the constructor applies it with FUN_00143820), the blink due at once,
    /// the expression BASE.
    pub fn new(table: &FaceTable) -> Face {
        Face { output: table.base.clone(), ..Face::default() }
    }

    /// Whether it has a face to drive: a model without a face table (`Face::default`) has no
    /// controller in the game (FUN_00112ae0 makes none: its lines play on its voice alone).
    fn has_table(&self) -> bool {
        !self.output.is_empty()
    }

    /// FUN_001435d0, the controller's part: it stops what it was saying (FUN_00143570), then, if
    /// the sound library has `sound` (FUN_0017beb0: else nothing more, and false), keeps the
    /// entry's viseme tracks (+4 = the entry's +0x1c), speaks (+0x54) for the sound's length (+0x50:
    /// the sound system's vtable +0x64, `SoundBank::length`) and notes the clock (+0x44 = +0x40).
    /// `ago` is how long before now, in its clock, the line began: the frame's dt for a speak that
    /// comes after this frame's tick but stands for one before it (the chatter's: the game's
    /// manager update, 0xd9ac7, runs before the objects' ticks, 0xd9d0b), so its start is that much
    /// earlier and its time left that much shorter, as that tick would have left them; 0 for one
    /// that comes after the tick in the game too (a dialog-trigger's, from the triggers' loop,
    /// 0xd9d20). The voice (the audio) is the caller's.
    pub fn speak(&mut self, sounds: &SoundBank, sound: u32, ago: f32) -> bool {
        self.stop();
        if sounds.params(sound).is_none() {
            return false;
        }
        self.say(sound, sounds.visemes(sound), sounds.length(sound).unwrap_or(0.0), ago);
        true
    }

    /// The line `speak` starts, once stopped: `sound` with its `tracks`, `length` s long, begun
    /// `ago` s before now in the clock (its start and its time left both that much back).
    fn say(&mut self, sound: u32, tracks: Option<Arc<Visemes>>, length: f32, ago: f32) {
        self.sound = Some(sound);
        self.tracks = tracks;
        self.speaking = true;
        self.left = length - ago;
        self.start = self.clock - ago;
    }

    /// FUN_00143570, the controller's part: the line dropped (+0 the voice, +4 the tracks, +0x50,
    /// +0x54 all 0) and the 15 mouth weights zeroed; the expressions' are left to their fade. The
    /// face itself isn't written: it keeps the last pose until something shows again.
    pub fn stop(&mut self) {
        self.sound = None;
        self.tracks = None;
        self.left = 0.0;
        self.speaking = false;
        self.weights[..MOUTH_VISEMES].fill(0.0);
    }

    /// The line it is saying, while it speaks (+0x54 and +0x50 over 0): the sound, the time since
    /// it began (+0x40 - +0x44) and its time left.
    fn talking(&self) -> Option<(u32, f32, f32)> {
        self.sound.filter(|_| self.speaking && self.left > 0.0).map(|s| (s, self.clock - self.start, self.left))
    }

    /// FUN_001438d0, the expression fade: with the shown expression the target, its weight rises
    /// FADE_RATE a second to 1 (BASE has none to raise); otherwise the shown one's falls FADE_RATE
    /// a second to 0, and there the target is shown. True while the shown one isn't BASE.
    fn fade(&mut self, dt: f32) -> bool {
        if self.shown == self.target {
            if self.target != BASE {
                let w = &mut self.weights[MOUTH_VISEMES + self.target];
                *w = (*w + FADE_RATE * dt).min(1.0);
            }
        } else {
            let w = &mut self.weights[MOUTH_VISEMES + self.shown];
            *w = (*w - FADE_RATE * dt).max(0.0);
            if *w == 0.0 {
                self.shown = self.target;
            }
        }
        self.shown != BASE
    }

    /// FUN_001439b0, the blend: speaking is +0x54 with +0x50 over 0; the expression fades (`fade`)
    /// and says whether one shows. With neither, the line is stopped (0x143cf3: FUN_00143570) and
    /// nothing written; speaking a sound without tracks and no expression, nothing either. Else the
    /// output starts from the base pose, a speaking line's tracks write their weights at the time
    /// since it began (FUN_0017aa30: `Visemes::write`), and the visemes are blended in order by the
    /// weights (`FaceTable::blend`). A test pose (BF_TEST_FACE) with a weight up is blended in
    /// their place.
    fn blend(&mut self, table: &FaceTable, dt: f32) {
        let speaking = self.speaking && self.left > 0.0;
        let expression = self.fade(dt);
        if let Some(w) = self.test.filter(|w| w.iter().any(|&x| x > 0.0)) {
            table.blend(&w, &mut self.output);
            return;
        }
        if !expression {
            if !speaking {
                self.stop();
                return;
            }
            if self.tracks.is_none() {
                return;
            }
        }
        if speaking {
            if let Some(tracks) = &self.tracks {
                tracks.write(self.clock - self.start, &mut self.weights);
            }
        }
        table.blend(&self.weights, &mut self.output);
    }

    /// One tick (FUN_001437d0; `dt` 0 does nothing, as FUN_00143d10's guard): the clock goes on
    /// (+0x40), a line's time left down (+0x50; at 0 or under it no longer speaks, 0x1437ee-
    /// 0x1437f9), then the blend (FUN_001439b0, `blend`) and the blink's clock (FUN_00143d10).
    /// Returns whether a blink started.
    pub fn tick(&mut self, table: &FaceTable, dt: f32) -> bool {
        if dt <= 0.0 {
            return false;
        }
        self.clock += dt;
        if self.speaking {
            self.left -= dt;
            if self.left <= 0.0 {
                self.speaking = false;
            }
        }
        self.blend(table, dt);
        self.since_blink += dt;
        let Some(blink) = &table.blink else { return false };
        // (the clip must be over, and then only the interval counts: see the module notes for
        // the random test that sits between them in the game)
        if blink.duration < self.since_blink && self.since_blink > table.blink_interval {
            self.since_blink = 0.0;
            self.blinked = true;
            self.blinks += 1;
            return true;
        }
        false
    }

    /// Put the face on a character's pose (FUN_00143820 and the blink player's nodes): the face
    /// bones take the output, the blink's lids their place in the clip, eased in from where the
    /// last blink left them (the first from the model's own).
    pub fn apply(&self, table: &FaceTable, game: &Game, pose: &mut [(Quat, Vec3)]) {
        table.write(&self.output, pose);
        if self.blinked {
            table.write_blink(game, self.since_blink, self.blinks > 1, pose);
        }
    }
}

/// The weights over 0, for BF_FACE_LOG (`if=0.179 wet=0.160`): by name, three decimals as the
/// takes' face logs have them.
fn weights_text(weights: &[f32; VISEMES]) -> String {
    let named: Vec<String> = weights.iter().zip(face::VISEME_NAMES).filter(|(w, _)| **w > 0.0).map(|(w, n)| format!("{n}={w:.3}")).collect();
    named.join(" ")
}

/// The face hook of every speech path with a speaker (#325): `u`'s face controller says `sound`
/// (`Face::speak`: FUN_001435d0's part on the controller), the line begun `ago` s of its clock
/// before now (see `Face::speak`); `by` names the path for BF_FACE_LOG. The chatter calls it as a
/// sound starts on a character's voice (play_chatter.rs `Chatter::voice_start`), the dialog-trigger
/// as its character says its line (play_sound_triggers.rs `speakers`). Nothing for a character
/// without a face table (it has no controller).
pub fn speak(u: &mut Player, sounds: &SoundBank, sound: u32, ago: f32, by: &str) {
    if !u.facial.has_table() {
        return;
    }
    let said = u.facial.speak(sounds, sound, ago);
    if std::env::var("BF_FACE_LOG").is_ok() {
        let length = sounds.length(sound).unwrap_or(0.0);
        // (a library entry whose file the map's banks lack has no length: it ends at once)
        let short = if length > 0.0 { "" } else { ", no length (its file isn't in the loaded banks): it ends at once" };
        let what = match (&u.facial.tracks, said) {
            (_, false) => "not in the sound library: only stopped".to_string(),
            (Some(t), _) => format!("{length:.3} s, {} viseme track{}{short}", t.tracks.len(), if t.tracks.len() == 1 { "" } else { "s" }),
            (None, _) => format!("{length:.3} s, no speech block: the face isn't moved{short}"),
        };
        println!("t={:.2} {}: says h_{sound:08x} ({what}; {by})", u.sim_time, unit_name(u));
    }
}

/// The face hook of a line's stop (#325): `u`'s face controller stops what it says (`Face::stop`,
/// FUN_00143570): a chatter cut-off (0x13fbde), a dialog-trigger's STOP (FUN_001267f0 at
/// 0x1267ff). `by` names it for BF_FACE_LOG.
pub fn stop(u: &mut Player, by: &str) {
    if let Some((s, t, _)) = u.facial.talking().filter(|_| std::env::var("BF_FACE_LOG").is_ok()) {
        println!("t={:.2} {}: stops h_{s:08x} at {t:.3} s ({by})", u.sim_time, unit_name(u));
    }
    u.facial.stop();
}

/// BF_FACE_COST: whether it is on (read once: the steps run each frame for every character).
fn costing() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("BF_FACE_COST").is_ok())
}

/// BF_FACE_COST's tally since its last report: the nanoseconds the faces' steps took, how many
/// steps, and how many of them with a line speaking.
static COST_NS: AtomicU64 = AtomicU64::new(0);
/// The steps counted.
static COST_STEPS: AtomicU32 = AtomicU32::new(0);
/// The steps of a face that was speaking.
static COST_SPEAKING: AtomicU32 = AtomicU32::new(0);

/// A character's face for this frame, on its final body pose (`step_player` calls it after the
/// aim): ticked by `dt`, the blink logged under BF_FACE_LOG. A model without a face is left as
/// it is, and so is a character **not in the world yet** (`Player::absent`: a squad member whose
/// spawn-trigger hasn't made its copy; the controlled one's step runs all the same, for the game's
/// clock). The game makes a controller with the copy (0x112b37 / 0x12f848) and ticks it from the
/// copy's first update, with its blink clock at 10 s, so the first blink is the arrival and the
/// four-second cycle runs from there: **a character's blink phase is when it was made**. The
/// squad's four are copies made 0.5 s apart in e34's opening, and the takes' controllers show it
/// (todo/255 take01-03, measured: each controller's clock +0x40 against the session's clock puts
/// its start at 9.16 s for Flint, 9.63 Brutus, 10.14 Hawk and 10.64 Tex, and their blinks are
/// 1.48, 1.01 and 0.51 s ahead of Tex's in the four-second cycle). A `dt` of 0 draws the face
/// without a tick (a knock-down's get-up steps twice in a frame, `step_player`'s `tick_face`).
/// BF_FACE_LOG: its blinks, and while it speaks its weights each tick, and the line's end;
/// BF_FACE_COST: the step's time.
pub fn step(p: &mut Player, l: &Loaded, game: &Game, dt: f32, pose: &mut [(Quat, Vec3)]) {
    let Some(table) = &l.model.face else { return };
    if p.absent {
        return;
    }
    let started = costing().then(std::time::Instant::now);
    run(p, table, dt, "");
    p.facial.apply(table, game, pose);
    count(started, p);
}

/// A knocked-down living character's face for this frame (`update_player`, which gives it no step
/// while the ragdoll has its body): its controller ticks all the same, for the game ticks every
/// living character's controller from its update (FUN_0011caa0 at 0x11dc87; 0x11dc7b tests only
/// the dead bit), so a line said as the hit knocks it down runs on its own time; nothing is drawn
/// (the ragdoll writes the bones), and once the get-up draws the face again it shows where the
/// line is by then. (The game's knock-down is a clip, which leaves the face bones to the
/// controller, so its mouth would be seen moving: the demo's ragdoll holds the face it fell with.)
pub fn tick(p: &mut Player, dt: f32) {
    let Some(l) = p.loaded.take() else { return };
    if let Some(table) = l.model.face.as_ref().filter(|_| !p.absent) {
        let started = costing().then(std::time::Instant::now);
        run(p, table, dt, " (knocked down, not drawn)");
        count(started, p);
    }
    p.loaded = Some(l);
}

/// The controller's tick with BF_FACE_LOG's lines (each blink; while a line speaks its weights;
/// the line's end), `note` after the character's name.
fn run(p: &mut Player, table: &FaceTable, dt: f32, note: &str) {
    let was = p.facial.talking();
    let blinked = p.facial.tick(table, dt);
    if dt <= 0.0 || std::env::var("BF_FACE_LOG").is_err() {
        return;
    }
    let name = format!("{}{note}", unit_name(p));
    if blinked {
        println!("t={:.2} {name}: blink {}", p.sim_time, p.facial.blinks);
    }
    match (p.facial.talking(), was) {
        (Some((s, t, left)), _) => println!("t={:.2} {name}: h_{s:08x} at {t:.4} s, {left:.3} s left: {}", p.sim_time, weights_text(&p.facial.weights)),
        (None, Some((s, t, _))) => println!("t={:.2} {name}: h_{s:08x} said ({t:.3} s; the face keeps its last pose)", p.sim_time),
        _ => {}
    }
}

/// BF_FACE_COST: a step's time since `started`, and whether its face spoke, into the tally.
fn count(started: Option<std::time::Instant>, p: &Player) {
    let Some(t0) = started else { return };
    COST_NS.fetch_add(t0.elapsed().as_nanos() as u64, Ordering::Relaxed);
    COST_STEPS.fetch_add(1, Ordering::Relaxed);
    COST_SPEAKING.fetch_add(p.facial.talking().is_some() as u32, Ordering::Relaxed);
}

/// BF_FACE_COST: each second of the controlled character's clock, the faces' steps' time (the
/// controller's tick and the pose written: `step`; a knocked-down one's tick, `tick`) over the
/// frames since the last report, the steps a frame and how many spoke.
fn report_cost(player: Res<Player>, mut since: Local<(f32, u32)>) {
    if !costing() {
        return;
    }
    let (last, frames) = &mut *since;
    *frames += 1;
    if player.sim_time - *last < 1.0 {
        return;
    }
    let ns = COST_NS.swap(0, Ordering::Relaxed);
    let steps = COST_STEPS.swap(0, Ordering::Relaxed);
    let speaking = COST_SPEAKING.swap(0, Ordering::Relaxed);
    println!("t={:.2} face cost: {:.1} us a frame over {} frames ({:.1} face steps a frame, {:.1} speaking; {:.2} us a step)", player.sim_time,
             ns as f64 / 1000.0 / *frames as f64, *frames, steps as f64 / *frames as f64, speaking as f64 / *frames as f64,
             if steps > 0 { ns as f64 / 1000.0 / steps as f64 } else { 0.0 });
    *last = player.sim_time;
    *frames = 0;
}

/// A character spawned (`spawn_unit`): its controller made afresh for its model (the game makes
/// one with the character: 0x112ae0, the character vtable's +0x9c, at 0x112b37, and FUN_0012f440
/// for a spawned copy at 0x12f848). BF_FACE_LOG prints what its table holds, under its `name`.
pub fn spawned(p: &mut Player, name: &str, model: &Character, game: &Game) {
    p.facial = model.face.as_ref().map_or_else(Face::default, Face::new);
    if std::env::var("BF_FACE_LOG").is_ok() {
        println!("{}", describe(name, model));
    }
    if std::env::var("BF_FACE_CHECK").is_ok() {
        println!("{}", check(name, model, game));
    }
}

/// How far apart two rotations are, in degrees: the relative turn's angle by atan2, exact near 0
/// where acos is not (two equal quaternions measure 0.04 degrees apart by `angle_between`).
fn apart(a: Quat, b: Quat) -> f32 {
    let r = a * b.inverse();
    (2.0 * Vec3::new(r.x, r.y, r.z).length().atan2(r.w.abs())).to_degrees()
}

/// BF_FACE_CHECK: the table against the clips it was read from. For the base pose and then each
/// of the 22 visemes, the model's face bones as `Face` writes them (with that viseme's weight
/// set to 1 over its scale, so its pose shows whole) against `Character::pose` with the viseme's
/// own clip laid over the same body pose (the viewer's way, which holds the clip's first key), on
/// the bones the clip names: the largest turn of a rotation bone (degrees) and the largest move
/// of a translation bone (m). Both should be 0 to rounding; a viseme with no clip is skipped.
/// And the lids: how far the blink clip's first and last keys lie from the bind pose, which is
/// what they show until the first blink starts, and how far a first blink has taken them a frame
/// and two frames in (1/30, 2/30 s), eased in (`FaceTable::write_blink`) and bare.
pub fn check(name: &str, model: &Character, game: &Game) -> String {
    let Some(t) = &model.face else { return format!("face check: {name}: no face set") };
    let body = model.pose(game, 0, 0.0, None, 0.0);
    let (mut turn, mut off) = (0.0f32, 0.0f32);
    let mut worst = String::new();
    let mut compare = |what: &str, clip: usize, weights: [f32; VISEMES]| {
        let mut ours = body.clone();
        let mut out = vec![];
        t.blend(&weights, &mut out);
        t.write(&out, &mut ours);
        let reference = model.pose(game, 0, 0.0, Some(clip), 0.0);
        for &(bone, _) in &model.face_anims[clip].targets {
            let Some(b) = model.bones.iter().position(|&x| x == bone) else { continue };
            let d = apart(ours[b].0, reference[b].0);
            if d > turn {
                worst = format!(", worst {what} bone {bone:08x}");
            }
            turn = turn.max(d);
            off = off.max(ours[b].1.distance(reference[b].1));
        }
    };
    let mut skipped = 0;
    if let Some(clip) = model.face_anims.iter().position(|a| a.name == face::SC_BASE) {
        compare("base", clip, [0.0; VISEMES]);
    }
    for (i, v) in face::VISEME_NAMES.iter().enumerate() {
        let Some(clip) = model.face_anims.iter().position(|a| a.name == bf_viewer::bf::hash::h(&format!("Sc_{v}"))) else {
            skipped += 1;
            continue;
        };
        let mut weights = [0.0; VISEMES];
        weights[i] = 1.0 / t.visemes[i].scale;
        compare(v, clip, weights);
    }
    let (mut first, mut last) = (body.clone(), body.clone());
    t.write_blink_clip(game, 0.0, &mut first);
    t.write_blink_clip(game, f32::INFINITY, &mut last);
    let lids: Vec<String> = t.blink.iter().flat_map(|b| b.tracks.iter()).map(|&(bone, _)| {
        format!("{:08x} {:.1}/{:.1}", model.bones[bone], apart(body[bone].0, first[bone].0), apart(body[bone].0, last[bone].0))
    }).collect();
    // (a first blink's lids a frame and two into it, eased in from the bind pose, against the
    // bare clip's: the ease's lag, faces.md "The blink")
    let lid_at = |bone: usize, s: f32, eased: bool| {
        let mut p = body.clone();
        if eased {
            t.write_blink(game, s, false, &mut p);
        } else {
            t.write_blink_clip(game, s, &mut p);
        }
        apart(body[bone].0, p[bone].0)
    };
    let eased: Vec<String> = t.blink.iter().flat_map(|b| b.tracks.iter()).map(|&(bone, _)| {
        format!("{:08x} {:.1}/{:.1} {:.1}/{:.1}", model.bones[bone], lid_at(bone, 1.0 / 30.0, true), lid_at(bone, 1.0 / 30.0, false),
                lid_at(bone, 2.0 / 30.0, true), lid_at(bone, 2.0 / 30.0, false))
    }).collect();
    format!("face check: {name}: base and {} visemes against their clips: largest turn {turn:.4} deg, largest move {off:.6} m{worst} ({skipped} without a clip); lids' first/last key from the bind pose, degrees [{}]; a first blink at 1/30 and 2/30 s, eased/bare, degrees [{}]",
            face::VISEMES - skipped, lids.join(", "), eased.join(", "))
}

/// What a model's face table holds, in a line (BF_FACE_LOG, BF_DUMP_SOUNDS): the face bones by
/// kind, how many of `Sc_base`'s the model lacks, the blink's lids and what it leaves out, the
/// chance, and the visemes stretched.
pub fn describe(name: &str, model: &Character) -> String {
    let Some(t) = &model.face else { return format!("face: {name}: no face set") };
    let rotations = t.bones.iter().filter(|b| b.kind == Kind::Rotation).count();
    let blink = t.blink.as_ref().map_or("no blink clip".to_string(), |b| format!(
        "blink {} lids ({} of the clip's bones left out: face bones), {:.3} s every {} s",
        b.tracks.len(), b.skipped, b.duration, t.blink_interval));
    let scales: Vec<String> = t.visemes.iter().zip(face::VISEME_NAMES).filter(|(v, _)| v.scale != 1.0).map(|(v, n)| format!("{n} x{}", v.scale)).collect();
    format!("face: {name}: {} face bones ({rotations} rotation, {} translation, {} missing), {blink}; chance {} (never passes); stretched [{}]",
            t.bones.len(), t.bones.len() - rotations, t.missing, t.blink_chance, scales.join(", "))
}

/// One `BF_TEST_FACE` entry: at `at` seconds, on `who`, viseme `viseme` to `weight`.
struct Hook {
    at: f32,
    who: String,
    viseme: usize,
    weight: f32,
}

/// `BF_TEST_FACE`'s entries (`<s>,<who>,<viseme>,<weight>`, separated by `;`); an entry that
/// doesn't parse (a viseme that isn't one of the 22) is skipped with a warning.
fn hooks() -> Vec<Hook> {
    let Ok(v) = std::env::var("BF_TEST_FACE") else { return vec![] };
    v.split(';').filter(|e| !e.trim().is_empty()).filter_map(|e| {
        let f: Vec<&str> = e.split(',').map(str::trim).collect();
        let hook = (|| Some(Hook { at: f.first()?.parse().ok()?, who: f.get(1)?.to_lowercase(), viseme: face::viseme_index(f.get(2)?)?, weight: f.get(3)?.parse().ok()? }))();
        if hook.is_none() {
            warn!("BF_TEST_FACE: {e:?} isn't <s>,<who>,<viseme>,<weight>; skipped");
        }
        hook
    }).collect()
}

/// Whether `who` names `u`: `all`, `me` (`controlled`: it is the character the player holds), its
/// squad name ("tex") or its slot (0-3). The face hooks' and BF_TEST_SPEAK's (play_chatter.rs).
pub(super) fn is_whom(u: &Player, controlled: bool, who: &str) -> bool {
    match who {
        "all" => true,
        "me" => controlled,
        w => !u.own_type() && (u.data_name() == w || w.parse::<usize>().is_ok_and(|n| n == u.character)),
    }
}

/// BF_TEST_FACE: each entry sets its weight in the character's test pose (`Face::test`) once its
/// time comes (the controlled character's clock, `Player::sim_time`).
fn test_hooks(mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut pending: Local<Option<Vec<Hook>>>) {
    let now = player.sim_time;
    let list = pending.get_or_insert_with(hooks);
    if list.is_empty() {
        return;
    }
    let (due, rest): (Vec<Hook>, Vec<Hook>) = std::mem::take(list).into_iter().partition(|h| h.at <= now);
    *list = rest;
    for h in due {
        for (i, u) in std::iter::once(&mut *player).chain(squad.0.iter_mut()).enumerate() {
            if is_whom(u, i == 0, &h.who) {
                u.facial.test.get_or_insert([0.0; VISEMES])[h.viseme] = h.weight;
                if std::env::var("BF_FACE_LOG").is_ok() {
                    println!("t={now:.2} {}: {} weight {}", unit_name(u), face::VISEME_NAMES[h.viseme], h.weight);
                }
            }
        }
    }
}

/// One `BF_TEST_FACE_CAM` entry: at `at` seconds, the free camera goes `distance` m in front of
/// `who`'s face, `rise` m higher than `FACE_OVER_HEAD` over the head bone, looking along its
/// facing back at it.
struct CamHook {
    at: f32,
    who: String,
    distance: f32,
    rise: f32,
}

/// `BF_TEST_FACE_CAM`'s entries (`<s>,<who>[,<m>[,<rise>]]`, separated by `;`; 0.8 m and 0
/// without them).
fn cam_hooks() -> Vec<CamHook> {
    let Ok(v) = std::env::var("BF_TEST_FACE_CAM") else { return vec![] };
    v.split(';').filter(|e| !e.trim().is_empty()).filter_map(|e| {
        let f: Vec<&str> = e.split(',').map(str::trim).collect();
        let hook = (|| Some(CamHook { at: f.first()?.parse().ok()?, who: f.get(1)?.to_lowercase(), distance: f.get(2).map_or(Some(0.8), |d| d.parse().ok())?,
                                       rise: f.get(3).map_or(Some(0.0), |d| d.parse().ok())? }))();
        if hook.is_none() {
            warn!("BF_TEST_FACE_CAM: {e:?} isn't <s>,<who>[,<m>[,<rise>]]; skipped");
        }
        hook
    }).collect()
}

/// The head bone's frame in the world (its matrix in the pose last stepped, under the character's
/// own): None without a model, a head bone or a pose.
fn head_frame(u: &Player) -> Option<Mat4> {
    let l = u.loaded.as_ref()?;
    let head = l.model.bones.iter().position(|&b| b == face::HEAD_BONE)?;
    let root = Transform::from_translation(u.position + Vec3::Y * (u.height + u.lift_now)).with_rotation(Quat::from_rotation_y(u.yaw)).compute_matrix();
    Some(root * *u.last_world.get(head)?)
}

/// Where a `BF_TEST_FACE_CAM` camera sits, in the head bone's frame of the character it frames:
/// the camera follows the head, so the framing holds while the idle turns it.
struct Following {
    who: String,
    /// the camera's place and the way it looks, in the head bone's frame
    at: Vec3,
    look: Vec3,
}

/// BF_TEST_FACE_CAM: the free camera put in front of a face and kept there as the head moves, for
/// the close-ups the lip-sync and the blink are checked on (the characters' heads stand at
/// different heights; Brutus's is low and forward, and their idles turn them). Runs after the
/// characters step, so it has the pose they just made. Its start: `FACE_OVER_HEAD` over the
/// head bone, `distance` m ahead of the character along its facing, looking back.
fn face_cam(player: Res<Player>, squad: Res<Squad>, mut tools: ResMut<testtools::Tools>, mut pending: Local<Option<Vec<CamHook>>>,
            mut following: Local<Option<Following>>) {
    let now = player.sim_time;
    let find = |who: &str| std::iter::once(&*player).chain(squad.0.iter()).enumerate().find(|(i, u)| is_whom(u, *i == 0, who)).map(|(_, u)| u);
    // (a hook whose character has no pose yet waits for the next frame)
    pending.get_or_insert_with(cam_hooks).retain(|h| {
        if h.at > now {
            return true;
        }
        let framed = find(&h.who).and_then(|u| {
            let m = head_frame(u)?;
            let ahead = Quat::from_rotation_y(u.yaw) * Vec3::NEG_Z;
            let at = m.transform_point3(Vec3::ZERO) + Vec3::Y * (face::FACE_OVER_HEAD + h.rise) + ahead * h.distance;
            let to_head = m.inverse();
            Some(Following { who: h.who.clone(), at: to_head.transform_point3(at), look: to_head.transform_vector3(-ahead) })
        });
        let done = framed.is_some();
        if let Some(f) = framed {
            *following = Some(f);
        }
        !done
    });
    if let Some(f) = following.as_ref() {
        if let Some(m) = find(&f.who).and_then(head_frame) {
            let look = m.transform_vector3(f.look).normalize();
            // (the free camera's yaw is atan2(-x, -z) of its view, its pitch the view's rise)
            tools.freecam = Some(testtools::FreeCam { pos: m.transform_point3(f.at), yaw: (-look.x).atan2(-look.z), pitch: look.y.clamp(-1.0, 1.0).asin() });
        }
    }
}

/// The faces' systems: the test hook, before the characters step (so a weight set this frame is
/// blended this frame), and the camera hook and BF_FACE_COST's report after.
pub fn plugin(app: &mut App) {
    app.add_systems(Update, test_hooks.before(update_player).run_if(in_state(AppState::Playing)))
        .add_systems(Update, (face_cam, report_cost).after(update_player).run_if(in_state(AppState::Playing)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bf_viewer::bf::audio::VisemeTrack;
    use bf_viewer::bf::face::{Blink, FaceBone, Viseme};

    /// A one-bone table by hand (a rotation bone, base 0.2 rad about x, `bump` 0.6) with a blink
    /// clip of 0.333 s that moves nothing, every `interval` s.
    fn table(interval: f32) -> FaceTable {
        let rot = |a: f32| { let q = Quat::from_rotation_x(a); [q.x, q.y, q.z, q.w] };
        let mut visemes: Vec<Viseme> = (0..VISEMES).map(|_| Viseme { pose: vec![rot(0.2)], scale: 1.0 }).collect();
        visemes[0].pose = vec![rot(0.6)];
        FaceTable {
            bones: vec![FaceBone { name: 1, bone: 0, kind: Kind::Rotation }],
            base: vec![rot(0.2)],
            visemes,
            blink: Some(Blink { duration: 0.333, tracks: vec![], skipped: 0 }),
            blink_interval: interval,
            blink_chance: 0.05,
            missing: 0,
        }
    }

    /// The times (s) of the blinks a face makes in `seconds` s of ticks of `dt`.
    fn blinks(t: &FaceTable, dt: f32, seconds: f32) -> Vec<f32> {
        let mut f = Face::new(t);
        (0..(seconds / dt) as usize).filter_map(|n| f.tick(t, dt).then_some(n as f32 * dt)).collect()
    }

    /// FUN_00143d10 and FUN_00143380: the blink clock starts at 10 s, so the first blink is at the
    /// first tick; then it zeroes and the next comes once the time since passes the type's
    /// interval, 4 s for the squad (3 for the Seers): the interval and at most a frame, whatever
    /// the frame rate.
    #[test]
    fn a_face_blinks_at_once_and_then_every_interval() {
        for (interval, dt) in [(4.0, 1.0 / 30.0), (4.0, 1.0 / 60.0), (4.0, 1.0 / 15.0), (3.0, 1.0 / 30.0), (5.0, 1.0 / 30.0)] {
            let at = blinks(&table(interval), dt, 31.0);
            assert_eq!(at[0], 0.0, "the first at the first tick");
            assert!(at.len() >= 5, "{at:?}");
            for w in at.windows(2) {
                let gap = w[1] - w[0];
                assert!(gap > interval - 1e-3 && gap < interval + dt * 1.5, "interval {interval}, dt {dt}: a gap of {gap}");
            }
        }
    }

    /// The phase is the first tick (todo/255 take01-03: the squad's four, whose controllers started
    /// 0.5 s apart, blink 0.5 s apart in the same four-second cycle): two faces made alike and first
    /// ticked 15 ticks (0.5 s) apart blink 15 ticks apart, at every cycle.
    #[test]
    fn a_faces_blinks_follow_its_first_tick() {
        let t = table(4.0);
        let dt = 1.0 / 30.0;
        let (mut a, mut b) = (Face::new(&t), Face::new(&t));
        let (mut at, mut bt) = (vec![], vec![]);
        for n in 0..600usize {
            if a.tick(&t, dt) {
                at.push(n);
            }
            if n >= 15 && b.tick(&t, dt) {
                bt.push(n);
            }
        }
        assert_eq!((at[0], bt[0]), (0, 15), "each blinks at its first tick");
        assert!(at.len() >= 4 && at.len() == bt.len(), "{at:?} {bt:?}");
        for (x, y) in at.iter().zip(&bt) {
            assert_eq!(y - x, 15);
        }
    }

    /// A tick of no time changes nothing (the game's guard: `dt` 0 skips the clock), a type with
    /// no blink clip never blinks, and the blink's clock needs the clip to be over (0.333 s).
    #[test]
    fn a_stopped_clock_and_a_type_without_a_blink_do_not_blink() {
        let t = table(4.0);
        let mut f = Face::new(&t);
        assert!(!f.tick(&t, 0.0));
        assert!(f.tick(&t, 0.03));
        // (a blink at 0.03 s: the next needs more than 4 s)
        assert!(!f.tick(&t, 3.9));
        assert!(f.tick(&t, 0.2));
        let mut none = table(4.0);
        none.blink = None;
        assert!(blinks(&none, 1.0 / 30.0, 20.0).is_empty());
    }

    /// The jaw's turn (rad) a face shows on the one-bone table.
    fn turn(t: &FaceTable, f: &Face) -> f32 {
        let mut pose = [(Quat::IDENTITY, Vec3::ZERO)];
        t.write(&f.output, &mut pose);
        pose[0].0.to_axis_angle().1
    }

    /// A `bump` track rising from 0 at 0.1 s to 100 % at 0.3 s and back to 0 at 0.5 s.
    fn bump_line() -> Arc<Visemes> {
        Arc::new(Visemes { tracks: vec![VisemeTrack::new(0, &[(0.1, 0.0), (0.3, 100.0), (0.5, 0.0)])] })
    }

    /// FUN_001439b0 at rest: nothing speaking and no expression, so nothing is written (the face
    /// stays the base pose it was made with) and the mouth's weights are zeroed each tick
    /// (FUN_00143570); a test pose (BF_TEST_FACE) with a weight up shows in their place, and once
    /// its weights are 0 again the face keeps the last pose written. A face with no table has
    /// nothing to show.
    #[test]
    fn the_face_is_the_base_until_something_shows_and_keeps_the_last_pose() {
        let t = table(4.0);
        let mut f = Face::new(&t);
        f.weights[0] = 1.0;
        f.tick(&t, 0.03);
        assert!((turn(&t, &f) - 0.2).abs() < 1e-4);
        assert_eq!(f.weights[0], 0.0, "the idle stop zeroes the mouth");
        f.test = Some([0.0; VISEMES]);
        f.test.as_mut().unwrap()[0] = 1.0;
        f.tick(&t, 0.03);
        assert!((turn(&t, &f) - 0.6).abs() < 1e-4);
        f.test.as_mut().unwrap()[0] = 0.0;
        f.tick(&t, 0.03);
        assert!((turn(&t, &f) - 0.6).abs() < 1e-4);
        assert!(Face::default().output.is_empty() && !Face::default().has_table());
    }

    /// FUN_001435d0 and FUN_001437d0 with FUN_001439b0: a line's tracks drive the mouth at the
    /// time since it began, the clock going on by each tick (a speak before the tick: the first
    /// tick reads it at dt). It speaks for its length: the tick that takes the time left to 0 or
    /// under ends it, the mouth's weights are zeroed (the idle stop) and the face keeps the last
    /// pose blended.
    #[test]
    fn a_line_drives_the_mouth_by_its_tracks_for_its_length() {
        let t = table(4.0);
        let mut f = Face::new(&t);
        let dt = 0.1;
        f.tick(&t, dt);
        f.say(0xEC50_FB88, Some(bump_line()), 0.45, 0.0);
        let mut seen = vec![];
        for _ in 0..6 {
            f.tick(&t, dt);
            seen.push((f.weights[0], f.talking().is_some()));
            // (a tick of no time, the get-up's second step of a frame, moves nothing on)
            let was = f.talking();
            f.tick(&t, 0.0);
            assert_eq!(f.talking(), was);
        }
        // read at 0.1, 0.2, 0.3, 0.4 s; the fifth tick takes 0.45 s left under 0
        let want = [(0.0, true), (0.5, true), (1.0, true), (0.5, true), (0.0, false), (0.0, false)];
        for (k, ((w, on), (ww, won))) in seen.iter().zip(want).enumerate() {
            assert!((w - ww).abs() < 1e-5 && *on == won, "tick {k}: {seen:?}");
        }
        // the last blend was at 0.4 s, bump at 0.5: the jaw halfway (0.4 rad) and left there
        assert!((turn(&t, &f) - 0.4).abs() < 1e-3, "{}", turn(&t, &f));
        assert!(f.sound.is_none() && f.tracks.is_none());
    }

    /// The chatter's speak comes after the frame's tick in the demo but before it in the game, so
    /// it backs its start and its time left up by the frame (`ago`): its next tick reads the line
    /// at two frames, as the game's second frame does, and its last at the same time as the
    /// game's (a 0.45 s line read last at 0.4 s, its time left then 0.05 s).
    #[test]
    fn a_line_begun_a_frame_ago_is_read_a_frame_on() {
        let t = table(4.0);
        let mut f = Face::new(&t);
        let dt = 0.1;
        f.tick(&t, dt);
        f.say(1, Some(bump_line()), 0.45, dt);
        f.tick(&t, dt);
        assert!((f.weights[0] - 0.5).abs() < 1e-5, "read at 0.2 s: {}", f.weights[0]);
        let (_, at, left) = f.talking().unwrap();
        assert!((at - 0.2).abs() < 1e-5 && (left - 0.25).abs() < 1e-5, "{at} {left}");
        f.tick(&t, dt);
        f.tick(&t, dt);
        let (_, at, left) = f.talking().unwrap();
        assert!((at - 0.4).abs() < 1e-5 && (left - 0.05).abs() < 1e-5, "{at} {left}");
        f.tick(&t, dt);
        assert!(f.talking().is_none());
    }

    /// A sound with no speech block speaks (its time runs) but moves nothing: FUN_001439b0
    /// writes nothing while it speaks without tracks and shows no expression. A stop
    /// (FUN_00143570) ends a line at once, the mouth's weights zeroed, the expressions' kept.
    #[test]
    fn a_line_without_tracks_moves_nothing_and_a_stop_ends_one() {
        let t = table(4.0);
        let mut f = Face::new(&t);
        f.say(2, None, 1.0, 0.0);
        f.weights[0] = 1.0;
        f.tick(&t, 0.1);
        assert!(f.talking().is_some());
        assert!((turn(&t, &f) - 0.2).abs() < 1e-4, "nothing written");
        assert_eq!(f.weights[0], 1.0, "and nothing stopped while it speaks");
        f.say(3, Some(bump_line()), 1.0, 0.0);
        f.tick(&t, 0.2);
        assert!(f.weights[0] > 0.0);
        f.weights[16] = 0.5;
        f.stop();
        assert!(f.talking().is_none());
        assert_eq!(f.weights[0], 0.0);
        assert_eq!(f.weights[16], 0.5);
    }

    /// FUN_001438d0: an expression other than BASE fades in at 2 a second to 1, and the blend
    /// writes while it shows; a change of target fades the shown one out first (to 0 in 0.5 s),
    /// then shows the target; BASE itself has no weight.
    #[test]
    fn an_expression_fades_at_two_a_second() {
        let t = table(4.0);
        let mut f = Face::new(&t);
        f.target = 0;
        f.tick(&t, 0.1);
        // (the shown one, BASE, has no weight: it is the target at once)
        assert_eq!(f.shown, 0);
        f.tick(&t, 0.1);
        assert!((f.weights[15] - 0.2).abs() < 1e-6, "{}", f.weights[15]);
        for _ in 0..10 {
            f.tick(&t, 0.1);
        }
        assert_eq!(f.weights[15], 1.0);
        f.target = BASE;
        f.tick(&t, 0.25);
        assert!((f.weights[15] - 0.5).abs() < 1e-6 && f.shown == 0);
        f.tick(&t, 0.25);
        assert_eq!((f.weights[15], f.shown), (0.0, BASE));
    }

    /// BF_TEST_FACE names a viseme as the game does and a character by its squad name or slot.
    #[test]
    fn the_test_hook_names_its_characters() {
        let tex = Player::new(3);
        assert!(is_whom(&tex, false, "tex") && is_whom(&tex, false, "3") && is_whom(&tex, false, "all"));
        assert!(!is_whom(&tex, false, "flint") && !is_whom(&tex, false, "me") && is_whom(&tex, true, "me"));
        assert_eq!(face::viseme_index("Roar"), Some(10));
    }
}

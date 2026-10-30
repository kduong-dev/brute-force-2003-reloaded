//! Faces (issue #255): what a character's face shows in play, as default.xbe's face controller
//! (char+0x5b8, FUN_00143380) drives it. The game plays no face clip while a character talks: the
//! controller writes the face bones' joints every frame from the 22 viseme weights over the base
//! pose (`Sc_base`), and only the blink is a clip. This first slice has the controller and the
//! blink; the weights stay 0 until the lip-sync and the expressions drive them.
//!
//!  - **The table** (`bf/face.rs`, FUN_00142cb0 and FUN_00142df0): per model, the face bones from
//!    `Sc_base`, its pose, and the 22 `Sc_<viseme>` poses with the type's scales.
//!  - **The face bones** belong to the controller (FUN_00143380 flags each node, +0x44 bit 5,
//!    which the animation binder FUN_00234190 skips): a body clip never moves them, so the demo
//!    lays no face clip over its poses either (it did, `Sc_bump`, the b/m/p mouth, before this),
//!    and `step` puts the controller's pose on them after every layer, overlay and aim turn of
//!    the body.
//!  - **At rest** the face shows the base pose, written at the controller's construction
//!    (FUN_00143380's last call) and again by every blend while a weight is up; with every weight
//!    0 the game writes nothing, so the bones keep the last pose written (FUN_001439b0 leaves
//!    them when nothing shows): the same here (`Face::output`).
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
//!    wrote.
//!  - **The phase is when the character was made**: the first blink is the first tick and the
//!    cycle runs on from it, so characters made apart blink apart (the squad's copies, 0.5 s
//!    apart in e34, as the takes' controllers are) and ones made together blink together. A
//!    character not in the world yet (`Player::absent`) isn't ticked (`step`).
//!  - **Alive only**: the game ticks the controller from the character's update while it lives
//!    (FUN_0011caa0, 0x11dc7b), so `step` runs in `step_player`: a dead character's face keeps
//!    the pose it died with (the ragdoll starts from `last_world`, which has it), a cutscene's
//!    characters blink like the rest. A knocked-down (alive) character's isn't ticked here: the
//!    ragdoll holds the pose from the knock.
//!
//! **The weights are the hook** the next slices drive: `Face::weights`, 22 floats in the VE_
//! order (`face::VISEME_NAMES`: 0-14 the mouth shapes, 15-21 the expressions), each 0 to 1. The
//! lip-sync will write the sound's tracks into them (FUN_0017aa30), the expression fade
//! (FUN_001438d0) the last seven.
//!
//! Test hooks:
//!  - `BF_FACE_LOG=1` prints each face made (its bones, its blink, its stretch list) and each
//!    blink as it starts.
//!  - `BF_FACE_CHECK=1` prints, for each face made, how closely its table reproduces the clips
//!    it was read from (`check`).
//!  - `BF_TEST_FACE=<s>,<who>,<viseme>,<weight>[;...]`: at `s` seconds set a viseme's weight (0
//!    to 1) on `who`: `me` (the controlled character), a squad name or 0-3 (`tex`, `2`), or
//!    `all`. It stays until another entry changes it. The mouth at a weight of 1 shows the
//!    viseme's clip, which bf_viewer's Q / E shows too (a check of the blend, `Face::tick`).
//!    With every weight back at 0 the face keeps the last pose blended (the game's idle
//!    controller writes nothing): set `base` to 1 to go back to the rest face.
//!  - `BF_TEST_FACE_CAM=<s>,<who>[,<m>[,<rise>]][;...]`: at `s` seconds the free camera goes `m`
//!    metres (0.8) in front of `who`'s face, `rise` m (0) higher (negative: lower), looking at it
//!    (`who` as above, the first it names), and stays there as the head moves: the close-ups for
//!    a captured face, whatever height the head is and however the idle turns it. Brutus's head
//!    hangs forward and low: `1.1,-0.3` frames it.

use bf_viewer::bf::face::{self, FaceTable, Kind, VISEMES};

use super::*;

/// The blink clock at a controller's construction (FUN_00143380: `+0x4c = 0x41200000`, 10.0): far
/// over any type's interval, so the first tick blinks.
const FIRST_BLINK_CLOCK: f32 = 10.0;

/// A character's face controller (the game's object at char+0x5b8, 0x64 bytes): the 22 weights,
/// the pose the face bones show and the blink's clock. The model's table (`Character::face`)
/// holds the data; this holds what changes.
#[derive(Clone, Debug)]
pub struct Face {
    /// the viseme weights (the controller's +0x34 array, 22 floats), in `face::VISEME_NAMES`'
    /// order, 0 to 1 (one above 1 counts as 1, one below 0 as 0): what the lip-sync and the
    /// expressions write. All 0 in this slice but the test hook's
    pub weights: [f32; VISEMES],
    /// the pose the face bones show now, one entry per face bone (the output pose, +0x24): the
    /// base at the start, the last blend's after
    output: Vec<[f32; 4]>,
    /// seconds since the last blink (+0x4c), 10 at the start
    since_blink: f32,
    /// a blink has started: the lids are the clip's, held at its last key once it ends
    blinked: bool,
    /// how many blinks have started (the log's count)
    blinks: u32,
}

impl Default for Face {
    /// A character with no face: nothing to show.
    fn default() -> Self {
        Face { weights: [0.0; VISEMES], output: vec![], since_blink: FIRST_BLINK_CLOCK, blinked: false, blinks: 0 }
    }
}

impl Face {
    /// FUN_00143380: a controller for a model with face table `table`: every weight 0, the base
    /// pose as the output (the constructor applies it with FUN_00143820), the blink due at once.
    pub fn new(table: &FaceTable) -> Face {
        Face { output: table.base.clone(), ..Face::default() }
    }

    /// One tick (FUN_001437d0; `dt` 0 does nothing, as FUN_00143d10's guard): the blend while
    /// something shows (FUN_001439b0: with nothing showing the game leaves the bones as they
    /// are), then the blink's clock (FUN_00143d10). Returns whether a blink started.
    pub fn tick(&mut self, table: &FaceTable, dt: f32) -> bool {
        if dt <= 0.0 {
            return false;
        }
        // (what shows, in the game: a line speaking with tracks, or an expression that isn't
        // BASE. Neither exists yet, so a weight up stands in; the lip-sync's slice puts the
        // game's test here, which also blends the base pose through a line's silences)
        if self.weights.iter().any(|&w| w > 0.0) {
            table.blend(&self.weights, &mut self.output);
        }
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
    /// bones take the output, the blink's lids their place in the clip.
    pub fn apply(&self, table: &FaceTable, game: &Game, pose: &mut [(Quat, Vec3)]) {
        table.write(&self.output, pose);
        if self.blinked {
            table.write_blink(game, self.since_blink, pose);
        }
    }
}

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
/// 1.48, 1.01 and 0.51 s ahead of Tex's in the four-second cycle).
pub fn step(p: &mut Player, l: &Loaded, game: &Game, dt: f32, pose: &mut [(Quat, Vec3)]) {
    let Some(table) = &l.model.face else { return };
    if p.absent {
        return;
    }
    if p.facial.tick(table, dt) && std::env::var("BF_FACE_LOG").is_ok() {
        println!("t={:.2} {}: blink {}", p.sim_time, unit_name(p), p.facial.blinks);
    }
    p.facial.apply(table, game, pose);
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
/// what they show until the first blink starts.
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
    t.write_blink(game, 0.0, &mut first);
    t.write_blink(game, f32::INFINITY, &mut last);
    let lids: Vec<String> = t.blink.iter().flat_map(|b| b.tracks.iter()).map(|&(bone, _)| {
        format!("{:08x} {:.1}/{:.1}", model.bones[bone], apart(body[bone].0, first[bone].0), apart(body[bone].0, last[bone].0))
    }).collect();
    format!("face check: {name}: base and {} visemes against their clips: largest turn {turn:.4} deg, largest move {off:.6} m{worst} ({skipped} without a clip); lids' first/last key from the bind pose, degrees [{}]",
            face::VISEMES - skipped, lids.join(", "))
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
/// squad name ("tex") or its slot (0-3).
fn is_whom(u: &Player, controlled: bool, who: &str) -> bool {
    match who {
        "all" => true,
        "me" => controlled,
        w => !u.own_type() && (u.data_name() == w || w.parse::<usize>().is_ok_and(|n| n == u.character)),
    }
}

/// BF_TEST_FACE: each entry sets its weight once its time comes (the controlled character's
/// clock, `Player::sim_time`).
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
                u.facial.weights[h.viseme] = h.weight;
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
/// blended this frame), and the camera hook after.
pub fn plugin(app: &mut App) {
    app.add_systems(Update, test_hooks.before(update_player).run_if(in_state(AppState::Playing)))
        .add_systems(Update, face_cam.after(update_player).run_if(in_state(AppState::Playing)));
}

#[cfg(test)]
mod tests {
    use super::*;
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

    /// With every weight 0 the face is the base pose; a weight up blends it in, and once the
    /// weights are 0 again the face keeps the last pose written, as the game's idle controller
    /// writes nothing (FUN_001439b0).
    #[test]
    fn the_face_is_the_base_until_a_weight_is_up_and_keeps_the_last_pose() {
        let t = table(4.0);
        let mut f = Face::new(&t);
        let turn = |f: &Face| {
            let mut pose = [(Quat::IDENTITY, Vec3::ZERO)];
            t.write(&f.output, &mut pose);
            pose[0].0.to_axis_angle().1
        };
        f.tick(&t, 0.03);
        assert!((turn(&f) - 0.2).abs() < 1e-4);
        f.weights[0] = 1.0;
        f.tick(&t, 0.03);
        assert!((turn(&f) - 0.6).abs() < 1e-4);
        f.weights[0] = 0.0;
        f.tick(&t, 0.03);
        assert!((turn(&f) - 0.6).abs() < 1e-4);
        // a face with no table has nothing to show
        assert!(Face::default().output.is_empty());
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

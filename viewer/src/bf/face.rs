//! The game's facial animation data (#255): a character type's face table and the blend that
//! turns viseme weights into a face pose. The faces are not played as clips while a character
//! talks; each living character's face controller (char+0x5b8, `play_faces.rs`) writes its face
//! bones every frame from a weighted blend of 22 static poses over a base pose, and these are
//! those poses.
//!
//!  `<name>face` animation set   one clip per pose, named `Sc_<viseme>`: the 15 mouth shapes
//!      (`bump cage church earth eat fave if new oat ox roar size though told wet`, the
//!      Microsoft SAPI viseme set) and 7 expressions (`angry base confused great_pain happy sad
//!      surprised`). Each is a still pose: its one key (the first) is what the game reads. Two
//!      more clips aren't poses: `Sc_blink` (0.333 s, the lids, played once every few seconds)
//!      and `Sc_rest` (a display object's, not used in play).
//!  `Sc_base`   (duration 0, one key) the rest pose, and, by its targets in order, the face
//!      bone list: the bones the controller owns (the game's animation binder skips them, so a
//!      body clip never moves them). A rotation channel makes a rotation bone, a vector channel
//!      a translation bone (FUN_00142df0).
//!  the type's `<face viseme-file h_09109ae2 h_f5a7b9d2>`   the set to read, the blink interval
//!      (seconds, 3-5) and the blink chance, and a list of `<h_fa110e09 enum scale>` records
//!      that stretch a viseme (Flint's `if` x 1.5, Tex's `if` x 3).
//!
//! The game builds the table once per type (FUN_00142cb0 and FUN_00142df0, type+0x1ac); here
//! `FaceTable::build` does, as a character's model loads.

use bevy::math::{Quat, Vec3};

use super::character::{sample, AnimDef, Game};
use super::hash::h;

/// How many visemes there are: the VE_ table (default.xbe 0x3be090) has 22 records.
pub const VISEMES: usize = 22;
/// The mouth shapes are visemes 0 to 14 (VE_BUMP to VE_WET); the expressions are 15 to 21
/// (VE_ANGRY to VE_SURPRISED). FUN_00143570 zeroes the first 15 weights when a line stops.
pub const MOUTH_VISEMES: usize = 15;
/// The 22 viseme names in the game's order (the VE_ table 0x3be090 and the strings at 0x3c02e8,
/// without the VE_ prefix and in lower case). The face table looks each up as the clip `Sc_` +
/// name; a sound's viseme track names it by its hash (0x3c0290).
pub const VISEME_NAMES: [&str; VISEMES] = [
    "bump", "cage", "church", "earth", "eat", "fave", "if", "new", "oat", "ox", "roar", "size", "though", "told", "wet",
    "angry", "base", "confused", "great_pain", "happy", "sad", "surprised",
];

/// The head bone's name hash ("Bip01 Head": 0xf6e2e9f1, the target of every body clip's head
/// channel): where a face is, for the viewer's and the demo's close-ups.
pub const HEAD_BONE: u32 = h("Bip01 Head");
/// How far over the head bone's origin a face's middle is (m): the eyes and mouth of the squad
/// are 0.05-0.15 m over the bone (a camera aimed there frames the face; measured by eye on
/// bf_viewer's close-ups, not a game value).
pub const FACE_OVER_HEAD: f32 = 0.08;

/// `Sc_base`'s name hash (0x0f896415, the literal FUN_00142df0 compares): the rest pose.
pub const SC_BASE: u32 = h("Sc_base");
/// `Sc_blink`'s name hash (0x1bb7a714, the literal at 0x143d45 FUN_00143d10 passes): the blink.
pub const SC_BLINK: u32 = h("Sc_blink");

/// The viseme index of a name ("bump", "great_pain"; any case), if it is one of the 22.
pub fn viseme_index(name: &str) -> Option<usize> {
    VISEME_NAMES.iter().position(|n| n.eq_ignore_ascii_case(name))
}

/// What a face clip is called: `Sc_bump` for a viseme's, `Sc_blink`, `Sc_rest`; None for a clip
/// of another name (Flint's set has one the game never looks up, h_febf7c82).
pub fn clip_name(hash: u32) -> Option<String> {
    VISEME_NAMES.iter().chain(&["blink", "rest"]).map(|n| format!("Sc_{n}")).find(|n| h(n) == hash)
}

/// A type's `<face>` settings (the parse record's +0x394 / +0x398 and list at +0x3a0, parsed by
/// FUN_001901a0, FUN_00190210 and FUN_001900f0): what the table takes besides the clips.
#[derive(Clone, Debug, PartialEq)]
pub struct FaceParams {
    /// h_09109ae2 (parse +0x394, an int; FUN_00142cb0 keeps it as a float at table+0x34): the
    /// seconds between blinks. 4 on the squad, the militia and the mutants, 3 on the Seers, 5 on
    /// Kingman. The blink fires once the time since the last passes it (FUN_00143d10)
    pub blink_interval: f32,
    /// h_f5a7b9d2 (parse +0x398, table+0x38): the chance of a blink before the interval is up,
    /// 0.05 on most types (0.09 on the Seers and Kingman). The test it feeds can't pass (the
    /// notes of play_faces.rs), so it only has to be kept
    pub blink_chance: f32,
    /// the `<h_e42866a1>` list: (viseme, scale) pairs that replace a viseme's scale of 1
    /// (FUN_00142cb0 writes each into its record). Flint: `if` 1.5; Tex: `if` 3; Hawk: nine
    /// mouth shapes 1.5
    pub scales: Vec<(usize, f32)>,
}

impl Default for FaceParams {
    /// A model loaded without its type's `<face>`: the squad's values (4 s, 0.05, no stretch). A
    /// stand-in, not a game value: every type that has a face names its own
    fn default() -> Self {
        FaceParams { blink_interval: 4.0, blink_chance: 0.05, scales: vec![] }
    }
}

/// How a face bone is posed: a rotation bone (a quaternion channel, h_06677bcb: kind 0 in the
/// game's bone list) takes a quaternion, a translation bone (a vector channel, h_f36d8810: kind
/// 1) a position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Rotation,
    Translation,
}

/// One bone of the face bone list: an entry of `Sc_base`'s targets that the model has.
#[derive(Clone, Copy, Debug)]
pub struct FaceBone {
    /// the bone's name hash (the target's name)
    pub name: u32,
    /// its index in the model's skeleton (`Character::bones`)
    pub bone: usize,
    pub kind: Kind,
}

/// One viseme's record (FUN_00142df0's 0x1c-byte records): its pose over the face bones, and
/// its scale.
#[derive(Clone, Debug)]
pub struct Viseme {
    /// per face bone, 4 floats (x y z w): a rotation bone's quaternion as `Sc_<name>` has it; a
    /// translation bone's difference between the clip's position and `Sc_base`'s (xyz, w 0). A
    /// bone the clip doesn't name keeps a copy of `Sc_base`'s value, which the blend skips when
    /// nothing has moved the bone yet. A viseme with no clip is `Sc_base` throughout
    pub pose: Vec<[f32; 4]>,
    /// how far its weight goes: the weight (at most 1) times this (record +0x18): 1 unless the
    /// type's `<face>` list stretches it
    pub scale: f32,
}

/// The blink: the lids' clip (`Sc_blink`, 0.333 s) and the bones it moves.
#[derive(Clone, Debug)]
pub struct Blink {
    /// the clip's length (the game compares it with the time since the last blink: the clip
    /// must be over before the next one starts)
    pub duration: f32,
    /// (model bone, channel) of each target the clip plays on. FUN_00234190 binds a target only
    /// if its node isn't flagged (+0x44 bit 5), and FUN_00143380 flags every face bone, so a
    /// target that is also in the face bone list never moves
    pub tracks: Vec<(usize, u32)>,
    /// how many of the clip's targets were left out for being face bones (Hawk's four lids: they
    /// are in her `Sc_base`, so her blink moves nothing)
    pub skipped: usize,
}

/// A character type's face table (FUN_00142cb0, 0x3c bytes at type+0x1ac), for one model.
#[derive(Clone, Debug)]
pub struct FaceTable {
    /// the face bone list: `Sc_base`'s targets in order (table+0x24), those the model has
    pub bones: Vec<FaceBone>,
    /// the base pose, per face bone (table+0x14): what the face shows with every weight 0
    pub base: Vec<[f32; 4]>,
    /// the 22 visemes in `VISEME_NAMES`' order (table+4)
    pub visemes: Vec<Viseme>,
    /// the blink, if the set has a `Sc_blink`
    pub blink: Option<Blink>,
    /// seconds between blinks (table+0x34) and the blink chance (table+0x38): the type's
    pub blink_interval: f32,
    pub blink_chance: f32,
    /// `Sc_base`'s targets the model doesn't have (the game writes no face at all when any is
    /// missing, FUN_00143820's count test; the demo leaves that bone out). 0 on every model
    pub missing: usize,
}

/// A clip's first key as a pose entry: a quaternion channel's rotation (x y z w), or a vector
/// channel's position (x y z, w 0). FUN_00142df0 reads each target's channel at key 0 (vtable
/// +0x1c, the argument 0 pushed at 0x142ed9 and 0x14314b). None for any other kind of channel (the
/// game's code takes only the two).
fn first_key(game: &Game, channel: u32) -> Option<(Kind, [f32; 4])> {
    match sample(game.channels.get(&channel)?, 0.0) {
        (Some(q), None) => Some((Kind::Rotation, [q.x, q.y, q.z, q.w])),
        (None, Some(v)) => Some((Kind::Translation, [v.x, v.y, v.z, 0.0])),
        _ => None,
    }
}

/// FUN_00144e50(1e-5): whether two 4-vectors agree in every component to within 1e-5 (each
/// squared difference under 1e-10; 0x3727c5ac is the 1e-5).
fn close(a: [f32; 4], b: [f32; 4]) -> bool {
    const EPSILON: f32 = 1.0e-5;
    a.iter().zip(&b).all(|(x, y)| (x - y) * (x - y) < EPSILON * EPSILON)
}

/// FUN_00233b10: a normalised lerp of two quaternions (x y z w), `(1 - t) a + t b` with `b`
/// negated when `a . b` is negative (the short way round), then normalised (the game with
/// rsqrtss). `t` isn't clamped: a scale over 1 takes it past `b`.
fn nlerp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let dot: f32 = a.iter().zip(&b).map(|(x, y)| x * y).sum();
    let s = if dot < 0.0 { -1.0 } else { 1.0 };
    let r: [f32; 4] = std::array::from_fn(|k| (a[k] - a[k] * t) + b[k] * t * s);
    let len = r.iter().map(|x| x * x).sum::<f32>().sqrt();
    if len > 0.0 { r.map(|x| x / len) } else { a }
}

impl FaceTable {
    /// The table of a model whose clips' face set is `face` (`Character::face_anims`): the game's
    /// FUN_00142df0 and FUN_00142cb0. `bones` is the model's skeleton (`Character::bones`). None
    /// when the set has no `Sc_base` (a type without a face set; the game would crash on a set
    /// without one, which none of the 39 face sets in the data is).
    pub fn build(game: &Game, bones: &[u32], face: &[AnimDef], params: &FaceParams) -> Option<FaceTable> {
        let base_clip = face.iter().find(|a| a.name == SC_BASE)?;
        // the bone list and the base pose: Sc_base's targets in order, their first keys
        let mut list: Vec<FaceBone> = vec![];
        let mut base: Vec<[f32; 4]> = vec![];
        let mut missing = 0;
        for &(name, channel) in &base_clip.targets {
            let Some((kind, value)) = first_key(game, channel) else { continue };
            let Some(bone) = bones.iter().position(|&b| b == name) else {
                missing += 1;
                continue;
            };
            list.push(FaceBone { name, bone, kind });
            base.push(value);
        }
        // the 22 records: a copy of the base, then each target of the clip `Sc_<name>` the list
        // has: a rotation bone's quaternion as it is, a translation bone's less the base's
        let mut visemes: Vec<Viseme> = VISEME_NAMES.iter().map(|name| {
            let mut pose = base.clone();
            if let Some(clip) = face.iter().find(|a| a.name == h(&format!("Sc_{name}"))) {
                for &(bone, channel) in &clip.targets {
                    let (Some(i), Some((kind, value))) = (list.iter().position(|f| f.name == bone), first_key(game, channel)) else { continue };
                    if kind != list[i].kind {
                        continue;
                    }
                    pose[i] = match kind {
                        Kind::Rotation => value,
                        Kind::Translation => std::array::from_fn(|k| value[k] - base[i][k]),
                    };
                }
            }
            Viseme { pose, scale: 1.0 }
        }).collect();
        for &(viseme, scale) in &params.scales {
            if let Some(v) = visemes.get_mut(viseme) {
                v.scale = scale;
            }
        }
        // the blink clip's targets: those a body clip's binder would bind (not face bones)
        let blink = face.iter().find(|a| a.name == SC_BLINK).map(|clip| {
            let mut tracks = vec![];
            let mut skipped = 0;
            for &(name, channel) in &clip.targets {
                let Some(bone) = bones.iter().position(|&b| b == name) else { continue };
                if list.iter().any(|f| f.name == name) {
                    skipped += 1;
                } else {
                    tracks.push((bone, channel));
                }
            }
            Blink { duration: clip.duration, tracks, skipped }
        });
        Some(FaceTable { bones: list, base, visemes, blink, blink_interval: params.blink_interval, blink_chance: params.blink_chance, missing })
    }

    /// The face pose for these viseme weights (FUN_001439b0's loop): the base pose, then every
    /// viseme with a weight over 0, in order (mouth shapes first, expressions last): the weight
    /// (held at 1) times the viseme's scale; each face bone whose viseme value isn't already
    /// the running one (FUN_00144e50) takes it: a rotation bone `nlerp`s from the running value
    /// toward the viseme's quaternion by the weight (FUN_00233b10), a translation bone adds the
    /// viseme's difference times the weight (FUN_00144e20). So the mouth is a chain of blends
    /// of absolute poses, not a sum of offsets; a weight of 1 shows the viseme's clip exactly.
    /// `out` is cleared and filled with one entry per face bone.
    ///
    /// One quirk of the game's is kept: a viseme's record is a copy of the base, so a bone its clip
    /// doesn't name holds the base's own value. For a rotation bone that pulls the running value
    /// back toward the base by the weight; for a translation bone the value stands as its
    /// "difference", so the base's absolute position is added again, weighted. The skip test
    /// (`close`) is what keeps either from happening while nothing has moved the bone; once an
    /// earlier viseme has, the running value no longer matches and it does. No face set in the
    /// data can meet it (all 39 were checked: no viseme leaves out a bone that an earlier one
    /// names, `base`, which moves nothing, apart), so it never shows; it would on a set that did.
    pub fn blend(&self, weights: &[f32; VISEMES], out: &mut Vec<[f32; 4]>) {
        out.clear();
        out.extend_from_slice(&self.base);
        for (viseme, &weight) in self.visemes.iter().zip(weights) {
            // (a weight of 0 or less does nothing, and a NaN neither)
            if !(weight > 0.0) {
                continue;
            }
            let w = weight.min(1.0) * viseme.scale;
            for (i, bone) in self.bones.iter().enumerate() {
                if close(out[i], viseme.pose[i]) {
                    continue;
                }
                match bone.kind {
                    Kind::Rotation => out[i] = nlerp(out[i], viseme.pose[i], w),
                    Kind::Translation => out[i] = std::array::from_fn(|k| out[i][k] + viseme.pose[i][k] * w),
                }
            }
        }
    }

    /// Put a face pose (`blend`'s output, or `base`) on a skeleton's local transforms
    /// (FUN_00143820: each face bone's joint takes its entry, a quaternion joint the rotation, a
    /// vector joint the position). The bones the pose names are replaced, none other touched.
    pub fn write(&self, output: &[[f32; 4]], pose: &mut [(Quat, Vec3)]) {
        for (bone, o) in self.bones.iter().zip(output) {
            let Some(local) = pose.get_mut(bone.bone) else { continue };
            match bone.kind {
                Kind::Rotation => local.0 = Quat::from_xyzw(o[0], o[1], o[2], o[3]).normalize(),
                Kind::Translation => local.1 = Vec3::new(o[0], o[1], o[2]),
            }
        }
    }

    /// The blink's lids `t` seconds into the clip (held at its last key from its end on: the
    /// nodes keep what the clip last wrote), on a skeleton's local transforms: what the
    /// animation player does with the bound targets of `Sc_blink`.
    pub fn write_blink(&self, game: &Game, t: f32, pose: &mut [(Quat, Vec3)]) {
        let Some(blink) = &self.blink else { return };
        for &(bone, channel) in &blink.tracks {
            let (Some(c), Some(local)) = (game.channels.get(&channel), pose.get_mut(bone)) else { continue };
            let (q, v) = sample(c, t.clamp(0.0, blink.duration));
            if let Some(q) = q {
                local.0 = q;
            }
            if let Some(v) = v {
                local.1 = v;
            }
        }
    }

    /// The face a character shows at rest, once its first blink (at its first tick) is over: the
    /// base pose on the face bones, the lids at the blink clip's last key. What a viewer shows as
    /// "the game's face". The lids can differ from the bind pose's: Brutus's rest keys are 18 and
    /// 7.5 degrees off it, Tex's and Flint's the same as it.
    pub fn write_rest(&self, game: &Game, pose: &mut [(Quat, Vec3)]) {
        self.write(&self.base, pose);
        self.write_blink(game, f32::INFINITY, pose);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A two-bone table by hand: a rotation bone (the jaw, base turned 0.2 rad about x) and a
    /// translation bone (a brow at y 0.5), and two visemes: `bump` (jaw 0.6 rad about x, brow
    /// 0.1 down) and `angry` (jaw as base, brow 0.3 down), `bump` stretched 1.5.
    fn table() -> FaceTable {
        let rot = |a: f32| { let q = Quat::from_rotation_x(a); [q.x, q.y, q.z, q.w] };
        let mut visemes: Vec<Viseme> = (0..VISEMES).map(|_| Viseme { pose: vec![rot(0.2), [0.0; 4]], scale: 1.0 }).collect();
        visemes[0] = Viseme { pose: vec![rot(0.6), [0.0, -0.1, 0.0, 0.0]], scale: 1.5 };
        visemes[15] = Viseme { pose: vec![rot(0.2), [0.0, -0.3, 0.0, 0.0]], scale: 1.0 };
        FaceTable {
            bones: vec![FaceBone { name: 1, bone: 0, kind: Kind::Rotation }, FaceBone { name: 2, bone: 1, kind: Kind::Translation }],
            base: vec![rot(0.2), [0.0, 0.5, 0.0, 0.0]],
            visemes,
            blink: None,
            blink_interval: 4.0,
            blink_chance: 0.05,
            missing: 0,
        }
    }

    fn angle(q: [f32; 4]) -> f32 {
        2.0 * q[3].clamp(-1.0, 1.0).acos()
    }

    /// With every weight 0 the face is the base pose; a weight of 1 shows the viseme's own pose
    /// (the rotation absolute, the position the base plus the difference); half a weight is
    /// halfway, as a normalised lerp has it.
    #[test]
    fn weights_blend_from_the_base_to_the_viseme() {
        let t = table();
        let mut out = vec![];
        let mut w = [0.0; VISEMES];
        t.blend(&w, &mut out);
        assert_eq!(out, t.base);
        // (bump is stretched 1.5, so a weight of 1 goes 1.5 of the way: use `base` as a viseme
        // with no stretch to check the exact pose)
        let mut t1 = table();
        t1.visemes[0].scale = 1.0;
        w[0] = 1.0;
        t1.blend(&w, &mut out);
        assert!((angle(out[0]) - 0.6).abs() < 1e-5, "{}", angle(out[0]));
        assert!((out[1][1] - 0.4).abs() < 1e-6, "{:?}", out[1]);
        w[0] = 0.5;
        t1.blend(&w, &mut out);
        assert!((angle(out[0]) - 0.4).abs() < 1e-3, "{}", angle(out[0]));
        assert!((out[1][1] - 0.45).abs() < 1e-6);
    }

    /// A viseme's scale stretches its weight past the pose (Flint's `if` x 1.5): the rotation
    /// goes 1.5 of the way from the base, a position 1.5 of the difference. A weight over 1 is
    /// held at 1 before the scale; a negative one does nothing.
    #[test]
    fn a_scale_stretches_the_weight_and_a_weight_stops_at_one() {
        let t = table();
        let mut out = vec![];
        let mut w = [0.0; VISEMES];
        w[0] = 1.0;
        t.blend(&w, &mut out);
        // (base 0.2 rad, bump 0.6: 1.5 of the way is 0.8; the nlerp's angle is near that)
        assert!((angle(out[0]) - 0.8).abs() < 0.01, "{}", angle(out[0]));
        assert!((out[1][1] - 0.35).abs() < 1e-6, "{:?}", out[1]);
        let once = out.clone();
        w[0] = 7.0;
        t.blend(&w, &mut out);
        assert_eq!(out, once);
        w[0] = -1.0;
        t.blend(&w, &mut out);
        assert_eq!(out, t.base);
        w[0] = f32::NAN;
        t.blend(&w, &mut out);
        assert_eq!(out, t.base);
    }

    /// The visemes go in order, each from what the ones before left: an expression after a mouth
    /// shape adds its position difference on top (0.5 - 0.1 - 0.3 at full weights, with bump's
    /// stretch 1.5: 0.5 - 0.15 - 0.3), and its rotation, equal to the base's, leaves the
    /// rotation alone only when it matches the running value (here it doesn't: bump moved it, so
    /// angry pulls it back toward the base).
    #[test]
    fn the_visemes_chain_in_order() {
        let t = table();
        let mut out = vec![];
        let mut w = [0.0; VISEMES];
        w[0] = 1.0;
        w[15] = 1.0;
        t.blend(&w, &mut out);
        assert!((out[1][1] - 0.05).abs() < 1e-6, "{:?}", out[1]);
        // angry's jaw is the base 0.2, a full step from bump's 0.8: back at 0.2
        assert!((angle(out[0]) - 0.2).abs() < 1e-3, "{}", angle(out[0]));
    }

    /// A viseme value that matches the running one within 1e-5 is skipped (FUN_00144e50): a
    /// translation bone a viseme doesn't name holds the base's own position as its "difference"
    /// and must not be added to the bone again.
    #[test]
    fn a_bone_a_viseme_does_not_name_stays_put() {
        let mut t = table();
        // cage names neither bone: its record is a copy of the base
        t.visemes[1] = Viseme { pose: t.base.clone(), scale: 1.0 };
        let mut out = vec![];
        let mut w = [0.0; VISEMES];
        w[1] = 1.0;
        t.blend(&w, &mut out);
        assert_eq!(out, t.base);
    }

    /// FUN_00233b10 takes the short way round: a quaternion and its negative are the same
    /// rotation, so blending toward the negated one gives the same face.
    #[test]
    fn the_nlerp_takes_the_short_way() {
        let a = Quat::from_rotation_x(0.2);
        let b = Quat::from_rotation_x(0.6);
        let (a, b) = ([a.x, a.y, a.z, a.w], [b.x, b.y, b.z, b.w]);
        let neg = b.map(|x| -x);
        let (p, q) = (nlerp(a, b, 0.5), nlerp(a, neg, 0.5));
        for k in 0..4 {
            assert!((p[k] - q[k]).abs() < 1e-6);
        }
        assert!((angle(p) - 0.4).abs() < 1e-3);
    }

    /// The viseme names are the game's: their hashes are the 0x3c0290 table's (bump
    /// e4d50c6f, surprised 002e0bed), `Sc_` + name the clips' (`Sc_bump` eff8ac8d, `Sc_base`
    /// 0f896415, `Sc_blink` 1bb7a714), and an index is found by name in any case.
    #[test]
    fn the_names_are_the_games() {
        assert_eq!(h("bump"), 0xe4d5_0c6f);
        assert_eq!(h("cage"), 0x1e50_90b9);
        assert_eq!(h("surprised"), 0x002e_0bed);
        assert_eq!(h("Sc_bump"), 0xeff8_ac8d);
        assert_eq!(SC_BASE, 0x0f89_6415);
        assert_eq!(SC_BLINK, 0x1bb7_a714);
        assert_eq!(viseme_index("Great_Pain"), Some(18));
        assert_eq!(viseme_index("blink"), None);
        assert_eq!(clip_name(0x0f89_6415).as_deref(), Some("Sc_base"));
        assert_eq!(clip_name(0x1bb7_a714).as_deref(), Some("Sc_blink"));
        assert_eq!(clip_name(0xfebf_7c82), None);
    }
}

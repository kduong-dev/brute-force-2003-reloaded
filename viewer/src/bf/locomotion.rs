//! Locomotion for a playable character: picking idle / walk / run / sprint / backpedal / strafe
//! clips out of a character's animation set, root-motion extraction and pose blending.
//!
//! Animation names are hashes, so clips are classified from their data: the root channel's
//! displacement over the clip (direction and speed; characters face -Z, sprints are fastest),
//! how cleanly the pose loops, how upright the posture is (crouched variants sit lower) and how
//! much the body moves (idles move least).

use bevy::math::{Quat, Vec3, Vec4};

use super::character::{sample, Character, Game};

/// Head bone (same name hash in every character's skeleton).
const HEAD_BONE: u32 = 0xF6E2_E9F1;

#[derive(Clone, Debug)]
pub struct ClipStats {
    pub index: usize,
    pub duration: f32,
    /// root displacement per second over the clip (model space; forward is -Z)
    pub velocity: Vec3,
    /// largest bone-position difference (relative to the root) between first and last frame
    pub loop_error: f32,
    /// mean head height: crouched / kneeling variants are lower
    pub height: f32,
    /// mean horizontal reach of the bones from the root: arms-out reference poses are wide
    pub span: f32,
    /// mean bone motion relative to the root between samples
    pub energy: f32,
}

#[derive(Clone, Debug, Default)]
pub struct Locomotion {
    pub idle: Option<usize>,
    pub walk: Option<usize>,
    pub run: Option<usize>,
    pub sprint: Option<usize>,
    pub walk_back: Option<usize>,
    pub run_back: Option<usize>,
    /// The only sideways clips in the squad's sets are dodge rolls (the head drops near the
    /// ground); aimed movement is legs-run plus an upper-body twist instead (see `twist`).
    pub dodge_left: Option<usize>,
    pub dodge_right: Option<usize>,
}

/// Root position of `anim` at time t (from its root channel; zero if it has none).
pub fn root_position(ch: &Character, game: &Game, anim: usize, t: f32) -> Vec3 {
    let root = ch.bones[0];
    ch.anims.get(anim)
        .and_then(|a| a.targets.iter().find(|(b, _)| *b == root))
        .and_then(|(_, c)| game.channels.get(c))
        .and_then(|c| sample(c, t).1)
        .unwrap_or(Vec3::ZERO)
}

/// Root displacement while advancing from t0 by dt, wrapping around the clip end.
pub fn root_delta(ch: &Character, game: &Game, anim: usize, t0: f32, dt: f32) -> Vec3 {
    let dur = ch.anims[anim].duration.max(1e-3);
    let t1 = t0 + dt;
    if t1 <= dur {
        root_position(ch, game, anim, t1) - root_position(ch, game, anim, t0)
    } else {
        let wraps = (t1 / dur).floor();
        let full = root_position(ch, game, anim, dur) - root_position(ch, game, anim, 0.0);
        (root_position(ch, game, anim, dur) - root_position(ch, game, anim, t0))
            + full * (wraps - 1.0)
            + (root_position(ch, game, anim, t1 - wraps * dur) - root_position(ch, game, anim, 0.0))
    }
}

pub fn stats(ch: &Character, game: &Game) -> Vec<ClipStats> {
    const N: usize = 10;
    let mut out = vec![];
    for (i, a) in ch.anims.iter().enumerate() {
        let dur = a.duration;
        if dur < 0.2 {
            continue;
        }
        let mut rel: Vec<Vec<Vec3>> = vec![];
        let (mut height, mut span) = (0.0, 0.0);
        let head = ch.bones.iter().position(|b| *b == HEAD_BONE);
        for k in 0..N {
            let t = dur * k as f32 / (N - 1) as f32 * 0.999;
            let w = ch.world(&ch.pose(game, i, t, None, 0.0));
            let root = w[0].w_axis.truncate();
            let r: Vec<Vec3> = w.iter().map(|m| m.w_axis.truncate() - root).collect();
            height += match head {
                Some(hb) => w[hb].w_axis.y,
                None => w.iter().map(|m| m.w_axis.y).fold(f32::MIN, f32::max),
            };
            span += r.iter().map(|v| (v.x * v.x + v.z * v.z).sqrt()).fold(0.0, f32::max);
            rel.push(r);
        }
        let energy = rel.windows(2).map(|p| p[0].iter().zip(&p[1]).map(|(a, b)| (*a - *b).abs().element_sum()).sum::<f32>()
            / p[0].len() as f32).sum::<f32>() / (N - 1) as f32;
        let loop_error = rel[0].iter().zip(&rel[N - 1]).map(|(a, b)| (*a - *b).abs().max_element()).fold(0.0, f32::max);
        let velocity = (root_position(ch, game, i, dur * 0.999) - root_position(ch, game, i, 0.0)) / dur;
        out.push(ClipStats { index: i, duration: dur, velocity, loop_error, height: height / N as f32, span: span / N as f32, energy });
    }
    out
}

pub fn pick(stats: &[ClipStats]) -> Locomotion {
    // most upright clip passing `filter`; ties broken by `prefer` (higher is better)
    let best = |filter: &dyn Fn(&ClipStats) -> bool, prefer: &dyn Fn(&ClipStats) -> f32| -> Option<usize> {
        let c: Vec<&ClipStats> = stats.iter().filter(|s| filter(s)).collect();
        let top = c.iter().map(|s| s.height).fold(f32::MIN, f32::max);
        c.into_iter().filter(|s| s.height >= top - 0.03)
            .max_by(|a, b| prefer(a).total_cmp(&prefer(b))).map(|s| s.index)
    };
    let fwd = |s: &ClipStats| -s.velocity.z;
    let straight = |s: &ClipStats| s.velocity.x.abs() < 0.25 * s.velocity.z.abs();
    let loops = |s: &ClipStats, e: f32| s.loop_error < e;
    Locomotion {
        idle: pick_idle(stats),
        walk: best(&|s| straight(s) && (1.0..2.6).contains(&fwd(s)) && loops(s, 0.05), &|s| -(fwd(s) - 1.8).abs()),
        run: best(&|s| straight(s) && (3.0..6.0).contains(&fwd(s)) && loops(s, 0.1), &|s| -(fwd(s) - 4.6).abs()),
        sprint: best(&|s| straight(s) && fwd(s) >= 6.0 && loops(s, 0.1), &|s| fwd(s)),
        walk_back: best(&|s| straight(s) && (1.0..2.6).contains(&-fwd(s)) && loops(s, 0.05), &|s| -(-fwd(s) - 1.6).abs()),
        run_back: best(&|s| straight(s) && (2.6..6.0).contains(&-fwd(s)) && loops(s, 0.1), &|s| -fwd(s)),
        dodge_left: dodge(stats, -1.0),
        dodge_right: dodge(stats, 1.0),
    }
}

/// Standing idle: tallest head (not kneeling / crouched), arms down (not an arms-out reference
/// pose: narrow span), then the calmest (least motion, so no gestures).
fn pick_idle(stats: &[ClipStats]) -> Option<usize> {
    let c: Vec<&ClipStats> = stats.iter().filter(|s| {
        (s.velocity.x * s.velocity.x + s.velocity.z * s.velocity.z).sqrt() < 0.1 && s.velocity.y.abs() < 0.1
            && s.duration >= 1.2 && s.loop_error < 0.05 && s.energy > 3e-4
    }).collect();
    let top = c.iter().map(|s| s.height).fold(f32::MIN, f32::max);
    let tall: Vec<&&ClipStats> = c.iter().filter(|s| s.height >= top - 0.03).collect();
    let narrow = tall.iter().map(|s| s.span).fold(f32::MAX, f32::min);
    tall.into_iter().filter(|s| s.span <= narrow + 0.1).min_by(|a, b| a.energy.total_cmp(&b.energy)).map(|s| s.index)
}

/// Sideways dodge roll toward `sign` (-1 left / +1 right): mostly sideways root motion.
fn dodge(stats: &[ClipStats], sign: f32) -> Option<usize> {
    stats.iter()
        .filter(|s| s.velocity.x * sign > 0.8 && s.velocity.z.abs() < 0.3 * s.velocity.x.abs())
        .min_by(|a, b| a.loop_error.total_cmp(&b.loop_error))
        .map(|s| s.index)
}

/// Spine chain used for aiming: the head's ancestors below the pelvis, plus the head.
pub fn aim_chain(ch: &Character) -> Vec<usize> {
    let Some(head) = ch.bones.iter().position(|b| *b == HEAD_BONE) else { return vec![] };
    let mut chain = vec![head];
    let mut b = head;
    while let Some(p) = ch.parent[b] {
        if ch.parent[p].is_none() || ch.parent[ch.parent[p].unwrap()].is_none() {
            break;                                           // stop above root and pelvis
        }
        chain.push(p);
        b = p;
    }
    chain.reverse();                                         // lowest spine bone first
    chain
}

/// Twist the upper body by `angle` radians about the model's up axis, spread evenly over the
/// aim chain (each bone turns its share around the vertical through its own joint).
pub fn twist(ch: &Character, local: &mut [(Quat, Vec3)], chain: &[usize], angle: f32) {
    turn(ch, local, chain, Vec3::Y, angle);
}

/// Rotate the chain by `angle` about a model-space `axis`, spread evenly over its bones (each
/// turns its share about the axis through its own joint), so whatever hangs from the last bone
/// turns by the full angle.
pub fn turn(ch: &Character, local: &mut [(Quat, Vec3)], chain: &[usize], axis: Vec3, angle: f32) {
    if chain.is_empty() || angle.abs() < 1e-4 {
        return;
    }
    let share = Quat::from_axis_angle(axis.normalize(), angle / chain.len() as f32);
    for &b in chain {
        let world = ch.world(local);
        let Some(p) = ch.parent[b] else { continue };
        let parent_rot = world[p].to_scale_rotation_translation().1;
        // new world rotation = share * old world rotation  =>  local' = P^-1 * share * P * local
        local[b].0 = (parent_rot.inverse() * share * parent_rot * local[b].0).normalize();
    }
}

/// Weighted blend of local poses (rotations nlerp'd with sign alignment, translations lerp'd).
pub fn blend(poses: &[(Vec<(Quat, Vec3)>, f32)]) -> Vec<(Quat, Vec3)> {
    let Some((first, _)) = poses.first() else { return vec![] };
    let total: f32 = poses.iter().map(|(_, w)| *w).sum::<f32>().max(1e-6);
    (0..first.len()).map(|b| {
        let mut q = Vec4::ZERO;
        let mut t = Vec3::ZERO;
        let reference = Vec4::from(first[b].0);
        for (p, w) in poses {
            let v = Vec4::from(p[b].0);
            q += if v.dot(reference) < 0.0 { -v } else { v } * (*w / total);
            t += p[b].1 * (*w / total);
        }
        (Quat::from_vec4(q).normalize(), t)
    }).collect()
}

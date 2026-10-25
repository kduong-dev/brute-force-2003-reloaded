//! Bolts (issue #136): the shots of bullet-type 0 and 3 (BTYPE_PROJECTILE, PHYSICAL_PROJECTILE)
//! fly as the game's pooled objects and find what they hit on the way, from default.xbe's blast
//! pool: FUN_0022bac0 (the launch), FUN_00225220 (the pool's tick) and FUN_00225440 (a bolt's
//! flight), and the damage handler each hit is dealt through (FUN_00224320, FUN_00224770 ->
//! 0x22d0b0). The guns: the A10 and A50 Bioreactive, the three LZRs, the two cutters, the Ion
//! Beamer, the Jax-iC / iP / iR, the MNR-7 Bio, the three Saryls, and the creatures' Psionic
//! Blast. No definition in the data uses bullet-type 3.
//!
//!  - **The launch** (FUN_0022bac0, from the fire loop's FUN_0022f1d0): at the muzzle (the
//!    shot's origin), along the shot (after the accuracy's turn), at the bullet's speed (+0x1c);
//!    its life (+0x28, and +0x30 its start) the bullet's range in the holder's hands
//!    (FUN_002324c0) over the speed; the damage roll made now (+0x20, FUN_0022c940: Damage
//!    min..max, then the shooter's skill's factor, `skills::roll_factor`, and its power-ups',
//!    `powerups::roll_factor`), with the shooter
//!    (+0x48) and its team (+0x6c). Its last checked point (+0x70) is the muzzle and its sweep
//!    counter (byte +3) the pool's reset's 3 (FUN_00225000). The bullet's h_1835f15e,
//!    h_fe4f3035 and h_11caf57b become its bytes +1 and +2 and its flag 0x20 (#238, below).
//!  - **Each game frame** (a 30 Hz tick, shots::TICK; FUN_00225220): its life runs down by the
//!    frame's time, and once it's out the bolt goes where it is; then FUN_00225440 counts the
//!    sweep counter down. Not yet 0, the bolt moves speed x dt along its way unchecked; at 0 the
//!    counter is set to 3 again and a ray (FUN_000ddd10) is swept from the last checked point
//!    over the way moved since and this frame's step. Nothing met: it moves on, and the last
//!    checked point is where it is. Met: it's set back 0.1 % of the way from the hit toward the
//!    last checked point (HIT_BACK), which is set there too, the body struck is gathered
//!    (FUN_00224320, the hit part's factor asked for) and dealt in the same frame (FUN_00224770),
//!    and it ends (the next frame takes it away, the flag 4 FUN_00225220 reads), passes through
//!    it or bounces off it (`Bolt::after_hit`, #238).
//!  - **What a sweep meets** (its filter, vtable 0x394848 with the mask 0xfffffff8: the bodies of
//!    PID 0, 1 and 2, `Arena::ray_shot`): the terrain and the placed objects, a breakable among
//!    them, but not the level's invisible blockers; a character's body (standing, play.rs's
//!    `ray_body`; knocked down, play_missile.rs's `ray_lying`; a corpse, its ragdoll's bodies,
//!    play_ragdoll.rs `Ragdoll::first_hit`); a Sentry. Never its shooter (the filter holds the shooter's body,
//!    +0x48's at its +0x6c), nor what it last passed through (+0x40, for that one sweep).
//!  - **A hit** (the damage handler 0x22d0b0, `deal`): a character takes the launch roll x the
//!    shooter's skill x the struck hit part's factor (FUN_00125940), x0.7 on its own side, then
//!    its own step and damage-type factor as it lands (play_damage.rs, through play.rs's
//!    `land_player_shot` for the controlled character's bolts and `land_ai_shot` for the rest);
//!    the world plays its material's hit effect, debris and sound with the launch roll as the
//!    record (play.rs `world_hit`) and takes the bullet's hole (play_fx.rs); a breakable object
//!    takes the roll (play_scenery.rs `ObjectHits`); a Sentry goes off (play_sentry.rs); a body on
//!    the ground is shoved and bleeds (`Ragdoll::shot`). Whatever it strikes, the bullet's own
//!    explosion plays there first (FUN_0022cfc0, play.rs `shot_explosion`: the cutters' white
//!    star, #53). A liquid's surface it crosses does nothing (FUN_00225440's sweep has no liquid
//!    code, todo/146-liquids leads §5.1: only an instant ray splashes, play.rs `projectiles`), and
//!    it goes on to the bed (whether the game's stops on the liquid isn't traced).
//!  - **Through, or off** (#238, `Bolt::after_hit`): with the pass flag it goes through an object
//!    while n1 lasts and a character while n2 does, and stops at anything else (the terrain
//!    always); without it, while n1 or n2 lasts, it bounces off what it hit, its way turned about
//!    the surface's normal. The Ion Beamer passes through up to five objects and stops at the
//!    first character; the cutters' blades bounce off up to five objects and stop on a character
//!    or the terrain. Each thing it hits on the way is dealt its hit.
//!
//! The timing, measured on all eight bolt guns of weapons batch 1 (todo/57-jax-ic/spec.md §4: 7 m
//! at 80 m/s 0.1 s in 16 of 16 hits, 25 m at 100 m/s 0.3 s in 12 of 12; the fire-to-hit sounds
//! 0.21 s apart at 10 m and 80 m/s, 6 frames; the Saryl-45's within 10 m 0.09-0.1 s; the burst in
//! the picture at the shot + 0.2 s at 10 m; the ball one step out a frame after the shot): the
//! sweeps land on game frames 3, 6, 9 ... counted from the frame after the shot's, 0.1 s apart.
//! The code would count the launch frame as the bolt's first tick (the world update runs the
//! objects, where the shot is made, before the pool's tick: 0xd9ae2, then 0xd9b07), a frame
//! sooner; the analyst couldn't settle it from the code and left it to the takes' frame counts
//! (todo/weapons-plan/answers-batch1.md §1), so a bolt's first tick is the frame after its shot's
//! (`launch_bolts` runs after `fly_bolts`). Between sweeps a bolt flies on unchecked, into what it
//! struck (the Jax-iR's ball gone in the wall 0.117-0.183 s after the shot, its burst at 0.2 s:
//! todo/59 take01). Its life ends its sweeps: the Jax-iC's last (tick 21 of its 22.5) reaches 56
//! m of its 60, and a target beyond is never met.
//!
//! Before #136 the demo picked a bolt's hit at its shot (the body or the world on its line then)
//! and landed it on the sweep that would reach it: a target that stepped out of the way was still
//! hit, one that stepped in wasn't. Before #238 every bolt stopped at its first hit.
//!
//! FUN_00225220 also sets the flight effect's user parameter to life / start each frame
//! (FUN_0021c380): no bolt's flight effect varies with it (every curve of the data's 22 has one
//! variant), so it isn't kept. The bolt's flight voice (FUN_0022bac0's h_03dc7b9a pick) isn't
//! played (answers-batch1.md §3: never heard in the takes). A bolt that ends on a loose object
//! pushes it (FUN_002b3bc0, the object's +0x26 bit 0x40): the demo's loose pickups are knocked by
//! the sweeps' lines instead (`Sweeps`).
//!
//! `BF_SHOT_LOG=1` prints each bolt's launch (its roll, life, range and hit counts), each sweep
//! that meets something (what, how far from its last checked point, its game frame, and whether
//! it goes on through it or off it) and its end.

use super::*;
use super::damage::{self, Difficulty};

/// FUN_00225440's sweep counter (byte +3, FUN_00225000's 3): a sweep every this many game frames.
const SWEEP_TICKS: u8 = 3;
/// FUN_00225440 sets a bolt that met something back from the hit point toward its last checked
/// point by this share of the way between them (x 0.001), and checks on from there.
const HIT_BACK: f32 = 0.001;
/// A bolt's life is out once what's left of it is this or less (s): FUN_00225440 ends it at a
/// life of 0 or less, and the demo's sum of frame times lands a hair either side of a life that's
/// a whole number of frames (the LZR-10's 80 / 100 = 0.8 s, 24 of them). The demo's.
const LIFE_SLACK: f32 = 1e-4;

/// A bolt in flight (the pool object FUN_0022bac0 fills; its offsets in the notes).
#[derive(Component)]
pub(super) struct Bolt {
    /// the shot it was fired as: the weapon's effects, decal, ammo-type, damage-type and range
    shot: Shot,
    /// its number (BF_SHOT_LOG)
    id: u32,
    /// who fired it (+0x48), their team (+0x6c), and whether that was the controlled character
    /// (its hits land as the player's shots do: play.rs `land_player_shot`)
    owner: Who,
    team: u8,
    human: bool,
    /// where it is (+4), its way (+0x10, a unit vector) and speed (+0x1c)
    pos: Vec3,
    dir: Vec3,
    speed: f32,
    /// its life left (+0x28, s)
    life: f32,
    /// the sweep counter (+3)
    counter: u8,
    /// the last checked point (+0x70)
    last: Vec3,
    /// the damage roll made at its launch (+0x20, FUN_0022c940), the shooter's skill's factor on
    /// it and its power-ups' (BRUTE's x2, `powerups::roll_factor`)
    roll: f32,
    skill: f32,
    power: f32,
    /// the hits it goes on after (#238, `after_hit`): byte +1 (n1, the bullet's h_1835f15e,
    /// `ShotData::object_hits`) and byte +2 (n2, h_fe4f3035, `ShotData::character_hits`), each
    /// the low byte of the int FUN_0022bac0 copies; and its flag 0x20 (h_11caf57b,
    /// `ShotData::pass_through`): it passes through, else it bounces
    n1: u8,
    n2: u8,
    pass: bool,
    /// what it last passed through (+0x40): its next sweep leaves it out, and only that one
    skip: Option<Struck>,
    /// game time not yet run in ticks (s)
    clock: f32,
    /// game frames flown since its launch (BF_SHOT_LOG)
    ticks: u32,
}

/// What a game frame of a bolt comes to (`Bolt::advance`).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Advance {
    /// its life is out: it goes, unmoved
    Out,
    /// it moved on unchecked
    Moved,
    /// its sweep is due: from its last checked point, over this far, this frame's step (the move
    /// it makes if the sweep meets nothing)
    Sweep { from: Vec3, len: f32, step: f32 },
}

/// What a hit is to the rules after it (`Bolt::after_hit`, play_rays.rs's budgets): the ray
/// result's kind (FUN_000dd8e0) and the object's type. The terrain (kind 1, no object); a game
/// object that isn't a character (kind 2 of an object's body: the placed objects, a broken one's
/// remains, a Sentry; the test floor's pillars, the demo's); a character (kind 2 of a bone body,
/// object type 3: standing, lying or dead).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Terrain,
    Object,
    Character,
}

/// What came of a bolt's hit (`Bolt::after_hit`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Outcome {
    /// the bolt ends there (flag 4)
    Ends,
    /// it goes on through it, leaving it out of its next sweep
    Passes,
    /// it goes on off it, turned about the surface's normal
    Bounces,
}

impl Bolt {
    /// One game frame of FUN_00225220 -> FUN_00225440 up to the sweep: the life runs down by the
    /// frame (out: it ends where it is), the sweep counter drops; not yet 0 the bolt moves on,
    /// at 0 it's set to SWEEP_TICKS again and the sweep is due over the way moved since the last
    /// checked point and this frame's step.
    fn advance(&mut self) -> Advance {
        self.ticks += 1;
        self.life -= shots::TICK;
        if self.life <= LIFE_SLACK {
            return Advance::Out;
        }
        self.counter -= 1;
        let step = self.speed * shots::TICK;
        if self.counter != 0 {
            self.pos += self.dir * step;
            return Advance::Moved;
        }
        self.counter = SWEEP_TICKS;
        Advance::Sweep { from: self.last, len: self.pos.distance(self.last) + step, step }
    }

    /// FUN_00225440 after a sweep's hit `id` of `kind` (its surface's `normal`), once the hit is
    /// gathered (#238):
    ///  - With the pass flag (0x20): an object while n1 is left takes one off it, a character
    ///    while n2 is left one off that, and the bolt goes on through it, leaving it out of its
    ///    next sweep (+0x40, the object's handle); anything else ends it, the terrain always.
    ///  - Without it, while either count is left: an object takes one off n1, or off n2 when n1 is
    ///    out; anything else (the terrain, a character) one off n2; and the bolt's way is turned
    ///    about the surface's normal (d - 2 (d.n) n, the ray result's normal). With the count it
    ///    would take at 0 it ends.
    ///
    /// The Ion Beamer (pass, n1 5, n2 0) goes through up to five objects and stops at the first
    /// character or the terrain; the cutters (n1 5, n2 0) bounce off up to five objects and stop
    /// on a character or the terrain; every other bolt (0, 0) ends at its first hit.
    fn after_hit(&mut self, kind: Kind, id: Struck, normal: Vec3) -> Outcome {
        if self.pass {
            match kind {
                Kind::Object if self.n1 > 0 => self.n1 -= 1,
                Kind::Character if self.n2 > 0 => self.n2 -= 1,
                _ => return Outcome::Ends,
            }
            self.skip = Some(id);
            return Outcome::Passes;
        }
        if kind == Kind::Object && self.n1 > 0 {
            self.n1 -= 1;
        } else if self.n2 > 0 {
            self.n2 -= 1;
        } else {
            return Outcome::Ends;
        }
        self.dir = (self.dir - 2.0 * self.dir.dot(normal) * normal).normalize_or(-self.dir);
        Outcome::Bounces
    }
}

/// The bolts' shared state: the next number.
#[derive(Resource, Default)]
struct BoltState {
    next: u32,
}

/// This frame's sweeps (from the last checked point, along the bolt's way, as far as what it
/// met) and pass-through rays' lines (play_rays.rs): the loose pickups they pass are knocked as by
/// a shot's line (play_pickups.rs `physics`).
#[derive(Resource, Default)]
pub(super) struct Sweeps(pub Vec<(Vec3, Vec3, f32)>);

/// The bolts' systems (`plugin`): play_rays.rs's runs after them (`Sweeps` is started afresh in
/// `fly_bolts`).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BoltSystems;

/// The systems, after the characters have moved and fired (`update_player`): the bolts in flight,
/// then this frame's new ones (a bolt is drawn at the muzzle in its shot's frame and flies from
/// the next); before the shots are drawn and cleared (`update_weapons`, `projectiles`: named, where
/// it held only through `play_sounds`' place in play.rs's chain before #287), the grenades (a
/// Sentry a bolt struck goes off this frame, play_sentry.rs's `trip` among them), the scenery
/// (play_scenery.rs, `ObjectHits`), the pickups (`Sweeps`), the hits' blood (play_fx.rs) and the
/// sounds.
pub fn plugin(app: &mut App) {
    app.init_resource::<BoltState>().init_resource::<Sweeps>()
        .add_systems(OnEnter(AppState::Playing), |mut state: ResMut<BoltState>, mut sweeps: ResMut<Sweeps>| {
            *state = BoltState::default();
            sweeps.0.clear();
        })
        .add_systems(Update, (fly_bolts, launch_bolts).chain().in_set(BoltSystems)
            .after(update_player).before(super::grenade::GrenadeSystems).before(super::pickups::PickupSystems)
            .before(super::fx::FxSystems).before(play_sounds).before(update_weapons).run_if(in_state(AppState::Playing)));
}

/// What a sweep (or a pass-through ray, play_rays.rs) met: how far along, what, which body it
/// is, and its surface's normal (facing back along the line).
#[derive(Clone, Copy, Debug)]
pub(super) struct Met {
    pub t: f32,
    pub what: What,
    pub id: Struck,
    pub normal: Vec3,
}

/// What a sweep or a pass-through ray can meet (see the module notes).
#[derive(Clone, Copy, Debug)]
pub(super) enum What {
    /// the world: its world material, whether it's the terrain, and the breakable object it is,
    /// if one
    World { material: i64, terrain: bool, breakable: Option<usize> },
    /// a character standing, or knocked down
    Body(Who),
    /// a corpse (its ragdoll)
    Corpse(Who),
    /// a Sentry mine
    Mine(Entity),
}

impl What {
    /// What it is to the hit rules (`Kind`).
    pub(super) fn kind(&self) -> Kind {
        match self {
            What::World { terrain: true, .. } => Kind::Terrain,
            What::World { .. } | What::Mine(_) => Kind::Object,
            What::Body(_) | What::Corpse(_) => Kind::Character,
        }
    }
}

/// One thing a sweep or a ray can meet, by its body (a sweep's single-hit query meets a body at
/// its nearest face, FUN_000dd8e0, and a bolt leaves out the one it last passed through, +0x40; a
/// pass-through ray meets a body at each face it crosses, `Arena::ray_shot_all`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Struck {
    /// a body of the map's collision (`Arena::ray_shot`'s, `Tri::body`)
    Body(u32),
    /// one of the flat test floor's pillars (its number in `pillars`)
    Pillar(usize),
    /// a breakable the test tools put down without collision (its scenery list index)
    Placed(usize),
    /// a character
    Unit(Who),
    /// a Sentry mine
    Mine(Entity),
}

/// The flat test floor's own things a line from `from` along `dir` meets within `len` m, but
/// `skip`: its ground (with no arena, the plane at GROUND), its pillars' boxes and the boxes of
/// the breakables the test tools put down, as play.rs's `ray_hit` and play_shots.rs's
/// `hit_normal` meet them. The pillars count as objects: the demo's.
fn floor_met(from: Vec3, dir: Vec3, len: f32, skip: Option<Struck>, ground: bool) -> Vec<Met> {
    let mut out = vec![];
    if ground && dir.y < -1e-4 {
        let t = (GROUND - from.y) / dir.y;
        if t > 1e-3 && t <= len {
            out.push(Met { t, what: What::World { material: 0, terrain: true, breakable: None }, id: Struck::Body(world::FLAT_BODY), normal: Vec3::Y });
        }
    }
    for (i, c) in pillars().enumerate().filter(|&(i, _)| skip != Some(Struck::Pillar(i))) {
        let (lo, hi) = (c - Vec3::new(0.3, 1.5, 0.3), c + Vec3::new(0.3, 1.5, 0.3));
        let Some(t) = ray_aabb(from, dir, lo, hi).filter(|&t| t <= len) else { continue };
        let at = from + dir * t;
        let d = (at - c) / Vec3::new(0.3, 1.5, 0.3);
        let k = if d.x.abs() >= d.y.abs().max(d.z.abs()) { 0 } else if d.y.abs() >= d.z.abs() { 1 } else { 2 };
        let mut n = Vec3::ZERO;
        n[k] = d[k].signum();
        out.push(Met { t, what: What::World { material: 0, terrain: false, breakable: None }, id: Struck::Pillar(i), normal: n });
    }
    let skip_placed = match skip { Some(Struck::Placed(b)) => Some(b), _ => None };
    for (t, i) in super::scenery::rays_placed(from, dir, len).into_iter().filter(|&(_, i)| Some(i) != skip_placed) {
        out.push(Met { t, what: What::World { material: 0, terrain: false, breakable: Some(i) }, id: Struck::Placed(i), normal: -dir });
    }
    out
}

/// A body of the map's collision a shot's ray met (`Arena::ray_shot`), as a `Met`.
pub(super) fn arena_met(h: world::ShotHit) -> Met {
    Met { t: h.t, what: What::World { material: h.material as i64, terrain: h.terrain, breakable: h.breakable }, id: Struck::Body(h.body), normal: h.normal }
}

/// What the world holds first on a sweep's line within `len` m of `from` (`What::World`), but
/// `skip`: the map's collision less its blockers (`Arena::ray_shot`, with its world material,
/// `ray_material`'s front-face rule); on the flat test floor its own things (`floor_met`) and the
/// pieces of scenery the test tools put on it (the floor's arena).
fn world_met(from: Vec3, dir: Vec3, len: f32, skip: Option<Struck>) -> Option<Met> {
    let arena = world::arena();
    let skip_body = match skip { Some(Struck::Body(b)) => Some(b), _ => None };
    let shot = arena.as_ref().and_then(|a| a.ray_shot(from, dir, len, skip_body)).map(arena_met);
    if arena.as_ref().is_some_and(|a| !a.is_flat()) {
        return shot;
    }
    shot.into_iter().chain(floor_met(from, dir, len, skip, arena.is_none())).min_by(|a, b| a.t.total_cmp(&b.t))
}

/// Every crossing of the world's surfaces a pass-through ray's line makes within `len` m of
/// `from`, nearest first (play_rays.rs): the map's (`Arena::ray_shot_all`: a closed object twice,
/// where the line goes in and where it comes out, #287) and, on the flat test floor, its ground
/// and its own things (`floor_all`).
pub(super) fn world_all(from: Vec3, dir: Vec3, len: f32) -> Vec<Met> {
    let arena = world::arena();
    let mut out: Vec<Met> = arena.as_ref().map(|a| a.ray_shot_all(from, dir, len).into_iter().map(arena_met).collect()).unwrap_or_default();
    if arena.as_ref().is_none_or(|a| a.is_flat()) {
        out.extend(floor_all(from, dir, len, arena.is_none()));
    }
    out.sort_by(|a, b| a.t.total_cmp(&b.t));
    out
}

/// `floor_met` for the multi-hit ray (`world_all`): its ground (with no arena), and each pillar's
/// and placed breakable's box twice, where the line goes in and where it comes out, the way out
/// with the normal of the face it leaves by, turned back along the line as a ray result's is.
fn floor_all(from: Vec3, dir: Vec3, len: f32, ground: bool) -> Vec<Met> {
    let mut out: Vec<Met> = floor_met(from, dir, len, None, ground).into_iter().filter(|m| !matches!(m.id, Struck::Pillar(_) | Struck::Placed(_))).collect();
    for (i, c) in pillars().enumerate() {
        let half = Vec3::new(0.3, 1.5, 0.3);
        let Some((t0, t1)) = ray_aabb_both(from, dir, c - half, c + half) else { continue };
        for (t, out_way) in [(t0, false), (t1, true)].into_iter().filter(|&(t, _)| t > 1e-3 && t <= len) {
            let d = (from + dir * t - c) / half;
            let k = if d.x.abs() >= d.y.abs().max(d.z.abs()) { 0 } else if d.y.abs() >= d.z.abs() { 1 } else { 2 };
            let mut n = Vec3::ZERO;
            n[k] = d[k].signum();
            // (the face's outward normal; on the way out it faces along the line: turned back)
            let n = if out_way { -n } else { n };
            out.push(Met { t, what: What::World { material: 0, terrain: false, breakable: None }, id: Struck::Pillar(i), normal: n });
        }
    }
    for (t, i, _) in super::scenery::crossings_placed(from, dir, len) {
        out.push(Met { t, what: What::World { material: 0, terrain: false, breakable: Some(i) }, id: Struck::Placed(i), normal: -dir });
    }
    out
}

/// Where a ray (from `origin` along unit `dir`) enters the box `lo`..`hi`, if it does ahead of
/// its start (play.rs `ray_hit`'s pillar test).
fn ray_aabb(origin: Vec3, dir: Vec3, lo: Vec3, hi: Vec3) -> Option<f32> {
    ray_aabb_both(origin, dir, lo, hi).map(|(t0, _)| t0).filter(|&t0| t0 > 0.0)
}

/// Where a ray (from `origin` along unit `dir`) enters the box `lo`..`hi` and where it leaves it,
/// if it meets it (the way in at 0 if it starts inside).
fn ray_aabb_both(origin: Vec3, dir: Vec3, lo: Vec3, hi: Vec3) -> Option<(f32, f32)> {
    let (mut t0, mut t1) = (0.0f32, f32::MAX);
    for k in 0..3 {
        if dir[k].abs() < 1e-6 {
            if origin[k] < lo[k] || origin[k] > hi[k] {
                return None;
            }
        } else {
            let (a, b) = ((lo[k] - origin[k]) / dir[k], (hi[k] - origin[k]) / dir[k]);
            t0 = t0.max(a.min(b));
            t1 = t1.min(a.max(b));
        }
    }
    (t0 <= t1).then_some((t0, t1))
}

/// The characters (and their bodies on the ground) and the Sentries a line from `from` along
/// `dir` meets within `len` m, each once, but its shooter `owner`, one out of the world
/// (`Player::absent`) and `skip`: standing, the cylinder (`ray_body`), its normal out from the
/// axis; knocked down, its ragdoll's bones (play_missile.rs's `ray_lying`); dead, its ragdoll
/// (`Ragdoll::first_hit`); a Sentry, its sphere (`MineTargets::first_on`). The lying, the dead
/// and the Sentries are met with the normal straight back along the line: the demo's (no bolt
/// bounces off them: none has the character count, and a Sentry a cutter's blade meets goes off).
pub(super) fn units_met(from: Vec3, dir: Vec3, len: f32, owner: Who, skip: Option<Struck>, units: &[&Player],
                        mines: &super::sentry::MineTargets) -> Vec<Met> {
    let mut out = vec![];
    for u in units.iter().filter(|u| u.who() != owner && !u.absent && skip != Some(Struck::Unit(u.who()))) {
        let id = Struck::Unit(u.who());
        if u.dead {
            if let Some((t, _)) = u.ragdoll.as_ref().and_then(|r| r.first_hit(from, dir, len)).filter(|&(t, _)| t <= len) {
                out.push(Met { t, what: What::Corpse(u.who()), id, normal: -dir });
            }
            continue;
        }
        if knocked_down(u) {
            if let Some(t) = super::missile::ray_lying(from, dir, u, len) {
                out.push(Met { t, what: What::Body(u.who()), id, normal: -dir });
            }
        } else if let Some(t) = ray_body(from, dir, u, len) {
            let at = from + dir * t;
            let normal = Vec3::new(at.x - u.position.x, 0.0, at.z - u.position.z).normalize_or(-dir);
            out.push(Met { t, what: What::Body(u.who()), id, normal });
        }
    }
    out.extend(mines.all_on(from, dir, len).into_iter().filter(|&(e, _)| skip != Some(Struck::Mine(e)))
        .map(|(e, t)| Met { t, what: What::Mine(e), id: Struck::Mine(e), normal: -dir }));
    out
}

/// What a bolt's sweep meets first (see the module notes): from `from` along `dir` within `len`
/// m, the world (`world_met`) and the characters and Sentries (`units_met`), but `skip`, the
/// thing it last passed through.
fn sweep(from: Vec3, dir: Vec3, len: f32, owner: Who, skip: Option<Struck>, units: &[&Player], mines: &super::sentry::MineTargets) -> Option<Met> {
    world_met(from, dir, len, skip).into_iter().chain(units_met(from, dir, len, owner, skip, units, mines))
        .min_by(|a, b| a.t.total_cmp(&b.t))
}

/// A character's name for the log (BF_SHOT_LOG).
pub(super) fn name(u: &Player) -> String {
    format!("{}{}", unit_name(u), u.npc.map_or(String::new(), |n| format!(" (NPC {})", n.id)))
}

/// The bolts in flight, a 30 Hz game frame at a time (shots::TICK; FUN_00225220 ->
/// FUN_00225440): their lives, moves and sweeps, what each sweep meets dealt then, and whether the
/// bolt goes on through it or off it (see the module notes). A bolt that ends, on a hit or with
/// its life, goes this frame.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn fly_bolts(mut commands: Commands, time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut game: ResMut<GameData>,
             mut ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>, (mut images, mut materials): (ResMut<Assets<Image>>, ResMut<Assets<StandardMaterial>>),
             mut bolts: Query<(Entity, &mut Bolt, &mut Transform)>,
             (mut chance, mut holes, mut mines, mut objects, mut sweeps): (ResMut<GameRandom>, ResMut<super::fx::HoleRequests>,
                                                                           ResMut<super::sentry::MineTargets>, ResMut<super::scenery::ObjectHits>, ResMut<Sweeps>),
             difficulty: Res<Difficulty>, test: Option<Res<super::testmap::TestMap>>, camera: Query<&GlobalTransform, With<MainCamera>>) {
    sweeps.0.clear();
    let dt = frame_dt(&time);
    let log = std::env::var("BF_SHOT_LOG").is_ok();
    let listener = camera.iter().next().map(|c| c.translation());
    let instant_kill = test.as_ref().is_some_and(|t| t.instant_kill);
    for (e, mut b, mut tr) in &mut bolts {
        b.clock += dt;
        let mut gone = false;
        while b.clock >= shots::TICK && !gone {
            b.clock -= shots::TICK;
            let (from, len, step) = match b.advance() {
                Advance::Out => {
                    if log {
                        println!("t={:.2} bolt {} ({}) ends at its life's end after {} game frames, {:.1} m from its muzzle", player.sim_time, b.id,
                                 game.0.weapons.get(&b.shot.weapon).map_or("?", |d| d.label.as_str()), b.ticks, b.pos.distance(b.shot.origin));
                    }
                    gone = true;
                    break;
                }
                Advance::Moved => continue,
                Advance::Sweep { from, len, step } => (from, len, step),
            };
            let dir = b.dir;
            // (what it last passed through is left out of this sweep only)
            let skip = b.skip.take();
            let met = {
                let units: Vec<&Player> = std::iter::once(&*player).chain(squad.0.iter()).collect();
                sweep(from, dir, len, b.owner, skip, &units, &mines)
            };
            let reach = met.map_or(len, |m| m.t);
            sweeps.0.push((from, dir, reach));
            // (a liquid's surface on the way: nothing, the blast pool has no liquid code; an
            // instant ray's splash is play.rs `projectiles`', todo/146-liquids leads §5.1)
            let Some(met) = met else {
                b.pos += dir * step;
                b.last = b.pos;
                continue;
            };
            // met: set back from the hit toward the last checked point, which is set there
            let at = from + dir * met.t;
            b.pos = at - (at - from) * HIT_BACK;
            b.last = b.pos;
            let now = player.sim_time;
            let strike = Strike { shot: &b.shot, owner: b.owner, team: b.team, human: b.human, roll: b.roll, skill: b.skill, power: b.power, falloff: 1.0 };
            let what = deal(&mut commands, &mut game.0, ale.as_deref_mut(), &mut images, &mut materials, &strike, met, from, dir,
                            &mut player, &mut squad, &mut chance, &mut holes, &mut mines, &mut objects, &difficulty, listener, instant_kill);
            let outcome = b.after_hit(met.what.kind(), met.id, met.normal);
            if log {
                // (a level blocker its sweep passed, which a shot met before #136)
                let blocker = world::arena().and_then(|a| a.ray(from, dir, met.t)).filter(|&t| t < met.t - 1e-3)
                    .map_or(String::new(), |t| format!(" (past a level blocker {t:.2} m on)"));
                let next = match outcome {
                    Outcome::Ends => String::new(),
                    Outcome::Passes => format!("; passes through it (n1 {} n2 {} left)", b.n1, b.n2),
                    Outcome::Bounces => format!("; bounces off it along {:.3} (n1 {} n2 {} left)", b.dir, b.n1, b.n2),
                };
                println!("t={now:.2} bolt {} sweep at game frame {} ({:.2} s): {what} {:.1} m on from its last checked point {:.1} m out{blocker}{next}", b.id,
                         b.ticks, b.ticks as f32 * shots::TICK, met.t, at.distance(b.shot.origin));
            }
            match outcome {
                Outcome::Ends => gone = true,
                // (turned along its new way: its flight effect rides it)
                Outcome::Bounces => {
                    let way = b.dir;
                    tr.look_to(way, Vec3::Y);
                }
                Outcome::Passes => {}
            }
        }
        if gone {
            commands.entity(e).despawn();
            continue;
        }
        // (drawn where it is by now: the time not yet run in ticks carries it on, so it doesn't
        // stand still every other frame at 60 fps)
        tr.translation = b.pos + b.dir * (b.speed * b.clock);
    }
}

/// One hit to deal through the damage handler 0x22d0b0 (`deal`): the shot that made it, who fired
/// it (their team, and whether the controlled character), the hit record's roll and the shooter's
/// skill's and power-ups' factors on it, and an instant ray's falloff over its range (1 for a
/// bolt).
pub(super) struct Strike<'a> {
    pub shot: &'a Shot,
    pub owner: Who,
    pub team: u8,
    pub human: bool,
    pub roll: f32,
    pub skill: f32,
    pub power: f32,
    pub falloff: f32,
}

/// A hit `met` on a line from `from` along `dir` dealt (the damage handler 0x22d0b0, see the
/// module notes): a bolt's on its sweep, a pass-through ray's at its shot (play_rays.rs). What it
/// was, for the log.
#[allow(clippy::too_many_arguments)]
pub(super) fn deal(commands: &mut Commands, game: &mut Game, ale: Option<&mut bf_viewer::ale_fx::AleAssets>, images: &mut Assets<Image>,
                   materials: &mut Assets<StandardMaterial>, k: &Strike, met: Met, from: Vec3, dir: Vec3, player: &mut Player, squad: &mut Squad,
                   chance: &mut GameRandom, holes: &mut super::fx::HoleRequests, mines: &mut super::sentry::MineTargets,
                   objects: &mut super::scenery::ObjectHits, difficulty: &Difficulty, listener: Option<Vec3>, instant_kill: bool) -> String {
    let s = k.shot;
    let at = from + dir * met.t;
    let now = player.sim_time;
    // (the record: the roll x the shooter's skill and power-ups x the falloff; a character's adds
    // its part)
    let record = k.roll * k.skill * k.power * k.falloff;
    // the bullet's own explosion where it strikes a body, a corpse or a Sentry (0x22d0b0 ->
    // FUN_0022cfc0, `shot_explosion`: the cutters' white star; a world hit's is world_hit's)
    let mut ale = ale;
    if !matches!(met.what, What::World { .. }) {
        if let Some(ale) = ale.as_deref_mut() {
            let hit = Shot { origin: from, dir, dist: met.t, hit: true, normal: met.normal, ..*s };
            shot_explosion(commands, game, ale, images, materials, &hit, at, met.normal, chance, listener, player);
        }
    }
    match met.what {
        What::World { material, terrain, breakable } => {
            // the bullet's hole, the material's hit effect, debris and sound, a breakable
            // object's damage
            if s.decal != 0 {
                holes.0.push((s.decal, at, met.normal));
            }
            let hit = Shot { origin: from, dir, dist: met.t, hit: true, normal: met.normal, ..*s };
            if let Some(ale) = ale {
                world_hit(commands, game, ale, images, materials, &hit, at, chance, listener, player, Some((record, material)));
            }
            if let Some(target) = breakable {
                objects.0.push(super::scenery::ObjectHit { target, amount: record, damage_type: s.damage_type, at, dir });
            }
            format!("hits the world ({}{}, material {material})", if terrain { "the terrain" } else { "an object" },
                    breakable.map_or(String::new(), |b| format!(", breakable {b}")))
        }
        What::Body(who) => {
            let Some((u, controlled)) = unit_mut(player, squad, who) else { return "hits a body that's gone".into() };
            // the struck part (FUN_00224320 asks FUN_00125940), x0.7 on the shooter's own side
            let bone = damage::struck_bone(u, game, from, dir, met.t);
            let hit = damage::Hit::new(k.roll, "roll", s.damage_type).skill(k.skill).power(k.power).by(k.owner, k.team, u).falloff(k.falloff)
                .struck(u, game, bone).push(game.weapons.get(&s.weapon).map_or(0.0, |w| w.push));
            let landing = PendingHit { delay: 0.0, member: 0, hit, dir, local: at - u.position, ammo: s.ammo };
            let lying = knocked_down(u);
            // (the player's shot's rules on anyone but the controlled character: control may have
            // passed while it flew)
            if k.human && !controlled {
                land_player_shot(u, &landing, game, difficulty, now, instant_kill);
            } else {
                land_ai_shot(u, &landing, game, difficulty, now);
            }
            // (one knocked down is shoved too, as a shot's line shoves it, play.rs)
            if let Some(kd) = u.knock.as_mut() {
                kd.ragdoll.shot(from, dir, met.t);
            }
            format!("hits {}{}", name(u), if lying { " (knocked down)" } else { "" })
        }
        What::Corpse(who) => {
            let Some((u, _)) = unit_mut(player, squad, who) else { return "hits a body that's gone".into() };
            if let Some(hit) = u.ragdoll.as_mut().and_then(|r| r.shot(from, dir, met.t)) {
                u.blood.push(Bleed::shown(hit, dir, s.ammo));
            }
            format!("hits {}'s body", name(u))
        }
        What::Mine(e) => {
            // (at once: the distance is for its log, at no time a metre)
            mines.strike(e, met.t, f32::MAX, now);
            "hits a Sentry".into()
        }
    }
}

/// This frame's bolts launched (FUN_0022bac0): each shooter's new shots of a bolt gun become bolts
/// at the muzzle with their flight effect riding on them (play.rs `start_flight`), their damage
/// roll made now (GameRandom, FUN_0022c940's draw) with the shooter's skill's and power-ups'
/// factors, and their bullet's hit counts and pass flag (#238).
#[allow(clippy::too_many_arguments)]
fn launch_bolts(mut commands: Commands, mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut game: ResMut<GameData>,
                mut ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>, mut images: ResMut<Assets<Image>>,
                mut materials: ResMut<Assets<StandardMaterial>>, mut chance: ResMut<GameRandom>, mut state: ResMut<BoltState>) {
    let log = std::env::var("BF_SHOT_LOG").is_ok();
    let now = player.sim_time;
    for (k, u) in std::iter::once(&mut *player).chain(squad.0.iter_mut()).enumerate() {
        let new: Vec<Shot> = u.shots.iter().filter(|s| s.bolt).copied().collect();
        for s in new {
            let (lo, hi) = shot_rolls(&s);
            let roll = lo + (hi - lo) * chance.unit();
            let skill = skills::roll_factor(u);
            let power = super::powerups::roll_factor(u);
            let speed = s.speed.max(1.0);
            let life = s.range / speed;
            // (FUN_0022bac0 copies the ints' low bytes: pbVar2[1] = type+0x23c, [2] = +0x240)
            let data = game.0.weapons.get(&s.weapon).map(|d| d.shots.clone()).unwrap_or_default();
            let (n1, n2) = (data.object_hits as u8, data.character_hits as u8);
            state.next += 1;
            let b = Bolt { shot: s, id: state.next, owner: u.who(), team: u.team, human: k == 0, pos: s.origin, dir: s.dir, speed, life,
                           counter: SWEEP_TICKS, last: s.origin, roll, skill, power, n1, n2, pass: data.pass_through, skip: None, clock: 0.0, ticks: 0 };
            if log {
                println!("t={now:.2} bolt {} launched by {} from {:.2} along {:.3} at {speed} m/s: roll {roll:.2} x skill {skill:.2} x power-up {power:.2}, life {life:.3} s (range {:.1} m), n1 {n1} n2 {n2}{}",
                         b.id, name(u), s.origin, s.dir, s.range, if b.pass { " passing through" } else { "" });
            }
            let e = commands.spawn((Transform::from_translation(s.origin).looking_to(s.dir, Vec3::Y), Visibility::default(), b, Name::new("bolt"))).id();
            // (its flight effect's seed: a draw of the shots' random state)
            let seed = chance.stirred();
            if let Some(ale) = ale.as_deref_mut() {
                start_flight(&mut commands, &mut game.0, ale, &mut images, &mut materials, s.flight, e, seed);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A bolt as `launch_bolts` makes one: `speed` m/s along -z from the origin, its life `range`
    /// / `speed`, the hit counts `n1`, `n2` and the pass flag.
    fn bolt(speed: f32, range: f32, n1: u8, n2: u8, pass: bool) -> Bolt {
        let shot = Shot { weapon: 0, origin: Vec3::ZERO, dir: Vec3::NEG_Z, dist: range, hit: false, speed, flight: 0, hit_fx: 0, damage: [1.0, 2.0],
                          damage_type: 0, ammo: 0, decal: 0, normal: Vec3::Z, falloff: 0, range, bolt: true, body: false, pass: false, scoped: false };
        Bolt { shot, id: 1, owner: Who::Squad(3), team: 0, human: true, pos: Vec3::ZERO, dir: Vec3::NEG_Z, speed, life: range / speed,
               counter: SWEEP_TICKS, last: Vec3::ZERO, roll: 1.0, skill: 1.0, power: 1.0, n1, n2, pass, skip: None, clock: 0.0, ticks: 0 }
    }

    /// The game frames a bolt that meets nothing sweeps on, each sweep's start (m from the
    /// muzzle) and length, and the frame its life ends on.
    fn flight(b: &mut Bolt) -> (Vec<(u32, f32, f32)>, u32) {
        let mut sweeps = vec![];
        loop {
            match b.advance() {
                Advance::Out => return (sweeps, b.ticks),
                Advance::Moved => {}
                Advance::Sweep { from, len, step } => {
                    sweeps.push((b.ticks, from.length(), len));
                    // (nothing met: on, and checked up to here)
                    b.pos += b.dir * step;
                    b.last = b.pos;
                }
            }
        }
    }

    /// #136: a bolt sweeps on game frames 3, 6, 9 ..., each sweep over the three frames' way since
    /// the last (8 m at the Jax-iC's 80 m/s), and its life (60 / 80 = 0.75 s) ends it on frame 23
    /// with no sweep past frame 21's, which reaches 56 m (FUN_00225220 / FUN_00225440).
    #[test]
    fn sweeps_every_third_frame_until_its_life_is_out() {
        let (sweeps, end) = flight(&mut bolt(80.0, 60.0, 0, 0, false));
        assert_eq!(end, 23);
        assert_eq!(sweeps.iter().map(|s| s.0).collect::<Vec<_>>(), vec![3, 6, 9, 12, 15, 18, 21]);
        for (k, &(_, from, len)) in sweeps.iter().enumerate() {
            assert!((from - 8.0 * k as f32).abs() < 1e-3, "sweep {k} from {from}");
            assert!((len - 8.0).abs() < 1e-3, "sweep {k} over {len}");
        }
    }

    /// A life that's a whole number of frames ends on its last one (LIFE_SLACK): the LZR-10's 80
    /// m at 100 m/s, 24 frames, its last sweep on frame 21.
    #[test]
    fn a_whole_frames_life_ends_on_its_last_frame() {
        let (sweeps, end) = flight(&mut bolt(100.0, 80.0, 0, 0, false));
        assert_eq!((end, sweeps.last().map(|s| s.0)), (24, Some(21)));
    }

    /// #238: a cutter's blade (n1 5, n2 0, no pass) bounces off five objects, its way turned
    /// about each normal, and ends on the sixth; it ends on the terrain and on a character at
    /// once; a bolt with no counts ends at its first hit.
    #[test]
    fn a_cutters_blade_bounces_off_five_objects() {
        let mut b = bolt(120.0, 75.0, 5, 0, false);
        let wall = Vec3::new(0.0, 0.0, 1.0);
        for k in 0..5 {
            b.dir = Vec3::new(0.6, 0.0, -0.8);
            assert_eq!(b.after_hit(Kind::Object, Struck::Body(7), wall), Outcome::Bounces, "bounce {k}");
            assert!((b.dir - Vec3::new(0.6, 0.0, 0.8)).length() < 1e-5, "turned about the normal: {:?}", b.dir);
            assert_eq!(b.skip, None);
        }
        assert_eq!(b.after_hit(Kind::Object, Struck::Body(7), wall), Outcome::Ends);
        assert_eq!(bolt(120.0, 75.0, 5, 0, false).after_hit(Kind::Terrain, Struck::Body(0), wall), Outcome::Ends);
        assert_eq!(bolt(120.0, 75.0, 5, 0, false).after_hit(Kind::Character, Struck::Unit(Who::Squad(0)), wall), Outcome::Ends);
        assert_eq!(bolt(80.0, 60.0, 0, 0, false).after_hit(Kind::Object, Struck::Body(7), wall), Outcome::Ends);
    }

    /// #238: without the pass flag a hit on an object with n1 out takes n2 instead (FUN_00225440
    /// at 0x225b0a), and the terrain and a character take n2.
    #[test]
    fn a_bounce_takes_n2_once_n1_is_out() {
        let mut b = bolt(100.0, 80.0, 0, 2, false);
        assert_eq!(b.after_hit(Kind::Object, Struck::Body(3), Vec3::Z), Outcome::Bounces);
        assert_eq!(b.after_hit(Kind::Terrain, Struck::Body(1), Vec3::Y), Outcome::Bounces);
        assert_eq!((b.n1, b.n2), (0, 0));
        assert_eq!(b.after_hit(Kind::Character, Struck::Unit(Who::Squad(1)), Vec3::Z), Outcome::Ends);
    }

    /// #238: the Ion Beamer's bolt (pass, n1 5, n2 0) passes through five objects, each left out
    /// of the next sweep, and stops at the sixth, at a character and at the terrain.
    #[test]
    fn the_ion_beamers_bolt_passes_through_five_objects() {
        let mut b = bolt(100.0, 90.0, 5, 0, true);
        for k in 0..5u32 {
            assert_eq!(b.after_hit(Kind::Object, Struck::Body(k), Vec3::Z), Outcome::Passes);
            assert_eq!(b.skip, Some(Struck::Body(k)));
            assert_eq!(b.dir, Vec3::NEG_Z, "it keeps its way");
        }
        assert_eq!(b.after_hit(Kind::Object, Struck::Body(9), Vec3::Z), Outcome::Ends);
        assert_eq!(bolt(100.0, 90.0, 5, 0, true).after_hit(Kind::Character, Struck::Unit(Who::Squad(2)), Vec3::Z), Outcome::Ends);
        assert_eq!(bolt(100.0, 90.0, 5, 0, true).after_hit(Kind::Terrain, Struck::Body(0), Vec3::Y), Outcome::Ends);
    }
}

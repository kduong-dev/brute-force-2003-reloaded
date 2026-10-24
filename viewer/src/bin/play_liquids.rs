//! A character in a liquid (#152): what a liquid does to the bodies in it, as default.xbe does it
//! (todo/146-liquids leads.md §2-§4, answer-spec.md Q1 and Q9, spec.md Part 4; the code map's
//! Levels rows FUN_00221af0-FUN_00222400), checked against #146's takes 01-03 (Brutus walking
//! into sdm_e13's lava, HP 75 and 5000, logged at 20 Hz) and #28's take08 (Tex running into the
//! tutorial's lava pit on STANDARD). And what a liquid plays (#5): its splashes, rings and sounds
//! (todo/5-liquid-sounds leads.md §0-§2; the code map's Sound rows FUN_00179250-FUN_0017f580).
//!
//!  - **Who's in.** The liquid's phantom (FUN_002218e0) reports each hit-part body of a living
//!    character (PID_BONE: the eleven bones of its motion set's `<collision>` list; the movement
//!    capsule never pairs with it, FUN_00169a30) as it enters or leaves the liquid's collision, a
//!    closed volume under the surface. FUN_00221af0 maps the body to its part (FUN_00115000) and
//!    keeps a wade entry per part (FUN_00222310 / FUN_00222400), and FUN_00220e30 counts them
//!    (char+0x104). Here a part is in while the lowest corner of its collision surface, posed as
//!    drawn (`damage::part_bodies`), is inside the volume (`Arena::liquid_inside`); a body without
//!    parts counts as one at its feet.
//!  - **The damage.** Only the liquid type's character rate, h_e0489ea0 (`LIQUID_CHARACTER_RATE`:
//!    lava 30, sludge 50, toxic waste 30, the waters and chloride 0): a frame on which one of a
//!    living character's parts enters queues one hit of the whole rate (0x221b98), any frame with a
//!    part in queues the rate x dt (FUN_00221ec0, 0x2222c7), and the queue holds one hit per
//!    object a frame (FUN_00221a40), so an entering frame deals the rate once. No depth (nothing
//!    measures it), no tick, nothing more for going under: fully under, the parts are still in the
//!    volume and the rate runs on (take03: 30.2 HP/s 2.9-3 m under, alive 9 s).
//!  - **The hit.** Each queued hit, an entry's and each frame's share alike, goes to the
//!    character's damage handler as DTYPE_FLAME, no attacker (0x221780 -> 0x125600; play.rs
//!    `burn`, through `take_hit`: a stamina shield takes it first, nothing in a cutscene): the
//!    difficulty step (STANDARD's squad x0.6: take08's 18 a hit and 18 a second; SDM none: the
//!    takes' 30), Vengar's 0.85 and the type's factor; no hit part. It floors nobody (take03:
//!    thirteen entry hits of 30, Brutus walking on), and draws no blood but FUN_00124a70's
//!    decals, which the entry hits are big enough for and the per-frame shares never
//!    (play_fx.rs). Only an entry hit may draw a hurt line (the demo's: take03 has no sound
//!    between the foot events while Brutus stands burning, todo/5-liquid-sounds leads §4.1).
//!  - **Splashes** (#5, FUN_00221af0 0x221ca9-0x221dc2). A part going in casts its surface ray
//!    (FUN_00222480: from the top of its bounding sphere, `Character::part_spheres`, 2r straight
//!    down against that liquid, `Arena::liquid_below`). Meeting the surface, the character keeps
//!    its height (char+0x414, `Wading::surface`) and the part splashes there, big (the type's
//!    third effect and sound) for a radius of at least h_e12e7b9d (0.1 m: every part of the
//!    squad's, 0.14-0.45 m) else small; at any speed, a walk, a jump or a fall alike. A part whose
//!    sphere doesn't reach the surface as it goes in (a calf in ankle-deep lava, a part going in
//!    under it) splashes nothing (take03: the first four parts in without a surface hit). The
//!    sound is positional at the surface point, at volume 1.0 with a pitch of U x 0.1 - 0.05
//!    octaves (`splash`; play.rs `WorldSound`, heard to 15 m).
//!  - **Rings** (FUN_00221ec0). Every RING_EVERY s at most, each wade entry casts its part's ray
//!    again: where its surface point has moved more than h_049350bb (0.1 m) from its last ring's
//!    (the part's node at its entry, at first), and not straight up or down, a ring (the fourth
//!    effect) there, facing the move, at user parameter clamp((moved - 0.1) / dt x h_fd22997d, 0,
//!    1). Silent.
//!  - **Wading steps** (FUN_00221ec0 0x221fd9, FUN_00119020): each game frame a part is in, the
//!    character takes the liquid's fourth sound as its next step's (char+0x410, `Wading::sound`);
//!    play.rs's foot events ask `step`: the wading step when the surface of the last splash is
//!    over 0.25 m above the foot, no step sound at all when it isn't, the ground's only out of
//!    the liquids. Nothing plays for going under, staying under, getting out or burning
//!    (todo/5-liquid-sounds leads §2.5).
//!  - **Objects** (FUN_00221af0's OBJECT bodies: grenades, play_grenade.rs, and missiles,
//!    play_missile.rs; `object_enters`). One going in, when it wasn't in a liquid already, at
//!    h_1f6de6f9 (0.4 m/s) or faster, splashes by its model's bounding radius (the grenades' 0.081
//!    m small, the Sentry's 0.33 m big). A shot's surface splash is play.rs `projectiles`'s, for
//!    instant rays only.
//!  - Falls and landings are play.rs's: a fall into a liquid hurts by the drop to its bed
//!    (FUN_0012b740 has no liquid check), and the landing's sound is left out while a part is in
//!    one (0x12d842).
//!
//! Not done: the ragdoll's parts (the demo's ragdoll has no part bodies: a body knocked down or
//! dead splashes nothing, where take01's corpse splashed as its parts met the surface); the wet
//! footprints (char+0x418); the idle animation's foot events (take03's step every ~2.1 s standing
//! waist-deep: the demo's steps come from feet planting on the move, play.rs `step_player`);
//! debris, pickups and bodies going in; the sound manager's queue and its budget of 16 positional
//! voices a frame (FUN_001793e0: every sound here plays).
//!
//! Test hooks: BF_LIQUID_LOG=1 prints each character's parts in a liquid as the count changes, its
//! entry hits, its splashes, rings and wading steps, and its hit points each second in one;
//! BF_SOUND_LOG=1 prints each liquid sound with its volume, pitch and distance (play.rs
//! `play_sounds`); BF_TEST_WALK=1 makes BF_TEST_GOTO walk, as the takes' half-stick (play.rs
//! `autopilot`).

use super::*;
use bf_viewer::bf::character::{LiquidType, LIQUID_BIG_SPLASH, LIQUID_CHARACTER_RATE, LIQUID_SPLASH, LIQUID_WADE};
use damage::PartBody;

/// DTYPE_FLAME, the damage type the liquid's update gives its hits (the record's +4, 0x221780;
/// the DTYPE_ table's 5): what a character type's factors are looked up by (FUN_002232d0).
const DTYPE_FLAME: i64 = 5;

/// FUN_00221ec0: a wade entry rings at most this often (s; 0x3a4f90, against the liquid's clock).
const RING_EVERY: f32 = 0.2;

/// FUN_00221af0 (0x221d77-0x221dc2): a splash's sound plays at this volume (its block's +0x48,
/// set: it replaces the bank's 0.5) ...
const SPLASH_VOLUME: f32 = 1.0;
/// ... with a pitch of U x the first - the second (octaves; 0x3a4ff0, 0x3a4f94; FUN_0017f580's
/// unit): +-0.05 octave.
const SPLASH_PITCH: (f32, f32) = (0.1, 0.05);

/// FUN_00119020 (0x1191bb): a step in a liquid makes no sound while the surface of the last splash
/// is this high or less above the foot (m; 0x3a5028).
const WADE_DEPTH: f32 = 0.25;
/// FUN_00119020 (0x119256): the controlled character's own step's share of its volume (0x3a510c).
const OWN_STEP: f32 = 0.75;
/// FUN_00119020 (0x119290-0x1192a4): a step's pitch, U x the first - the second (octaves;
/// 0x3a5154, 0x3a5478): +-0.35 octave.
const STEP_PITCH: (f32, f32) = (0.7, 0.35);

/// char+0x414 before a splash has found a surface: -1 (#146's take03 watch log, `surf` -1 until
/// the first surface hit at 9.90 s). Nothing resets it after.
const NO_SURFACE: f32 = -1.0;

/// A character's hit parts in the liquids: the game's wade entries for them (FUN_00222310 /
/// FUN_00222400) and their count (char+0x104, `in_liquid`), as of the last game frame the
/// liquids ran (`run_liquids`), and the two fields the footsteps read.
#[derive(Clone, Debug)]
pub struct Wading {
    /// each part (its bone; 0 for a body without parts, at its feet) and the liquid it's in (its
    /// index among the level's liquids, `Arena::liquid_inside`)
    pub parts: Vec<(u32, Option<u32>)>,
    /// each part's body at the end of the last frame drawn (`damage::part_bodies`): where a game
    /// frame between two drawn ones finds it (`run_liquids`)
    bodies: Vec<PartBody>,
    /// the wade entries of the parts in a liquid, with their last rings (`Entry`)
    entries: Vec<Entry>,
    /// char+0x410: the liquid's wading step (its fourth sound), written each game frame a part is
    /// in (0x221fd9) and taken by the next foot event (`step`); 0 none
    sound: u32,
    /// char+0x414: the surface height the last splash's ray found (0x221cc6), NO_SURFACE before
    /// one has; kept after the character comes out
    surface: f32,
    /// time (s) since the last game frame the liquids ran (`TICK`s)
    since: f32,
    /// (BF_LIQUID_LOG: the game time of the last hit points line)
    logged: f32,
}

impl Default for Wading {
    fn default() -> Self {
        Wading { parts: vec![], bodies: vec![], entries: vec![], sound: 0, surface: NO_SURFACE, since: 0.0, logged: 0.0 }
    }
}

impl Wading {
    /// How many of its parts are in a liquid (char+0x104).
    pub fn count(&self) -> usize {
        self.parts.iter().filter(|p| p.1.is_some()).count()
    }
}

/// A part's wade entry (FUN_00222310: made as the part goes in, gone as it comes out): its bone,
/// the liquid, and its last ring for FUN_00221ec0 (where: the part's node as it went in, then each
/// ring's surface point, the entry's +0xc; when, in game time: its +0x1c, the liquid's clock).
#[derive(Clone, Copy, Debug)]
struct Entry {
    bone: u32,
    liquid: u32,
    ring_at: Vec3,
    ring_time: f32,
}

/// Whether any of `u`'s parts is in a liquid (char+0x104 > 0): what the game's landing tests
/// (0x12d842) and its footsteps (FUN_00119020).
pub fn in_liquid(u: &Player) -> bool {
    u.wading.count() > 0
}

/// Whether `p` is in a liquid that burns characters: one whose character rate h_e0489ea0 is above
/// 0 (lava 30, sludge 50, toxic waste 30; not the waters or chloride, whose rates are 0; the
/// objects' rate doesn't count, which is 0 on toxic waste).
pub fn burns_at(a: &world::Arena, p: Vec3) -> bool {
    a.liquid_inside(p).is_some_and(|(_, _, t)| t.damage[LIQUID_CHARACTER_RATE] > 0.0)
}

/// How far below its feet (m) a character's hit parts reach, standing (its calves' collision
/// surfaces round the soles): ground within this of a liquid's surface puts a part in it. The
/// demo's, from what it measures: Brutus walking into sdm_e13's lava has his first part in with
/// his feet 0.10 m over the surface (#146's take01 has the game's at 0.05 m under it).
const PART_REACH: f32 = 0.15;

/// How far ahead (m) a wingman looks for ground in a liquid that burns before setting off toward
/// its place (`burning_ahead`): the ledge probe's points (play.rs LEDGE_PROBE).
const BURNING_AHEAD: [f32; 3] = [0.4, 0.8, 1.2];

/// Whether the AI's character `u`, going `dir` (unit, level), would step into a liquid that burns
/// it within BURNING_AHEAD (#1): `steps_into_burning` at each of those points. The game's AI has
/// no liquid code (no AI routine reads the liquid state or the liquid objects, todo/146-liquids
/// leads §4.2); what keeps its squad and its enemies out is their routes over the level's world
/// points, of which no harmful liquid has one with its floor under the surface on the SDM maps
/// (sdm_e13's lava 0 of the 536 over it, 2 at a 1.3 m eye offset; sdm_m03's sludge 0 of 141;
/// sdm_m07's lava 0 of 284, 1), where water has many (sdm_e10's pond 87-272). The demo's AI
/// doesn't walk those routes yet, so this and play.rs's refusal of such a step stand in for them,
/// as its ledge probe does for their keeping off drops (#264): a wingman waits at the shore, an
/// enemy or a cutscene's walk stops there. Whether the game's squad follows its leader in when he
/// walks into lava (a direct move to him) isn't traced: #146's shot A4, not filmed.
pub fn burning_ahead(a: &world::Arena, u: &Player, dir: Vec2) -> bool {
    let feet = u.position.y + GROUND + u.height.max(0.0);
    let at = Vec2::new(u.position.x, u.position.z);
    BURNING_AHEAD.iter().any(|&d| steps_into_burning(a, at, feet, at + dir * d))
}

/// Whether the AI's step from `from` (its feet at `feet`) to `to` is onto ground in a liquid that
/// burns (#1; see `burning_ahead`, PART_REACH): from outside one, any; from inside, one that isn't
/// higher than where it is, so it can still climb out. play.rs undoes such a step, as it undoes
/// one off a ledge.
pub fn steps_into_burning(a: &world::Arena, from: Vec2, feet: f32, to: Vec2) -> bool {
    let inside = burns_at(a, Vec3::new(from.x, feet - PART_REACH, from.y));
    a.floor_at(to.x, to.y, feet + STEP_UP).is_some_and(|(g, _, _)| (!inside || g <= feet) && burns_at(a, Vec3::new(to.x, g - PART_REACH, to.y)))
}

/// The liquids on the characters, once they have moved and been posed, before the frame's sounds
/// and effects are played.
pub fn plugin(app: &mut App) {
    app.add_systems(Update, run_liquids.after(update_player).before(play_sounds).before(spawn_splashes).run_if(in_state(AppState::Playing)));
}

/// What the liquids do to each character and what they play (see the module notes), a game frame
/// at a time: the liquid's update runs once a game frame (`shots::TICK`, 30 Hz), and one hit a
/// frame is the rule, so a body's parts that enter on one frame give one hit and on two give two
/// (take08 of #28: Tex's fall into the pit, 5 hits of 18 over 0.27 s; the demo the same at 30 Hz
/// steps). A game frame between two drawn ones finds the parts' bodies between where the two put
/// them (linear), so BF_CAPTURE's 15 Hz steps and a 60 Hz screen deal what the game's 30 Hz does.
/// Nothing while the game's clock is stopped (`frame_dt`).
fn run_liquids(time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>,
               difficulty: Option<Res<damage::Difficulty>>) {
    let dt = frame_dt(&time);
    let (Some(a), Some(difficulty)) = (world::arena(), difficulty) else { return };
    if dt <= 0.0 {
        return;
    }
    let log = std::env::var("BF_LIQUID_LOG").is_ok();
    let now = player.sim_time;
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        if u.dead || u.absent || u.stand_in {
            // (its wade entries go with its parts; char+0x414 stays)
            let surface = u.wading.surface;
            u.wading = Wading { surface, ..Default::default() };
            continue;
        }
        // its parts' bodies now (a body knocked down keeps where its parts were: the demo's
        // ragdoll has no part bodies to test); without parts, one at its feet, with no sphere
        let bodies = if u.knock.is_some() {
            u.wading.bodies.clone()
        } else {
            let bodies = damage::part_bodies(u, &game.0);
            if bodies.is_empty() {
                let feet = u.position + Vec3::Y * (GROUND + u.height.max(0.0));
                vec![PartBody { bone: 0, low: feet, node: feet, centre: feet, radius: -1.0 }]
            } else {
                bodies
            }
        };
        let was = std::mem::take(&mut u.wading.bodies);
        u.wading.since += dt;
        while u.wading.since >= shots::TICK && !u.dead {
            u.wading.since -= shots::TICK;
            // this game frame's place in the drawn frame (1: its end), and the parts there
            let k = (1.0 - u.wading.since / dt).clamp(0.0, 1.0);
            let here: Vec<PartBody> = bodies.iter().map(|b| was.iter().find(|w| w.bone == b.bone).map_or(*b, |w| w.lerp(b, k))).collect();
            game_frame(u, &game.0, &a, &difficulty, &here, now - u.wading.since, log);
        }
        u.wading.bodies = bodies;
        if log && u.wading.count() > 0 && now - u.wading.logged >= 1.0 {
            u.wading.logged = now;
            println!("t {now:.2}: {} in a liquid, {} parts: {:.2} hp, at {:.2}", unit_name(u), u.wading.count(), u.health,
                     u.position + Vec3::Y * (GROUND + u.height.max(0.0)));
        }
    }
}

/// One game frame of the liquids on `u`, its parts' bodies now `bodies`, at game time `at`:
///  - the enter handler (FUN_00221af0) for each part gone into a liquid: its wade entry and its
///    splash (`part_splash`); a part come out loses its entry, silently (FUN_00221de0);
///  - per liquid it has a part in, the whole rate if a part entered it this frame (a part in it
///    that wasn't), else the rate's share of the frame (FUN_00221af0, FUN_00221ec0, 0x221610);
///  - the wade list's update (FUN_00221ec0, `wade`): the wading step and the rings.
fn game_frame(u: &mut Player, game: &Game, a: &world::Arena, difficulty: &damage::Difficulty, bodies: &[PartBody], at: f32, log: bool) {
    let parts: Vec<(u32, Option<u32>)> = bodies.iter().map(|b| (b.bone, a.liquid_inside(b.low).map(|l| l.0))).collect();
    let mut liquids: Vec<(u32, bool)> = vec![];
    for (b, &(bone, l)) in bodies.iter().zip(&parts) {
        let Some(l) = l else { continue };
        let entered = !u.wading.parts.iter().any(|&(x, was)| x == bone && was == Some(l));
        match liquids.iter_mut().find(|x| x.0 == l) {
            Some(x) => x.1 |= entered,
            None => liquids.push((l, entered)),
        }
        if entered {
            // (FUN_00222310: the entry starts its rings from the part's node, now)
            u.wading.entries.retain(|e| e.bone != bone);
            u.wading.entries.push(Entry { bone, liquid: l, ring_at: b.node, ring_time: at });
            part_splash(u, a, l, b, at, log);
        }
    }
    u.wading.entries.retain(|e| parts.iter().any(|&(b, l)| b == e.bone && l == Some(e.liquid)));
    let before = u.wading.count();
    u.wading.parts = parts;
    for (l, entered) in liquids {
        let Some((kind, rate)) = a.liquid(l).map(|t| (t.kind, t.damage[LIQUID_CHARACTER_RATE])) else { continue };
        if rate <= 0.0 || u.dead {
            continue;
        }
        // (each hit through the character's damage handler, the entry's and each frame's share
        // alike: 0x221780 -> 0x125600 every game frame; `burn`)
        let mut hit = damage::Hit::new(if entered { rate } else { rate * shots::TICK }, "liquid", DTYPE_FLAME);
        let amount = if entered { hit.land(u, game, difficulty, at) } else { hit.on(u, game, difficulty) };
        let point = u.position + Vec3::Y * GROUND;
        burn(u, game, amount, point, entered);
        if log && entered {
            println!("t {at:.2}: {} {} of its parts in liquid {l} (type {kind}): a part in, hit {amount:.2}: {:.2} hp", unit_name(u),
                     u.wading.count(), u.health);
        }
        if log && u.dead {
            println!("t {at:.2}: {} dies in liquid {l} (type {kind}), {} of its parts in", unit_name(u), u.wading.count());
        }
    }
    wade(u, a, bodies, at, log);
    if log && u.wading.count() != before {
        println!("t {at:.2}: {} {} of its parts in a liquid (was {before}): {:.2} hp", unit_name(u), u.wading.count(), u.health);
    }
}

/// Where a bounding sphere (`centre`, `radius`) meets liquid `l`'s surface: FUN_00222480's ray from
/// its top straight down 2r against that liquid (`Arena::liquid_below`). None for a body without a
/// sphere (FUN_00236670 fails: -1) or one wholly over or under the surface.
fn surface_point(a: &world::Arena, l: u32, centre: Vec3, radius: f32) -> Option<Vec3> {
    if radius <= 0.0 {
        return None;
    }
    a.liquid_below(l, centre.x, centre.z, centre.y + radius, 2.0 * radius).map(|y| Vec3::new(centre.x, y, centre.z))
}

/// Part `b` of `u`'s gone into liquid `l` (FUN_00221af0's splash for a character's part, at any
/// speed): where its sphere's ray meets the surface, the character keeps the height (char+0x414,
/// 0x221cc6) and the part splashes there, big for a radius of at least the type's h_e12e7b9d
/// (0x221cec), else small (`splash`). No surface met, nothing.
fn part_splash(u: &mut Player, a: &world::Arena, l: u32, b: &PartBody, at: f32, log: bool) {
    let Some(t) = a.liquid(l) else { return };
    let Some(p) = surface_point(a, l, b.centre, b.radius) else {
        if log {
            println!("t {at:.2}: {} part h_{:08x} into liquid {l} (type {}): its sphere (r {:.2}) doesn't reach the surface, no splash",
                     unit_name(u), b.bone, t.kind, b.radius);
        }
        return;
    };
    u.wading.surface = p.y;
    let slot = if b.radius >= t.big_radius { LIQUID_BIG_SPLASH } else { LIQUID_SPLASH };
    if log {
        println!("t {at:.2}: {} part h_{:08x} into liquid {l} (type {}): a {} splash at {p:.2} (r {:.2}): effect h_{:08x}, sound {:08x}",
                 unit_name(u), b.bone, t.kind, if slot == LIQUID_BIG_SPLASH { "big" } else { "small" }, b.radius, t.effects[slot], t.sounds[slot]);
    }
    splash(u, &t, slot, p);
}

/// A splash of liquid type `t`'s `slot` (LIQUID_SPLASH or LIQUID_BIG_SPLASH) at the surface point
/// `p`, into `to`'s queues (`Player::splashes`, `Player::world_sounds`): its effect upright at
/// user parameter 1.0 (FUN_000def50, 0x221d59), its sound there at SPLASH_VOLUME with the
/// SPLASH_PITCH jitter (FUN_00179250 kind 5; the slot's list has one id, FUN_00076310's pick).
pub fn splash(to: &mut Player, t: &LiquidType, slot: usize, p: Vec3) {
    if t.effects[slot] != 0 {
        to.splashes.push((Transform::from_translation(p), t.effects[slot], 1.0));
    }
    if t.sounds[slot] != 0 {
        let pitch = to.unit() * SPLASH_PITCH.0 - SPLASH_PITCH.1;
        to.world_sounds.push(WorldSound { id: t.sounds[slot], at: Some(p), volume: Some(SPLASH_VOLUME), pitch, kind: KIND_SPLASH });
    }
}

/// The wade list's update for `u`'s entries (FUN_00221ec0), its parts' bodies `bodies`, at game
/// time `at`. Each entry gives the character the liquid's wading step (char+0x410, 0x221fd9). Then,
/// once RING_EVERY s have passed since its last ring (0x222008), it casts its part's surface ray
/// again: no surface, only the time is kept; a surface point more than h_049350bb from the last
/// ring's, and not straight above or below it, a ring there (the type's fourth effect, through
/// FUN_000def50 at 0x2221d1), facing the level part of the move (FUN_0001d0c0: Bevy's
/// `looking_to`), at user parameter clamp((moved - h_049350bb) / dt x h_fd22997d, 0, 1), and the
/// point and time are kept; closer, nothing (the next frame tries again).
fn wade(u: &mut Player, a: &world::Arena, bodies: &[PartBody], at: f32, log: bool) {
    let mut entries = std::mem::take(&mut u.wading.entries);
    for e in &mut entries {
        let Some(t) = a.liquid(e.liquid) else { continue };
        u.wading.sound = t.sounds[LIQUID_WADE];
        let ring = t.effects[LIQUID_WADE];
        if ring == 0 {
            e.ring_time = at;
            continue;
        }
        if at - RING_EVERY <= e.ring_time {
            continue;
        }
        let Some(b) = bodies.iter().find(|b| b.bone == e.bone) else { continue };
        let Some(p) = surface_point(a, e.liquid, b.centre, b.radius) else {
            e.ring_time = at;
            continue;
        };
        let d = p - e.ring_at;
        let moved = d.length();
        if moved <= t.ring_distance || (d.x == 0.0 && d.z == 0.0) {
            continue;
        }
        let sp = ((moved - t.ring_distance) / shots::TICK * t.ring_scale).clamp(0.0, 1.0);
        let way = Vec3::new(d.x, 0.0, d.z).normalize();
        u.splashes.push((Transform::from_translation(p).looking_to(way, Vec3::Y), ring, sp));
        e.ring_at = p;
        e.ring_time = at;
        if log {
            println!("t {at:.2}: {} part h_{:08x} rings liquid {} at {p:.2} (moved {moved:.2} m): effect h_{ring:08x} at {sp:.2}", unit_name(u),
                     e.bone, e.liquid);
        }
    }
    u.wading.entries = entries;
}

/// What a foot event plays (`step`).
pub enum Step {
    /// the ground's step sound (play.rs's pick from the surface's list)
    Ground,
    /// the liquid's wading step
    Wade(WorldSound),
    /// nothing
    Silent,
}

/// What a foot event of `u`'s plays, its foot (the bone) at `foot`, as FUN_00119020 picks it
/// (0x11914f-0x1192c5; todo/5-liquid-sounds leads §2.3):
///  - its wading step (`Wading::sound`, char+0x410) is taken, and cleared; without one, out of the
///    liquids (char+0x104 0) the ground's step plays, in one nothing does;
///  - in a liquid the step is silent while the surface of the last splash (`Wading::surface`,
///    char+0x414) is WADE_DEPTH or less above the foot, and at least min(depth, 1) loud otherwise;
///  - its volume: max(that, the skill's noise, `skills::noise`) x the gait's factor (`gait`) x
///    OWN_STEP for the controlled character, whose own step plays 2D (FUN_00119020 writes its
///    point into the block's +0x10 slot, which the cull reads as no position: inferred), another's
///    at its foot; its pitch U x 0.7 - 0.35 octaves (STEP_PITCH).
///
/// So the first step after coming out still plays the wading step the last frame in left, at the
/// noise's volume (no depth then: the code's order). The foot events are play.rs's (feet planting
/// on the move; the game's clips' own events, the idle's among them, aren't read).
pub fn step(u: &mut Player, foot: Vec3) -> Step {
    let id = std::mem::take(&mut u.wading.sound);
    let inside = in_liquid(u);
    if id == 0 && !inside {
        return Step::Ground;
    }
    let mut base = skills::noise(u);
    let depth = u.wading.surface - foot.y;
    if inside {
        if depth <= WADE_DEPTH {
            if std::env::var("BF_LIQUID_LOG").is_ok() {
                let why = if u.wading.surface == NO_SURFACE { "no splash has found the surface yet".to_string() } else { format!("{depth:.2} m under the surface") };
                println!("t {:.2}: {} steps in a liquid, {why}: no step sound", u.sim_time, unit_name(u));
            }
            return Step::Silent;
        }
        base = base.max(depth.min(1.0));
    }
    if id == 0 {
        return Step::Silent;
    }
    let volume = base * gait(u) * if u.controlled { OWN_STEP } else { 1.0 };
    let pitch = u.unit() * STEP_PITCH.0 - STEP_PITCH.1;
    if std::env::var("BF_LIQUID_LOG").is_ok() {
        println!("t {:.2}: {} wades a step {depth:.2} m under the surface{}: sound {id:08x} vol {volume:.2} pitch {pitch:+.2}", u.sim_time,
                 unit_name(u), if inside { "" } else { " (out of it now)" });
    }
    Step::Wade(WorldSound { id, at: (!u.controlled).then_some(foot), volume: Some(volume), pitch, kind: KIND_STEP })
}

/// The gait's factor on a step's volume, FUN_00119020 (0x119059-0x1190a4) by the motion type
/// (char+0x1f8): 1.0 for MT_DASH (8) and the running ones (FUN_00126a40: MT_RUN, MT_BACKPEDAL and
/// the four MT_MOVE_* runs), 0.35 for MT_CROUCH_WALK (7), 0.5 for the rest (the walks, a dodge).
/// The demo's gaits: a sprint dashes; a run, forward or back, runs; a move crouched is the crouch
/// walk (its back and side steps too: inferred); the walks and the aimed side steps (the
/// `f_side_walk` clips: FUN_00126b40's MOVE_*_SLOW, inferred) walk.
fn gait(u: &Player) -> f32 {
    if matches!(u.action, Action::Dodge { .. }) {
        return 0.5;
    }
    if matches!(u.action, Action::Crouched) && u.gait != Gait::Idle {
        return 0.35;
    }
    match u.gait {
        Gait::Sprint | Gait::Run | Gait::RunBack => 1.0,
        _ => 0.5,
    }
}

/// An object's body in the liquids (FUN_00221af0's OBJECT bodies: a grenade, a missile), after a
/// step of its centre from `from` to `to` at `speed`: `body` the radius of its body (in a liquid
/// while its lowest point is in the volume), `radius` its model's bounding radius
/// (`Game::archetype_sphere`), `wet` whether it's in one (its obj+0x104 above 0; kept here). Going
/// in when it wasn't in one already, at the type's h_1f6de6f9 (0.4 m/s) or faster, it splashes
/// (0x221ba2-0x221c16): the liquid type, where (its lowest point's crossing of the surface in the
/// step) and which slot (big for a radius of at least h_e12e7b9d, 0x221cec), for `splash`. The
/// game's phantom finds the overlap at its physics step, whose sphere ray can miss a fast object's
/// surface (FUN_00222480 reaches 2r); the demo takes the crossing, which it always reaches.
pub fn object_enters(a: &world::Arena, wet: &mut bool, from: Vec3, to: Vec3, body: f32, radius: f32, speed: f32) -> Option<(LiquidType, Vec3, usize)> {
    let inside = a.liquid_inside(to - Vec3::Y * body);
    let was = std::mem::replace(wet, inside.is_some());
    let (_, surface, t) = inside.filter(|_| !was)?;
    if speed < t.splash_speed {
        return None;
    }
    let (high, low) = (from.y - body, to.y - body);
    let k = if high > low { ((high - surface) / (high - low)).clamp(0.0, 1.0) } else { 1.0 };
    let at = from.lerp(to, k);
    let slot = if radius >= t.big_radius { LIQUID_BIG_SPLASH } else { LIQUID_SPLASH };
    Some((t, Vec3::new(at.x, surface, at.z), slot))
}

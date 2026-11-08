//! Effect types spawned in the world (issue #373): the effect object's own work beyond its
//! particles, from default.xbe's effect object and its effect type's records, checked against
//! the barrels' and the rack's takes (todo/85-interactive-scenery, #366's takes).
//!
//! An effect type (objecttypes `<effect>`, `bf/character.rs` `EffectType`) lists what an object of
//! it does where it's started: its ALE effects and light effect, its sounds, and two lists the
//! effect object's tick (0x21b7a0) works through each game frame it runs, after its particles:
//!  - **damage areas** (`<h_fb0a5f1d><Damage ..>`, parser FUN_00191170, the object's own damage
//!    code FUN_0021ade0): `AreaDamage`, below;
//!  - **shakes** (`<shake-effects><shake ..>`, parser FUN_00191470, FUN_0021b220 and FUN_0021b620):
//!    `EffectShake`, below. Its `<rumble-effects>` (FUN_001912f0, FUN_0021b410: the pad's motors)
//!    aren't played: the demo has no pad vibration.
//!
//! Anything that starts an effect type asks for it with an `EffectSpawn` pushed on
//! `EffectSpawns`: the breaking scenery for a debris list's effect (the barrels', the rack's:
//! play_scenery.rs, which owned the damage areas until #373 generalised them), a grenade's blast
//! for the damage of its explosion's effect type (a Molotov's cloud, #78: `fly_grenades`), the
//! test hook BF_TEST_EFFECT, and the tickets to come (the Supermutant's pound, #212, the Molotov
//! Mutant's burst, #211). `start_effects` makes the look and sounds (unless the caller did) and
//! sets the areas and shakes going; `area_damage` and `shake_view` run them.
//!
//! **Damage areas** (FUN_0021ade0, read in full). Per frame, for each record while h_f724cb8c <
//! the effect's age <= h_f724cb8c + `duration`: every target within `range` of the effect's place
//! (a grid query, then 3D, to the target's place: no ray, so no line of sight and no cover) whose
//! factor for the record's Type is above 0 and whose hit points are above 0 takes amount x dt,
//! x (1 - d / range) for falloff 1 (DFALL_LINEAR), (1 - d / range)^2 x (amount x dt)^2 for 2
//! (DFALL_EXPONENTIAL: squared value and all), in full for 0 (NONE) and 3 (HALF_LIFE). It goes
//! to the target's damage handler (vtable slot 0, 0x125600) as a record with no attacker and no
//! body, so the target's difficulty step, skill and modifiers and its factor for the damage type
//! all apply (play_damage.rs `Hit`), and the objects (breakables) in range take it as the same
//! (`Scenery::area_at_work`: the rack's explosion sets off the barrels around it). The record
//! carries h_ed582b3c, the area's push, scaled by ((1 - w) x 0.4 + 0.6) (w the world's load,
//! FUN_000d9090), as the factor of a kill's shove (#366: `area_damage`'s `death_shoves`: a
//! body-less request thrown from the effect's place; a Type-3 kill is vaporised instead). So
//! the Molotov's cloud (h_1a256542: 78 FLAME a second for 1 s within 7 m, falloff 1, push 1)
//! and the pound (h_ece33110: 50 POWERBLADE a second for 1 s within 7.5 m, falloff 0, push 2.5)
//! need nothing of their own here. The player hurt by one has the red screen tint of a blast.
//!
//! **Shakes** (FUN_0021b220 then FUN_0021b620, from the same tick): for each record while its
//! start < the effect's age <= start + `duration`, every character within `range` of the
//! effect's place (the same point as the damage's) that has a camera following it gets
//! FUN_0006bdd0 with the amplitude lerp(h_f944a10f, h_f6d585b1, (age - start) / duration), x 1 - d /
//! range for falloff 1 and (1 - d / range)^2 x amplitude^2 for 2, no hold and no fall: the
//! camera's one-frame amplitude, set again each frame (`CameraShake::hold_each_frame`,
//! play_shake.rs). Only the controlled character's camera exists here. The pound's shake: 0.7
//! falling to 0 over 2 s, from 1 s on, within 10 m, falloff 2 (so 0.49 at the pound at 1 s).
//!
//! The effect object runs from its start until its last ALE effect has finished (FUN_0021a850
//! empties a slot when its ALE instance answers true to its vtable +0x2c with 4, read as
//! "finished", and the tick ends the object when all eight slots are empty, 0x21bc34), so a
//! damage area or a shake outlasting its effects' particles is cut short in the game; the demo
//! runs each record its whole length, as the barrels' and the rack's always did. The line
//! `BF_DAMAGE_LOG` prints as an effect starts says when a record is longer than its ALE effects
//! (the cloud's run 5.77 s and the pound's 10.93 s, `Compiled::duration`, an estimate: neither).
//!
//! **When** (`Rules`): the effect object's age grows by dt in each of its ticks before the window
//! test, so a record starting at 0 deals its first damage at age dt, in the spawn's frame or the
//! next (todo/200-effect-skills/answers.md Q1): the clock runs from the spawn (#200's takes: the
//! Orbital Beam's and the Wrath's first tick in the sample of the press). The barrels' and the
//! rack's first HP loss about 0.1 s after their hit points reach 0 (takes 01-04 and 07) is their
//! break chain's frames (the debris list spawned one entry a frame, the pieces first), and a
//! grenade's blast's 0.1 s its blast object's delay: those two spawns keep AREA_DELAY before the
//! clock, the fit they were checked with (`Rules::Checked`), as they keep CENTRE_UP for the point
//! of a character they measure to, where the code measures to its root node (`reach_point`).
//! Every other spawn (a skill's, the test hooks') is the game's effect object (`Rules::Game`). A
//! skill's effect has an owner (`EffectSpawn::owner`), and its areas' hits are his (the damage
//! record's attacker, FUN_0021ade0's +0x14 = the effect's +0xcc).
//!
//! **An effect on a body** (#397). An effect object can be given a parent, and then follows it: a
//! level's `<parent Type="1">` link makes the effect object's tick place it at its parent's
//! transform each frame (FUN_0015ed30, run by FUN_0021a700), and for a character parent that is its
//! position record, the hips (vtable +0x40, as the damage areas' reach measures to). Two effect types
//! of common's objecttypes are put on a body here, the demo's use of them: no body in the game's data
//! or code carries either. They are called by the names of their ALE effects (the types' own aren't
//! recovered):
//!  - `vengar` (h_fb1e5895): an aura named after Vengar, max-life 5e11 s. **Unused in the game**:
//!    nothing in its data or code spawns the type, and the ALE effect `vengar` (h_f8db2797) is
//!    named by this type alone.
//!  - `pshield` (h_13687abf): **not unused in the game.** The type is unreferenced, but it is a
//!    duplicate, but for its name, of h_1218cdc1 (objecttypes-common, the record after it), the
//!    `hit-effect` of every `<shield>` (shield-type 1 in m02_a, m02_b, m04_c, m07_c, m09_c, m14_c
//!    and m14_d, type 2 in objecttypes-dm and sdm_shrike; enemies.md, "The stamina shield"): the game
//!    plays the same ALE effect `pshield` (h_fb0d848c) when a shielded character is hit. It's a cone
//!    emitter of small flares, `pshield_Cone.emt`, with a **lifespan of 0.3 s**, pulled by a turbulence
//!    field, aligned by its alignment-type 2 to the hit's normal; its bounding sphere has a radius of
//!    1.02 m centred (0.06, 0.06, -0.33) from the effect's origin. A 0.3 s burst at a hit, then: run
//!    continuously on a body (`AleEffect::new` starts the emitter over each 0.3 s), it is the demo's
//!    construct, and a body has no hit normal here, so alignment 2 falls back to the effect's own
//!    frame (FUN_0021a850).
//!
//! `BodyRequests` asks for one on a character; `start_body_effects`
//! makes the effect type's ALE effects and light effect riding it (`AleAssets::load_flight`: the
//! particles that are attached travel with the effect, the others stay where they're born, as a bolt's
//! flight effect's do), with the type's user parameter, bounding sphere and sounds, and
//! `follow_bodies` keeps them at the character's hips (`missile::root_point`) turned its way (its
//! yaw: the game's is the whole transform). A repeated request for the same type on the same
//! character takes it off (`TakenAway`, as the game's destructor does: FUN_0021a4d0), and two in one
//! frame count once (`distinct`). The type's damage areas and shakes aren't run for a body effect
//! (neither of these two has any), and it ends with its character: dead or gone. Not traced: what
//! the game's code would have done with a body effect, so how it's attached is the demo's reading.
//!
//! Test hook: `BF_TEST_EFFECT=<s>,<effect type>,<x>,<z>[,<y>][;...]` starts the effect type (an
//! `h_xxxxxxxx` hash, or a name) at sim time `s` at (x, z), on the floor there or at world height
//! `y`, with everything a spawned effect has (look, sounds, damage areas, shakes);
//! `BF_TEST_EFFECT=<s>,<effect type>,<who>` puts it on a character's body instead (`who`: `me`, the
//! controlled character; a squad member's name, `brutus`, `flint`, `hawk` or `tex`; or `npc<n>`, the
//! n-th test map NPC) and a second entry for the same type and character takes it off. A name that
//! isn't an effect type but is an ALE effect's (one of the 139 nothing in the game starts,
//! play_testfx.rs: `BF_TEST_EFFECT=2,tornado,0,-8`) plays that effect once there. Logs:
//! `BF_DAMAGE_LOG=1` prints each effect started and each damage tick (the time, the effect, the
//! record's age, the distance and falloff share, the victim, what it takes after every factor,
//! its hit points left) and each area's end with its victims' totals; `BF_COMBAT_LOG=1` the ticks
//! alone (its older form), `BF_SCENERY_LOG=1` the area's end, and `BF_SHAKE_LOG=1` the shake the
//! effects hold on the camera, four times a second.

use super::*;
use super::grenade::{ScreenTint, BLAST_HEARD, BLAST_QUIET};
use bf_viewer::bf::character::{AreaDamage, EffectShake};
use bf_viewer::bf::hash::h;

/// The `Rules::Checked` effects' clock starts this long after they're spawned: the takes' first HP
/// loss is ~0.1 s after the object's hitpoints reach 0, for the barrel (takes 01-04: 2.04 ->
/// 2.13-2.14 s) and the rack alike (take07); the same as a grenade's DAMAGE_DELAY. A fit of the
/// barrels' break chain, not the effect's own: the effect object's first tick deals its first
/// damage (age = dt, FUN_0021ade0 from 0x21b7a0), and the barrel's death queues its debris list,
/// spawned one entry a frame (FUN_00157260, FUN_0015af20), the pieces first, so its effect comes a
/// frame or two after the hit points reach 0, which the takes' 20 Hz logs read as 0.05-0.15 s
/// (todo/200-effect-skills/answers.md Q1). Kept for the spawns it was checked on (see `Rules`).
const AREA_DELAY: f32 = 0.1;
/// A character's point that a `Rules::Checked` damage area (and a shake) measures to (m above the
/// feet): the barrel's range of 3 m from its centre (0.435 m up) reached Tex 2.7 m away along the
/// ground and not 2.9 m (takes 04 / 05), so a point 1.2-1.74 m up. The middle of that: a fit. The
/// game measures to the character's root node (its vtable +0x40, FUN_0010fda0: the hips,
/// `missile::root_point`) from the effect object's place (answers.md Q2), as `Rules::Game` does:
/// for the barrels that place is the object's origin on the floor, not the centre (#366), which
/// fits takes 04 / 05 too (2.95 m and 3.13 m at Tex's 1.19 m hips).
const CENTRE_UP: f32 = 1.45;

/// The rules an effect starts by. The game has one effect object, whose clock starts at its spawn
/// (todo/200-effect-skills/answers.md Q1); the delays the takes show are in whatever spawns it.
/// The demo's spawns that come after such a chain carry fits checked against their takes, kept
/// until each is checked again against the code's reading (answers.md Q1, Q2):
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rules {
    /// the breaking scenery's (the barrels', the rack's: their death queues the debris list, one
    /// entry a frame, FUN_0015af20) and a grenade's blast's (its blast object's delay before its
    /// effect and its damage, FUN_00225220, the grenade's DAMAGE_DELAY), as they were checked (#85,
    /// #373): the chain stood for by AREA_DELAY before the clock, the characters measured to
    /// CENTRE_UP above their feet from `at`, the ALE effects by the grenades' rules
    /// (`AleAssets::load_recorded`)
    Checked,
    /// the effect object as the code has it, made at once (a skill's spawner, FUN_0013e600; the test
    /// hooks'; #200): the clock from the spawn, so the first damage at age dt (0x21b7a0 ->
    /// FUN_0021ade0, Q1; the takes' first tick of the Orbital Beam's and the Wrath's 0-0.1 s area is
    /// in the sample of the press), the characters measured to their root node (Q2: d^2 < range^2,
    /// 3D, from the effect's place), and the ALE effects by the game's count and shapes
    /// (`load_recorded_facing`, the triggers' and the hits' rules: the Orbital Beam's `godray.tga`
    /// column is a shape-3 quad standing on its particle 55 m up, which the grenades' rules draw
    /// centred there, out of sight)
    Game,
}
/// How often BF_SHAKE_LOG reports the shake held on the camera (s).
const SHAKE_LOG_EVERY: f32 = 0.25;

/// A request to start an effect type where something went off. `start_effects` takes them each
/// frame.
#[derive(Clone, Debug)]
pub struct EffectSpawn {
    /// the effect type (an objecttypes `<effect>`'s base name)
    pub kind: u32,
    /// the effect's place: its ALE effects play here and its damage areas and shakes measure
    /// from it (a barrel's: its archetype's centre, `Breakable::centre`)
    pub at: Vec3,
    /// where a kill's shove goes from: the effect object's own place in the game (#366: the
    /// barrel's origin, not its centre); `at` for a spawn that has no other
    pub origin: Vec3,
    /// the effect's turn (a barrel's: its placement's)
    pub rotation: Quat,
    /// the breakable object (the scenery's list index) it came from, which its own areas leave
    /// alone: the barrel that broke
    pub source: Option<usize>,
    /// whether to play the ALE effects, the light effect and the sounds, and to throw the loose
    /// bodies about (pickups, debris: `pickups::Blasts`); false for a caller that does those
    /// itself (a grenade's blast, whose volume, ground and push it has worked out)
    pub look: bool,
    /// the character whose skill started it (FUN_0013e600 sets the effect object's +0xcc and +0xd4
    /// to its owner and the owner's player record, #200): FUN_0021ade0 puts the two into every
    /// damage record its areas deal (+0x14, the attacker, and +0x1c), so their hits are that
    /// character's (play_chatter.rs `Cause::by`: its kill line). None: a barrel's, a grenade's, a
    /// hook's
    pub owner: Option<Who>,
    /// the user parameter its ALE effects run at (FUN_0021c380; a skill's spawner FUN_0013e600 sets
    /// 1 - U x skill+0x30). 0 for the others, as they always ran
    pub user: f32,
    /// the rules it starts by (`Rules`)
    pub rules: Rules,
}

impl EffectSpawn {
    /// Effect type `kind` started at `at` with all it has: its look, its sounds, its damage areas
    /// and its shakes, unturned, from no breakable object, by the game's rules.
    pub fn new(kind: u32, at: Vec3) -> Self {
        EffectSpawn { kind, at, origin: at, rotation: Quat::IDENTITY, source: None, look: true, owner: None, user: 0.0, rules: Rules::Game }
    }

    /// Effect type `kind` started by a skill's spawner (FUN_0013e410 -> FUN_0013e600,
    /// play_effect_skills.rs): at `at` turned by `rotation`, owned by `owner`, at user parameter
    /// `user`, by the game's rules.
    pub fn of_skill(kind: u32, at: Vec3, rotation: Quat, owner: Who, user: f32) -> Self {
        EffectSpawn { rotation, owner: Some(owner), user, ..Self::new(kind, at) }
    }

    /// The same, by the checked rules: for a spawn that comes after a chain (the breaking
    /// scenery's, a grenade's blast's: `Rules::Checked`).
    pub fn checked(mut self) -> Self {
        self.rules = Rules::Checked;
        self
    }

    /// The seconds before its clock starts (`Clock::wait`): AREA_DELAY by the checked rules, none
    /// by the game's.
    fn wait(&self) -> f32 {
        match self.rules {
            Rules::Checked => AREA_DELAY,
            Rules::Game => 0.0,
        }
    }

    /// The same, but only its damage areas and shakes: the caller plays the look and sounds and
    /// throws the loose bodies (`look`).
    pub fn without_look(mut self) -> Self {
        self.look = false;
        self
    }
}

/// The effects to start this frame (`EffectSpawn`): systems that spawn effect types push here
/// before `start_effects` runs (`break_scenery` does, in the same frame).
#[derive(Resource, Default)]
pub struct EffectSpawns(pub Vec<EffectSpawn>);

/// The effect type h_fb1e5895 of common's objecttypes, the one the cut-content catalogue calls
/// `vengar` (3.1): an aura named after Vengar, its one ALE effect `vengar` (h_f8db2797) and one
/// sound (h_effbbcaf, play-length 2); max-life 5e11 s. The type's own name isn't recovered (the
/// hash of "vengar" is the ALE effect's). Nothing in the game's data or code spawns it.
pub const VENGAR_TYPE: u32 = 0xFB1E_5895;
/// The effect type h_13687abf, the catalogue's `pshield`: **unspawned, but not unused in the game.**
/// It is a duplicate, but for its name, of h_1218cdc1, the shields' hit-effect (shield-type 1 in
/// m02_a, m02_b, m04_c, m07_c, m09_c, m14_c and m14_d, type 2 in objecttypes-dm and sdm_shrike), so the
/// game plays its one ALE effect `pshield` (h_fb0d848c; alignment-type 2, the struck surface's normal,
/// which falls back to the effect's own frame with none, FUN_0021a850) when a shielded character is
/// hit, a 0.3 s burst (`pshield_Cone.emt`'s lifespan); no sound; max-life FLT_MAX. Its name isn't
/// recovered either, and nothing references this copy. Here it runs continuously on a body: the
/// demo's.
pub const PSHIELD_TYPE: u32 = 0x1368_7ABF;

/// An effect type as a hook or the menu names it: `h_xxxxxxxx`, `vengar` or `pshield` (the two
/// unspawned types, by their ALE effects' names: the types' own aren't recovered), else the hash of
/// the name.
fn effect_type_named(s: &str) -> Option<u32> {
    let s = s.trim();
    match s.strip_prefix("h_") {
        Some(hex) => u32::from_str_radix(hex, 16).ok(),
        None if s.eq_ignore_ascii_case("vengar") => Some(VENGAR_TYPE),
        None if s.eq_ignore_ascii_case("pshield") => Some(PSHIELD_TYPE),
        None if !s.is_empty() => Some(h(s)),
        None => None,
    }
}

/// A request to put an effect type on a character's body (see the module notes): the object menu's
/// `vengar` and `pshield` entries and BF_TEST_EFFECT's `<who>` form push them; asked again for the
/// type a character already carries, it takes it off.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct BodyRequest {
    /// the effect type
    pub(super) kind: u32,
    /// the character that carries it (`Player::who`: it stays with that character when control
    /// passes to another)
    pub(super) who: Who,
}

/// A frame's body requests with each repeat of the same type for the same character left out, in
/// order: the effect a request makes isn't in the world until the frame's commands are applied, so a
/// second request in the frame would start another copy (or, for a type already carried, take it off
/// a second time), and a click and a hook can land in one frame.
fn distinct(requests: Vec<BodyRequest>) -> Vec<BodyRequest> {
    let mut out: Vec<BodyRequest> = vec![];
    for r in requests {
        if !out.contains(&r) {
            out.push(r);
        }
    }
    out
}

/// The body effects to start or take off this frame (`BodyRequest`): `start_body_effects` takes
/// them.
#[derive(Resource, Default)]
pub(super) struct BodyRequests(pub(super) Vec<BodyRequest>);

/// One ALE effect (or the light effect) of an effect type riding a character's body: which
/// character and which type. `follow_bodies` keeps it at the character's hips.
#[derive(Component)]
pub(super) struct BodyEffect {
    who: Who,
    kind: u32,
}

/// An effect record's clock: it waits `wait` s after the effect starts, then counts its age.
#[derive(Clone, Copy, Debug)]
struct Clock {
    wait: f32,
    age: f32,
}

impl Clock {
    /// Runs the clock `dt` s. The stretch of its age this frame covers, `(from, to)`, or None
    /// while it still waits; the frame it stops waiting counts only the part after the wait.
    fn step(&mut self, dt: f32) -> Option<(f32, f32)> {
        let before = self.wait;
        self.wait -= dt;
        if self.wait > 0.0 {
            return None;
        }
        let from = self.age;
        self.age += if before > 0.0 { -self.wait } else { dt };
        Some((from, self.age))
    }
}

/// A damage area at work (see the module notes): one `<Damage>` record of an effect started.
struct Area {
    /// the effect type, for the log
    effect: u32,
    /// the breakable it came from (not hurt by it)
    source: Option<usize>,
    /// where it is: what the range is measured from
    at: Vec3,
    /// where a kill's shove goes from (`EffectSpawn::origin`; #366's take01 and take02: the
    /// records' directions are the victim's root body's place less this, to three digits)
    origin: Vec3,
    /// the effect's owner (`EffectSpawn::owner`): its hits' attacker
    owner: Option<Who>,
    /// the rules it started by (`Rules`): the point of a character it measures to
    rules: Rules,
    /// the record (`EffectType::damage`)
    damage: AreaDamage,
    /// seconds until it starts (AREA_DELAY and the record's h_f724cb8c), then its age since it
    /// started
    clock: Clock,
    /// per character it reached: (who: a squad member, or a test map NPC by its number, #128;
    /// its age when first and last hurting them, HP)
    victims: Vec<(Who, f32, f32, f32)>,
}

/// A shake at work: one `<shake>` record of an effect started.
struct ShakeRun {
    /// the effect type, for the log
    effect: u32,
    /// where the effect is: what the range is measured from
    at: Vec3,
    /// the record (`EffectType::shakes`)
    shake: EffectShake,
    /// the effect's own clock (from AREA_DELAY by the checked rules, from the spawn by the game's):
    /// the record's start and duration are on its age
    clock: Clock,
    /// the rules it started by (`Rules`): the point of the character it measures to
    rules: Rules,
}

/// The point of `u` an effect's damage area or shake measures to (FUN_0021ade0's target vtable
/// +0x40, see `Rules`): its root node by the game's rules (`missile::root_point`, the hips: 1.19 m
/// above an idle Tex's feet, the lock point of #200's takes), CENTRE_UP above its feet by the
/// checked ones.
fn reach_point(u: &Player, rules: Rules) -> Vec3 {
    match rules {
        Rules::Checked => u.position + Vec3::Y * (GROUND + u.height + CENTRE_UP),
        Rules::Game => super::missile::root_point(u),
    }
}

/// The effects at work: their damage areas, their shakes and their sounds waiting for their delay.
#[derive(Resource, Default)]
struct EffectsAtWork {
    /// the damage areas running, one per `<Damage>` record of each effect started
    areas: Vec<Area>,
    /// the shakes running, one per `<shake>` record
    shakes: Vec<ShakeRun>,
    /// effect sounds waiting for their delay: (s left, id, where)
    sounds: Vec<(f32, u32, Vec3)>,
    /// BF_TEST_EFFECT's entries started so far
    test_fired: usize,
    /// the sim time the shake last reported to BF_SHAKE_LOG (negative: not yet)
    shake_logged: f32,
}

/// The effects systems on a frame: they run among the scenery's (play_scenery.rs `SceneryStep`),
/// between `break_scenery`, which asks for the effects a breaking object spawns, and
/// `set_off_mines`, which reads the reach of the areas at work, so the barrels' and the rack's
/// areas start on the frame they always did.
pub fn plugin(app: &mut App) {
    app.init_resource::<EffectSpawns>()
        .init_resource::<BodyRequests>()
        .init_resource::<EffectsAtWork>()
        .add_systems(OnEnter(AppState::Playing), |mut spawns: ResMut<EffectSpawns>, mut bodies: ResMut<BodyRequests>, mut work: ResMut<EffectsAtWork>| {
            spawns.0.clear();
            bodies.0.clear();
            *work = EffectsAtWork { shake_logged: -1.0, ..default() };
        })
        .add_systems(Update, (test_effect, start_effects, area_damage, shake_view).chain()
            .in_set(super::scenery::SceneryStep).after(super::scenery::break_scenery).before(super::scenery::set_off_mines)
            .run_if(in_state(AppState::Playing)))
        // (the body effects follow their characters once those have moved this frame)
        .add_systems(Update, (start_body_effects, follow_bodies).chain().after(update_player).after(start_effects).run_if(in_state(AppState::Playing)));
}

/// A blast sound's volume heard at `listener`: full at the blast, BLAST_QUIET from BLAST_HEARD m
/// (the grenades' choice).
fn blast_volume(at: Vec3, listener: Vec3) -> f32 {
    (1.0 - at.distance(listener) / BLAST_HEARD).clamp(BLAST_QUIET, 1.0)
}

/// Test hook BF_TEST_EFFECT: one entry, an effect type to start at a sim time and place, or on a
/// character's body.
#[derive(Clone, Debug, PartialEq)]
struct TestEffect {
    /// the controlled character's sim time (s) it starts at
    time: f32,
    /// the effect type
    kind: u32,
    /// where, on the ground plane
    x: f32,
    z: f32,
    /// the world height; None: the floor there
    y: Option<f32>,
    /// the character it goes on, by the word the hook gave (`who_named`); None: at x, z
    body: Option<String>,
}

/// BF_TEST_EFFECT's entries: `<s>,<effect type>,<x>,<z>[,<y>]` or `<s>,<effect type>,<who>` separated
/// by `;`, the effect type as `effect_type_named` reads it (`h_xxxxxxxx`, `vengar`, `pshield`, else a
/// name hashed, as the other hooks' names are). An entry that isn't four or five fields with numbers
/// where they belong, or three with a word that isn't a number, is skipped.
fn parse_test_effects(v: &str) -> Vec<TestEffect> {
    v.split(';').filter_map(|entry| {
        let f: Vec<&str> = entry.split(',').map(str::trim).collect();
        if !(3..=5).contains(&f.len()) {
            return None;
        }
        let kind = effect_type_named(f[1])?;
        let time: f32 = f[0].parse().ok()?;
        // (three values: a character instead of a place; a bare number is neither)
        if f.len() == 3 {
            return (!f[2].is_empty() && f[2].parse::<f32>().is_err())
                .then(|| TestEffect { time, kind, x: 0.0, z: 0.0, y: None, body: Some(f[2].to_string()) });
        }
        Some(TestEffect { time, kind, x: f[2].parse().ok()?, z: f[3].parse().ok()?, y: f.get(4).and_then(|y| y.parse().ok()), body: None })
    }).collect()
}

/// The character a hook's word means: `me` (the controlled character), a squad member by name
/// (`brutus`, `flint`, `hawk`, `tex`: whoever of those is in the squad), or `npc<n>` (the n-th test
/// map NPC spawned, `Npc::id`; `npc:3` too). None for a word that names nobody there.
fn who_named(word: &str, player: &Player, squad: &Squad) -> Option<Who> {
    let w = word.trim().to_ascii_lowercase();
    let units = || std::iter::once(player).chain(squad.0.iter());
    if w == "me" {
        return Some(player.who());
    }
    if let Some(id) = w.strip_prefix("npc").and_then(|n| n.trim_start_matches(':').parse::<u32>().ok()) {
        return units().find(|u| u.npc.is_some_and(|n| n.id == id)).map(|u| u.who());
    }
    // (the squad's four: CHARACTERS' fifth is the slot of every other type, with no name)
    let slot = super::CHARACTERS[..4].iter().position(|c| *c == w)?;
    units().find(|u| u.npc.is_none() && u.character == slot).map(|u| u.who())
}

/// Test hook: BF_TEST_EFFECT starts the effect types it lists (see the module notes), at the
/// controlled character's sim time: at a place, or on a character's body (`BodyRequests`).
fn test_effect(player: Res<Player>, squad: Res<Squad>, mut spawns: ResMut<EffectSpawns>, mut bodies: ResMut<BodyRequests>,
               mut work: ResMut<EffectsAtWork>, mut entries: Local<Option<Vec<TestEffect>>>) {
    let list = entries.get_or_insert_with(|| std::env::var("BF_TEST_EFFECT").map(|v| parse_test_effects(&v)).unwrap_or_default());
    while let Some(e) = list.get(work.test_fired).filter(|e| e.time <= player.sim_time) {
        work.test_fired += 1;
        if let Some(word) = &e.body {
            match who_named(word, &player, &squad) {
                Some(who) => {
                    if log_damage() || log_body() {
                        println!("t {:.2}: BF_TEST_EFFECT puts h_{:08x} on {word}", player.sim_time, e.kind);
                    }
                    bodies.0.push(BodyRequest { kind: e.kind, who });
                }
                None => println!("t {:.2}: BF_TEST_EFFECT: no character {word} for h_{:08x}", player.sim_time, e.kind),
            }
            continue;
        }
        let y = e.y.unwrap_or_else(|| floor_y(e.x, e.z, player.position.y + GROUND + player.height.max(0.0) + 2.0, world::Body::Thing));
        let at = Vec3::new(e.x, y, e.z);
        if log_damage() {
            println!("t {:.2}: BF_TEST_EFFECT starts h_{:08x} at {at:.2}", player.sim_time, e.kind);
        }
        spawns.0.push(EffectSpawn::new(e.kind, at));
    }
}

/// Whether BF_DAMAGE_LOG is on: the effects' own log, each effect started and each damage tick
/// in full (BF_COMBAT_LOG, the damage's, prints the ticks too, without the detail: `area_damage`).
fn log_damage() -> bool {
    std::env::var("BF_DAMAGE_LOG").is_ok()
}

/// An ALE effect played as an effect that isn't an effect type (the object menu's unused ALE
/// effects, #397) whose `Compiled::duration` (an estimate: its longest emitter's lifespan, or the 3 s
/// that counts for one outlasting 100 s, plus its longest particle life) is more than this many
/// seconds is taken as one that never ends: of the 139, `singeflame` (105 s, a flame that stays),
/// `tornado_master` and `tornado_master_40` (1e9 s, a particle life with no end) and
/// `breakup_console_master` (63 s, which shows nothing here). The 28 s godray and the 30 s stun
/// effect are the longest that run their length.
const ALE_ENDLESS: f32 = 30.0;
/// How long one that never ends is played (s): the 3 s `Compiled::duration` counts for an emitter
/// that outlasts 100 s. Its emitters stop then and the particles already made live out their lives,
/// as at the end of any effect (`AleExpire` takes the effect itself away, not its particles).
const ALE_ENDLESS_RUNS: f32 = 3.0;

/// How long an ALE effect of `duration` s (`Compiled::duration`) is played: all of it, or
/// ALE_ENDLESS_RUNS if it's over ALE_ENDLESS.
fn ale_runs(duration: f32) -> f32 {
    if duration > ALE_ENDLESS { ALE_ENDLESS_RUNS } else { duration }
}

/// Starts the effects asked for (`EffectSpawns`): each one's ALE effects and light effect where it
/// is, in its turn, run once (the barrels' and the grenades' way), its sounds after their delays
/// (those waiting from earlier frames play first, so a sound with no delay plays on the frame
/// after the spawn, as the barrels' always did), a push to the loose bodies about for each damage
/// area that has a push (not when the caller plays its own look), and its damage areas and
/// shakes set going on the effect's clock. A request for a hash that isn't an effect type is looked
/// up as an ALE effect (#397: the unused ones the object menu lists, `BF_TEST_EFFECT=<s>,<name>,..`)
/// and played once where it's asked for, with the rules the triggers' effects have.
#[allow(clippy::too_many_arguments)]
fn start_effects(mut commands: Commands, time: Res<Time>, mut spawns: ResMut<EffectSpawns>, mut work: ResMut<EffectsAtWork>, mut game: ResMut<GameData>,
                 mut ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>, (mut images, mut materials): (ResMut<Assets<Image>>, ResMut<Assets<StandardMaterial>>),
                 mut player: ResMut<Player>, mut loose: ResMut<super::pickups::Blasts>) {
    let dt = frame_dt(&time);
    let p = &mut *player;
    work.sounds.retain_mut(|(left, id, at)| {
        *left -= dt;
        if *left <= 0.0 {
            p.sound_queue.push((*id, blast_volume(*at, p.position)));
        }
        *left > 0.0
    });
    let log = log_damage();
    for s in std::mem::take(&mut spawns.0) {
        let Some(fx) = game.0.effect_type_defs.get(&s.kind).cloned() else {
            // not an effect type: an ALE effect named by its hash, one of the unused ones the object
            // menu lists (play_testfx.rs, #397), played once where it's asked for, as the effects
            // triggers make are (`load_recorded_facing`: the game's count, shapes and facing). Only
            // looked up where the caller leaves the look to this system (`look`) and the ALE assets
            // are there
            let looked_up = if s.look { ale.as_deref_mut() } else { None };
            match looked_up.map(|ale| ale.load_recorded_facing(&mut game.0, &mut images, &mut materials, s.kind)) {
                Some(Some(c)) => {
                    let estimate = c.duration();
                    let life = ale_runs(estimate);
                    if log || log_body() {
                        let capped = if life < estimate { format!(" (an estimate of {estimate:.0} s: taken as one that never ends)") } else { String::new() };
                        println!("t {:.2}: ALE effect {} (h_{:08x}) plays at {:.2} for {life:.2} s{capped}", p.sim_time, c.name, s.kind, s.at);
                    }
                    let seed = (s.at.x * 977.0 + s.at.z * 131.0) as u32;
                    commands.spawn((Transform::from_translation(s.at).with_rotation(s.rotation), Visibility::default(),
                                    bf_viewer::ale_fx::AleEffect::once(c, 0.0, seed), AleExpire(life), Name::new(format!("ALE effect h_{:08x}", s.kind))));
                }
                // (looked up, and the library has no effect of that hash)
                Some(None) => println!("effect h_{:08x}: no such effect type, and no ALE effect of that name hash", s.kind),
                // (not looked up: the caller plays its own look, or the ALE assets aren't there)
                None => println!("effect h_{:08x}: no such effect type", s.kind),
            }
            continue;
        };
        // the longest of its ALE effects (the effect object's life, the notes), for the log
        let mut runs = 0.0f32;
        let listed = fx.effects.iter().filter(|&&e| e != 0).count();
        if s.look {
            if let Some(ale) = ale.as_deref_mut() {
                for (k, &e) in fx.effects.iter().chain([&fx.light]).filter(|&&e| e != 0).enumerate() {
                    // (by the game's count and shapes, or the grenades' rules the checked spawns
                    // were drawn with: `Rules`)
                    let compiled = match s.rules {
                        Rules::Game => ale.load_recorded_facing(&mut game.0, &mut images, &mut materials, e),
                        Rules::Checked => ale.load_recorded(&mut game.0, &mut images, &mut materials, e),
                    };
                    if let Some(c) = compiled {
                        let life = c.duration();
                        if k < listed {
                            runs = runs.max(life);
                        }
                        let seed = (s.at.x * 977.0 + s.at.z * 131.0) as u32 ^ k as u32;
                        commands.spawn((Transform::from_translation(s.at).with_rotation(s.rotation), Visibility::default(),
                                        bf_viewer::ale_fx::AleEffect::once(c, s.user, seed), AleExpire(life)));
                    }
                }
            }
            for &(id, delay, _, _) in &fx.sounds {
                work.sounds.push((delay, id, s.at));
            }
        }
        for dmg in &fx.damage {
            work.areas.push(Area { effect: s.kind, source: s.source, at: s.at, origin: s.origin, owner: s.owner, rules: s.rules, damage: *dmg,
                                   clock: Clock { wait: s.wait() + dmg.delay, age: 0.0 }, victims: vec![] });
            // loose bodies about (pickups, earlier debris) are thrown by it: the push its damage
            // message carries (h_ed582b3c) is what throws a body, so an area with none (the acid's
            // and gas clouds', push 0) leaves them lying. The barrels' and the rack's are all 2 or
            // more. (A caller that plays its own look throws them itself)
            if s.look && dmg.push > 0.0 {
                loose.0.push((s.at, dmg.range));
            }
        }
        for sh in &fx.shakes {
            work.shakes.push(ShakeRun { effect: s.kind, at: s.at, shake: *sh, clock: Clock { wait: s.wait(), age: 0.0 }, rules: s.rules });
        }
        if log {
            let look = if s.look { format!("ALE effects run {runs:.2} s, {} sounds", fx.sounds.len()) } else { "the caller plays its look".to_string() };
            // (a skill's: its owner, its user parameter, the game's rules)
            let owned = s.owner.map_or(String::new(), |w| format!(", owner {}", who_name(w, &game.0)));
            let rules = match s.rules {
                Rules::Game => format!(", the game's rules (the clock from the spawn, to the root nodes, the game's ALE count and shapes), user parameter {:.2}", s.user),
                Rules::Checked => String::new(),
            };
            let owned = owned + &rules;
            println!("t {:.2}: effect h_{:08x} starts at {:.2}: {} damage areas (amount, type, s, range, falloff) {:?}, {} shakes (start, s, range, falloff, from, to) {:?}, {look}{owned}",
                     p.sim_time, s.kind, s.at, fx.damage.len(), fx.damage.iter().map(|d| (d.amount, d.damage_type, d.duration, d.range, d.falloff)).collect::<Vec<_>>(),
                     fx.shakes.len(), fx.shakes.iter().map(|k| (k.start, k.duration, k.range, k.falloff, k.from, k.to)).collect::<Vec<_>>());
            for d in fx.damage.iter().filter(|d| s.look && runs > 0.0 && d.delay + d.duration > runs) {
                println!("  (a damage area runs to {:.2} s, past its ALE effects' {runs:.2} s: the game's effect object would end first)", d.delay + d.duration);
            }
        }
    }
}

/// Whether the body effects' log is on: BF_TOOLS_LOG (the tools'; BF_DAMAGE_LOG, the effects' own,
/// prints them too, `log_damage`).
fn log_body() -> bool {
    std::env::var("BF_TOOLS_LOG").is_ok()
}

/// Where a character's body effects ride: its hips (`missile::root_point`, the effect object's
/// parent transform for a character, vtable +0x40) turned the way it faces (its yaw: the demo's
/// stand-in for the game's whole transform).
fn body_pose(u: &Player) -> Transform {
    Transform::from_translation(super::missile::root_point(u)).with_rotation(Quat::from_rotation_y(u.yaw))
}

/// A body effect's name for a message: "Vengar aura" for the unused type `vengar` (h_fb1e5895), "Shield hit effect
/// (pshield)" for h_13687abf, the unspawned duplicate of the shields' hit-effect (`PSHIELD_TYPE`),
/// else its hash.
fn body_name(kind: u32) -> String {
    match kind {
        VENGAR_TYPE => "Vengar aura".into(),
        PSHIELD_TYPE => "Shield hit effect (pshield)".into(),
        other => format!("effect h_{other:08x}"),
    }
}

/// Starts or takes off the body effects asked for (`BodyRequests`). A character that already carries
/// the type has it taken off: its ALE effects marked `TakenAway` (their particles go with them, as the
/// game's destructor takes an effect object's nodes out of the ALE manager at once: FUN_0021a4d0).
/// Otherwise each of the type's ALE effects and its light effect is made riding the character
/// (`AleAssets::load_flight`, `AleEffect::new`: it runs on, its emitters starting over as their
/// lifespans end), at the type's user parameter and with its bounding sphere for the distance LOD,
/// at the character's hips; the type's sounds play after their delays. A request for a character
/// that's gone, or a type the data lacks, is logged and skipped; two requests in one frame for the
/// same type and character count once (`distinct`). The message says what was done.
#[allow(clippy::too_many_arguments)]
fn start_body_effects(mut commands: Commands, mut requests: ResMut<BodyRequests>, mut work: ResMut<EffectsAtWork>, mut game: ResMut<GameData>,
                      mut ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>, (mut images, mut materials): (ResMut<Assets<Image>>, ResMut<Assets<StandardMaterial>>),
                      player: Res<Player>, squad: Res<Squad>, carried: Query<(Entity, &BodyEffect)>, mut status: ResMut<UsePanel>) {
    for r in distinct(std::mem::take(&mut requests.0)) {
        let Some(u) = std::iter::once(&*player).chain(squad.0.iter()).find(|u| u.who() == r.who) else {
            println!("effect h_{:08x}: no such character to carry it", r.kind);
            continue;
        };
        let (who, what) = (unit_name(u), body_name(r.kind));
        let on: Vec<Entity> = carried.iter().filter(|(_, b)| b.who == r.who && b.kind == r.kind).map(|(e, _)| e).collect();
        if !on.is_empty() {
            for e in &on {
                commands.entity(*e).insert(bf_viewer::ale_fx::TakenAway);
            }
            if log_damage() || log_body() {
                println!("t {:.2}: {what} (h_{:08x}) taken off {who}: {} effects gone", player.sim_time, r.kind, on.len());
            }
            status.message = Some((format!("{what} off {who}"), 2.0));
            continue;
        }
        let Some(fx) = game.0.effect_type_defs.get(&r.kind).cloned() else {
            println!("effect h_{:08x}: no such effect type", r.kind);
            continue;
        };
        let Some(ale) = ale.as_deref_mut() else { continue };
        let at = body_pose(u);
        let seed = r.who.seed().wrapping_mul(0x9E37_79B9) ^ r.kind;
        let mut made: Vec<String> = vec![];
        for (k, &e) in fx.effects.iter().chain([&fx.light]).filter(|&&e| e != 0).enumerate() {
            let Some(c) = ale.load_flight(&mut game.0, &mut images, &mut materials, e) else { continue };
            made.push(c.name.clone());
            let run = bf_viewer::ale_fx::AleEffect::new(c, fx.user, seed ^ k as u32).with_lod_sphere(fx.bounds);
            commands.spawn((at, GlobalTransform::from(at), Visibility::default(), run, BodyEffect { who: r.who, kind: r.kind },
                            Name::new(format!("{what} on {who}"))));
        }
        for &(id, delay, _, _) in &fx.sounds {
            work.sounds.push((delay, id, at.translation));
        }
        if log_damage() || log_body() {
            println!("t {:.2}: {what} (h_{:08x}) put on {who} at {:.2}: ALE effects {:?}, {} sounds, user parameter {}, bounds {:?}", player.sim_time, r.kind,
                     at.translation, made, fx.sounds.len(), fx.user, fx.bounds);
        }
        status.message = Some((format!("{what} on {who}"), 2.0));
    }
}

/// Keeps the body effects at their characters' hips (`body_pose`), their transform written as the
/// effect's own and as its global one (an effect with no parent is placed from its global transform
/// when the ALE system emits, before the transforms are propagated: ale_fx.rs `follow_camera`), and
/// takes off those whose character is dead or gone.
fn follow_bodies(mut commands: Commands, player: Res<Player>, squad: Res<Squad>,
                 mut effects: Query<(Entity, &BodyEffect, &mut Transform, &mut GlobalTransform)>) {
    for (e, b, mut at, mut global) in &mut effects {
        match std::iter::once(&*player).chain(squad.0.iter()).find(|u| u.who() == b.who).filter(|u| !u.dead) {
            Some(u) => {
                *at = body_pose(u);
                *global = GlobalTransform::from(*at);
            }
            None => {
                commands.entity(e).insert(bf_viewer::ale_fx::TakenAway);
            }
        }
    }
}

/// An area's damage this frame (`value`: amount x the frame's part of its time) at `d` m by its
/// falloff, as FUN_0021ade0 deals it: 1 (DFALL_LINEAR) x (1 - d / range); 2 (DFALL_EXPONENTIAL)
/// (1 - d / range)^2 x value^2 (squared value and all, as the code has it); the others, 0
/// (NONE) and 3 (HALF_LIFE: the rack's), in full. The takes' flat rack damage at 4 and 7 m
/// agrees.
pub(super) fn falloff(damage: &AreaDamage, value: f32, d: f32) -> f32 {
    let k = 1.0 - d / damage.range.max(1e-3);
    match damage.falloff {
        1 => value * k,
        2 => k * k * value * value,
        _ => value,
    }
}

/// The damage areas: from their start for their duration, amount x dt x factor a frame to every
/// character within range with no test of the way between (x its difficulty step,
/// play_damage.rs), and to the breakable objects in range (chain reactions). See the module
/// notes.
#[allow(clippy::too_many_arguments)]
fn area_damage(time: Res<Time>, mut work: ResMut<EffectsAtWork>, mut scenery: ResMut<super::scenery::Scenery>, mut player: ResMut<Player>,
               mut squad: ResMut<Squad>, game: Res<GameData>, mut tint: ResMut<ScreenTint>, difficulty: Res<super::damage::Difficulty>) {
    let dt = frame_dt(&time);
    let now = player.sim_time;
    let combat = std::env::var("BF_COMBAT_LOG").is_ok();
    let detail = log_damage();
    let mut hits = super::scenery::AreaHits::default();
    for a in work.areas.iter_mut() {
        // the part of this frame the area is at work
        let Some((from, to)) = a.clock.step(dt) else { continue };
        let inside = (to.min(a.damage.duration) - from).max(0.0);
        if inside <= 0.0 {
            continue;
        }
        let value = a.damage.amount * inside;
        // the characters in range (FUN_0021ade0 measures each target's place to the effect's,
        // with no ray test; the weapons' blasts, FUN_00224a90, ray-test characters only, and not
        // these)
        for (k, u) in std::iter::once(&mut *player).chain(squad.0.iter_mut()).enumerate() {
            if u.dead {
                continue;
            }
            // (d < range: FUN_0021ade0's d^2 < range^2, strict)
            let middle = reach_point(u, a.rules);
            let d = middle.distance(a.at);
            if d >= a.damage.range {
                continue;
            }
            // (through the character's handler: its difficulty step, then its factor for the Type,
            // play_damage.rs; FUN_0021ade0 calls the target's vtable slot 0, 0x125600. No
            // FRIENDLY share: the record isn't a weapon's, FUN_0022c4d0, so the owner and his team
            // take what anyone takes)
            let mut hit = super::damage::Hit::new(falloff(&a.damage, value, d), "area", a.damage.damage_type);
            if let Some(owner) = a.owner {
                hit = hit.source(owner);
            }
            let amount = hit.on(u, &game.0, &difficulty);
            if amount <= 0.0 {
                continue;
            }
            // the record FUN_0021ade0 builds (FUN_00222f60's, no flags): its attacker (+0x14) the
            // effect's owner (+0xcc), a skill's character, else none
            let cause = chatter::Cause { by: a.owner, ..chatter::Cause::bare(a.damage.damage_type) };
            let first = !a.victims.iter().any(|v| v.0 == u.who());
            if first || amount >= u.health {
                // hurt as by a blast (blood, its hurt line); dies from it as from any hurt
                let away = Vec3::new(u.position.x - a.at.x, 0.0, u.position.z - a.at.z).normalize_or(Vec3::X);
                hurt(u, &game.0, amount, cause, away * 3.0 + Vec3::Y * 2.0, middle, -1);
                // (a kill's own shoves, #366: FUN_0021ade0's record names no body (+0x20 = 0), its
                // way is from the effect's place (the object's origin) to the victim's point (the
                // root body, the hips: `missile::root_point`) and its push the area's, so the
                // CBT_DEAD handler's hit request is body-less and the up-kick merges into it: one
                // request of magnitude 2 x the share + 1 and factor push + 36, thrown at the
                // all-body cap (take01: the rack's 12.7 m/s, 20 degrees up). A type-3 kill is
                // vaporised and asks for none)
                if u.dead {
                    let root = super::missile::root_point(u);
                    ragdoll::death_shoves(u, ragdoll::Target::None, root, (root - a.origin).normalize_or_zero(), amount, a.damage.damage_type,
                                          ragdoll::area_push(a.damage.push));
                }
                if k == 0 {
                    tint.0 = 0.0;
                }
            } else {
                // (a hit through the handler all the same: its hurt line, FUN_00125ad0)
                chatter::hurt_line(u, cause, amount);
                u.health -= amount;
            }
            if combat || detail {
                let mut line = format!("t {now:.2}: area h_{:08x} at {d:.2} m: {} takes {amount:.2} (x{:.2} the step{}) -> {:.1} / {:.0}", a.effect, unit_name(u),
                                       hit.step, if hit.victim_power != 1.0 { format!(", x{:.2} its power-up", hit.victim_power) } else { String::new() },
                                       u.health, u.max_health);
                if detail {
                    line += &format!(" [age {:.3} (+{:.3}), falloff {} share {:.3}, type {} x{:.3}, push {}{}]", to, to - from, a.damage.falloff,
                                     falloff(&a.damage, 1.0, d), a.damage.damage_type, hit.vulnerability, a.damage.push, if u.dead { ", killed" } else { "" });
                }
                println!("{line}");
            }
            match a.victims.iter_mut().find(|v| v.0 == u.who()) {
                Some(v) => { v.2 = to; v.3 += amount; }
                None => a.victims.push((u.who(), to, to, amount)),
            }
        }
        // the breakable objects in range (to their centres)
        scenery.area_at_work(&a.damage, a.at, value, a.source, &mut hits);
    }
    scenery.deal_area_hits(&game.0, hits, now);
    // areas done
    work.areas.retain(|a| {
        let over = a.clock.wait <= 0.0 && a.clock.age >= a.damage.duration;
        if over && (combat || detail || std::env::var("BF_SCENERY_LOG").is_ok()) {
            println!("t {now:.2}: area h_{:08x} over ({} x {} s, range {}, type {}, falloff {})", a.effect, a.damage.amount, a.damage.duration,
                     a.damage.range, a.damage.damage_type, a.damage.falloff);
            for &(who, first, last, total) in &a.victims {
                println!("  {} hurt from +{first:.2} to +{last:.2} s: {total:.1} HP", who_name(who, &game.0));
            }
        }
        !over
    });
}

/// The amplitude shake `s` holds `d` m from the effect when the effect is `age` s old, if it holds
/// one then: FUN_0021b620's, for a shake with range, within its window `start` < age <= `start` +
/// `duration` and its range. The amplitude lerps from `from` to `to` over the duration; falloff 1
/// takes it times 1 - d / range, falloff 2 takes (1 - d / range)^2 x the amplitude squared (as
/// the damage areas' falloff 2 squares its amount), the others leave it.
fn shake_amplitude(s: &EffectShake, age: f32, d: f32) -> Option<f32> {
    if !(s.range > 0.0 && s.start < age && age <= s.start + s.duration && d < s.range) {
        return None;
    }
    let amp = (s.to - s.from) * ((age - s.start) / s.duration) + s.from;
    let k = 1.0 - d / s.range;
    Some(match s.falloff {
        1 => k * amp,
        2 => k * k * amp * amp,
        _ => amp,
    })
}

/// The shakes at work: the strongest of those that reach the controlled character is held on the
/// view this frame (`CameraShake::hold_each_frame`; FUN_0021b620 gives the camera of every
/// character in range FUN_0006bdd0's one-frame amplitude, and the follow camera is the controlled
/// character's). A shake that's run its length is dropped.
fn shake_view(time: Res<Time>, mut work: ResMut<EffectsAtWork>, player: Res<Player>, mut shake: ResMut<super::shake::CameraShake>) {
    let dt = frame_dt(&time);
    // (the strongest, and which effect and how far, for the log)
    let mut strongest: Option<(f32, u32, f32, f32)> = None;
    for run in work.shakes.iter_mut() {
        let Some((_, age)) = run.clock.step(dt) else { continue };
        let d = reach_point(&player, run.rules).distance(run.at);
        if let Some(amp) = shake_amplitude(&run.shake, age, d).filter(|_| !player.dead) {
            if strongest.is_none_or(|s| amp > s.0) {
                strongest = Some((amp, run.effect, d, age));
            }
        }
    }
    work.shakes.retain(|r| r.clock.wait > 0.0 || r.clock.age <= r.shake.start + r.shake.duration);
    shake.hold_each_frame(strongest.map_or(0.0, |s| s.0));
    if let Some((amp, effect, d, age)) = strongest {
        let due = work.shake_logged < 0.0 || player.sim_time - work.shake_logged >= SHAKE_LOG_EVERY;
        if due && std::env::var("BF_SHAKE_LOG").is_ok() {
            work.shake_logged = player.sim_time;
            println!("t {:.2}: shake held {amp:.3} by h_{effect:08x} (age {age:.2} s, {d:.2} m)", player.sim_time);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Molotov's cloud as the data has it (h_1a256542): 78 FLAME a second for 1 s within 7 m,
    /// falloff 1, push 1.
    fn cloud() -> AreaDamage {
        AreaDamage { amount: 78.0, damage_type: 5, push: 1.0, delay: 0.0, duration: 1.0, range: 7.0, falloff: 1 }
    }

    /// The pound's shake as the data has it (h_ece33110): from 1 s, 2 s long, 10 m, falloff 2,
    /// 0.7 falling to 0.
    fn pound() -> EffectShake {
        EffectShake { start: 1.0, duration: 2.0, range: 10.0, falloff: 2, from: 0.7, to: 0.0 }
    }

    /// FUN_0021ade0's falloffs: linear is x (1 - d / range), exponential squares the share and the
    /// value, none and half-life leave it, and at the range's edge a falloff gives nothing.
    #[test]
    fn the_falloffs_are_the_effect_objects() {
        let linear = cloud();
        assert!((falloff(&linear, 78.0 / 30.0, 0.0) - 2.6).abs() < 1e-5);
        assert!((falloff(&linear, 78.0 / 30.0, 3.5) - 1.3).abs() < 1e-5);
        assert!(falloff(&linear, 78.0 / 30.0, 7.0).abs() < 1e-6);
        let exponential = AreaDamage { falloff: 2, range: 10.0, ..linear };
        assert!((falloff(&exponential, 2.0, 5.0) - 1.0).abs() < 1e-6, "(1 - 0.5)^2 x 2^2");
        for kind in [0, 3] {
            assert_eq!(falloff(&AreaDamage { falloff: kind, ..linear }, 2.6, 6.9), 2.6);
        }
    }

    /// A clock waits, then counts the age: the frame it stops waiting counts the part past the
    /// wait only (the old barrel code's `inside`), and the frames after count whole.
    #[test]
    fn a_clock_counts_from_the_end_of_its_wait() {
        let mut c = Clock { wait: 0.1, age: 0.0 };
        assert_eq!(c.step(0.0667), None);
        let (from, to) = c.step(0.0667).unwrap();
        assert!((from - 0.0).abs() < 1e-6 && (to - 0.0333).abs() < 1e-3, "{from} {to}");
        let (from, to) = c.step(0.0667).unwrap();
        assert!((from - 0.0333).abs() < 1e-3 && (to - 0.1).abs() < 1e-3);
        // a wait that ends exactly on a frame's end starts the age at 0 and counts nothing yet
        let mut c = Clock { wait: 0.0667, age: 0.0 };
        assert_eq!(c.step(0.0667), Some((0.0, 0.0)));
        assert_eq!(c.step(0.0667), Some((0.0, 0.0667)));
    }

    /// The whole second of a damage area deals the amount, however the frames split it: 78 FLAME
    /// at the cloud's centre over 15 fps frames, and over 60 fps ones.
    #[test]
    fn a_second_of_damage_adds_up_to_the_amount() {
        for fps in [15.0f32, 30.0, 60.0] {
            let (dt, damage) = (1.0 / fps, cloud());
            let mut c = Clock { wait: AREA_DELAY, age: 0.0 };
            let mut dealt = 0.0;
            for _ in 0..(3.0 * fps) as usize {
                if let Some((from, to)) = c.step(dt) {
                    let inside = (to.min(damage.duration) - from).max(0.0);
                    dealt += falloff(&damage, damage.amount * inside, 0.0);
                }
            }
            assert!((dealt - 78.0).abs() < 0.01, "{fps} fps: {dealt}");
        }
    }

    /// FUN_0021b620: the pound's shake starts after its first second (nothing at 1.0 s, 0.49 at
    /// the pound just after), falls with the amplitude's square, is cut by the range, and ends
    /// after its three seconds.
    #[test]
    fn the_pounds_shake_ramps_down_over_its_window() {
        let s = pound();
        assert_eq!(shake_amplitude(&s, 1.0, 0.0), None);
        assert!((shake_amplitude(&s, 1.0 + 1e-4, 0.0).unwrap() - 0.49).abs() < 1e-3);
        // halfway through: amplitude 0.35, squared 0.1225; at 5 m a quarter of that
        assert!((shake_amplitude(&s, 2.0, 0.0).unwrap() - 0.1225).abs() < 1e-5);
        assert!((shake_amplitude(&s, 2.0, 5.0).unwrap() - 0.030625).abs() < 1e-5);
        assert_eq!(shake_amplitude(&s, 2.0, 10.0), None);
        assert!(shake_amplitude(&s, 3.0, 0.0).unwrap().abs() < 1e-6);
        assert_eq!(shake_amplitude(&s, 3.01, 0.0), None);
        // linear falloff takes the share once, none leaves the amplitude
        let linear = EffectShake { falloff: 1, ..s };
        assert!((shake_amplitude(&linear, 1.5, 5.0).unwrap() - 0.5 * 0.525).abs() < 1e-5);
        let none = EffectShake { falloff: 0, ..s };
        assert!((shake_amplitude(&none, 1.5, 5.0).unwrap() - 0.525).abs() < 1e-5);
        // a shake with no range is nothing
        assert_eq!(shake_amplitude(&EffectShake { range: 0.0, ..s }, 1.5, 0.0), None);
    }

    /// BF_TEST_EFFECT: the entries, their effect types as hashes or names, an optional height;
    /// a malformed entry is skipped.
    #[test]
    fn the_test_hook_reads_its_entries() {
        let got = parse_test_effects("2.0,h_1a256542,0,-10; 4.5, h_ece33110, 3.5, 4, 1.25 ;bad;1,,0,0;6,pound,1,2");
        assert_eq!(got.len(), 3);
        assert_eq!(got[0], TestEffect { time: 2.0, kind: 0x1a25_6542, x: 0.0, z: -10.0, y: None, body: None });
        assert_eq!(got[1], TestEffect { time: 4.5, kind: 0xece3_3110, x: 3.5, z: 4.0, y: Some(1.25), body: None });
        assert_eq!(got[2].kind, h("pound"));
    }

    /// BF_TEST_EFFECT puts a type on a body with three values (#397): the time, the type and a word
    /// for the character; `vengar` and `pshield` name the two unspawned types (the hash of the word is
    /// the ALE effect's, not the type's); a number alone, an empty word or a word with a place after
    /// it makes no entry.
    #[test]
    fn the_test_hook_puts_an_effect_on_a_body() {
        let got = parse_test_effects("3,vengar,me; 4, pshield ,Tex ;5,h_fb1e5895,npc2;6,vengar,5;7,vengar,;8,vengar,me,1;9,pshield,me,-2,3");
        assert_eq!(got.len(), 3, "{got:?}");
        assert_eq!(got[0], TestEffect { time: 3.0, kind: VENGAR_TYPE, x: 0.0, z: 0.0, y: None, body: Some("me".into()) });
        assert_eq!(got[1].kind, PSHIELD_TYPE);
        assert_eq!(got[1].body.as_deref(), Some("Tex"));
        assert_eq!(got[2].kind, 0xfb1e_5895);
        assert_eq!(got[2].body.as_deref(), Some("npc2"));
        // (the two types' hashes are the data's: h_fb1e5895 and h_13687abf, not the hashes of their ALE effects' names)
        assert_eq!((VENGAR_TYPE, PSHIELD_TYPE), (0xfb1e_5895, 0x1368_7abf));
        assert_ne!(h("vengar"), VENGAR_TYPE);
        assert_eq!(effect_type_named("PSHIELD"), Some(PSHIELD_TYPE));
        assert_eq!(effect_type_named("h_1368_7abf"), None, "a hash is hex digits only");
        assert_eq!(effect_type_named("h_13687abf"), Some(PSHIELD_TYPE));
        assert_eq!(effect_type_named("pound"), Some(h("pound")));
        assert_eq!(effect_type_named(""), None);
    }

    /// A hook's word names a character: `me` the controlled one, a squad member by name (only if
    /// the squad has them), `npc<n>` a test map NPC; nothing else.
    #[test]
    fn a_word_names_a_character() {
        let me = Player::new(3);
        let mut npc = Player::new(1);
        npc.npc = Some(super::super::testtools::Npc { id: 2, fight: false, face: 0.0, follow: false });
        let npc_who = npc.who();
        let squad = Squad(vec![Player::new(0), npc]);
        assert_eq!(who_named("me", &me, &squad), Some(Who::Squad(3)));
        assert_eq!(who_named(" TEX ", &me, &squad), Some(Who::Squad(3)));
        assert_eq!(who_named("brutus", &me, &squad), Some(Who::Squad(0)));
        assert_eq!(who_named("flint", &me, &squad), None, "the squad has no Flint: the NPC isn't one");
        assert_eq!(who_named("npc2", &me, &squad), Some(npc_who));
        assert_eq!(who_named("npc:2", &me, &squad), Some(npc_who));
        assert_eq!(who_named("npc3", &me, &squad), None);
        assert_eq!(who_named("", &me, &squad), None);
        assert_eq!(who_named("nobody", &me, &squad), None);
    }

    /// The two types are named in a message by what they are (the second the shields' hit effect, not
    /// a "personal shield": it duplicates h_1218cdc1), any other by its hash.
    #[test]
    fn a_body_effect_has_a_name() {
        assert_eq!(body_name(VENGAR_TYPE), "Vengar aura");
        assert_eq!(body_name(PSHIELD_TYPE), "Shield hit effect (pshield)");
        assert_eq!(body_name(0x1234_abcd), "effect h_1234abcd");
    }

    /// Two requests for the same type and character in one frame are one; another type or another
    /// character is kept, and so is the order.
    #[test]
    fn a_repeat_in_one_frame_counts_once() {
        let r = |kind, who| BodyRequest { kind, who };
        let (me, other) = (Who::Squad(3), Who::Npc(2, 7));
        let got = distinct(vec![r(VENGAR_TYPE, me), r(PSHIELD_TYPE, me), r(VENGAR_TYPE, me), r(VENGAR_TYPE, other), r(PSHIELD_TYPE, me)]);
        assert_eq!(got, [r(VENGAR_TYPE, me), r(PSHIELD_TYPE, me), r(VENGAR_TYPE, other)]);
        assert!(distinct(vec![]).is_empty());
    }

    /// An ALE effect whose estimate is over 30 s is played for 3 s; the others for their length, the
    /// longest of the 139 that run it (the 30 s stun effect) among them.
    #[test]
    fn an_effect_that_never_ends_is_cut_to_three_seconds() {
        assert_eq!(ale_runs(1.0e9), 3.0, "tornado_master: a particle life with no end");
        assert_eq!(ale_runs(105.02), 3.0, "singeflame");
        assert_eq!(ale_runs(30.001), 3.0);
        assert_eq!(ale_runs(30.0), 30.0, "stunned_fx");
        assert_eq!(ale_runs(5.74), 5.74, "tornado");
        assert_eq!(ale_runs(0.0), 0.0);
    }
}

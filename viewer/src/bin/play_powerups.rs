//! Power-ups (#15): BRUTE POWER, FORCE POWER, HEALTH POWER and STAMINA POWER, the glowing orbs of
//! the multiplayer maps (viewer/docs/pickups.md, "Power-ups"). Every one the levels place is a
//! spawn-trigger's template (#278): what lies in the world is the trigger's copy at its transform
//! with its idle effect (play_pickup_spawns.rs), and taking it sends PICKUP_OBJECT from the copy,
//! which starts the timer that makes the next (sdm_e34: STAMINA and HEALTH POWER 30 s, BRUTE 180
//! s, FORCE 165 s). Read from default.xbe (each function cited is in viewer/docs/code-map.md) and
//! checked against todo/35-special-skills take20 (STAMINA POWER on sdm_e34) and
//! todo/85-interactive-scenery take28 (FORCE POWER on, a barrel at 1 m) and take29 (without).
//!
//!  - **Taking one** is the walk-over of an item (FUN_0011c540 -> FUN_0011c170, as a lying gun's,
//!    play_weapon_pickups.rs): any living member of a player's squad, the AI's too, whose
//!    position (the hips' node) has the orb at most 1.8 m below and 1.0 m above and under
//!    sqrt(1.8) m across (`in_reach`), if the item set's "can take" (+0x38) agrees (`can_take`):
//!    HEALTH and STAMINA POWER's (IFSET_POWERUP_MEDKIT, 0x148d40) while its character is hurt
//!    and it heals (h_0a811e94), or its stamina isn't full and it gives stamina (h_11884f2e);
//!    BRUTE's and FORCE's always (0x280040). Refused, the HUD of the player whose character it
//!    is flashes the bar it would fill (0x146900, play_hud.rs `BarFlash`; take20 3.50-4.55 s).
//!    Taken: the item's pickup sound at the character (FUN_0014a490 -> FUN_00146190, the type's
//!    +0x188), PICKUP_OBJECT from it, the pickup line ("STAMINA POWER", FUN_0011c6a0:
//!    play_pickups.rs `PickupFeed`; take20 9.48-13.6 s), and it's taken out of the world
//!    (FUN_000db3e0): its idle effect's particles live out their lives where it was (take20's
//!    blue glow at Tex's feet, 9.4-9.9 s).
//!  - **What it does** (`take`): the item goes into the inventory (FUN_0014ce90), whose item set
//!    uses it at once (its +0x28, 0x148ae0 -> +0x54) and lets the stack go:
//!    - HEALTH / STAMINA POWER (0x148de0): health + h_0a811e94 up to the most, stamina +
//!      h_11884f2e up to the most (FUN_001258d0). take20: 21.60 -> 71.68, +50.
//!    - BRUTE / FORCE POWER (FUN_00148b30): a modifier made from the item (0x148be0 / 0x148c40:
//!      the factor h_0a811e94, BRUTE's 2 and FORCE's 50 x 0.01; the time h_11884f2e, 45 s),
//!      added to the character's list (char+0x55c, FUN_00122900), then its "on" (0x15f610 /
//!      0x15f760): the character's material controllers of kind 8 / 0x10 switched on with the
//!      item's state-0 event material (the shell, below), and a commentary cue (0x0f5428da /
//!      0xf9994826) that every commentary file lists with no sound and no text. Taking one of a
//!      kind the character has on only puts its time back to the item's (0x11c458-0x11c4a8): one
//!      modifier a kind.
//!  - **While it lasts** (`Power`, `Powers`): BRUTE's modifier multiplies every damage roll its
//!    character makes by its factor (FUN_0022c940's +0x55c hooks, its +0x10 0x15f670:
//!    `roll_factor`), FORCE's every hit it takes but a BLADED one by its (0x125600 at 0x12573f,
//!    its +0x14 0x15f7c0: `damage_factor`; take28 19.0 where take29 took 39.4). Each time runs
//!    down in its character's update, alive or dead (FUN_0011a840 -> +4 = 0x10ebe0 ->
//!    FUN_0010ea00); at 0 the modifier goes, its destructor switching its controllers off. No
//!    HUD shows it (nothing but these reads the list). Each hook also plays the item's state-2
//!    event (min(1, amount x 0.04), FUN_0010eb70): no power-up has one.
//!  - **The shell** (`Look`, `ShellMaterial`, src/shell.wgsl): the controllers draw the character
//!    again with the state-0 event's material (FUN_002135a0; the controller's draw 0x2135e0, in
//!    its states 0-2), a wrapper (h_eb58ab52: BRUTE's h_1959b73b, the glare key 120, FORCE's
//!    h_f5581a7c, 150) of a shell material (class h_0112da6f): the body pushed out along its
//!    normals by `offset` (0.03 m) and covered, added to the picture, with a texture looked up
//!    by its normal in view space and scrolled by the material's clock (BRUTE's red arcs
//!    h_fab1de97 x 0.5, down 0.6 a second; FORCE's blue h_0f91fbc8 x 0.8, up 0.8). take28's
//!    "Tex glows blue-white" from start to end. A controller switched off fades out over 1 s and
//!    is drawn through it (0x210a90 state 2): the shell stays 1 s past its modifier
//!    (SHELL_FADE). The game's controllers of those kinds belong to the types FUN_00210060 gives
//!    them: every multiplayer squad type (mp_brutus 0x119, mp_hawk 0x1f, mp_tex and mp_flint
//!    0x19), but not the campaign's brutus (0x101) and hawk (7). The demo's squad plays the
//!    campaign types on every map, so here every character gets the shell.
//!
//! Test hooks: `BF_TEST_POWERUP=<brute|force|health|stamina|h_<type>>[@<s>][;...]` gives the
//! controlled character that power-up at that time (TEST_AT by default) as a walk-over would,
//! refused (the bar flashing) when it can't take it; `BF_PICKUP_LOG=1` prints each power-up taken
//! or refused and each modifier's start, refresh and end; `BF_COMBAT_LOG=1` the factors on each
//! hit ("x the shooter's power-up 2.00", "x the victim's power-up 0.50").

use super::*;
use bevy::{
    asset::weak_handle,
    pbr::{MaterialPipeline, MaterialPipelineKey, MaterialPlugin, NotShadowCaster},
    render::{
        mesh::{skinning::SkinnedMesh, MeshVertexBufferLayoutRef},
        render_resource::{AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, RenderPipelineDescriptor, ShaderRef,
                          SpecializedMeshPipelineError, TextureFormat},
        view::NoFrustumCulling,
    },
};
use bf_viewer::bf::character::{ItemType, H_WRAP_ALPHA};
use bf_viewer::bf::hash::h;
use bf_viewer::level_scene::{Placed, SpawnTemplate};

/// The power-ups' item sets, by function-type (the IFSET_ enum, its names at 0x3be3cc;
/// FUN_00146930 makes them at 0x146afe, 0x146b4c and 0x146b69): IFSET_POWERUP_MEDKIT (HEALTH and
/// STAMINA POWER, and the Garo fruit, play_pickups.rs's), IFSET_POWERUP_BRUTAL_DAMAGE (BRUTE
/// POWER), IFSET_POWERUP_FORCE_SHIELD (FORCE POWER).
pub const POWERUP_MEDKIT: i64 = 19;
pub const BRUTAL_DAMAGE: i64 = 20;
pub const FORCE_SHIELD: i64 = 21;
/// FORCE POWER's factor is its h_0a811e94 (50) x this (0x148c7f, the float at 0x3a4f98); BRUTE's
/// is its h_0a811e94 as it is (0x148c1a).
const FORCE_SCALE: f32 = 0.01;
/// DTYPE_BLADED (the DTYPE_ table at 0x3bd9d4): the hits FORCE POWER lets through whole (0x15f7cb).
const DTYPE_BLADED: i64 = 2;
/// How long (s) a power-up's shell is drawn after its modifier ends: its controllers' fade-out
/// (+0xc, 1.0, set by their +0x10 0x213570), through which they're drawn (0x2135e0: states 0-2).
const SHELL_FADE: f32 = 1.0;
/// The shell material's class (h_0112da6f: registered at 0xa3a20, factory vtable 0x3a08f0, its
/// instance's 0x3a0900) and its parameters (its tables 0xa3540 and 0xa35b0): the texture and
/// colour `color`, the scroll `translate-u` / `translate-v` (a second) and `offset` (m).
const SHELL_CLASS: u32 = 0x0112_DA6F;
const H_TRANSLATE_U: u32 = 0xFB0B_FF34;
const H_TRANSLATE_V: u32 = 0xE002_AE8E;
const H_OFFSET: u32 = 0x01A4_86AF;
/// When BF_TEST_POWERUP gives a power-up it names no time for (s).
const TEST_AT: f32 = 0.5;
/// How long (s) each character's body is drawn with an invisible shell as its model first shows,
/// so bevy has built the shell's pipelines before a power-up is taken: it compiles a pipeline the
/// first time something is drawn with it and skips the draw until it's ready, and the first
/// FORCE shell came 1.1 s after its take without this (the game's shows at once). play_stealth.rs's
/// WARM_UP does the same for Hawk's refraction.
const WARM_UP: f32 = 1.0;
/// The shell's shader (src/shell.wgsl).
const SHELL_SHADER: Handle<Shader> = weak_handle!("6b1f3c2e-9d47-4a85-b0e6-3f8a71c25d94");

/// A power-up modifier's kind: the item set it comes from and the material controllers it
/// switches (FUN_002107d0's kind, kept at the modifier's +0x2c: what FUN_0011c170 matches a second
/// one of the kind by).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// BRUTE POWER (IFSET_POWERUP_BRUTAL_DAMAGE, the modifier vtable 0x39c064): kind 8 (0x15f637)
    Brute,
    /// FORCE POWER (IFSET_POWERUP_FORCE_SHIELD, vtable 0x39bf7c): kind 0x10 (0x15f787)
    Force,
}

impl Kind {
    /// The modifier kind of an item set (function-type), None for the others.
    pub fn of(function: i64) -> Option<Kind> {
        match function {
            BRUTAL_DAMAGE => Some(Kind::Brute),
            FORCE_SHIELD => Some(Kind::Force),
            _ => None,
        }
    }

    /// Its name for the log.
    fn name(self) -> &'static str {
        match self {
            Kind::Brute => "BRUTE",
            Kind::Force => "FORCE",
        }
    }
}

/// A modifier on a character (one of the game's list at char+0x55c): its kind, its factor (the
/// modifier's +0x30), its time left (+4, s) and the item type that made it.
#[derive(Clone, Debug, PartialEq)]
pub struct Power {
    pub kind: Kind,
    pub factor: f32,
    pub left: f32,
    pub item: u32,
}

/// A shell drawn on a character (its controllers of a kind on, with a material: see the module
/// notes): the kind, the material (the item's state-0 event's, `ItemType::look`) and, once its
/// modifier has ended, the fade-out's time left (s).
#[derive(Clone, Debug, PartialEq)]
pub struct Look {
    pub kind: Kind,
    pub material: u32,
    pub fading: Option<f32>,
}

/// A character's power-ups (`Player::powers`): its modifiers, in the order they came (the list's,
/// FUN_00122900 appends), and its shells.
#[derive(Clone, Debug, Default)]
pub struct Powers {
    pub list: Vec<Power>,
    pub looks: Vec<Look>,
}

/// What a take did, for the log.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Taken {
    /// HEALTH or STAMINA POWER: the health and the stamina after
    Restored { health: f32, stamina: f32 },
    /// a modifier started
    Started(Kind),
    /// a modifier of the kind was on: its time put back
    Refreshed(Kind),
}

/// The factor `u`'s modifiers put on a damage roll it makes (FUN_0022c940, after the skill's
/// +0xec: each modifier's +0x10 in turn): BRUTE's x its factor (0x15f670), FORCE's leaves it
/// (0x10e8f0).
pub fn roll_factor(u: &Player) -> f32 {
    u.powers.list.iter().filter(|p| p.kind == Kind::Brute).map(|p| p.factor).product()
}

/// The factor `u`'s modifiers put on a hit of `damage_type` it takes (0x125600 at 0x12573f, after
/// the skill's +0xd8: each modifier's +0x14 in turn): FORCE's x its factor unless the hit is
/// BLADED (0x15f7c0), BRUTE's leaves it (0x262710).
pub fn damage_factor(u: &Player, damage_type: i64) -> f32 {
    if damage_type == DTYPE_BLADED {
        return 1.0;
    }
    u.powers.list.iter().filter(|p| p.kind == Kind::Force).map(|p| p.factor).product()
}

/// Whether item type `kind` is one of the power-ups this module takes: BRUTE's and FORCE's sets,
/// or IFSET_POWERUP_MEDKIT's with an idle effect (HEALTH and STAMINA POWER: the orbs; the set's
/// other item, the Garo fruit, lies on the ground and is play_pickups.rs's).
fn powerup(game: &Game, kind: u32) -> Option<&ItemType> {
    game.items.get(&kind).filter(|i| Kind::of(i.function).is_some() || (i.function == POWERUP_MEDKIT && game.idle_effects.contains_key(&kind)))
}

/// Whether an orb at `at` is within the walk-over's reach of a character at `position` (its
/// node: the hips, `Player::position`): FUN_0011c170's -1.8..+1.0 m up or down and dx^2 + dz^2
/// under 1.8 (play_weapon_pickups.rs WALK_DOWN / WALK_UP / WALK_ACROSS_SQ).
pub fn in_reach(position: Vec3, at: Vec3) -> bool {
    let (dx, dy, dz) = (at.x - position.x, at.y - position.y, at.z - position.z);
    (-weapon_pickups::WALK_DOWN..=weapon_pickups::WALK_UP).contains(&dy) && dx * dx + dz * dz < weapon_pickups::WALK_ACROSS_SQ
}

/// The item set's "can take" (+0x38) for `u` and power-up `item`: IFSET_POWERUP_MEDKIT's 0x148d40
/// when it heals (h_0a811e94) and `u` is hurt, or it gives stamina (h_11884f2e) and `u`'s isn't
/// full; BRUTE's and FORCE's sets' 0x280040, always.
pub fn can_take(u: &Player, item: &ItemType) -> bool {
    match item.function {
        POWERUP_MEDKIT => (item.health != 0.0 && u.health < u.max_health) || (item.stamina != 0.0 && u.stamina < u.max_stamina),
        _ => true,
    }
}

/// `u` takes power-up `item` (item type `kind`; see the module notes): HEALTH / STAMINA POWER's
/// health and stamina (0x148de0); BRUTE's or FORCE's modifier, started with its shell
/// (FUN_00148b30) or, one of the kind on, its time put back (FUN_0011c170 at 0x11c488). None for an
/// item that isn't a power-up.
pub fn take(u: &mut Player, item: &ItemType, kind: u32) -> Option<Taken> {
    if item.function == POWERUP_MEDKIT {
        u.health = (u.health + item.health).min(u.max_health);
        u.stamina = (u.stamina + item.stamina).clamp(0.0, u.max_stamina);
        return Some(Taken::Restored { health: u.health, stamina: u.stamina });
    }
    let k = Kind::of(item.function)?;
    if let Some(p) = u.powers.list.iter_mut().find(|p| p.kind == k) {
        p.left = item.stamina;
        return Some(Taken::Refreshed(k));
    }
    let factor = if k == Kind::Force { item.health * FORCE_SCALE } else { item.health };
    u.powers.list.push(Power { kind: k, factor, left: item.stamina, item: kind });
    // its "on": the controllers of its kind on with the item's state-0 material (a fading shell
    // of the kind is on again at once: 0x210a90 takes state 2 straight to 1)
    if item.look != 0 {
        u.powers.looks.retain(|l| l.kind != k);
        u.powers.looks.push(Look { kind: k, material: item.look, fading: None });
    }
    Some(Taken::Started(k))
}

/// One frame of `p`'s modifiers and shells: each modifier's time down by `dt` (FUN_0010ea00), at 0
/// taken off (FUN_0011a840), its destructor (0x15f5b0 / 0x15f700) switching its kind's controllers
/// off, so its shell fades out over SHELL_FADE; each fading shell's time down, gone at 0 (state 3,
/// not drawn). Returns the kinds that ended.
pub fn tick(p: &mut Powers, dt: f32) -> Vec<Kind> {
    let mut ended = vec![];
    for m in &mut p.list {
        m.left -= dt;
        if m.left <= 0.0 {
            ended.push(m.kind);
        }
    }
    p.list.retain(|m| m.left > 0.0);
    for l in &mut p.looks {
        if let Some(t) = &mut l.fading {
            *t -= dt;
        } else if ended.contains(&l.kind) {
            l.fading = Some(SHELL_FADE);
        }
    }
    p.looks.retain(|l| l.fading.is_none_or(|t| t > 0.0));
    ended
}

/// The line a power-up taken adds (FUN_0011c6a0, play_pickups.rs `PickupFeed`): its display name
/// (h_1dd3a49d, "STAMINA POWER"), else its stringtable-name.
fn line_name(game: &Game, item: &ItemType) -> String {
    game.strings.get(&item.message).cloned().filter(|_| item.message != 0).unwrap_or_else(|| item.label.clone())
}

/// The bars the HUD flashes for a power-up refused (0x148d40's "no": 0x146900 on the health bar
/// when the item heals, on the stamina bar when it gives stamina).
fn flash(bars: &mut hud::BarFlash, item: &ItemType) {
    if item.health != 0.0 {
        hud::BarFlash::start(&mut bars.health);
    }
    if item.stamina != 0.0 {
        hud::BarFlash::start(&mut bars.stamina);
    }
}

/// The modifiers, the shells and the test hook, after the pickups' frame and before the sounds;
/// the shells drawn once the characters have moved.
pub fn plugin(app: &mut App) {
    app.world_mut().resource_mut::<Assets<Shader>>().insert(&SHELL_SHADER, Shader::from_wgsl(include_str!("../shell.wgsl"), file!()));
    app.add_plugins(MaterialPlugin::<ShellMaterial> { prepass_enabled: false, shadows_enabled: false, ..default() })
        .init_resource::<Shells>()
        .init_resource::<TestPowerups>()
        .add_systems(OnEnter(AppState::Playing), (|mut commands: Commands| {
            commands.insert_resource(Shells::default());
            commands.insert_resource(TestPowerups::default());
        }).after(setup))
        .add_systems(Update, (test_powerups, take_powerups, tick_powers).chain().after(pickups::PickupSystems).before(play_sounds)
            .run_if(in_state(AppState::Playing)))
        .add_systems(Update, draw_shells.after(update_player).run_if(in_state(AppState::Playing)));
}

/// The walk-over of the power-ups (see the module notes): each orb in the world (a spawn-trigger's
/// copy; not a template), checked against the player, then each squad member (FUN_0011c540 runs
/// for every living member of a player's squad, the AI's too; the test map's NPCs take nothing):
/// the first in reach whose "can take" agrees takes it. A refusal flashes the bar on the player's
/// HUD only (0x6ef00 finds the player whose character it is: an AI member has none).
#[allow(clippy::too_many_arguments)]
fn take_powerups(mut commands: Commands, game: Res<GameData>, mut player: ResMut<Player>, mut squad: ResMut<Squad>,
                 mut feed: ResMut<pickups::PickupFeed>, mut events: ResMut<triggers::TriggerEvents>, mut bars: ResMut<hud::BarFlash>,
                 orbs: Query<(Entity, &Placed, &Transform), Without<SpawnTemplate>>) {
    let log = std::env::var("BF_PICKUP_LOG").is_ok();
    for (e, p, t) in &orbs {
        if p.tag != h("inventory-object") {
            continue;
        }
        let Some(item) = powerup(&game.0, p.kind) else { continue };
        let at = t.translation;
        for k in 0..=squad.0.len() {
            let u = if k == 0 { &mut *player } else { &mut squad.0[k - 1] };
            if u.dead || u.absent || !u.in_squad() || u.loaded.is_none() || !in_reach(u.position, at) {
                continue;
            }
            if !can_take(u, item) {
                if k == 0 {
                    flash(&mut bars, item);
                }
                continue;
            }
            let Some(taken) = take(u, item, p.kind) else { break };
            if item.sound != 0 {
                u.sound_queue.push((item.sound, 1.0));
            }
            // (PICKUP_OBJECT from the copy: its trigger's relay times the next one, #278)
            if p.name != 0 {
                events.0.push(triggers::ObjectEvent { from: p.name, signal: bf_viewer::bf::triggers::sig::PICKUP_OBJECT, subject: u.trigger_name, data: -1 });
            }
            feed.add(line_name(&game.0, item), item.amount.max(1));
            commands.entity(e).despawn();
            if log {
                println!("t {:.2}: {} took {} h_{:08x} at {at:.2}: {taken:?}", u.sim_time, unit_name(u), item.label, p.name);
            }
            break;
        }
    }
}

/// BF_TEST_POWERUP's entries, resolved once the map's item types are in: the item type, when (s)
/// and whether it's been given.
#[derive(Resource, Default)]
struct TestPowerups {
    read: bool,
    list: Vec<(u32, f32, bool)>,
}

/// The item type a BF_TEST_POWERUP name means on this map: `brute` and `force` their sets'
/// first type, `health` and `stamina` the IFSET_POWERUP_MEDKIT orb that gives that, `h_<hex>` that
/// type (the lowest hash where the map's files define several alike).
fn test_item(game: &Game, name: &str) -> Option<u32> {
    let name = name.trim().to_lowercase();
    if let Some(hex) = name.strip_prefix("h_") {
        return u32::from_str_radix(hex, 16).ok().filter(|k| game.items.contains_key(k));
    }
    let want = |kind: u32, i: &ItemType| match name.as_str() {
        "brute" => i.function == BRUTAL_DAMAGE,
        "force" => i.function == FORCE_SHIELD,
        "health" => i.function == POWERUP_MEDKIT && i.health > 0.0 && game.idle_effects.contains_key(&kind),
        "stamina" => i.function == POWERUP_MEDKIT && i.stamina > 0.0 && game.idle_effects.contains_key(&kind),
        _ => false,
    };
    game.items.iter().filter(|(k, i)| want(**k, i)).map(|(k, _)| *k).min()
}

/// BF_TEST_POWERUP=<name>[@<s>][;...]: gives the controlled character each power-up at its time
/// as a walk-over would (`can_take`, `take`, the sound and the line; a refusal flashes the bar).
fn test_powerups(game: Res<GameData>, mut player: ResMut<Player>, mut feed: ResMut<pickups::PickupFeed>, mut bars: ResMut<hud::BarFlash>,
                 mut test: ResMut<TestPowerups>) {
    if !test.read {
        test.read = true;
        let Ok(v) = std::env::var("BF_TEST_POWERUP") else { return };
        for entry in v.split(';').filter(|s| !s.trim().is_empty()) {
            let (name, at) = entry.split_once('@').map_or((entry, TEST_AT), |(n, s)| (n, s.trim().parse().unwrap_or(TEST_AT)));
            match test_item(&game.0, name) {
                Some(kind) => test.list.push((kind, at, false)),
                None => warn!("BF_TEST_POWERUP: no power-up {name:?} on this map"),
            }
        }
    }
    let log = std::env::var("BF_PICKUP_LOG").is_ok();
    for (kind, at, done) in &mut test.list {
        if *done || player.sim_time < *at || player.dead || player.loaded.is_none() {
            continue;
        }
        *done = true;
        let Some(item) = game.0.items.get(kind) else { continue };
        if !can_take(&player, item) {
            flash(&mut bars, item);
            if log {
                println!("t {:.2}: BF_TEST_POWERUP: {} refused {}", player.sim_time, unit_name(&player), item.label);
            }
            continue;
        }
        let taken = take(&mut player, item, *kind);
        if item.sound != 0 {
            player.sound_queue.push((item.sound, 1.0));
        }
        feed.add(line_name(&game.0, item), item.amount.max(1));
        if log {
            println!("t {:.2}: BF_TEST_POWERUP: {} took {} h_{kind:08x}: {taken:?}", player.sim_time, unit_name(&player), item.label);
        }
    }
}

/// Every character's modifiers and shells, each frame (`tick`): FUN_0011a840 runs them in the
/// character's update, the dead's too.
fn tick_powers(time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>) {
    let dt = frame_dt(&time);
    if dt <= 0.0 {
        return;
    }
    let log = std::env::var("BF_PICKUP_LOG").is_ok();
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        if u.powers.list.is_empty() && u.powers.looks.is_empty() {
            continue;
        }
        for k in tick(&mut u.powers, dt) {
            if log {
                println!("t {:.2}: {}'s {} POWER ended", u.sim_time, unit_name(u), k.name());
            }
        }
    }
}

/// The shell material (class h_0112da6f; see the module notes and src/shell.wgsl), one for each
/// power-up's look material, shared by every character it covers as the game's material is.
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct ShellMaterial {
    /// rgb the material's `color` (c27), a the glare key: the wrapper's h_e59d69a0 / 255
    #[uniform(0)]
    color: Vec4,
    /// x, y `translate-u`, `translate-v` (a second), z the material's clock (s: its +0x28, which
    /// its update 0xa37c0 counts up by the frame's time), w `offset` (m)
    #[uniform(1)]
    params: Vec4,
    /// its `color` texture, as linear amounts (see src/shell.wgsl), sampled as its material says
    #[texture(2)]
    #[sampler(3)]
    texture: Handle<Image>,
}

impl Material for ShellMaterial {
    fn vertex_shader() -> ShaderRef {
        SHELL_SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHELL_SHADER.into()
    }

    /// Among the see-through things, after the bodies it covers (bevy's added mode).
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Add
    }

    /// Added to the picture (0xa37e0: ONE / ONE), its alpha replaced by the glare key (the
    /// wrapper's stencil REPLACE, 0x9ea90; src/glare.rs keys by the picture's alpha); no depth
    /// written, culled as the bodies are.
    fn specialize(_pipeline: &MaterialPipeline<Self>, descriptor: &mut RenderPipelineDescriptor, _layout: &MeshVertexBufferLayoutRef,
                  _key: MaterialPipelineKey<Self>) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = descriptor.fragment.as_mut() {
            for target in fragment.targets.iter_mut().flatten() {
                target.blend = Some(BlendState {
                    color: BlendComponent { src_factor: BlendFactor::One, dst_factor: BlendFactor::One, operation: BlendOperation::Add },
                    alpha: BlendComponent { src_factor: BlendFactor::One, dst_factor: BlendFactor::Zero, operation: BlendOperation::Add },
                });
            }
        }
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            depth.depth_write_enabled = false;
        }
        Ok(())
    }
}

/// The shells' materials made, by look material (None: it isn't a shell material, or its texture
/// is missing), the clock they run on (the materials' +0x28), the warm-up's invisible material
/// and each body's warm-up time left (WARM_UP), by its model's root.
#[derive(Resource, Default)]
struct Shells {
    materials: HashMap<u32, Option<Handle<ShellMaterial>>>,
    clock: f32,
    warm: Option<Handle<ShellMaterial>>,
    warming: HashMap<Entity, f32>,
}

/// Marks a body part's shell copy: a child of the part with its mesh and skin, drawn with the
/// look's `ShellMaterial`; whose body (the model's root) and which look material (WARM_LOOK: the
/// warm-up's).
#[derive(Component)]
struct Shell {
    root: Entity,
    material: u32,
}

/// The `Shell::material` of a warm-up copy (no look material is 0).
const WARM_LOOK: u32 = 0;

/// The shell material for look material `look` (see `ShellMaterial`): the wrapper's resolved
/// material (bf/character.rs `add_materials` gives the wrapper's name the wrapped material's class,
/// texture and constants, and keeps its h_e59d69a0), None when it isn't of the shell class.
fn shell_material(game: &mut Game, images: &mut Assets<Image>, materials: &mut Assets<ShellMaterial>, look: u32) -> Option<Handle<ShellMaterial>> {
    if game.material_type(look) != SHELL_CLASS {
        return None;
    }
    let texture = game.material_texture(look)?;
    let mips = game.texture_mips(texture)?;
    let image = images.add(bf_viewer::scene::mip_image(&mips, TextureFormat::Rgba8Unorm, game.texture_address(look, texture)));
    let first = |k: u32, or: f32| game.material_constant(look, k).and_then(|v| v.first().copied()).unwrap_or(or);
    let rgb = game.material_constant(look, h("color")).map(|v| match v.len() {
        0 => Vec3::ONE,
        1 | 2 => Vec3::splat(v[0]),
        _ => Vec3::new(v[0], v[1], v[2]),
    }).unwrap_or(Vec3::ONE);
    let key = (first(H_WRAP_ALPHA, 255.0) / 255.0).clamp(0.0, 1.0);
    Some(materials.add(ShellMaterial { color: rgb.extend(key), params: Vec4::new(first(H_TRANSLATE_U, 0.0), first(H_TRANSLATE_V, 0.0), 0.0, first(H_OFFSET, 0.0)),
                                       texture: image }))
}

/// The shells each frame (see the module notes): the materials' clock on, each character with a
/// look gets a shell copy of each of its body's skinned parts with the look's material (its guns
/// are drawn as they are: take28's are), and copies whose look has ended go. Each body also wears
/// an invisible one for WARM_UP s as its model first shows (`warm_material`).
#[allow(clippy::too_many_arguments)]
fn draw_shells(mut commands: Commands, time: Res<Time>, player: Res<Player>, squad: Res<Squad>, mut game: ResMut<GameData>, mut shells: ResMut<Shells>,
               mut materials: ResMut<Assets<ShellMaterial>>, mut images: ResMut<Assets<Image>>, children: Query<&Children>,
               parts: Query<(&Mesh3d, &SkinnedMesh), (With<MeshMaterial3d<StandardMaterial>>, Without<Shell>)>, copies: Query<(Entity, &Shell)>) {
    let dt = frame_dt(&time);
    shells.clock += dt;
    let clock = shells.clock;
    for handle in shells.materials.values().flatten() {
        if let Some(m) = materials.get_mut(handle) {
            m.params.z = clock;
        }
    }
    let units = || std::iter::once(&*player).chain(squad.0.iter());
    // the warm-up's time on each body
    for left in shells.warming.values_mut() {
        *left = (*left - dt).max(0.0);
    }
    for l in units().filter_map(|u| u.loaded.as_ref()) {
        shells.warming.entry(l.root).or_insert(WARM_UP);
    }
    // the (body, look) pairs drawn now
    let mut wanted: Vec<(Entity, u32)> = units().filter_map(|u| u.loaded.as_ref().map(|l| (l.root, &u.powers.looks)))
        .flat_map(|(root, looks)| looks.iter().map(move |l| (root, l.material))).collect();
    wanted.extend(shells.warming.iter().filter(|(_, &left)| left > 0.0).map(|(&root, _)| (root, WARM_LOOK)));
    let mut have: Vec<(Entity, u32)> = vec![];
    for (e, s) in &copies {
        if wanted.contains(&(s.root, s.material)) {
            have.push((s.root, s.material));
        } else {
            commands.entity(e).despawn();
        }
    }
    let log = std::env::var("BF_PICKUP_LOG").is_ok();
    for (root, look) in wanted {
        if have.contains(&(root, look)) {
            continue;
        }
        let handle = if look == WARM_LOOK {
            Some(shells.warm.get_or_insert_with(|| warm_material(&mut images, &mut materials)).clone())
        } else {
            match shells.materials.get(&look) {
                Some(h) => h.clone(),
                None => {
                    let made = shell_material(&mut game.0, &mut images, &mut materials, look);
                    if made.is_none() {
                        warn!("power-up look h_{look:08x} isn't a shell material (class h_{:08x}): no shell", game.0.material_type(look));
                    }
                    shells.materials.insert(look, made.clone());
                    made
                }
            }
        };
        let Some(handle) = handle else { continue };
        let mut n = 0;
        for part in children.iter_descendants(root) {
            let Ok((mesh, skin)) = parts.get(part) else { continue };
            commands.spawn((Shell { root, material: look }, stealth::BodyCopy, mesh.clone(), skin.clone(), MeshMaterial3d(handle.clone()),
                            Transform::IDENTITY, Visibility::Inherited, NoFrustumCulling, NotShadowCaster, ChildOf(part)));
            n += 1;
        }
        // (its parts come the frame after its spawn: until then, try again)
        if n > 0 {
            have.push((root, look));
            if log && look != WARM_LOOK {
                println!("shell h_{look:08x} on {n} parts of {root}");
            }
        }
    }
}

/// The warm-up's shell material (WARM_UP): nothing added (colour 0, no offset) and the picture's
/// alpha left at the bodies' own 1 (the glare's "no key"), over a 1 x 1 white texture.
fn warm_material(images: &mut Assets<Image>, materials: &mut Assets<ShellMaterial>) -> Handle<ShellMaterial> {
    let white = images.add(Image::new(bevy::render::render_resource::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                                      bevy::render::render_resource::TextureDimension::D2, vec![255; 4], TextureFormat::Rgba8Unorm,
                                      bevy::asset::RenderAssetUsages::default()));
    materials.add(ShellMaterial { color: Vec4::new(0.0, 0.0, 0.0, 1.0), params: Vec4::ZERO, texture: white })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A power-up item type: `function`, h_0a811e94 `health`, h_11884f2e `stamina`, the state-0
    /// event's material `look`.
    fn item(function: i64, health: f32, stamina: f32, look: u32) -> ItemType {
        ItemType { function, stack_limit: 2000, amount: 1, health, stamina, kind: 0, gives: 0, sound: 0, use_sound: 0, label: String::new(), icon: 0,
                   message: 0, group: 8, gameflag: vec![], use_type: 0, selectable: false, look }
    }

    /// The data's four (sdm_e34's objecttypes): BRUTE 2 / 45, FORCE 50 / 45, HEALTH 50 / 0,
    /// STAMINA 0 / 50.
    fn brute() -> ItemType {
        item(BRUTAL_DAMAGE, 2.0, 45.0, 0x1959_B73B)
    }
    fn force() -> ItemType {
        item(FORCE_SHIELD, 50.0, 45.0, 0xF558_1A7C)
    }

    /// Tex on sdm_e34 (take20): 70 hit points, 100 stamina.
    fn tex() -> Player {
        let mut u = Player::new(3);
        u.max_health = 70.0;
        u.health = 70.0;
        u.max_stamina = 100.0;
        u.stamina = 100.0;
        u
    }

    /// FUN_0011c170's reach: -1.8..+1.0 m up, under sqrt(1.8) across.
    #[test]
    fn the_walk_over_reach() {
        let at = Vec3::new(-84.07, 29.78, 60.58);
        assert!(in_reach(at + Vec3::new(1.3, 0.4, 0.0), at));
        assert!(!in_reach(at + Vec3::new(1.35, 0.4, 0.0), at));
        assert!(in_reach(at + Vec3::new(0.0, 1.79, 0.0), at));
        assert!(!in_reach(at + Vec3::new(0.0, 1.81, 0.0), at));
        assert!(in_reach(at - Vec3::new(0.0, 0.99, 0.0), at));
        assert!(!in_reach(at - Vec3::new(0.0, 1.01, 0.0), at));
    }

    /// take20: STAMINA POWER isn't taken at full stamina; at 21.6 it gives +50 (71.68 with the
    /// frame's refill). HEALTH POWER only while hurt, up to the most.
    #[test]
    fn stamina_and_health_power() {
        let stamina = item(POWERUP_MEDKIT, 0.0, 50.0, 0);
        let health = item(POWERUP_MEDKIT, 50.0, 0.0, 0);
        let mut u = tex();
        assert!(!can_take(&u, &stamina));
        assert!(!can_take(&u, &health));
        u.stamina = 21.6;
        assert!(can_take(&u, &stamina) && !can_take(&u, &health));
        assert_eq!(take(&mut u, &stamina, 1), Some(Taken::Restored { health: 70.0, stamina: 71.6 }));
        u.health = 40.0;
        assert!(can_take(&u, &health));
        take(&mut u, &health, 2);
        assert_eq!(u.health, 70.0);
        u.stamina = 80.0;
        take(&mut u, &stamina, 1);
        assert_eq!(u.stamina, 100.0);
    }

    /// BRUTE: every roll x2; FORCE: every hit but a BLADED one x0.5; 45 s each; a second of a
    /// kind puts the time back, without a second modifier.
    #[test]
    fn brute_and_force_power() {
        let mut u = tex();
        assert!(can_take(&u, &brute()) && can_take(&u, &force()));
        assert_eq!(take(&mut u, &brute(), 1), Some(Taken::Started(Kind::Brute)));
        assert_eq!(roll_factor(&u), 2.0);
        assert_eq!(damage_factor(&u, 1), 1.0);
        assert_eq!(take(&mut u, &force(), 2), Some(Taken::Started(Kind::Force)));
        assert_eq!(damage_factor(&u, 10), 0.5);
        assert_eq!(damage_factor(&u, DTYPE_BLADED), 1.0);
        assert_eq!(roll_factor(&u), 2.0);
        tick(&mut u.powers, 30.0);
        assert_eq!(take(&mut u, &brute(), 1), Some(Taken::Refreshed(Kind::Brute)));
        assert_eq!(u.powers.list.len(), 2);
        assert_eq!(u.powers.list.iter().find(|p| p.kind == Kind::Brute).map(|p| p.left), Some(45.0));
        // FORCE ends 45 s after its take, BRUTE 45 s after its refresh
        assert_eq!(tick(&mut u.powers, 15.0), vec![Kind::Force]);
        assert_eq!(damage_factor(&u, 10), 1.0);
        assert_eq!(roll_factor(&u), 2.0);
        assert_eq!(tick(&mut u.powers, 30.0), vec![Kind::Brute]);
        assert_eq!(roll_factor(&u), 1.0);
    }

    /// The shell: on from the take, fading for SHELL_FADE once its modifier ends, then gone; a
    /// take during the fade puts it straight back on.
    #[test]
    fn the_shell_outlives_its_power_by_its_fade() {
        let mut u = tex();
        take(&mut u, &force(), 2);
        assert_eq!(u.powers.looks, vec![Look { kind: Kind::Force, material: 0xF558_1A7C, fading: None }]);
        tick(&mut u.powers, 45.0);
        assert!(u.powers.list.is_empty());
        assert_eq!(u.powers.looks[0].fading, Some(SHELL_FADE));
        tick(&mut u.powers, 0.5);
        assert_eq!(u.powers.looks.len(), 1);
        take(&mut u, &force(), 2);
        assert_eq!(u.powers.looks, vec![Look { kind: Kind::Force, material: 0xF558_1A7C, fading: None }]);
        tick(&mut u.powers, 45.0);
        tick(&mut u.powers, SHELL_FADE);
        assert!(u.powers.looks.is_empty());
    }
}

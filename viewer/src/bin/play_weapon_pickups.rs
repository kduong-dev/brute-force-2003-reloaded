//! Weapon pickups (#17): the guns that lie in the world, the use prompt that offers one, the swap
//! that takes it, and the ammo a squad member takes from one by walking over it. Written from
//! default.xbe's code (the addresses are in the comments and in viewer/docs/code-map.md; the
//! reading is todo/17-weapon-pickups/leads.md's) and checked against the takes: todo/28-tutorial
//! take11 (the tutorial's L-Shot-50), todo/61-l-shot-50 take04 (Tex: the refusal, then the swap),
//! todo/52-confed-lzr-50 take04 (Brutus: the switch, then the swap) and todo/DNA + Weapon
//! Pickups.mp4 (refusals and the walk-over's lines).
//!
//!  - **A lying gun** (`LyingGun`) is a level's weapon-object placement (object type 20 WEAPON,
//!    reticule-action 2 RACT_PICKUP) that isn't a spawn-trigger's template, in its placement's
//!    pose; a spawn-trigger's copy of one (every multiplayer map's guns: play_pickup_spawns.rs,
//!    #278); or a gun a character let go of: the one a swap drops, and a dead character's (the
//!    ragdoll's start FUN_00127290 drops both slots' guns, FUN_0014dfd0; and release_weapon's
//!    dead path, FUN_00117660 at 0x118a64: play.rs `update_weapons`), which fall and tumble as
//!    the loose pickups do (play_pickups.rs `Thrown`). Taken, it's gone: nothing puts a placed
//!    campaign weapon back (leads §4.6), and a copy's trigger makes the next.
//!  - **The vanish timers** (`Vanish`, the gun's +0x1a8; FUN_00157c40 counts it, at 0 the gun is
//!    taken out of the world): outside a mission 20 s on a gun a swap or a death drops and 10 s
//!    on one a walk-over takes rounds from; in a mission 300 s on a dead squad member's only; a
//!    use's start stops it (`vanish`, `death_vanish`).
//!  - **A creature's weapons too** (class 7, ammo-count -1, the 7 mm dummy model h_ff18f12d:
//!    m04_a's Psionic Blast x2 and Breath Attack at (-207, 44.5, 76.8), a dead Shrike Hound's
//!    Breath Attack): nothing in the code sets them apart. A placement whose h_1cb7d5f2 is false
//!    becomes a pickup (0x14a0f7 -> FUN_0014a170) whatever its class or model; the dead's slot
//!    guns are dropped (FUN_0014dfd0) whatever they are; the object update hands every unheld
//!    weapon to the use candidates (FUN_00157c40 -> FUN_000ba1c0: reticule-action set, +0x71 bit
//!    0x10 from the weapon's constructor 0x22b757 and bit 0x20 out of a holder), and neither the
//!    candidate loop (FUN_000bd180) nor the tracker (FUN_000bf1e0, FUN_000bf2f0) looks at a class,
//!    a model or the rounds. The squad's holders refuse class 7 (FUN_000a9910), so the prompt is
//!    the refusal with the red ring, on next to nothing to see. Not filmed: the shot would settle
//!    it, and where the game rests a model-less one (the demo's ground, the placement's 44.5 m).
//!  - **The target** (the HUD's use tracker FUN_000bf2f0 and its caller 0xbf110): of the lying
//!    guns at most HEIGHT above or below the controlled character's position (its node, the hips)
//!    and REACH across (last frame's at KEEP of its distance), in the front half of its aim
//!    (a pickup needs no more: other targets the dot's 0.5) and in its line of sight, the one most
//!    in front (the dot of the aim's heading with the way to it: the tracker keeps the lowest
//!    1 - dot). Not while scoped, in a cutscene or dead. A gate panel's prompt (play.rs `doors`)
//!    stays unless a gun is more in front.
//!  - **The prompt** (play_hints.rs `draw_prompt`, FUN_000aa600's weapon case): the character can
//!    take it when one of its two slots' holders takes the gun's class (FUN_000aad20 ->
//!    FUN_000a9910 -> the holders' 0x14baa0: `CharacterType::slot_classes`, the squad's
//!    `limit-weapon`): "Hold [X] to swap %s." (or "... (%d shots)" for a gun with no magazine prop
//!    and no ammo-regen: the iKhan-GPL), with the blue ring; if not, "%s cannot pick up %s." with
//!    the red ring (0xa9bf0's colours). While the use runs, no prompt, and the ring stays on the
//!    gun in its lighter blue (take11 300.6-301.1 s).
//!  - **The ring** (`ring`; the ring widget 0x3a02e8, its update 0xa9bf0 and the reticle
//!    controller's 0xa9510): it zooms in on a target from about 2.4 times its size, fading in
//!    (`Ring::open`, `RING_IN`), and out again when there's none (`RING_OUT`); a new target takes it
//!    at once if it was shown, and at half open at least if not (take11 291.8-292.4 s). It sits on
//!    the point the tracker keeps for its target (its +0x1c, FUN_000bf2f0's last argument):
//!    FUN_0015a480's, the target's hardpoint h_10f81d34 if its model has one (a gate panel's
//!    button), else its position. No weapon model has one (none of the 59 weapon types' meshes or
//!    their parts, over every level), so it's the gun's position; the reach and the line of sight
//!    are measured from FUN_0015a500's, h_ee389721 or the position, the same for a gun.
//!  - **The hold**: X (E; `BF_TEST_SWAP`, `BF_TEST_USE`) held HOLD on a target that can be taken
//!    starts the use (the player controller 0x1332a0 -> FUN_000a9710 -> FUN_0014a410 ->
//!    FUN_00121670); once per press. On one that can't, the press only grumbles: the chatter line
//!    f2ce8278, one time in ten (FUN_000a9910 with its argument 1).
//!  - **The use** (`GunUse`, `step`; FUN_00121670 and FUN_00120290): the slot the gun goes to by
//!    the type's pickup mode (`slot_for`, FUN_00121ac0); that slot is switched into the hand first
//!    if it isn't there (the normal switch clip, re-asked every frame: #52 take04's both panels);
//!    then, with no other overlay playing and the character free, the type's drop-and-pickup
//!    overlay (`pickup_motion`, FUN_00121920; the motion set's `<overlay>` of that motion type,
//!    at its `speed`), laid over the whole body, its root bone too, with the character held where
//!    it stands while it plays (its h_f6a410e6, `GunUse::holds_still`; play.rs `apply_overlay`
//!    and `step_player`, #293). An undefined one, or a use that can't start, gives up after
//!    USE_TIMEOUT.
//!  - **The clip's events** (FUN_00117660; `swap`): at release_weapon the gun in hand's rounds go
//!    into the squad's reserve of its ammo type as far as the stack takes them, and the gun drops
//!    as a lying gun with the rest; at grab_weapon PICKUP_OBJECT goes from the lying gun to the
//!    level's triggers (the tutorial's sniper hint, router h_fa7e22af), a new gun of its type takes
//!    the slot and the hand with the lying gun's rounds, loaded at once from the reserve
//!    (FUN_0022daf0 with 0), the HUD shows its name over its count (play_ammo.rs `HeldName`), and
//!    the lying gun is gone. The weapon panel is empty from the release to the grab, and the
//!    crosshair hidden (take11 300.8-301.1 s).
//!  - **Walking over a lying gun** (`walk_over`, FUN_0011c540 -> FUN_0011c170, every squad member
//!    alive, AI ones too): within WALK_DOWN below and WALK_UP above its position and under
//!    sqrt(WALK_ACROSS_SQ) across, a gun with rounds whose ammo type some squad member's gun uses
//!    gives them to the reserve (the rest kept), with the ammo item's pickup sound, PICKUP_OBJECT
//!    from it and the line "<n>x <ammo>"; with the stack full nothing goes and the line reads
//!    "<ammo> (full)" in red (the DNA video's "Bio Ammo (full)", then "2x Sonic Ammo"). The guns
//!    that count are what the members' slots hold (FUN_00122ed0 through FUN_0014dc50, the hand
//!    holder for the slot in hand): not a slot a release has emptied until its grab. It runs
//!    before the clips' events (`swap`), so it sees the frame's lying guns before a grab takes
//!    one; and a gun a pickup has grabbed is gone for it (FUN_000db3e0 at the grab), the frame it
//!    goes too.
//!  - The AI never swaps guns (no goal of the game's does: leads §9).
//!
//! Test hooks: BF_TEST_GUN=<weapon>[,<ahead m>[,<right m>[,<rounds>]]][;...] lays those guns on
//! the ground round the controlled character's start (3.5 m ahead by default, as the takes' carrier
//! was put, with a full clip), as a dead carrier's guns would lie; BF_TEST_SWAP=<s>[,<s>...] holds
//! X from each of those times for SWAP_HOLD s; BF_WEAPON_LOG=1 prints the lying guns, the target
//! changes, each use, its events, the walk-overs and the vanish timers.

use super::*;
use bf_viewer::bf::hash::h;
use bf_viewer::level_scene::Placed;

/// The use tracker's reach (FUN_000bf2f0; the tracker at the HUD's +0x30, built at 0xbcb90, sets
/// +0x50 = 1.7 and +0x54 = 2.5 at 0xbcbbb): a target at most HEIGHT m above or below the
/// character's position and REACH m away across. The target it already has is measured at KEEP of
/// its distance across ([0x3a5154] 0.7), so it's kept out to 2.5 / 0.7 = 3.6 m.
const HEIGHT: f32 = 1.7;
const REACH: f32 = 2.5;
const KEEP: f32 = 0.7;
/// X held this long (s) on a target uses it: the player controller's hold time, +0x50 =
/// 0x3e99999a, set once by its constructor (0x132f0c); take11: X down at 300.08 s, the prompt gone
/// by 300.6 through xemu's picture lag.
const HOLD: f32 = 0.3;
/// A use gives up when its clip hasn't started this long (s) after it began (FUN_00120290's
/// +0x620, 5.0 at the use's start).
const USE_TIMEOUT: f32 = 5.0;
/// The ring's zoom (`Ring::open`, the ring widget's +0x8c, eased to its +0x88: 1 with a target, 0
/// without, 0xa9569 / 0xa95f8): up at +0x90 (2.0) x 0.6 ([0x3a5038]) a second, down at 2.0 (its
/// update 0xa9bf0); a new target with the ring under half open sets it to a half ([0x3a4f74],
/// 0xa958e), so a ring that was out zooms in from 2.4 times its size over 0.42 s.
const RING_IN: f32 = 1.2;
const RING_OUT: f32 = 2.0;
const RING_NEW: f32 = 0.5;
/// The walk-over's reach (FUN_0011c170): the gun at most WALK_DOWN below the character's position
/// and WALK_UP above it ([-1.8, 1.0]), and dx^2 + dz^2 under WALK_ACROSS_SQ (1.8, about 1.34 m).
/// (FUN_0011c540 gathers the candidates within 2.0 m of the world grid first.)
const WALK_DOWN: f32 = 1.8;
const WALK_UP: f32 = 1.0;
const WALK_ACROSS_SQ: f32 = 1.8;
/// The pickup clips' events, named by hashing (leads §4.3): release_weapon (the gun in hand let
/// go) and grab_weapon (the new one taken). Clip times where a clip lacks one: the demo's.
const EV_RELEASE_WEAPON: u32 = 0xE85F_3539;
const EV_GRAB_WEAPON: u32 = 0x1303_B7C4;
/// The refusal's grumble: the chatter line f2ce8278 (FUN_000a9910 with its argument 1, the X
/// press, -> FUN_00140a60), which every squad chatter file has with h_ea21ae4b 10, read as the
/// chance (%) as the medkit's healed line is (play_pickups.rs HEALED_CHANCE).
const REFUSE_CHATTER: u32 = 0xF2CE_8278;
const REFUSE_CHANCE: usize = 10;
/// WTYPE_NONE: a `limit-weapon` class that takes nothing, and the class FUN_00121920 gives an
/// empty hand.
const WTYPE_NONE: i64 = 8;
/// IFSET_AMMO_BOX (the IFSET_ name table 0x3be3cc): an ammo item's function-type; its
/// h_e5f51266 is the ammo-type it holds (play_ammo.rs).
const AMMO_ITEM: i64 = 14;
/// BF_TEST_GUN's default place: 3.5 m ahead (the xemu playbook's carrier, placed so and killed),
/// let go from 0.5 m up, a little turned (the demo's).
const TEST_AHEAD: f32 = 3.5;
const TEST_DROP: f32 = 0.5;
/// BF_TEST_SWAP holds X this long (s) from each time: past HOLD, short of a second use.
const SWAP_HOLD: f32 = 0.5;
/// A dropped gun's spin as it leaves the hand (rad/s): the demo's, the drop's physics isn't traced
/// (FUN_0014a1d0 -> FUN_00210630; leads §4.6).
const DROP_SPIN: f32 = 2.0;
/// The vanish time (s, `Vanish`) outside a mission of a gun a swap lets go (release_weapon,
/// 0x41a00000 stored at 0x1189e5) or a dead character drops (FUN_0014dfd0, 0x14e062 / 0x14e0d0),
/// each only when the gun's timer isn't running.
const VANISH_DROPPED: f32 = 20.0;
/// The vanish time (s) in a mission of a gun a dead member of a player's squad drops
/// (FUN_0014dfd0, 0x43960000 at 0x14e072 / 0x14e0e4); anyone else's gets none there.
const VANISH_SQUAD_DEAD: f32 = 300.0;
/// The vanish time (s) outside a mission of a lying gun a walk-over takes rounds from
/// (FUN_0011c170, 0x41200000 at 0x11c3fd, whether it's running or not).
const VANISH_WALKED: f32 = 10.0;

/// A lying gun's vanish timer, the weapon's +0x1a8 (s): while it isn't 0 the object's update
/// counts it down by dt (FUN_00157c40 at 0x157d66-0x157da1, plain dt for an object of type 20
/// WEAPON), and once it's 0 or less the update returns 0 and the world takes the gun out
/// (FUN_000d9840 collects the objects whose update said so and hands each to FUN_000db3e0).
/// Started outside a mission (the session's level kind isn't 0x17506391: the multiplayer maps)
/// on a gun a swap lets go or a dead character drops (VANISH_DROPPED) and on one a walk-over
/// takes rounds from (VANISH_WALKED), in a mission on a dead squad member's (VANISH_SQUAD_DEAD);
/// cleared when a use starts on it (FUN_00121670 at 0x121827). A gun without one stays: a
/// level's, a spawn-trigger's copy, the campaign's dropped guns (as every take has them).
#[derive(Component, Clone, Copy, Debug)]
pub struct Vanish(pub f32);

/// The vanish timer (s) FUN_0014dfd0 starts on a gun a dead character drops: outside a
/// `mission` VANISH_DROPPED; in one VANISH_SQUAD_DEAD for a member of a player's squad (`squad`:
/// char+0x59c's squad, FUN_0005c0a0), else none.
pub fn death_vanish(mission: bool, squad: bool) -> Option<f32> {
    match (mission, squad) {
        (false, _) => Some(VANISH_DROPPED),
        (true, true) => Some(VANISH_SQUAD_DEAD),
        (true, false) => None,
    }
}

/// A lying gun the walk-over leaves alone (`walk_over`): the test map's rack, whose guns keep
/// their full clips for the swap (play_testmap.rs). Not the game's: a test convenience.
#[derive(Component)]
pub struct KeepClip;

/// A gun lying in the world (see the module notes): the game's weapon object out of any holder
/// (its +0x1b0 clear, reticule-action +0x79 = 2: FUN_0014a170). Its entity is a root (a placed
/// object, or a gun taken off a character), so its `Transform` is where it is in the world.
#[derive(Component, Clone, Copy, Debug)]
pub struct LyingGun {
    /// its weapon type (a `Game::weapons` key)
    pub kind: u32,
    /// its rounds (the weapon's +0x1f8): a placed one's its type's ammo-count (take11's "6 / 0";
    /// the shot list's F would settle whether the game leaves it unset, -1, which the grab reads
    /// the same), a dropped one's what it had
    pub clip: i64,
    /// its placement's name, the object the level's triggers know (taking it posts PICKUP_OBJECT
    /// from it); 0 for a dropped one
    pub name: u32,
}

/// A weapon pickup under way (`Player::gun_use`): the game's use of a lying gun (char+0x640 the
/// gun, char+0x1ec its state 4-6, char+0x7e0 the slot it goes to; FUN_00121670 starts it,
/// FUN_00120290 runs it, the clip's events finish it).
pub struct GunUse {
    /// the lying gun
    pub target: Entity,
    /// the slot it goes to (0 the first: char+0x7e0 less one)
    pub slot: usize,
    /// its weapon type's class (WTYPE: 0 pistol, 1 rifle, 2 cannon)
    class: i64,
    /// seconds left for the clip to start (`USE_TIMEOUT`)
    timeout: f32,
    /// the drop-and-pickup clip, once it's started
    clip: Option<PickupClip>,
    /// time into the clip (the clip's own seconds: the frame's time x its rate)
    pub time: f32,
    /// release_weapon and grab_weapon have come (`step`), and `swap` has yet to act on them
    pub released: bool,
    pub grabbed: bool,
    release_due: bool,
    grab_due: bool,
    /// cut short (knocked down: play.rs `update_player`): `swap` ends it. (Control handed over
    /// doesn't: the character finishes it under the AI, as the game's use update runs for anyone)
    pub cancelled: bool,
}

impl GunUse {
    /// The use is the character's still (char+0x640 set): from its start to the grab, which ends
    /// it (FUN_000c4f10), though the clip plays on.
    pub fn using(&self) -> bool {
        !self.grabbed
    }

    /// Between release_weapon and grab_weapon: nothing in the hand, the weapon panel empty.
    pub fn hand_empty(&self) -> bool {
        self.released && !self.grabbed
    }

    /// Its clip's overlay and time into it, while it plays (play.rs `step_player` poses it).
    pub(super) fn overlay(&self) -> Option<(&OverlayClip, f32)> {
        self.clip.as_ref().map(|c| (&c.overlay, self.time))
    }

    /// Whether its clip plays and holds the character where it stands (its `<overlay>`'s
    /// h_f6a410e6, `Overlay::holds`, which every motion set's pickup overlays have): the stick's
    /// move FUN_001195c0 skips the move and the turn while the playing overlay (char+0x214) has
    /// it (0x11982f-0x11986b). play.rs `step_player` (#293).
    pub fn holds_still(&self) -> bool {
        self.clip.as_ref().is_some_and(|c| c.holds && self.time < c.overlay.duration)
    }
}

/// A drop-and-pickup clip as it plays: the overlay, its rate (the `<overlay>`'s speed), its
/// release_weapon and grab_weapon times (the clip's own seconds), and whether it holds the
/// character still (the `<overlay>`'s h_f6a410e6, `GunUse::holds_still`).
struct PickupClip {
    overlay: OverlayClip,
    name: String,
    speed: f32,
    release: f32,
    grab: f32,
    holds: bool,
}

/// The lying guns' state: whether the level's have been found, the target last frame (its
/// entity), the hold on it, and whether X is armed (pressed and not yet used: the controller's
/// +0xa0 bit 4); `mission`: the map is played as a mission (the session's level kind 0x17506391,
/// play_damage.rs `multiplayer_map`), which the vanish timers ask (`Vanish`).
#[derive(Resource, Default)]
pub(super) struct Guns {
    ready: bool,
    target: Option<Entity>,
    held: f32,
    armed: bool,
    was_down: bool,
    log: bool,
    pub(super) mission: bool,
}

/// The lying guns found once the level's objects are in; each frame after the panels (`doors`)
/// and before the characters' step, the target, the prompt, the ring and the hold
/// (`use_target`); after the step, the walk-overs (`walk_over`), the clips' events (`swap`) and
/// the vanish timers (`vanish`), before the frame's sounds play and the guns are placed on the
/// bodies. The walk-over goes first: after a grab the new gun's ammo type counts, while the
/// lying gun it came from is only queued to go, and the walk-over would take its rounds a second
/// time (#17 review).
pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Playing), (|mut commands: Commands, game: Res<GameData>, current: Res<CurrentMap>| {
        commands.insert_resource(Guns { log: std::env::var("BF_WEAPON_LOG").is_ok(), mission: !damage::multiplayer_map(&game.0, &current.0), ..default() });
    }).after(setup))
        .add_systems(Update, (find_guns, use_target).chain().after(doors).before(update_player).run_if(in_state(AppState::Playing)))
        .add_systems(Update, (walk_over, swap, vanish).chain().after(update_player).before(play_sounds).before(update_weapons)
            .run_if(in_state(AppState::Playing)));
}

/// The level's lying guns, once its objects are in (as play_pickups.rs `find_pickups` waits for
/// the character): every weapon-object placement that isn't a spawn-trigger's template (out of
/// the world, `SpawnTemplate`: its copies are made lying guns by play_pickup_spawns.rs, #278),
/// with a full clip, in its placement's pose but raised where the ground under it covers part of
/// it (`lowest`). The arena's `settle_pickups` seats a pickup's model bottom on the floor as if it
/// stood upright, which sinks one lying on its side: the tutorial's L-Shot-50 (up axis along x),
/// placed 4.5 cm above the ground there, came out 1 cm under it, half buried; raised, it rests on
/// the ground as take11 shows it. And BF_TEST_GUN's (module notes).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn find_guns(mut commands: Commands, mut guns: ResMut<Guns>, mut game: ResMut<GameData>, player: Res<Player>,
             mut placed: Query<(Entity, &Placed, &mut Transform), (Without<LyingGun>, Without<bf_viewer::level_scene::SpawnTemplate>)>,
             mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>,
             mut images: ResMut<Assets<Image>>, mut bindposes: ResMut<Assets<bevy::render::mesh::skinning::SkinnedMeshInverseBindposes>>,
             (children, parts): (Query<&Children>, Query<(&Transform, Option<&Mesh3d>), Without<Placed>>)) {
    if guns.ready || player.loaded.is_none() {
        return;
    }
    guns.ready = true;
    for (e, p, mut t) in &mut placed {
        if p.tag != h("weapon-object") {
            continue;
        }
        let Some(def) = game.0.weapons.get(&p.kind) else { continue };
        commands.entity(e).insert(LyingGun { kind: p.kind, clip: def.ammo, name: p.name });
        let ground = floor_y(t.translation.x, t.translation.z, t.translation.y + 1.0);
        let low = lowest(e, t.rotation, &children, &parts, &meshes);
        if let Some(low) = low.filter(|l| t.translation.y + l < ground) {
            t.translation.y = ground - low;
        }
        if guns.log {
            println!("lying gun {} (h_{:08x}, placement h_{:08x}) at {:.2} (the ground {ground:.2}, its lowest point {:+.2}), {} rounds", def.label,
                     p.kind, p.name, t.translation, low.unwrap_or(0.0), def.ammo);
        }
    }
    // test hook: BF_TEST_GUN (see the module notes)
    let Ok(v) = std::env::var("BF_TEST_GUN") else { return };
    let mut assets = ModelAssets { meshes: &mut meshes, materials: &mut materials, images: &mut images, bindposes: &mut bindposes };
    for (k, entry) in v.split(';').map(str::trim).filter(|e| !e.is_empty()).enumerate() {
        let mut it = entry.split(',').map(str::trim);
        let name = it.next().unwrap_or_default();
        let Some(kind) = shots::test_weapon_key(&game.0.weapons, name) else {
            warn!("BF_TEST_GUN: no weapon {name}");
            continue;
        };
        let nums: Vec<f32> = it.filter_map(|x| x.parse().ok()).collect();
        let (ahead, right) = (nums.first().copied().unwrap_or(TEST_AHEAD), nums.get(1).copied().unwrap_or(0.0));
        let def = game.0.weapons[&kind].clone();
        let clip = nums.get(2).map_or(def.ammo, |&n| n as i64);
        let (fwd, side) = (Vec3::new(-player.yaw.sin(), 0.0, -player.yaw.cos()), Vec3::new(player.yaw.cos(), 0.0, -player.yaw.sin()));
        let feet = player.position + Vec3::Y * GROUND + fwd * ahead + side * right;
        let at = Vec3::new(feet.x, floor_y(feet.x, feet.z, feet.y + 1.0) + TEST_DROP, feet.z);
        // (lying across the way ahead, a little tipped and turned apart from each other)
        let turn = Quat::from_rotation_y(player.yaw + 1.3 + 0.7 * k as f32) * Quat::from_rotation_z(1.4);
        let Some(e) = gun_model(&mut commands, &mut game.0, &mut assets, &def, Transform::from_translation(at).with_rotation(turn)) else { continue };
        commands.entity(e).insert((LyingGun { kind, clip, name: 0 }, pickups::Thrown { velocity: Vec3::ZERO, spin: Vec3::new(0.5, 0.0, -0.5) }));
        if guns.log {
            println!("BF_TEST_GUN: {} (h_{kind:08x}) laid at {at:.2}, {clip} rounds", def.label);
        }
    }
}

/// A gun's model in the world at `transform` (its parts as the hand's gun has them,
/// `WeaponModel`), for BF_TEST_GUN. None when it doesn't load.
fn gun_model(commands: &mut Commands, game: &mut Game, assets: &mut ModelAssets, def: &WeaponDef, transform: Transform) -> Option<Entity> {
    let wm = WeaponModel::load(game, def.archetype).ok()?;
    let e = commands.spawn((transform, Visibility::Inherited, Name::new(format!("lying {}", def.label)))).id();
    for part in &wm.parts {
        let pe = commands.spawn((Transform::from_translation(part.offset).with_rotation(part.rotation), Visibility::Inherited, ChildOf(e))).id();
        spawn_static(commands, game, &part.geosets, assets, pe, true);
    }
    Some(e)
}

/// How far a placed model's lowest point lies above its origin (negative: below), turned
/// `rotation`: its meshes' vertices through their parts' transforms (the entity's children, as
/// level_scene.rs spawns them). None without a mesh.
fn lowest(e: Entity, rotation: Quat, children: &Query<&Children>, parts: &Query<(&Transform, Option<&Mesh3d>), Without<Placed>>,
          meshes: &Assets<Mesh>) -> Option<f32> {
    let mut low: Option<f32> = None;
    let mut stack: Vec<(Entity, Transform)> = children.get(e).map(|k| k.iter().map(|c| (c, Transform::from_rotation(rotation))).collect()).unwrap_or_default();
    while let Some((c, at)) = stack.pop() {
        let Ok((t, mesh)) = parts.get(c) else { continue };
        let at = at * *t;
        if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(v)) =
            mesh.and_then(|m| meshes.get(&m.0)).and_then(|m| m.attribute(Mesh::ATTRIBUTE_POSITION)) {
            for p in v {
                let y = at.transform_point(Vec3::from(*p)).y;
                low = Some(low.map_or(y, |l| l.min(y)));
            }
        }
        if let Ok(kids) = children.get(c) {
            stack.extend(kids.iter().map(|k| (k, at)));
        }
    }
    low
}

/// The controlled character's type's slot classes and pickup mode (`CharacterType::slot_classes`,
/// `pickup_mode`); a type without `limit-weapon` takes every class in either slot (the front's
/// constructor 0x14c5d0 sets the holders' tables to 1 for each) and its mode is 0.
fn limits(game: &Game, kind: u32) -> ([[i64; 2]; 2], i64) {
    game.character_types.get(&kind).map_or(([[-1; 2]; 2], 0), |t| (t.slot_classes, t.pickup_mode))
}

/// Whether slot `slot`'s holder takes a gun of `class` (0x14baa0: the byte at the holder's
/// +0x34 + the class): one of the slot's `limit-weapon` classes (not WTYPE_NONE), or any of the
/// nine the table holds (0-8, every byte set by the front's constructor 0x14c5d0) when the type
/// gives none.
fn slot_takes(classes: [[i64; 2]; 2], slot: usize, class: i64) -> bool {
    let c = classes[slot];
    if c == [-1, -1] {
        return (0..=WTYPE_NONE).contains(&class);
    }
    c.iter().any(|&x| x == class && x != WTYPE_NONE)
}

/// The slot a picked-up gun of `class` goes to, 1 or 2 (0: none), FUN_00121ac0 by the type's
/// pickup `mode` (+0x15c): 0 slot 1; 1 (Flint, Hawk) the first slot that takes the class; 2
/// (Brutus) a cannon to slot 2, a rifle to the slot in hand (with both slots empty the slot not
/// in hand), no pistol; 3 (Tex) a rifle or a cannon to the slot in hand (with both empty, the slot
/// not in hand), no pistol. `in_hand` the slot in hand (1 or 2), `has` whether each slot has a gun.
fn slot_for(mode: i64, classes: [[i64; 2]; 2], class: i64, in_hand: usize, has: [bool; 2]) -> usize {
    let other = (in_hand == 1) as usize + 1;
    match mode {
        0 => 1,
        1 if slot_takes(classes, 0, class) => 1,
        1 if slot_takes(classes, 1, class) => 2,
        2 if class == 2 => 2,
        2 if class == 1 => if !has[0] && !has[1] { other } else { in_hand },
        3 if class == 1 || class == 2 => if !has[0] && !has[1] { other } else { in_hand },
        _ => 0,
    }
}

/// The pickup overlay's motion type (FUN_00121920, the OMT_ name table 0x3bfae0), by the type's
/// pickup `mode`: the slot the gun goes to (`target`, 1 or 2), the slot in hand, the class of the
/// gun in hand (`hand`, WTYPE_NONE without one) and the picked gun's (`picked`). 33 PICKUP_W1, 34
/// PICKUP_W2, 13-16 DROP_<hand>_PICKUP_<picked> (13 cannon/cannon, 14 cannon/rifle, 15
/// rifle/cannon, 16 rifle/rifle); with the target not in hand the HOLSTER and W2_ types (17-25,
/// 48-51), which no squad set defines (the switch has put the slot in hand by then). Brutus's
/// mode 2 plays 33 for either slot in hand (0x1219f2): his Sc_w2_* pickup clips are never used.
fn pickup_motion(mode: i64, target: usize, in_hand: usize, hand: i64, picked: i64) -> i64 {
    let rifle = (picked == 1) as i64;
    match mode {
        0 => 33,
        1 if target != 1 => if in_hand != 1 { 34 } else { 20 },
        1 => if in_hand != 2 { 33 } else { 24 },
        2 if target == in_hand => 33,
        2 if in_hand == 1 => 0x16 - (picked != 1) as i64,
        2 if target != 1 => if hand != 1 { 49 } else { 51 - (picked != 1) as i64 },
        2 => if hand == 1 { 25 } else { 23 },
        3 if target != in_hand => if hand != 1 { 17 } else { 19 - (picked != 1) as i64 },
        3 if hand == 1 => 15 + rifle,
        3 => 13 + rifle,
        _ => 0,
    }
}

/// The type's drop-and-pickup clip for this use (FUN_00120c00 -> FUN_00121920's motion type,
/// looked up in its motion set's `<overlay>`s as the use queues it: FUN_0012e9d0): the overlay's
/// script (prefix-type 0: the clip's whole name; else after the stance's "Sc_w1_" / "Sc_w2_"),
/// its speed, and the clip's release_weapon and grab_weapon times. None when the set has no such
/// overlay or the model no such clip.
fn pickup_clip(p: &Player, l: &Loaded, game: &Game, u: &GunUse) -> Option<PickupClip> {
    let ty = game.character_types.get(&p.kind)?;
    let hand = l.weapons.get(p.weapon).filter(|_| p.holding).map_or(WTYPE_NONE, |w| w.def.weapon_type);
    let omt = pickup_motion(ty.pickup_mode, u.slot + 1, p.weapon + 1, hand, u.class);
    let o = game.motion_sets.get(&ty.motion_set)?.overlays.iter().find(|o| o.motion_type == omt)?;
    let name = if o.prefix == 0 { o.script.clone() } else { format!("Sc_w{}_{}", stance_of(l, p.weapon) + 1, o.script) };
    let overlay = overlay_clip(&l.model, game, &name, l.feet)?;
    let release = overlay.event(EV_RELEASE_WEAPON).unwrap_or(overlay.duration * 0.25);
    let grab = overlay.event(EV_GRAB_WEAPON).unwrap_or(overlay.duration * 0.55);
    Some(PickupClip { overlay, name, speed: o.speed.max(0.01), release, grab, holds: o.holds })
}

/// Whether `p` can take a gun of `class` now (FUN_000a9910 for a weapon): not scoped (char+0x7a8),
/// not using something (char+0x640), and one of its slots takes the class.
fn can_take(p: &Player, game: &Game, class: i64) -> bool {
    let (classes, mode) = limits(game, p.kind);
    p.scope_level == 0 && p.gun_use.as_ref().is_none_or(|u| !u.using()) && mode != 4 && (slot_takes(classes, 0, class) || slot_takes(classes, 1, class))
}

/// The character's name for the refusal ("Tex cannot pick up Jax-iP."): its type's display name
/// (h_1dd3a49d), else its squad slot's name, capitalised.
fn character_name(p: &Player, game: &Game) -> String {
    if let Some(s) = game.character_types.get(&p.kind).and_then(|t| game.strings.get(&t.label)).filter(|s| !s.is_empty()) {
        return s.clone();
    }
    let mut c = CHARACTERS[p.character % CHARACTERS.len()].chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

/// Each frame after `doors`: the controlled character's target among the lying guns (see the
/// module notes), the prompt for it, the ring, and the hold that starts a use; BF_TEST_SWAP's X.
#[allow(clippy::too_many_arguments)]
pub(super) fn use_target(mut commands: Commands, time: Res<Time>, mut guns: ResMut<Guns>, mut player: ResMut<Player>, game: Res<GameData>,
              mut use_panel: ResMut<UsePanel>, lying: Query<(Entity, &LyingGun, &Transform, &Visibility)>) {
    let dt = frame_dt(&time);
    let t = player.sim_time;
    // test hook: BF_TEST_SWAP=<s>[,<s>...]
    if let Ok(v) = std::env::var("BF_TEST_SWAP") {
        player.use_held |= v.split(',').filter_map(|x| x.trim().parse::<f32>().ok()).any(|s| t >= s && t < s + SWAP_HOLD);
    }
    // X armed on its press, disarmed on its release and by a use (0x134b4e, 0x134be3)
    let down = player.use_held;
    if down && !guns.was_down {
        guns.armed = true;
        guns.held = 0.0;
    }
    if !down {
        guns.armed = false;
        guns.held = 0.0;
    }
    guns.was_down = down;
    let look = Vec2::new(-player.cam_yaw.sin(), -player.cam_yaw.cos());
    let at = player.position;
    let using = player.gun_use.as_ref().filter(|u| u.using()).map(|u| u.target);
    // the candidates: in reach, in front, in sight (FUN_000bf2f0), the most in front first; none
    // for a type that can't pick guns up (mode 4: FUN_000bf1e0)
    let offered = !player.dead && player.loaded.is_some() && !script::blocks_input(&player) && player.scope < 0.5
        && limits(&game.0, player.kind).1 != 4;
    let mut best: Option<(Entity, f32, Vec3, LyingGun)> = None;
    for (e, gun, g, vis) in &lying {
        if !offered || *vis == Visibility::Hidden {
            continue;
        }
        let p = g.translation;
        let d = Vec2::new(p.x - at.x, p.z - at.z);
        let across = d.length() * if guns.target == Some(e) { KEEP } else { 1.0 };
        if (p.y - at.y).abs() > HEIGHT || across > REACH {
            continue;
        }
        let dot = if d.length() > 1e-4 { d.normalize().dot(look) } else { 1.0 };
        if dot < 0.0 || best.as_ref().is_some_and(|b| b.1 >= dot) {
            continue;
        }
        // (the line of sight FUN_000de330 asks, from the character's position to it)
        let to = p - at;
        if ray_hit(at, to.normalize_or_zero(), (to.length() - 0.3).max(0.0)).is_some() {
            continue;
        }
        best = Some((e, dot, p, *gun));
    }
    // while a use runs, its gun stays the target (0xa9510 sets the tracker's lock while the use's
    // object is its target)
    if let Some(e) = using {
        best = lying.get(e).ok().map(|(e, gun, g, _)| (e, 1.0, g.translation, *gun));
    }
    // a gate panel's prompt (`doors`) keeps the frame unless the gun is more in front
    let panel_dot = use_panel.target.map(|p| Vec2::new(p.x - at.x, p.z - at.z).normalize_or_zero().dot(look));
    if let Some((e, dot, p, gun)) = best.filter(|b| using.is_some() || panel_dot.is_none_or(|pd| b.1 > pd)) {
        let def = game.0.weapons.get(&gun.kind);
        let class = def.map_or(WTYPE_NONE, |d| d.weapon_type);
        let takes = can_take(&player, &game.0, class);
        let name = def.map_or_else(|| format!("h_{:08x}", gun.kind), |d| d.display.clone());
        use_panel.prompt = match (using.is_some(), takes) {
            (true, _) => None,
            // ("(%d shots)" for a gun with no magazine prop and no ammo-regen: data+0x174 the
            // null id and data+0x14c 0)
            (false, true) => Some(Prompt::Swap { shots: def.filter(|d| d.magazine == 0 && d.ammo_regen <= 0.0).map(|_| gun.clip), name: name.clone() }),
            (false, false) => Some(Prompt::Refuse { who: character_name(&player, &game.0), name: name.clone() }),
        };
        use_panel.target = Some(p);
        use_panel.target_id = e.to_bits();
        // (the panel isn't the target: its hold doesn't run)
        use_panel.held = 0.0;
        use_panel.ring.colour = if using.is_some() { RingColour::InUse } else if takes { RingColour::Usable } else { RingColour::Refused };
        if guns.target != Some(e) && guns.log {
            let (classes, mode) = limits(&game.0, player.kind);
            println!("t {t:.2}: target {} at {p:.2} ({:.2} m across, dot {dot:.2}): class {class}, {} (slots {classes:?}, mode {mode})", name,
                     Vec2::new(p.x - at.x, p.z - at.z).length(), if takes { "can take" } else { "refused" });
        }
        guns.target = Some(e);
        // the hold (0x1332a0): X down with a target, armed; at HOLD the use (FUN_000a9710)
        if guns.armed && using.is_none() && player.gun_use.is_none() {
            guns.held += dt;
            if guns.held >= HOLD {
                guns.armed = false;
                guns.held = 0.0;
                if takes {
                    start_use(&mut player, &game.0, e, class, guns.log);
                    // (in use from this frame: no prompt, the ring's lighter blue; the gun's
                    // vanish timer stopped, FUN_00121670 at 0x121827)
                    if player.gun_use.is_some() {
                        use_panel.prompt = None;
                        use_panel.ring.colour = RingColour::InUse;
                        commands.entity(e).remove::<Vanish>();
                    }
                } else if player.random(100) < REFUSE_CHANCE {
                    say(&mut player, &game.0, REFUSE_CHATTER, 0.0);
                }
            }
        }
    } else {
        if guns.target.is_some() && guns.log {
            println!("t {t:.2}: no gun target");
        }
        guns.target = None;
        guns.held = 0.0;
        if use_panel.target.is_some() {
            use_panel.ring.colour = RingColour::Usable;
        }
    }
    ring(&mut use_panel, dt);
}

/// The ring's zoom (`Ring`, the widget's update 0xa9bf0 with the controller's 0xa9510): toward
/// open with a target (on it, at half open at least if it's a new one), toward closed without,
/// staying where it was last while it goes.
fn ring(use_panel: &mut UsePanel, dt: f32) {
    let (target, id) = (use_panel.target, use_panel.target_id);
    let r = &mut use_panel.ring;
    match target {
        Some(at) => {
            if r.on != id {
                r.open = r.open.max(RING_NEW);
                r.on = id;
            }
            r.at = Some(at);
            r.open = (r.open + RING_IN * dt).min(1.0);
        }
        None => {
            r.on = 0;
            r.open = (r.open - RING_OUT * dt).max(0.0);
            if r.open <= 0.0 {
                r.at = None;
            }
        }
    }
}

/// The use's start (FUN_00121670 for a weapon, through FUN_000a9710 and the item's +0x140,
/// FUN_0014a410): the slot the gun goes to (`slot_for`); the gun lowered (char+0x7dc = 0); the
/// clip comes once that slot is in hand (`step`). A slot the character has no gun in isn't
/// supported (the squad always has both): no use.
fn start_use(p: &mut Player, game: &Game, target: Entity, class: i64, log: bool) {
    let Some(l) = p.loaded.as_ref() else { return };
    let (classes, mode) = limits(game, p.kind);
    let has = [l.weapons.len() > 0, l.weapons.len() > 1];
    let slot = slot_for(mode, classes, class, p.weapon + 1, has);
    if slot == 0 || slot > l.weapons.len() {
        if log {
            println!("t {:.2}: {} can't put a class {class} gun in a slot (mode {mode}, slot {slot})", p.sim_time, unit_name(p));
        }
        return;
    }
    p.leave_scope();
    p.fire_state.lower();
    p.aim_hold = 0.0;
    if log {
        println!("t {:.2}: {} starts the pickup: class {class} to slot {slot} (slot {} in hand, mode {mode})", p.sim_time, unit_name(p), p.weapon + 1);
    }
    p.gun_use = Some(GunUse { target, slot: slot - 1, class, timeout: USE_TIMEOUT, clip: None, time: 0.0, released: false, grabbed: false,
                              release_due: false, grab_due: false, cancelled: false });
}

/// The use's frame, from play.rs `step_player` (FUN_00120290 and the clip): the slot it goes to
/// switched into the hand (the switch asked again every frame until it's there), then the clip
/// once no other overlay plays and the character is free, then the clip's time and its two events
/// (acted on by `swap`). Gives up at USE_TIMEOUT if the clip hasn't started.
pub(super) fn step(p: &mut Player, l: &Loaded, game: &Game, dt: f32) {
    let Some(mut u) = p.gun_use.take() else { return };
    if u.cancelled {
        p.gun_use = Some(u);
        return;
    }
    if let Some(c) = &u.clip {
        u.time += dt * c.speed;
        if !u.released && u.time >= c.release {
            // release_weapon: nothing in the hand from here to the grab
            u.released = true;
            u.release_due = true;
            p.holding = false;
            p.weapon_dirty = true;
        }
        if !u.grabbed && u.time >= c.grab {
            u.grabbed = true;
            u.grab_due = true;
        }
        // (kept to its end, and until `swap` has acted on its events)
        if u.time < c.overlay.duration || u.release_due || u.grab_due {
            p.gun_use = Some(u);
        }
        return;
    }
    u.timeout -= dt;
    if u.timeout <= 0.0 {
        if std::env::var("BF_WEAPON_LOG").is_ok() {
            println!("t {:.2}: {}'s pickup gives up (no clip in {USE_TIMEOUT} s)", p.sim_time, unit_name(p));
        }
        return;
    }
    if p.weapon != u.slot {
        if p.switching.is_none() {
            start_switch(p, l);
        }
    } else if p.switching.is_none() && p.reloading.is_none() && p.throwing.is_none() && p.using.is_none() && p.skill.toggling.is_none()
        && !skills::overlays_blocked(p) && skills::lets_act(p) {
        if let Some(c) = pickup_clip(p, l, game, &u) {
            if std::env::var("BF_WEAPON_LOG").is_ok() {
                println!("t {:.2}: {} plays {} ({:.2} s at x{}, release {:.2}, grab {:.2})", p.sim_time, unit_name(p), c.name, c.overlay.duration, c.speed,
                         c.release, c.grab);
            }
            u.clip = Some(c);
            u.time = 0.0;
        }
    }
    p.gun_use = Some(u);
}

/// The clips' events (FUN_00117660; see the module notes) for whoever's pickup is under way (the
/// controlled character's; one control passed from finishes it under the AI, as the game's use
/// update runs for any character): release_weapon's drop and grab_weapon's new gun, a use cut
/// short, and a use whose clip is over. A use cut short after the release takes its gun at once
/// (the demo's: the game's slot would stay empty); a dead character's just ends.
#[allow(clippy::too_many_arguments)]
fn swap(mut commands: Commands, mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut game: ResMut<GameData>, mut reserve: ResMut<ammo::Reserve>,
        mut events: ResMut<triggers::TriggerEvents>, mut held_name: ResMut<ammo::HeldName>, lying: Query<&LyingGun>,
        globals: Query<&GlobalTransform>, guns: Res<Guns>,
        (mut meshes, mut materials, mut images, mut bindposes): (ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>, ResMut<Assets<Image>>,
                                                                 ResMut<Assets<bevy::render::mesh::skinning::SkinnedMeshInverseBindposes>>)) {
    for (k, p) in std::iter::once(&mut *player).chain(squad.0.iter_mut()).enumerate() {
        let Some(mut u) = p.gun_use.take() else { continue };
        if p.dead {
            continue;
        }
        let Some(mut l) = p.loaded.take() else {
            p.gun_use = Some(u);
            continue;
        };
        if u.cancelled && u.released && !u.grabbed {
            u.grabbed = true;
            u.grab_due = true;
        }
        if std::mem::take(&mut u.release_due) {
            release(&mut commands, p, &mut l, &game.0, &mut reserve, &globals, guns.mission, guns.log);
        }
        if std::mem::take(&mut u.grab_due) {
            let mut assets = ModelAssets { meshes: &mut meshes, materials: &mut materials, images: &mut images, bindposes: &mut bindposes };
            grab(&mut commands, p, &mut l, &mut game.0, &mut assets, &mut reserve, &mut events, &lying, &u, guns.log);
            // (the HUD's name over the count is the controlled character's)
            if k == 0 {
                held_name.0 = ammo::NAME_TIME;
            }
        }
        let done = u.cancelled || u.clip.as_ref().is_some_and(|c| u.time >= c.overlay.duration);
        p.loaded = Some(l);
        if !done {
            p.gun_use = Some(u);
        } else if guns.log {
            println!("t {:.2}: {}'s pickup ends{}", p.sim_time, unit_name(p), if u.cancelled { " (cut short)" } else { "" });
        }
    }
}

/// release_weapon (FUN_00117660 at 0x118a2b): the gun in hand's rounds into the reserve of its
/// ammo type as far as its stack takes them (FUN_0022b580 the level's ammo item for the type,
/// FUN_0014ce90), the rest kept; the gun let go where the hand holds it (the hand holder's
/// +0x34(1), 0x14af20: its item set's drop, FUN_0014a170), falling and tumbling as a lying gun.
/// Its slot keeps an empty stand-in (`HeldWeapon::empty`) until the grab. The HUD's weapon row
/// leaving the hand plays the switch's first sound
/// (`SWITCH_SOUNDS`, FUN_000cbec0 through FUN_000bb220: the controlled character's only), as the
/// takes have it at the release: #52 take04 7.52 s, #61 take04 6.57 s, take11 300.73 s, matched
/// by spectrogram. Outside a `mission` the gun's vanish timer starts (VANISH_DROPPED: 0x1189e5,
/// the gun just left the hand, so its timer isn't running).
#[allow(clippy::too_many_arguments)]
fn release(commands: &mut Commands, p: &mut Player, l: &mut Loaded, game: &Game, reserve: &mut ammo::Reserve, globals: &Query<&GlobalTransform>,
           mission: bool, log: bool) {
    if p.controlled {
        p.sound_queue.push((SWITCH_SOUNDS[0], 0.9));
    }
    let w = p.weapon;
    let Some(held) = l.weapons.get_mut(w) else { return };
    let mut clip = p.ammo.get(w).map_or(0, |a| a[0]);
    let ammo_type = held.def.ammo_type;
    if ammo_item(game, ammo_type).is_some() {
        clip -= reserve.add(ammo_type, clip);
    }
    let at = globals.get(held.entity).map_or(Transform::from_translation(p.position), |g| g.compute_transform());
    let spin = Vec3::new(p.random(100) as f32 - 50.0, p.random(100) as f32 - 50.0, p.random(100) as f32 - 50.0) * (DROP_SPIN / 50.0);
    commands.entity(held.entity).remove::<ChildOf>().insert((at, Visibility::Inherited, pickups::Thrown { velocity: Vec3::ZERO, spin },
                                                              LyingGun { kind: held.def.name, clip, name: 0 }));
    if !mission {
        commands.entity(held.entity).insert(Vanish(VANISH_DROPPED));
    }
    commands.entity(held.flash).despawn();
    commands.entity(held.light).despawn();
    if log {
        println!("t {:.2}: {} releases {} at {:.2}: {} rounds left in it, reserve type {ammo_type} now {}{}", p.sim_time, unit_name(p), held.def.label,
                 at.translation, clip, reserve.get(ammo_type), if mission { String::new() } else { format!(", gone in {VANISH_DROPPED} s") });
    }
    let empty = commands.spawn((Transform::default(), Visibility::Hidden, ChildOf(l.root))).id();
    (held.entity, held.flash, held.light) = (empty, empty, empty);
    held.spinners.clear();
    (held.in_hand, held.stow, held.second_hand) = (None, [None; 3], None);
    held.empty = true;
    if let Some(a) = p.ammo.get_mut(w) {
        *a = [0, 0];
    }
    p.holding = false;
    p.weapon_dirty = true;
}

/// grab_weapon (FUN_00117660 at 0x117f62): PICKUP_OBJECT from the lying gun (its placement's name)
/// to the level's triggers; a new gun of its type in the slot (FUN_0014ce00) and the hand
/// (FUN_0014d020), with the lying gun's rounds, loaded at once from the reserve (FUN_0022daf0 with
/// 0: the rounds the clip lacks, as far as the reserve has them); the lying gun gone
/// (FUN_000db3e0). The weapon row taking the hand plays the switch's second sound (as at the
/// release: #52 take04 7.95 s, #61 take04 6.88 s).
#[allow(clippy::too_many_arguments)]
fn grab(commands: &mut Commands, p: &mut Player, l: &mut Loaded, game: &mut Game, assets: &mut ModelAssets, reserve: &mut ammo::Reserve,
        events: &mut triggers::TriggerEvents, lying: &Query<&LyingGun>, u: &GunUse, log: bool) {
    let Ok(gun) = lying.get(u.target).map(|g| *g) else { return };
    if p.controlled {
        p.sound_queue.push((SWITCH_SOUNDS[1], 0.9));
    }
    if gun.name != 0 {
        events.0.push(triggers::ObjectEvent { from: gun.name, signal: bf_viewer::bf::triggers::sig::PICKUP_OBJECT, subject: p.trigger_name, data: -1 });
    }
    commands.entity(u.target).despawn();
    let Some(def) = game.weapons.get(&gun.kind).cloned() else { return };
    // (its stance by the type's pickup mode, `stance_for`: Tex's rifle in his cannon's slot held as
    // a rifle, #297)
    let mode = limits(game, p.kind).1;
    let Some(held) = build_weapon(commands, game, assets, &l.model, l.root, def, u.slot, mode) else { return };
    let (size, ammo_type, regen) = (held.def.ammo.max(1), held.def.ammo_type, held.def.ammo_regen);
    if let Some(old) = l.weapons.get(u.slot) {
        commands.entity(old.entity).despawn();
    }
    let (label, stance) = (held.def.label.clone(), held.stance);
    l.weapons[u.slot] = held;
    // the switch clips go by the guns' classes (Tex's Sc_wc_<from>2<to>, Brutus's w2rifle / cannon)
    l.switch_clips = [switch_clip(&l.model, game, &l.weapons, 0), switch_clip(&l.model, game, &l.weapons, 1)];
    let mut clip = gun.clip.max(0);
    if regen <= 0.0 && clip < size && ammo_item(game, ammo_type).is_some() {
        clip += reserve.take(ammo_type, size - clip);
    }
    if let Some(a) = p.ammo.get_mut(u.slot) {
        *a = [clip, 0];
    }
    if let Some(c) = p.regen.get_mut(u.slot) {
        *c = Default::default();
    }
    p.weapon = u.slot;
    p.holding = true;
    p.weapon_dirty = true;
    p.fire_state.reset_burst(0);
    if log {
        println!("t {:.2}: {} grabs {label} into slot {} (stance w{}): {clip} rounds (the lying gun's {}), reserve type {ammo_type} {}", p.sim_time,
                 unit_name(p), u.slot + 1, stance + 1, gun.clip, reserve.get(ammo_type));
    }
}

/// The level's ammo item for an ammo-type (FUN_0022b580: the world's ammo item table at
/// world+0xbe0, by the gun's ammo-type): the `<inventory>` entry of IFSET_AMMO_BOX whose
/// h_e5f51266 is that type. None when the data has none (the LZRs' 11).
fn ammo_item(game: &Game, ammo_type: i64) -> Option<(u32, &bf_viewer::bf::character::ItemType)> {
    game.items.iter().filter(|(_, i)| i.function == AMMO_ITEM && i.kind == ammo_type && ammo_type > 0)
        .min_by_key(|(k, _)| **k).map(|(k, i)| (*k, i))
}

/// The ammo types of the guns in these slots, for the walk-over (FUN_00122ed0: what each slot's
/// holder holds now, FUN_0014dc50 giving the hand holder for the slot in hand): a slot a release
/// has emptied (`HeldWeapon::empty`) holds none.
fn slot_ammo<'a>(weapons: impl Iterator<Item = &'a HeldWeapon>) -> Vec<i64> {
    weapons.filter(|w| !w.empty).map(|w| w.def.ammo_type).collect()
}

/// Whether lying gun `e` is there for a walk-over this frame: shown, with rounds (FUN_0011c170:
/// +0x1f8 above 0), and not one a pickup has grabbed (`grabbed`: gone at the grab, FUN_000db3e0,
/// though its entity waits for the frame's commands).
fn walkable(e: Entity, gun: &LyingGun, vis: Visibility, grabbed: &[Entity]) -> bool {
    gun.clip > 0 && vis != Visibility::Hidden && !grabbed.contains(&e)
}

/// Squad members walking over a lying gun (FUN_0011c540 -> FUN_0011c170; see the module notes):
/// its rounds into the reserve when someone in the squad has a gun of its ammo type, the pickup
/// line, the ammo's pickup sound and PICKUP_OBJECT, and outside a mission its vanish timer
/// (VANISH_WALKED); "(full)" when nothing goes in. Before `swap`. Not the test map's rack
/// (`KeepClip`).
#[allow(clippy::too_many_arguments)]
fn walk_over(mut commands: Commands, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>, mut reserve: ResMut<ammo::Reserve>,
             mut feed: ResMut<pickups::PickupFeed>, mut events: ResMut<triggers::TriggerEvents>, guns: Res<Guns>,
             mut lying: Query<(Entity, &mut LyingGun, &Transform, &Visibility), Without<KeepClip>>) {
    let units = || std::iter::once(&*player).chain(squad.0.iter());
    // the squad's ammo types (FUN_00122ed0 over the squad: each living member's slots)
    let types = slot_ammo(units().filter(|u| u.in_squad() && !u.dead).filter_map(|u| u.loaded.as_ref()).flat_map(|l| &l.weapons));
    let grabbed: Vec<Entity> = units().filter_map(|u| u.gun_use.as_ref()).filter(|g| g.grabbed).map(|g| g.target).collect();
    for k in 0..=squad.0.len() {
        let u = if k == 0 { &mut *player } else { &mut squad.0[k - 1] };
        if u.dead || u.absent || !u.in_squad() || u.loaded.is_none() {
            continue;
        }
        let at = u.position;
        // (the gun a pickup is about to take too: FUN_0011c170 makes no exception; its rounds then
        // come back from the reserve at the grab)
        for (e, mut gun, g, vis) in &mut lying {
            if !walkable(e, &gun, *vis, &grabbed) {
                continue;
            }
            let p = g.translation;
            let (dy, dx, dz) = (p.y - at.y, p.x - at.x, p.z - at.z);
            if !(-WALK_DOWN..=WALK_UP).contains(&dy) || dx * dx + dz * dz >= WALK_ACROSS_SQ {
                continue;
            }
            let Some(ammo_type) = game.0.weapons.get(&gun.kind).map(|d| d.ammo_type) else { continue };
            let Some((_, item)) = ammo_item(&game.0, ammo_type) else { continue };
            if !types.contains(&ammo_type) {
                continue;
            }
            let name = game.0.strings.get(&item.message).cloned().filter(|_| item.message != 0).unwrap_or_else(|| item.label.clone());
            let added = reserve.add(ammo_type, gun.clip);
            feed.add(name.clone(), added);
            if added == 0 {
                continue;
            }
            gun.clip -= added;
            if item.sound != 0 {
                u.sound_queue.push((item.sound, 1.0));
            }
            if gun.name != 0 {
                events.0.push(triggers::ObjectEvent { from: gun.name, signal: bf_viewer::bf::triggers::sig::PICKUP_OBJECT, subject: u.trigger_name, data: -1 });
            }
            // (0x11c3fd: outside a mission, running or not)
            if !guns.mission {
                commands.entity(e).insert(Vanish(VANISH_WALKED));
            }
            if guns.log {
                println!("t {:.2}: {} takes {added}x {name} from a lying gun at {p:.2} ({} left in it), reserve type {ammo_type} {}{}", u.sim_time, unit_name(u),
                         gun.clip, reserve.get(ammo_type), if guns.mission { String::new() } else { format!(", gone in {VANISH_WALKED} s") });
            }
        }
    }
}

/// The vanish timers (`Vanish`, FUN_00157c40 for a weapon): each running one down by the frame's
/// dt, and a gun whose time is out taken out of the world (FUN_000d9840 -> FUN_000db3e0).
fn vanish(mut commands: Commands, time: Res<Time>, guns: Res<Guns>, player: Res<Player>, mut timed: Query<(Entity, &mut Vanish, &LyingGun)>) {
    let dt = frame_dt(&time);
    for (e, mut v, gun) in &mut timed {
        v.0 -= dt;
        if v.0 > 0.0 {
            continue;
        }
        commands.entity(e).try_despawn();
        if guns.log {
            println!("t {:.2}: a lying gun (h_{:08x}, {} rounds) vanishes: its timer ran out", player.sim_time, gun.kind, gun.clip);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FUN_0014dfd0's timers on a dead character's guns: 20 s outside a mission, 300 s in one for
    /// a squad member, none for the others there; release_weapon's and the walk-over's are 20 and
    /// 10 s (0x1189e5, 0x11c3fd).
    #[test]
    fn the_vanish_timers() {
        assert_eq!(death_vanish(false, false), Some(20.0));
        assert_eq!(death_vanish(false, true), Some(20.0));
        assert_eq!(death_vanish(true, true), Some(300.0));
        assert_eq!(death_vanish(true, false), None);
        assert_eq!((VANISH_DROPPED, VANISH_WALKED), (20.0, 10.0));
    }

    /// The squad's `limit-weapon` (objecttypes-common): Brutus 1 8 / 2 1, Flint 1 8 / 0 8, Hawk
    /// 0 8 / 1 8, Tex 2 1 / 2 1.
    const BRUTUS: [[i64; 2]; 2] = [[1, 8], [2, 1]];
    const FLINT: [[i64; 2]; 2] = [[1, 8], [0, 8]];
    const HAWK: [[i64; 2]; 2] = [[0, 8], [1, 8]];
    const TEX: [[i64; 2]; 2] = [[2, 1], [2, 1]];

    /// Who can take what (the takes: Brutus and Tex refuse the pistols, Hawk the cannons; an
    /// LZR-10 replaced Flint's Jax-iP, her pistol slot).
    #[test]
    fn the_slots_take_their_classes() {
        let takes = |c: [[i64; 2]; 2], class: i64| slot_takes(c, 0, class) || slot_takes(c, 1, class);
        assert!(!takes(BRUTUS, 0) && takes(BRUTUS, 1) && takes(BRUTUS, 2));
        assert!(!takes(TEX, 0) && takes(TEX, 1) && takes(TEX, 2));
        assert!(takes(HAWK, 0) && takes(HAWK, 1) && !takes(HAWK, 2));
        assert!(takes(FLINT, 0) && takes(FLINT, 1) && !takes(FLINT, 2));
        assert!(!slot_takes(FLINT, 0, 8));
        // no limit-weapon: every class of the nine bytes in either slot (0x14c5d0)
        assert!(slot_takes([[-1; 2]; 2], 1, 2));
        assert!(slot_takes([[-1; 2]; 2], 0, 7) && !slot_takes([[-1; 2]; 2], 0, 9));
        // a creature's weapon (class 7: m04_a's Psionic Blast) isn't the squad's to take
        assert!([BRUTUS, FLINT, HAWK, TEX].iter().all(|&c| !slot_takes(c, 0, 7) && !slot_takes(c, 1, 7)));
    }

    /// A gun of `ammo_type` in a slot, or the stand-in a release leaves (`empty`).
    fn held(ammo_type: i64, empty: bool) -> HeldWeapon {
        HeldWeapon { def: WeaponDef { ammo_type, ..default() }, stance: 0, entity: Entity::PLACEHOLDER, spinners: vec![], light: Entity::PLACEHOLDER,
                     data_light: false, flash: Entity::PLACEHOLDER, muzzle: Hardpoint { point: Vec3::ZERO, rot: Quat::IDENTITY }, shell: None,
                     fire_dir: Vec3::NEG_Z, in_hand: None, stow: [None; 3], second_hand: None, empty }
    }

    /// The walk-over's ammo types are what the slots hold (FUN_00122ed0): between a release and
    /// its grab the emptied slot's gun doesn't count (#17 review 2: a full stack's "(full)" line
    /// every frame over the gun just dropped).
    #[test]
    fn an_emptied_slot_uses_no_ammo() {
        let guns = [held(2, false), held(1, true)];
        assert_eq!(slot_ammo(guns.iter()), vec![2]);
        assert_eq!(slot_ammo([held(1, false)].iter()), vec![1]);
    }

    /// A gun a pickup has grabbed is gone for the walk-over that frame, though its entity waits
    /// for the frame's commands (#17 review 1: the #61 recreation with 10 rounds in reserve read
    /// "6 / 16", the L-Shot-50's 6 taken twice); a gun without rounds or hidden isn't there either.
    #[test]
    fn a_grabbed_gun_is_gone_for_the_walk_over() {
        let (a, b) = (Entity::from_raw(1), Entity::from_raw(2));
        let gun = LyingGun { kind: 0, clip: 6, name: 0 };
        assert!(walkable(a, &gun, Visibility::Inherited, &[b]));
        assert!(!walkable(a, &gun, Visibility::Inherited, &[a]));
        assert!(!walkable(a, &LyingGun { clip: -1, ..gun }, Visibility::Inherited, &[]));
        assert!(!walkable(a, &gun, Visibility::Hidden, &[]));
    }

    /// The clip holds the character where it stands while it plays, when its overlay says so
    /// (h_f6a410e6, FUN_001195c0; #293): not before it starts (the switch before it moves
    /// freely), nor once it's over, nor for an overlay without the flag.
    #[test]
    fn the_clip_holds_the_character_while_it_plays() {
        let clip = |holds| PickupClip { overlay: OverlayClip { clip: 0, duration: 1.0, events: vec![], mask: vec![], upper: vec![], whole: vec![] },
                                        name: String::new(), speed: 1.0, release: 0.2, grab: 0.6, holds };
        let mut u = GunUse { target: Entity::PLACEHOLDER, slot: 0, class: 1, timeout: USE_TIMEOUT, clip: None, time: 0.0, released: false,
                             grabbed: false, release_due: false, grab_due: false, cancelled: false };
        assert!(!u.holds_still());
        u.clip = Some(clip(true));
        assert!(u.holds_still());
        u.time = 0.99;
        assert!(u.holds_still());
        u.time = 1.0;
        assert!(!u.holds_still());
        u.time = 0.5;
        u.clip = Some(clip(false));
        assert!(!u.holds_still());
    }

    /// FUN_00121ac0's slot: Flint's pistol to slot 2 and rifle to 1; Brutus's cannon to 2 and rifle
    /// to the slot in hand; Tex's either to the slot in hand.
    #[test]
    fn a_gun_goes_to_its_slot() {
        let both = [true, true];
        assert_eq!(slot_for(1, FLINT, 0, 1, both), 2);
        assert_eq!(slot_for(1, FLINT, 1, 2, both), 1);
        assert_eq!(slot_for(1, HAWK, 0, 2, both), 1);
        assert_eq!(slot_for(2, BRUTUS, 2, 1, both), 2);
        assert_eq!(slot_for(2, BRUTUS, 1, 2, both), 2);
        assert_eq!(slot_for(2, BRUTUS, 0, 1, both), 0);
        assert_eq!(slot_for(3, TEX, 1, 1, both), 1);
        assert_eq!(slot_for(3, TEX, 2, 2, both), 2);
        assert_eq!(slot_for(3, TEX, 2, 1, [false, false]), 2);
    }

    /// FUN_00121920's motion types: Tex's DROP_<hand>_PICKUP_<picked> (take11: MK-ASLT in hand,
    /// L-Shot-50 picked: 16 DROP_RIFLE_PICKUP_RIFLE), Brutus's 33 for either slot, Flint's and
    /// Hawk's 33 / 34 by the slot.
    #[test]
    fn the_clip_follows_the_mode() {
        assert_eq!(pickup_motion(3, 1, 1, 1, 1), 16);
        assert_eq!(pickup_motion(3, 1, 1, 1, 2), 15);
        assert_eq!(pickup_motion(3, 2, 2, 2, 1), 14);
        assert_eq!(pickup_motion(3, 2, 2, 2, 2), 13);
        assert_eq!(pickup_motion(2, 2, 2, 1, 2), 33);
        assert_eq!(pickup_motion(2, 1, 1, 1, 1), 33);
        assert_eq!(pickup_motion(1, 1, 1, 1, 1), 33);
        assert_eq!(pickup_motion(1, 2, 2, 0, 0), 34);
        // the target not in hand: types no squad set defines
        assert_eq!(pickup_motion(1, 2, 1, 1, 0), 20);
        assert_eq!(pickup_motion(3, 2, 1, 1, 2), 18);
    }
}

//! Weapons: definitions from objecttypes-<level>.xmb, models from objects-<level>.xmb, and the
//! hardpoints that join a weapon to a character's hand.
//!
//!   <h_e275fb80 weapon-type=.. h_e93a46d6=RATE h_e33f08b8="FIRE SOUNDS" ammo-count=..>  weapon
//!     <base ... stringtable-name=LABEL> <base mesh-name=ARCHETYPE> <base name=WEAPON object-type=20/>
//!     <bullet h_113d3e3c=SPEED range=.. effect-name=FLIGHT h_08d28037=HIT> ...
//!     <h_f738f302 h_e41cb542="MUZZLE HARDPOINTS"/>   (a list: a turret names two)
//! SPEED is m/s (1: instant hit); FLIGHT and HIT are effect types (objecttypes <effect>): the
//! shot's tracer or bolt (laser_bolt_s, cuttertrail_ribbon, tracer...) and what it does where it
//! lands (laserhit_s, sminigun_hit...).
//!
//! A weapon archetype is either a plain mesh archetype or a compound (PART list) like the
//! minigun, whose barrel part spins about its joint axis. Both carry HARDPOINTs (point +
//! ORIENTATION quaternion stored w x y z, in the owner's local frame). A held weapon hangs from
//! the trigger hand: the character's TRIGGER_HAND hardpoint lined up with the weapon's
//! TRIGGER_GRIP:
//!   hand-local = hand.point + hand.rot * grip.rot^-1 * (v - grip.point)
//! Checked against the original clips: in the ready-pose clips (Sc_w2_rp_*) every squad weapon
//! then points straight ahead and level (within a few degrees) for every character, and in Tex's
//! carry clips the support hand rests on the gun's FOREGRIP (2.7 cm off on average). A weapon not
//! in hand is stowed the same way, its HOLSTER hardpoint on the character's STOW hardpoint for its
//! inventory slot: Tex's ramps (vertical), across Brutus's back (diagonal), Flint's and Hawk's
//! sidearms on the hip, barrel down.

use std::collections::HashMap;

use bevy::math::{Quat, Vec3};

use super::bxml::{Element, Value};
use super::character::{read_geoset, Game, Geoset};
use super::hash::h;

const H_WEAPON: u32 = 0xE275_FB80;
const H_HIT_EFFECT: u32 = 0x08D2_8037;
const H_RATE: u32 = 0xE93A_46D6;
const H_FIRE_SOUNDS: u32 = 0xE33F_08B8;
const H_BULLET_SPEED: u32 = 0x113D_3E3C;
/// weapon: the muzzle block (`<h_f738f302>`), which holds the muzzle hardpoints' names
const H_MUZZLE_REF: u32 = 0xF738_F302;
/// the muzzle block's h_e41cb542: a list of hardpoint names (the weapon data's vector at
/// +0x140; see `WeaponDef::muzzles`)
const H_MUZZLE_REF_NAME: u32 = 0xE41C_B542;
/// item base: HUD icon texture (+ its size, h_e2bc2c81 x h_f71189bb, 128 x 64 for guns)
const H_HUD_ICON: u32 = 0xE5EC_3F1F;
const H_RELOAD_SOUND: u32 = 0x0CBF_5B9C;
/// weapon: the empty-fire sound (named by its use, a guess at the name): played by
/// default.xbe's FUN_002327f0 when the trigger is pulled on an empty clip with nothing to
/// reload (the weapon type's +0x114). Ballistic guns h_f740ecdd, energy guns h_16bbf95c.
const H_EMPTY_SOUND: u32 = 0xE6AC_AFF7;
/// weapon: seconds per round recharged (named by its use, a guess at the name): read by
/// FUN_0022ebc0 from the weapon type's +0x14c; 0 for every gun but the LZR-10 (2), LZR-23
/// (1.25) and LZR-50 (1).
const H_AMMO_REGEN: u32 = 0x1D1E_0E9C;
/// grenade: the weapon definition its explosion uses
const H_PROJECTILE: u32 = 0x053C_429F;
/// object event sound (state 7); for the Frag it is the sound heard as the throw starts charging
const H_EVENT_SOUND: u32 = 0xFA04_E025;
/// bullet: impact sound (a grenade projectile's explosion)
const H_IMPACT_SOUND: u32 = 0xE561_8348;
/// bullet: the effect type it plays where it goes off (an explosion's flash, fireball, light and
/// sound: objecttypes `<effect>`, object-type 17)
const H_BULLET_EFFECT: u32 = 0xEC23_D593;
/// bullet: the ground decal it leaves (objecttypes `<decal>`; the Frag's scorch h_ff1b711e)
const H_BULLET_DECAL: u32 = 0x06A2_7365;
/// item base: how it is used (the XBE's IOU_ enum: 2 IOU_PLACE_ON_GROUND (Roller, Sentry),
/// 3 IOU_THROW_TO_USE (Frag and the other thrown grenades))
const H_USE_TYPE: u32 = 0x1EE2_F4ED;
/// item: a proximity mine's trigger radius (m, an int: 3 on every Sentry definition in every
/// level). default.xbe's range test 0x146cb0 loads it from the item type (+0x178) and squares
/// it against the 3D distance to each character.
const H_PROXIMITY_RADIUS: u32 = 0x0A81_1E94;
/// object base: the effects attached to the object (8 slots of `<h_081398d6 effect-name
/// hardpoint-name>`); a grenade's first is its trail (the Frag's h_10a5508f: grenade_trail and
/// the hiss 10318b29)
const H_ATTACHED_EFFECTS: u32 = 0x18B6_AB72;
/// Damage: h_04ea9251, seconds the damage is dealt over (the Gas's 5.5; 0 for the rest - an
/// inference from the Gas recording's ~5.7 s of steady damage)
const H_DAMAGE_TIME: u32 = 0x04EA_9251;
/// weapon: the crosshair's lock box scale (name not recovered; see `WeaponDef::lock_box`)
const H_LOCK_BOX: u32 = 0xE1E4_1C2D;
const H_PARTS: u32 = 0xE487_418A;
const H_MESHES: u32 = 0xEA78_3362;
const H_JOINT: u32 = 0x151A_CE78;
/// A joint's limits, `<h_fab3703e min max>` (parser FUN_0023e3a0: the joint record's +0x3c
/// pair). Sliding and hinge joints carry one (sdm_e34's doors -3.4e38..3.4e38, e10's hinged
/// door -2 pi..2 pi).
const H_JOINT_LIMITS: u32 = 0xFAB3_703E;

/// Weapon hardpoints (names are hashes).
pub const TRIGGER_GRIP: u32 = 0x1F34_55A8;
/// where the support hand goes
pub const FOREGRIP: u32 = 0xEA5E_02F2;
pub const MUZZLE: u32 = 0xECC0_AA78;
/// where a stowed weapon is held (every squad weapon has it)
pub const HOLSTER: u32 = 0x10AB_40AC;
/// Character hand hardpoints: the trigger hand (on the +X hand bone) and the support hand.
pub const TRIGGER_HAND: u32 = 0x1D4B_6D69;
pub const SUPPORT_HAND: u32 = 0xF847_4385;
/// Character stow hardpoints, one per inventory slot (on the back, or the hip for sidearms).
pub const STOW: [u32; 2] = [0xFF4C_691B, 0xF4CC_F78C];
/// Character hardpoint of the second hand's gun (Tex's berserk): the inventory's second-hand
/// holder (+0x178) names it (0x14c7b8, as +0x144's names TRIGGER_HAND at 0x14c791), and both hand
/// holders (vtable 0x39b138) hold a weapon by its TRIGGER_GRIP (+0x6c = 0x14c5b0). Only Tex's
/// model has it, on his left hand's part (h_f1becca5 in objects-common).
pub const SECOND_HAND: u32 = 0x15AB_6D29;

#[derive(Clone, Copy, Debug)]
pub struct Hardpoint {
    pub point: Vec3,
    pub rot: Quat,
}

impl Hardpoint {
    /// Transform taking `child` coordinates to `self`'s owner when `other` (a hardpoint of the
    /// child) is lined up with `self`: (rotation, translation).
    pub fn mount(&self, other: &Hardpoint) -> (Quat, Vec3) {
        let r = self.rot * other.rot.inverse();
        (r, self.point - r * other.point)
    }
}

#[derive(Clone, Debug, Default)]
pub struct WeaponDef {
    pub name: u32,
    pub label: String,
    pub weapon_type: i64,
    /// archetype of the model
    pub archetype: u32,
    /// shots per second
    pub rate: f32,
    pub ammo: i64,
    pub fire_sounds: Vec<u32>,
    pub bullet_speed: f32,
    pub range: f32,
    /// the muzzle hardpoints on the model, every name of the muzzle block's list (H_MUZZLE_REF_NAME,
    /// the weapon data's vector at +0x140), in order; MUZZLE when the block has no list. The game
    /// uses each: the muzzle effect is attached at every one (FUN_00232060, an effect object per
    /// muzzle at weapon+0x36c + 8 i), each one's aim is turned every frame (FUN_0022e4c0), and
    /// the fire loop takes turns (`next_muzzle`). Every hand-held gun names one, MUZZLE
    /// (h_ecc0aa78); two, h_f9eed613 and h_e2e787a9, on the four turrets (object-type 29:
    /// h_13e090b2, h_e2bbcbc5, h_f35e1e0e, h_fb6e2a46) in e05, e16, m03_b, m08_b, m09_b/c/x and
    /// m14_a-d, which the demo doesn't have; none ("") on 14 definitions without a model (e01's
    /// h_15889e57, mesh-name "")
    pub muzzles: Vec<u32>,
    /// weapon handling sound (picked up / drawn)
    pub pickup_sound: u32,
    /// HUD icon texture (0 if none)
    pub icon: u32,
    /// seconds (0 in most definitions: the game times reloads by animation)
    pub reload_time: f32,
    pub reload_sound: u32,
    /// dry fire (H_EMPTY_SOUND; 0 if none)
    pub empty_sound: u32,
    /// seconds per round the clip recharges by itself (H_AMMO_REGEN; 0: it reloads from the
    /// squad's reserve instead)
    pub ammo_regen: f32,
    /// grenades: fuse seconds (attribute `timer`; 0 for guns)
    pub fuse: f32,
    /// grenades: definition of the explosion (a weapon with Damage radius and impact sound)
    pub projectile: u32,
    /// grenades: heard when the throw starts charging (pin pulled / gauge)
    pub arm_sound: u32,
    /// HUD crosshair texture (`reticule-prefix`)
    pub reticle: u32,
    /// object sound (sound-name)
    pub object_sound: u32,
    pub impact_sound: u32,
    /// Damage radius
    pub blast_radius: f32,
    /// Damage max (at the centre of a blast) and min (a shot does min..max)
    pub damage: f32,
    pub damage_min: f32,
    /// how far aiming zooms in (h_efce7f77: the field of view is divided by it; 0, no zoom:
    /// MK-ASLT 2, Foley 356 3, L-Shot-50 5, L-Shot-75 10)
    pub zoom: f32,
    /// AMMO_* type (1 ballistic, 2 ballistic high rate, 5 rail, 6 shotgun, 7 cutter, 11 laser ...):
    /// picks the world material's hit sounds and effect
    pub ammo_type: i64,
    /// effect types: the shot in flight (bullet effect-name) and where it lands (h_08d28037)
    pub flight_effect: u32,
    pub hit_effect: u32,
    /// Damage damage-type (the combat-targets' per-type factors: the Frag's 10, the Gas's 4)
    pub damage_type: i64,
    /// Damage h_04ea9251: seconds the damage is spread over (see H_DAMAGE_TIME)
    pub damage_time: f32,
    /// bullet h_ec23d593: effect type played where the bullet goes off (0 if none)
    pub blast_effect: u32,
    /// bullet h_06a27365: ground decal it leaves (0 if none)
    pub decal: u32,
    /// items: function-type (the XBE's IFSET_ enum: 0 none (thrown grenades), 8
    /// IFSET_PROXIMITY_EXPLOSIVE (Sentry), 13 IFSET_ROLLING_BOMB (Roller))
    pub function_type: i64,
    /// items: group-type (2 IG_GRENADE, 3 IG_EXPLOSIVE; 0 for the enemies' grenades)
    pub group_type: i64,
    /// items: h_e5f51266, the item's index in its group ([type+0x198], beside the group at
    /// [type+0x194]): with the group the key of its stack in an inventory (FUN_00149d30). Frag 0,
    /// Gas 1, Sonic 4, Energy 5, Light 32; Sentry 1 (h_e5f1f063) or 3 (h_f73de83d), Roller 4
    /// (todo/196 leads §3.2)
    pub item_index: i64,
    /// items: h_1ee2f4ed, how it's used (see H_USE_TYPE)
    pub use_type: i64,
    /// items: how many fit in the inventory (stack-limit: 10 for the squad's grenades)
    pub stack_limit: i64,
    /// the effect type attached to the object (h_18b6ab72's first slot; 0 if none)
    pub attached_effect: u32,
    /// items: a proximity mine's trigger radius (H_PROXIMITY_RADIUS; 0 if none)
    pub proximity_radius: f32,
    /// what the fire loop reads besides the above: bursts, accuracy, tracers, muzzle and shell
    /// effects, damage falloff (see `ShotData`)
    pub shots: ShotData,
    /// Damage h_011cb154 (the Damage record's +0x10, weapon type +0x200): how a blast object's
    /// share falls off (FUN_0022c4d0): false, half the blast's roll anywhere inside the radius
    /// (the Therm Sweeper's, the Sweeper V's); true, the whole roll to half the radius, then
    /// (1 - 2 (d / r - 0.5))^2 of it to the edge
    pub blast_core: bool,
    /// a mesh projectile's bullet (bullet-type 2 with a projectile object, `MeshBullet`); None
    /// for every other bullet
    pub mesh_bullet: Option<MeshBullet>,
    /// `<weapon>` h_e1e41c2d (H_LOCK_BOX): the crosshair's lock box scale, s. The weapon-type
    /// handler stores it at type +0x2dc (0x193d7d -> 0x193dbd), the weapon data's +0x134, and
    /// FUN_000c5d20 puts it in the target selection's +0x34: the box round the crosshair is
    /// 33.6 x s HUD units each way (play_reticle.rs). LZR-50 0.85, Jax-iC 0.65, RVG50 Minigun
    /// 0.6 (read live in #141); 0 never locks (the iKhan-GPL)
    pub lock_box: f32,
    /// bullet h_f0a56297 (data+0x58, the bullet block's +0x20; `MeshBullet::after` for a mesh
    /// projectile): FUN_00232520 adds it x the bullet's speed to a mesh projectile's reach (the
    /// Therm Sweeper's 5 s x 30 m/s)
    pub bullet_after: f32,
    /// bullet `<mirv>` h_11f515c5 (data+0xb4, the bullet block's +0x7c, the WM_ enum;
    /// `MeshBullet::mirv_mode`): FUN_00232520 adds the range again to a mesh projectile's reach
    /// for WM_MIRV_ON_EXPLODE (2), 0.75 x it for WM_MIRV_ON_APOGEE (1)
    pub mirv_mode: i64,
}

impl WeaponDef {
    /// The first muzzle hardpoint (0: none), the one a gun with a single muzzle fires from: every
    /// gun the demo's characters hold.
    pub fn muzzle(&self) -> u32 {
        self.muzzles.first().copied().unwrap_or(0)
    }

    /// The muzzle after muzzle `i` (an index into `muzzles`): after each shot the fire loop moves
    /// the weapon's muzzle index (+0x1fc) on by one, back to 0 past the last, when it has more than
    /// one (FUN_0022f5e0, called once the shot, FUN_0022f1d0, has left from muzzle +0x1fc). So a
    /// two-muzzle turret fires its barrels in turn. Its answer (true for such a weapon) also has
    /// the fire loop FUN_0022f2a0 set off the muzzle effect after every shot (0x22f4e3, rather
    /// than going straight on to the next round of a zero-delay burst), and that effect
    /// (FUN_0022f170, at 0x22f56d) is the one at the index as moved: the flash shows at the barrel
    /// that fires next, not the one that fired (read from the code; no take of a turret).
    pub fn next_muzzle(&self, i: usize) -> usize {
        if self.muzzles.len() > 1 && i + 1 < self.muzzles.len() { i + 1 } else { 0 }
    }
}

/// The `<bullet>` of a mesh projectile (bullet-type 2, BTYPE_MESH_PROJECTILE: FUN_0022f1d0 case 2
/// launches the bullet's object, FUN_0022fb50). Offsets are the bullet block's (loaders
/// FUN_0018a8d0 / FUN_0018a7a0; the block is at +0x210 of the weapon type), with the projectile
/// flag each sets (FUN_00227dd0 copies them to the projectile's +0x220) and the code that reads
/// it (the projectile's tick FUN_00226cc0, the MIRV split FUN_00227f80). Names not in the
/// string table are read from their use.
#[derive(Clone, Debug, Default)]
pub struct MeshBullet {
    /// bullet mesh-name: the projectile's object type (objecttypes, object-type 21: the Therm
    /// Sweeper's missile h_e9327576, its model, attached effect and sound)
    pub object: u32,
    /// bullet radius (m): its physics body's (0.1 for the Therm Sweeper); its centre is this far
    /// off what it touches, where it goes off
    pub radius: f32,
    /// h_07cdf040 (+0x14, flag 8): after the straight phase it steers to its target (or aim point)
    pub homes: bool,
    /// h_ea5327e6 (+0x18): the straight phase, as a share of its life (projectile +0x230)
    pub straight: f32,
    /// h_f69b7d4c (+0x1c, flag 0x40): it looks for targets (at the split, 30 degrees; a child
    /// every 0.25 s, 60 degrees: FUN_002237c0, FUN_00226b80)
    pub seeks: bool,
    /// h_f0a56297 (+0x20): the second timer (s), run down once its life is out (+0x1d4)
    pub after: f32,
    /// h_0976a9dc (+0x28, flag 0x10): it explodes where it is when both timers are out
    /// (FUN_00227760)
    pub explode_at_end: bool,
    /// h_0b03ece5 (+0x29, flag 4): steered every frame, held to the bullet's speed (FUN_002266d0)
    pub steady: bool,
    /// h_0bafbd6f (+0x91, flag 0x20): the proximity fuse (a hostile within Damage radius x 0.375
    /// that it has passed, checked every 0.25 s)
    pub fuse: bool,
    /// `<mirv>` h_fe04300d (+0x78): how many it splits into (capped at 6 by FUN_00227f80)
    pub mirv_count: i64,
    /// `<mirv>` h_11f515c5 (+0x7c): WM_ enum at 0x3be7f4: 0 WM_NO_MIRV, 1 WM_MIRV_ON_APOGEE, 2
    /// WM_MIRV_ON_EXPLODE
    pub mirv_mode: i64,
    /// `<mirv>` h_e8ebd6e0 (+0x80): the children's object type (0: the bullet's own)
    pub mirv_object: u32,
    /// `<mirv>` spread-min / spread-max (+0x84 / +0x88): the split turns each child by
    /// spread-max x 90 degrees (FUN_00227f80; spread-min isn't read there)
    pub spread: [f32; 2],
}

/// The `<weapon>` attributes the shot code reads (play_shots.rs), with where the loader at
/// 0x193b70 stores them in the weapon data (data+X, the record a weapon object points at from
/// +0x228) and the function that uses them. Names not in the game's string table are the
/// tester's guesses from their use.
#[derive(Clone, Debug, Default)]
pub struct ShotData {
    /// h_019c314a (data+0xd0): shots per second while the holder's +0x7a8 is set (the scope,
    /// medium confidence): FUN_0022f0e0 / FUN_0022dc00 through vtable +0x15c (0x22da30)
    pub scoped_rate: f32,
    /// h_e60e2074 (data+0xd4): the muzzle effect's own rate parameter (FUN_00232060)
    pub effect_rate: f32,
    /// h_e6c60892 (data+0xd8): seconds between the shots of a burst (FUN_0022f0e0)
    pub burst_delay: f32,
    /// h_e4076713 (data+0xdc, int): shots per trigger cycle (FUN_0022f0e0). Bower 6, others 1
    pub burst_count: i64,
    /// h_e704fd69 (data+0x130): degrees each shot after a zero-delay burst's first is turned
    /// by from the one before, about the world's x axis and then its vertical (FUN_0022e1d0)
    pub burst_spread: f32,
    /// h_fd88830d (data+0x12c): muzzle effect type, one per muzzle hardpoint (FUN_00232060),
    /// set off on each shot (FUN_0022f170). 0 if none
    pub muzzle_effect: u32,
    /// h_ea1abbbe (data+0x150 list): shell-eject hardpoint on the weapon model (0 if none)
    pub shell_hardpoint: u32,
    /// h_19b21bf5 (data+0x160): the object type thrown out of it (h_16415157, a casing); 0 if
    /// none (FUN_00231b10 throws nothing then: the Bower has a hardpoint and no casing)
    pub shell_object: u32,
    /// h_fdc93b33 (data+0x164): casings per trigger cycle, added to weapon+0x208 (FUN_0022f2a0)
    pub shell_rate: f32,
    /// h_fb327295 (data+0x4..0x14, loader 0x18a1d0, read as ints): {min, max, h_01af2634 the
    /// cap while scoped, h_10b6f49b lost per shot, h_19707b5c regained per second}
    /// (FUN_00222ac0 / 00222b30 / 00222c10)
    pub accuracy: [f32; 5],
    /// Damage h_fb124e6c (data+0x34, the Damage record's +0x1c; loader 0x18a427): falloff mode
    /// over the range, FUN_00223780 (1 linear, 2 squared, 3 full to half range then linear,
    /// else none). Bower 2, every other gun 0
    pub falloff: i64,
    /// bullet bullet-type (bullet+8): the switch in FUN_0022f1d0. 4 is an instant ray
    /// (FUN_002317e0), whatever the bullet's speed says
    pub bullet_type: i64,
    /// bullet h_eeb9e75a (data+0xa4): shots without the flight effect between two with it
    /// (FUN_002317e0's counter at weapon+0x290). MK 2, Minigun 3, others 0
    pub tracer_gap: i64,
    /// bullet h_1835f15e (bullet +0x2c, data+0x64; FUN_0018a8d0): the hits on objects a shot
    /// goes on after (#238). A bolt's byte +1 (FUN_0022bac0 copies the low byte): with
    /// `pass_through` the objects it passes, else the bounces off them (FUN_00225440); an instant
    /// ray with `pass_through` takes n1 + n2 hits, this the walls' share (FUN_00230860). The
    /// cutters and the Ion Beamer 5, the Rails 1, others 0
    pub object_hits: i64,
    /// bullet h_fe4f3035 (bullet +0x30, data+0x68): the same for characters and, for a bolt's
    /// ricochet, for any hit that isn't an object's (byte +2). The Rails 500, others 0
    pub character_hits: i64,
    /// bullet h_11caf57b (bullet +0x34, data+0x6c, a byte): the shot passes through what it hits
    /// while `object_hits` / `character_hits` last (a bolt's flag 0x20). The Rails, the Ion
    /// Beamer (and the Saryl-45, with no hits to spend), not the cutters
    pub pass_through: bool,
}

/// `ShotData` of a `<weapon>` element and its `<bullet>`.
fn shot_data(w: &Element, bullet: Option<&&Element>) -> ShotData {
    let all = w.walk();
    let num = |e: Option<&Element>, a: u32| e.and_then(|e| e.attr(a)).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32)));
    let hash = |a: u32| w.attr(a).and_then(|v| v.as_hash()).filter(|&x| x != h(""));
    let accuracy = all.iter().find(|e| e.name == 0xFB32_7295).copied();
    let damage = all.iter().find(|e| e.name == h("Damage")).copied();
    let rate = num(Some(w), 0xE93A_46D6).unwrap_or(2.0);
    ShotData {
        scoped_rate: num(Some(w), 0x019C_314A).unwrap_or(rate),
        effect_rate: num(Some(w), 0xE60E_2074).unwrap_or(rate),
        burst_delay: num(Some(w), 0xE6C6_0892).unwrap_or(0.0),
        burst_count: num(Some(w), 0xE407_6713).map_or(1, |v| v as i64).max(1),
        burst_spread: num(Some(w), 0xE704_FD69).unwrap_or(0.0),
        muzzle_effect: hash(0xFD88_830D).unwrap_or(0),
        shell_hardpoint: hash(0xEA1A_BBBE).unwrap_or(0),
        shell_object: hash(0x19B2_1BF5).unwrap_or(0),
        shell_rate: num(Some(w), 0xFDC9_3B33).unwrap_or(1.0),
        accuracy: [h("min"), h("max"), 0x01AF_2634, 0x10B6_F49B, 0x1970_7B5C].map(|a| num(accuracy, a).unwrap_or(0.0)),
        falloff: damage.and_then(|d| d.attr(0xFB12_4E6C)).and_then(|v| v.as_i64().or(v.ints().first().copied())).unwrap_or(0),
        bullet_type: bullet.and_then(|b| b.attr(h("bullet-type"))).and_then(|v| v.as_i64().or(v.ints().first().copied())).unwrap_or(0),
        tracer_gap: num(bullet.copied(), 0xEEB9_E75A).map_or(0, |v| v as i64),
        object_hits: num(bullet.copied(), 0x1835_F15E).map_or(0, |v| v as i64),
        character_hits: num(bullet.copied(), 0xFE4F_3035).map_or(0, |v| v as i64),
        pass_through: bullet.and_then(|b| b.attr(0x11CA_F57B)).and_then(|v| v.as_i64()).is_some_and(|v| v != 0),
    }
}

/// Hash values of an attribute stored either as a list or as repeated attributes.
pub fn hashes(e: &Element, name: u32) -> Vec<u32> {
    e.attrs.iter().filter(|(k, _)| *k == name).flat_map(|(_, v)| match v {
        Value::List(l) => l.iter().filter_map(|x| x.as_hash()).collect::<Vec<_>>(),
        other => other.as_hash().into_iter().collect(),
    }).filter(|&x| x != 0).collect()
}

/// Weapon definitions of an objecttypes document.
pub fn parse_weapons(root: &Element, strings: &HashMap<u32, String>) -> Vec<WeaponDef> {
    let mut out = vec![];
    for w in root.walk().into_iter().filter(|e| e.name == H_WEAPON) {
        let all = w.walk();
        let find = |attr: u32| all.iter().find_map(|e| e.attr(attr));
        let Some(name) = all.iter().find(|e| e.attr(h("object-type")).is_some()).and_then(|e| e.attr(h("name"))).and_then(|v| v.as_hash())
            else { continue };
        let bullet = all.iter().find(|e| e.name == h("bullet"));
        let f32_of = |e: Option<&&Element>, a: u32, d: f32| e.and_then(|e| e.attr(a)).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32))).unwrap_or(d);
        let label = find(h("stringtable-name")).and_then(|v| v.as_hash()).and_then(|s| strings.get(&s).cloned())
            .unwrap_or_else(|| format!("h_{name:08x}"));
        out.push(WeaponDef {
            name,
            label,
            weapon_type: w.attr(h("weapon-type")).and_then(|v| v.as_i64()).unwrap_or(0),
            archetype: find(h("mesh-name")).and_then(|v| v.as_hash()).unwrap_or(0),
            rate: f32_of(Some(&w), H_RATE, 2.0),
            ammo: w.attr(h("ammo-count")).and_then(|v| v.as_i64()).unwrap_or(0),
            fire_sounds: hashes(w, H_FIRE_SOUNDS),
            bullet_speed: f32_of(bullet, H_BULLET_SPEED, 120.0),
            range: f32_of(bullet, h("range"), 75.0),
            muzzles: match all.iter().find(|e| e.name == H_MUZZLE_REF).filter(|e| e.attr(H_MUZZLE_REF_NAME).is_some()) {
                Some(e) => hashes(e, H_MUZZLE_REF_NAME),
                None => vec![MUZZLE],
            },
            pickup_sound: find(h("pickup-sound")).and_then(|v| v.as_hash()).unwrap_or(0),
            icon: find(H_HUD_ICON).and_then(|v| v.as_hash()).unwrap_or(0),
            reload_time: f32_of(Some(&w), h("reload-time"), 0.0),
            reload_sound: hashes(w, H_RELOAD_SOUND).first().copied().unwrap_or(0),
            empty_sound: hashes(w, H_EMPTY_SOUND).first().copied().filter(|&x| x != h("")).unwrap_or(0),
            ammo_regen: f32_of(Some(&w), H_AMMO_REGEN, 0.0),
            fuse: f32_of(Some(&w), h("timer"), 0.0),
            projectile: w.attr(H_PROJECTILE).and_then(|v| v.as_hash()).unwrap_or(0),
            reticle: w.attr(h("reticule-prefix")).and_then(|v| v.as_hash()).unwrap_or(0),
            arm_sound: find(H_EVENT_SOUND).and_then(|v| v.as_hash()).unwrap_or(0),
            object_sound: find(h("sound-name")).and_then(|v| v.as_hash()).unwrap_or(0),
            impact_sound: bullet.and_then(|b| b.attr(H_IMPACT_SOUND)).and_then(|v| v.as_hash()).unwrap_or(0),
            blast_radius: f32_of(all.iter().find(|e| e.name == h("Damage")), h("radius"), 0.0),
            damage: f32_of(all.iter().find(|e| e.name == h("Damage")), h("max"), 0.0),
            damage_min: f32_of(all.iter().find(|e| e.name == h("Damage")), h("min"), 0.0),
            zoom: f32_of(Some(&w), 0xEFCE_7F77, 0.0),
            ammo_type: w.attr(h("ammo-type")).and_then(|v| v.as_i64()).unwrap_or(1),
            flight_effect: bullet.and_then(|b| b.attr(h("effect-name"))).and_then(|v| v.as_hash()).filter(|&x| x != h("")).unwrap_or(0),
            hit_effect: bullet.and_then(|b| b.attr(H_HIT_EFFECT)).and_then(|v| v.as_hash()).filter(|&x| x != h("")).unwrap_or(0),
            damage_type: all.iter().find(|e| e.name == h("Damage")).and_then(|e| e.attr(h("damage-type"))).and_then(|v| v.ints().first().copied()).unwrap_or(0),
            damage_time: f32_of(all.iter().find(|e| e.name == h("Damage")), H_DAMAGE_TIME, 0.0),
            blast_effect: bullet.and_then(|b| b.attr(H_BULLET_EFFECT)).and_then(|v| v.as_hash()).filter(|&x| x != h("")).unwrap_or(0),
            decal: bullet.and_then(|b| b.attr(H_BULLET_DECAL)).and_then(|v| v.as_hash()).filter(|&x| x != h("")).unwrap_or(0),
            function_type: find(h("function-type")).and_then(|v| v.ints().first().copied()).unwrap_or(0),
            group_type: find(h("group-type")).and_then(|v| v.ints().first().copied()).unwrap_or(0),
            item_index: find(0xE5F5_1266).and_then(|v| v.ints().first().copied()).unwrap_or(0),
            use_type: find(H_USE_TYPE).and_then(|v| v.ints().first().copied()).unwrap_or(0),
            stack_limit: find(h("stack-limit")).and_then(|v| v.ints().first().copied()).unwrap_or(0),
            attached_effect: all.iter().find(|e| e.name == H_ATTACHED_EFFECTS)
                .and_then(|a| a.children.first()).and_then(|e| e.attr(h("effect-name"))).and_then(|v| v.as_hash())
                .filter(|&x| x != h("")).unwrap_or(0),
            proximity_radius: find(H_PROXIMITY_RADIUS).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32))).unwrap_or(0.0),
            shots: shot_data(w, bullet),
            blast_core: all.iter().find(|e| e.name == h("Damage")).and_then(|e| e.attr(0x011C_B154)).and_then(|v| v.as_i64()).is_some_and(|v| v != 0),
            mesh_bullet: bullet.and_then(|b| mesh_bullet(b)),
            lock_box: f32_of(Some(&w), H_LOCK_BOX, 0.0),
            bullet_after: f32_of(bullet, 0xF0A5_6297, 0.0),
            mirv_mode: bullet.and_then(|b| b.children.iter().find(|c| c.name == h("mirv"))).and_then(|m| m.attr(0x11F5_15C5))
                .and_then(|v| v.as_i64().or(v.as_f32().map(|f| f as i64))).unwrap_or(0),
        });
    }
    out
}

/// `MeshBullet` of a `<bullet>`: bullet-type 2 with a projectile object (mesh-name), else None.
fn mesh_bullet(b: &Element) -> Option<MeshBullet> {
    if b.attr(h("bullet-type")).and_then(|v| v.as_i64().or(v.ints().first().copied())) != Some(2) {
        return None;
    }
    let object = b.attr(h("mesh-name")).and_then(|v| v.as_hash()).filter(|&x| x != 0 && x != h(""))?;
    let num = |e: &Element, a: u32| e.attr(a).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32))).unwrap_or(0.0);
    let flag = |a: u32| b.attr(a).and_then(|v| v.as_i64()).is_some_and(|v| v != 0);
    let mirv = b.children.iter().find(|c| c.name == h("mirv"));
    Some(MeshBullet {
        object,
        radius: num(b, h("radius")),
        homes: flag(0x07CD_F040),
        straight: num(b, 0xEA53_27E6),
        seeks: flag(0xF69B_7D4C),
        after: num(b, 0xF0A5_6297),
        explode_at_end: flag(0x0976_A9DC),
        steady: flag(0x0B03_ECE5),
        fuse: flag(0x0BAF_BD6F),
        mirv_count: mirv.map_or(0, |m| num(m, 0xFE04_300D) as i64),
        mirv_mode: mirv.map_or(0, |m| num(m, 0x11F5_15C5) as i64),
        mirv_object: mirv.and_then(|m| m.attr(0xE8EB_D6E0)).and_then(|v| v.as_hash()).filter(|&x| x != h("")).unwrap_or(0),
        spread: mirv.map_or([0.0; 2], |m| [num(m, h("spread-min")), num(m, h("spread-max"))]),
    })
}

/// HARDPOINT children of an archetype.
pub fn parse_hardpoints(arch: &Element) -> HashMap<u32, Hardpoint> {
    let mut out = HashMap::new();
    for hp in arch.walk().into_iter().filter(|e| e.name == h("HARDPOINT")) {
        let Some(name) = hp.attr(h("name")).and_then(|v| v.as_hash()) else { continue };
        let floats = |n: &str| hp.child(h(n)).and_then(|e| e.text.as_ref()).map(|t| t.floats()).unwrap_or_default();
        let (p, q) = (floats("point"), floats("ORIENTATION"));
        if p.len() < 3 || q.len() < 4 {
            continue;
        }
        out.entry(name).or_insert(Hardpoint {
            point: Vec3::new(p[0], p[1], p[2]),
            rot: Quat::from_xyzw(q[1], q[2], q[3], q[0]).normalize(),     // stored w x y z
        });
    }
    out
}

pub struct WeaponPart {
    /// part-name (0 for a single-part model); animation targets name parts
    pub name: u32,
    /// its collision surface (Game::collision), in the part's frame
    pub physics: Option<u32>,
    pub geosets: Vec<Geoset>,
    /// position and orientation in the weapon's (or compound object's) frame
    pub offset: Vec3,
    pub rotation: Quat,
    /// spinning parts (minigun barrel): axis in the part's frame
    pub spin_axis: Option<Vec3>,
    /// sliding parts (door leaves: joint type h_fd8f670c): the direction they open, in the
    /// compound's frame
    pub slide_axis: Option<Vec3>,
    /// its joint to the part it hangs from, as the archetype gives it (None for the root part and
    /// a model of one part)
    pub joint: Option<PartJoint>,
}

/// A part's joint: `<h_151ace78 parent-part child-part><h_002cbac5 Type>` with its
/// `parent-point`, `child-point`, `ORIENTATION` (w x y z), `axis` and limits (the joint record's
/// +0, +4, +0x10, +0x1c, +0x2c and its +0x3c pair: parsers FUN_0023e3e0 and FUN_0023e3a0).
/// FUN_00234f90 places the part from these by the joint's type and the joint's value, which a
/// clip's channel sets (FUN_00234dc0); level_scene.rs `Leaf` moves door leaves so.
#[derive(Clone, Copy, Debug)]
pub struct PartJoint {
    /// the joint type: SLIDING_JOINT, TRANSFORM_JOINT, HINGE_JOINT, the fixed h_0fb850ce ...
    pub kind: u32,
    /// the part it hangs from (h("root") for the compound's root part)
    pub parent: u32,
    pub parent_point: Vec3,
    pub child_point: Vec3,
    pub orientation: Quat,
    /// as written (a sliding or hinge joint's direction, in the parent part's frame)
    pub axis: Vec3,
    /// the range FUN_00234dc0 clamps a sliding or hinge joint's value to; the whole float range
    /// for a joint without `<h_fab3703e>`
    pub limits: (f32, f32),
}

/// Joint type of a sliding (prismatic) joint: door and gate leaves.
pub const SLIDING_JOINT: u32 = 0xFD8F_670C;
/// Joint type of a transform joint: a clip's transform channel (DT_XFORM) sets the part's
/// translation and rotation (FUN_00234f90's 0xf35d5b3c; FUN_002340f0 lets only that data type
/// play on it). The Shrike doors' segments, m05's and m08's doors, m01's hatches.
pub const TRANSFORM_JOINT: u32 = 0xF35D_5B3C;
/// Joint type of a hinge: a clip's angle channel (character.rs `DT_ANGLE`, radians) turns the
/// part about the joint's axis at its parent-point (FUN_00234f90's 0xe94eb66a). e10's door
/// h_1e50b5e0, the fans of e05 and e40.
pub const HINGE_JOINT: u32 = 0xE94E_B66A;

pub struct WeaponModel {
    pub parts: Vec<WeaponPart>,
    pub hardpoints: HashMap<u32, Hardpoint>,
}

impl WeaponModel {
    pub fn load(game: &Game, archetype: u32) -> Result<Self, String> {
        let (arch, _) = game.archetype(archetype).ok_or_else(|| format!("weapon archetype h_{archetype:08x} not found"))?;
        let mut hardpoints = parse_hardpoints(arch);
        let mut parts = vec![];
        let part_list = part_list(arch);
        if part_list.is_empty() {
            parts.push(WeaponPart { name: 0, physics: game.archetype_physics(archetype), geosets: meshes(game, arch)?, offset: Vec3::ZERO, rotation: Quat::IDENTITY, spin_axis: None,
                                    slide_axis: None, joint: None });
        } else {
            for (part, part_arch) in part_list {
                let Some((pa, _)) = game.archetype(part_arch) else { continue };
                for (k, v) in parse_hardpoints(pa) {
                    hardpoints.entry(k).or_insert(v);
                }
                // joint to the parent (compounds here are one level deep: root + children)
                let joint = part_joint(arch, part);
                let (offset, rotation, spin_axis, slide_axis) = match &joint {
                    // the part sits at parent-point + child-point, both in the parent's frame (a
                    // building set lays its pieces out by the child-point: a hangar's roof bays
                    // 5 m apart, its bunks in pairs), and the joint's ORIENTATION (w x y z; e.g. a
                    // wall set's mirrored gate leaves) turns the part about its own origin. Read
                    // as parent-point - ORIENTATION * child-point, Bulgar's compound h_0baf091a
                    // came out jumbled: roof bays outside the building, bunks through the walls
                    // (issue #12). Each of its 19 pieces lies inside the building only this way.
                    Some(j) => {
                        let (spin, slide) = if j.axis.length_squared() > 0.5 {
                            if j.kind == SLIDING_JOINT { (None, Some(j.orientation * j.axis.normalize())) } else { (Some(j.axis.normalize()), None) }
                        } else {
                            (None, None)
                        };
                        (j.parent_point + j.child_point, j.orientation, spin, slide)
                    }
                    None => (Vec3::ZERO, Quat::IDENTITY, None, None),
                };
                parts.push(WeaponPart { name: part, physics: game.archetype_physics(part_arch), geosets: meshes(game, pa)?, offset, rotation, spin_axis, slide_axis, joint });
            }
        }
        Ok(WeaponModel { parts, hardpoints })
    }

    /// A compound archetype's parts as `load` lists them, each with its joint, without reading
    /// their meshes: what moves when its clips play (level_scene.rs `animated_parts`). Empty for
    /// a model of one part.
    pub fn joints(game: &Game, archetype: u32) -> Vec<(u32, Option<PartJoint>)> {
        let Some((arch, _)) = game.archetype(archetype) else { return vec![] };
        part_list(arch).into_iter().filter(|&(_, pa)| game.archetype(pa).is_some()).map(|(part, _)| (part, part_joint(arch, part))).collect()
    }
}

/// A compound archetype's parts (`<h_e487418a><PART part-name archetype-name>`): (part name,
/// its archetype). Empty for a model of one part.
fn part_list(arch: &Element) -> Vec<(u32, u32)> {
    arch.child(H_PARTS).map(|p| p.children_named(h("PART"))
        .map(|e| (e.attr(h("part-name")).map(part_hash).unwrap_or(0), e.attr(h("archetype-name")).and_then(|v| v.as_hash()).unwrap_or(0)))
        .collect()).unwrap_or_default()
}

/// Part `part`'s joint in compound `arch` (`PartJoint`; the joint whose child-part it is), None
/// without one (the root part) or with an empty one.
fn part_joint(arch: &Element, part: u32) -> Option<PartJoint> {
    let joint = arch.walk().into_iter().filter(|e| e.name == H_JOINT).find(|j| j.attr(h("child-part")).map(part_hash) == Some(part))?;
    let d = joint.children.first()?;
    let v = |n: &str| d.child(h(n)).and_then(|e| e.text.as_ref()).map(|t| t.floats())
        .filter(|f| f.len() >= 3).map(|f| Vec3::new(f[0], f[1], f[2])).unwrap_or(Vec3::ZERO);
    let orientation = d.child(h("ORIENTATION")).and_then(|e| e.text.as_ref()).map(|t| t.floats()).filter(|q| q.len() >= 4)
        .map_or(Quat::IDENTITY, |q| Quat::from_xyzw(q[1], q[2], q[3], q[0]).normalize());
    let num = |l: &Element, k: &str| l.attr(h(k)).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32)));
    let limits = d.child(H_JOINT_LIMITS).map_or((f32::MIN, f32::MAX), |l| (num(l, "min").unwrap_or(f32::MIN), num(l, "max").unwrap_or(f32::MAX)));
    Some(PartJoint {
        kind: d.attr(h("Type")).and_then(|t| t.as_hash()).unwrap_or(0),
        parent: joint.attr(h("parent-part")).map(part_hash).unwrap_or(0),
        parent_point: v("parent-point"),
        child_point: v("child-point"),
        orientation,
        axis: v("axis"),
        limits,
    })
}

/// part-name values are hashes, except the literal "root"
fn part_hash(v: &Value) -> u32 {
    v.as_hash().or_else(|| v.as_str().map(h)).unwrap_or(0)
}

/// Geosets of an archetype's most detailed mesh (lowest lod-switch).
fn meshes(game: &Game, arch: &Element) -> Result<Vec<Geoset>, String> {
    let Some(list) = arch.child(H_MESHES) else { return Ok(vec![]) };
    let Some(best) = list.children_named(h("mesh"))
        .min_by_key(|m| m.attr(h("lod-switch")).and_then(|v| v.as_i64()).unwrap_or(0))
        .and_then(|m| m.attr(h("mesh-name")).and_then(|v| v.as_hash())) else { return Ok(vec![]) };
    mesh_geosets(game, best)
}

/// A `<geoset>`'s h_f89cd77d (its handler 0x23ef60, the record's +0x28): `Geoset::indoor`.
const H_GEOSET_INDOOR: u32 = 0xF89C_D77D;

/// Geosets of a mesh (objects `<mesh name=...>`), e.g. a level's terrain or sky.
pub fn mesh_geosets(game: &Game, best: u32) -> Result<Vec<Geoset>, String> {
    let (mesh, ivd) = game.mesh(best).ok_or_else(|| format!("mesh h_{best:08x} not found"))?;
    let mut out = vec![];
    for g in mesh.walk().into_iter().filter(|e| e.name == h("geoset")) {
        let material = g.attr(h("material-name")).and_then(|v| v.as_hash()).unwrap_or(0);
        // lit by the indoor pair under LU_GEOSET (`Geoset::indoor`)
        let indoor = g.attr(H_GEOSET_INDOOR).and_then(|v| v.as_i64()).is_some_and(|v| v != 0);
        // one or more draw records; records over the same vertex buffer (terrain patches) are
        // merged: each uses the buffer's first `n` vertices
        let mut merged: Option<(i64, Geoset)> = None;
        for rec in g.walk().into_iter().filter(|e| e.name == 0xFDE5_0E9F) {
            let vb = rec.attr(0x0CDF_FA25).and_then(|v| v.as_i64()).unwrap_or(-1);
            let mut part = read_geoset(rec, ivd, &[], material)?.0;
            part.indoor = indoor;
            match merged.as_mut() {
                Some((at, m)) if *at == vb => {
                    let new = std::mem::take(&mut part.indices);
                    if part.positions.len() > m.positions.len() {
                        part.indices = std::mem::take(&mut m.indices);
                        *m = part;
                    }
                    m.indices.extend(new);
                }
                _ => {
                    if let Some((_, m)) = merged.take() {
                        out.push(m);
                    }
                    merged = Some((vb, part));
                }
            }
        }
        out.extend(merged.map(|(_, m)| m));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `<weapon>` named `name` whose muzzle block is `muzzle` (None: no block).
    fn weapon(name: u32, muzzle: Option<Value>) -> Element {
        let base = Element { name: h("base"), attrs: vec![(h("name"), Value::Hash(name)), (h("object-type"), Value::Int(20))], children: vec![], text: None };
        let mut children = vec![base];
        if let Some(m) = muzzle {
            children.push(Element { name: H_MUZZLE_REF, attrs: vec![(H_MUZZLE_REF_NAME, m)], children: vec![], text: None });
        }
        Element { name: H_WEAPON, attrs: vec![], children, text: None }
    }

    /// Every name of the list is kept (#246: the turrets' second was dropped), "" is none, a
    /// weapon without the block has MUZZLE; the fire loop takes the muzzles in turn
    /// (FUN_0022f5e0) and a single one stays put.
    #[test]
    fn muzzles_in_turn() {
        let (a, b) = (0xF9EE_D613, 0xE2E7_87A9);
        let root = Element { name: 0, attrs: vec![], children: vec![
            weapon(1, Some(Value::List(vec![Value::Hash(a), Value::Hash(b)]))),
            weapon(2, Some(Value::Hash(MUZZLE))),
            weapon(3, Some(Value::Hash(0))),
            weapon(4, None),
        ], text: None };
        let defs = parse_weapons(&root, &HashMap::new());
        let of = |n: u32| defs.iter().find(|d| d.name == n).unwrap();
        assert_eq!(of(1).muzzles, vec![a, b]);
        assert_eq!(of(1).muzzle(), a);
        assert_eq!((of(1).next_muzzle(0), of(1).next_muzzle(1)), (1, 0));
        assert_eq!(of(2).muzzles, vec![MUZZLE]);
        assert_eq!(of(2).next_muzzle(0), 0);
        assert!(of(3).muzzles.is_empty());
        assert_eq!(of(3).muzzle(), 0);
        assert_eq!(of(4).muzzles, vec![MUZZLE]);
    }
}

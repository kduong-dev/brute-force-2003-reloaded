//! Character types from the game data (#128): every character the archives define, keyed the
//! way the game keys it, with the motion set it plays and the placements that put it in a level.
//! The groundwork for the enemies (#110): the species tickets build on it.
//!
//!  objecttypes-<lib>.xmb  `<character><h_e275fb80 motion-archetype=ms_x ..>`: one type each,
//!      keyed by the hash of its `<base name>` (`CharacterType`). The type factory (FUN_0015fde0,
//!      archetype-type 4) makes a character type (FUN_0012df60) and its loader 0x12ed40 copies
//!      the parse record in (FUN_00190330 the attributes, FUN_00190580 the elements). A name is
//!      looked up library by library and the first hit wins (FUN_00160da0): here, the archive
//!      loaded first. The data has 198 definitions of 128 names; the 33 names defined in several
//!      archives are the same definition in each (checked against decompiled/xml).
//!  characters/ms_<x>.xmb  the motion set a type's motion-archetype names (FUN_00131a10): its
//!      animation set, the MOTION table, the overlays and the hit bones (`MotionSet`). Keyed by
//!      the file's stem, not its own name: ms_lf and ms_dm_lf both call themselves "lf" and their
//!      tables differ. Each of the 22 files is the same in every archive that has it.
//!  levels-<lvl>.xmb  `<character-object object-type-name=TYPE name=..>`: a placed character
//!      (`Placement`), whose own team, AI and inventory go over its type's (the character's
//!      property handler 0x112eb0). The level loads it dormant; a spawn trigger clones it (#102).
//!
//! What a character's clips are: a MOTION's script (ready-script with the gun up) by the rule of
//! FUN_00130c50 (`Motion::clip`), out of the animation set its motion set names; its face clips
//! are the set its `<face viseme-file>` names; its flesh is the world material of its parts'
//! collision (`Game::flesh`).

use std::collections::HashMap;

use bevy::math::{Mat4, Vec3};

use super::archive::Archive;
use super::bxml::{self, Element, Value};
use super::character::{skill_data, Character, Game, SkillData};
use super::collision::MATERIAL_ID;
use super::hash::h;
use super::level;
use super::weapon;

/// The objecttypes entries' element (`h_e275fb80`: every list's archetype record) and a
/// character's game-archetype part's display name (`<base h_1dd3a49d>`, a string id: h_19bcc17f
/// "Shrike Soldier").
const H_ENTRY: u32 = 0xE275_FB80;
const H_LABEL: u32 = 0x1DD3_A49D;
/// The type's flag h_e78efbb2 (name not recovered; see `CharacterType::player_type`).
const H_PLAYER_TYPE: u32 = 0xE78E_FBB2;
/// The footstep type (h_072ab4af, an enum: the squad 1-4, the campaign Shrikes and hounds 0).
const H_FOOTSTEP: u32 = 0x072A_B4AF;
/// The two blood decal lists: h_fb4bfc68 the hit's, h_efc2a874 the second (FUN_00190930's two
/// cursors, the type's vectors +0x1b4 / +0x1c4).
const H_DECALS: [u32; 2] = [0xFB4B_FC68, 0xEFC2_A874];
/// The attached effects (`<base><h_18b6ab72><h_081398d6 effect-name hardpoint-name>` x 8; names
/// not recovered).
const H_ATTACHED: u32 = 0x18B6_AB72;
const H_ATTACHED_EFFECT: u32 = 0x0813_98D6;
/// A shield's `<recharge>` values (FUN_00189a90): `delay` (h_f0a56297, name recovered by
/// hashing), h_1cf65a16 and h_fe14a5b7 (unrecovered); the m09_c Hunter Lord's 5, 30, 100.
const H_SHIELD_RECHARGE: [u32; 3] = [0xF0A5_6297, 0x1CF6_5A16, 0xFE14_A5B7];
/// The motion set's two ready blocks, `<npc-transition-ready in-time out-time>` (h_170db5cd) and
/// `<pc-transition-ready ..>` (h_f599b114), the `<motion-sequence>` dispatcher 0x195be0's (its
/// handlers 0x194880 / 0x1948c0); `in-time` is h_141433f1. Names recovered by hashing (#140).
const H_NPC_READY: u32 = 0x170D_B5CD;
const H_PC_READY: u32 = 0xF599_B114;
const H_IN_TIME: u32 = 0x1414_33F1;
/// A placement's attributes: h_17a554bf (a character a player can take control of), h_ec5337fb
/// (whether h_072837bb, the accuracy bonus, applies), and its drops `<h_0f77963d inventory-drop>`
/// and ammo and grenades `<inventory><h_10dd6db2 name count>`.
const H_CONTROLLABLE: u32 = 0x17A5_54BF;
const H_ACCURACY_ON: u32 = 0xEC53_37FB;
const H_ACCURACY: u32 = 0x0728_37BB;
/// A placement `<ai>`'s h_11cddd9f (its own `vision` counts, `PlacementAi::vision_own`) and
/// h_1b2327a0, field-of-view (`PlacementAi::field_of_view`); a type `<ai>`'s h_12a04758, its AI's
/// accuracy (`TypeAi::accuracy`).
const H_VISION_OWN: u32 = 0x11CD_DD9F;
const H_FIELD_OF_VIEW: u32 = 0x1B23_27A0;
const H_AI_ACCURACY: u32 = 0x12A0_4758;
const H_DROPS: u32 = 0x0F77_963D;
const H_AMMO_ITEM: u32 = 0x10DD_6DB2;
/// A type's death award `<base><h_1edc28b5 ..>` and its attributes in the parse record's order
/// (FUN_0018cfb0: +0 amount, +4 h_ffc62853, +5 h_06961311, +6 h_f61ecbaf, +7 h_199ff965, +8
/// h_e925f48b; see `Award`).
const H_AWARD: u32 = 0x1EDC_28B5;
const H_AWARD_PER_DAMAGE: u32 = 0xFFC6_2853;
const H_AWARD_ON_HIT: u32 = 0x0696_1311;
const H_AWARD_TO_SQUAD: u32 = 0xF61E_CBAF;
const H_AWARD_TO_OWN: u32 = 0x199F_F965;
const H_AWARD_KIND: u32 = 0xE925_F48B;

/// The MT_ motion types by value, without their MT_ prefix (default.xbe's name table 0x3bfdac,
/// values 0-49 in order; parser FUN_001876a0).
pub const MOTION_TYPES: [&str; 50] = [
    "NONE", "ATTACK0", "ATTACK1", "BACKPEDAL", "BACKSTEP", "COWER", "CROUCH", "CROUCH_WALK", "DASH", "DIE0", "DIE1", "DIE2",
    "DIE3", "DIVE", "DIZZY", "DODGE_LEFT", "DODGE_RIGHT", "DODGE_SHORT_LEFT", "DODGE_SHORT_RIGHT", "IDLE", "JUMP_CROUCH",
    "JUMP_FALL", "JUMP_LAND", "JUMP_LAND_NONINTERRUPTIBLE", "JUMP_LAUNCH", "KNOCKDOWN", "MOVE_LEFT", "MOVE_LEFT_BACK",
    "MOVE_LEFT_BACK_SLOW", "MOVE_LEFT_SLOW", "MOVE_RIGHT", "MOVE_RIGHT_BACK", "MOVE_RIGHT_BACK_SLOW", "MOVE_RIGHT_SLOW", "RUN",
    "SLIDE", "SLIDE_END", "SPECIAL_USE_A_START", "SPECIAL_USE_B_START", "SPECIAL_USE_C_START", "SPECIAL_USE_END",
    "SPECIAL_USE_IDLE", "STANDING_GRENADE_THROW", "STOP", "SURPRISE", "TOGGLE_SPECIAL", "TRANSITION", "WALK", "WORK", "LUNGE",
];
/// MT_IDLE (19): the idles, whose clips' root-heights the dump lists.
pub const MT_IDLE: i64 = 19;
/// The ALIGN_ teams by value + 1 (default.xbe's name table 0x3bc94c: ALIGN_NONE is -1), without
/// their ALIGN_ prefix. Hostility between them is the table at 0x3ffd80 (code map).
pub const TEAMS: [&str; 11] = ["NONE", "CONFEDERATION", "SHRIKE", "NEUTRAL", "TEAM1", "TEAM2", "TEAM3", "TEAM4", "PC_NO_TEAM", "HOSTILE", "SEER_SLAVE"];
/// ALIGN_NONE: a placement that leaves the team to its type (0x10fa60), or a type with none.
pub const ALIGN_NONE: i64 = -1;
/// The stances a MOTION's clips are named with (default.xbe 0x40cb30: "", "w1_", "w2_"): its
/// builder (FUN_00130c50) makes the w1_ and w2_ ones, and falls back to "".
pub const STANCES: [&str; 3] = ["", "w1_", "w2_"];
/// The suffixes a clip name can end in (0x40cafc), and the one each falls back to when the anim
/// set has no such clip (0x38db80: `_left_cr` and `_right_cr` to `_cr`, the rest to none).
pub const SUFFIXES: [&str; 6] = ["", "_cr", "_left", "_right", "_left_cr", "_right_cr"];
const SUFFIX_FALLBACK: [usize; 6] = [0, 0, 0, 0, 1, 1];
/// The root-height the game gives a type whose default motion's clip has none (FUN_00131a10 at
/// 0x131b2f: type+0x1b0).
pub const DEFAULT_ROOT_HEIGHT: f32 = 0.75;

/// A character type: one objecttypes `<character>` entry (see the module notes). The fields are
/// the parse record's that the loader 0x12ed40 copies into the runtime type; the offsets are the
/// runtime type's (`type+X`, see viewer/docs/code-map.md).
#[derive(Clone, Debug, Default)]
pub struct CharacterType {
    /// the hash of its `<base name>` (the placements' object-type-name): readable names for the
    /// campaign squad (brutus, flint, hawk, tex) and the deathmatch one (mp_brutus ...), hashes
    /// for the rest
    pub key: u32,
    /// the archive it was read from (the first loaded that defines it), and its objecttypes
    /// library there (`objecttypes-<library>.xmb`: the archive's own name in the campaign's;
    /// mp_common has six: dm, sdm_bf, sdm_feral, sdm_ferg, sdm_seers, sdm_shrike)
    pub archive: String,
    pub library: String,
    /// its display name: a string id (`<base h_1dd3a49d>`, into `Game::strings`), 0 none
    pub label: u32,
    /// its model: the compound archetype `<base mesh-name>` (type+0xc4, FUN_00154a20)
    pub mesh: u32,
    /// motion-archetype as written ("ms_lf"), and its hash: the motion set's key in
    /// `Game::motion_sets` (the file characters/<motion-archetype>, FUN_00131a10; type+0x1a4)
    pub motion_archetype: String,
    pub motion_set: u32,
    /// `<face viseme-file>`: the animation set of its face clips (brutusface on the ferals,
    /// seerface, militiaface ...), 0 none (Shrikes, hounds)
    pub face_set: u32,
    /// chatter-library (h_f18a456f): its voice lines' file (characters/<x>.xmb), "" none
    /// (FUN_001304b0)
    pub chatter: String,
    /// combat-target hitpoints (an int in the game: type+0x1c, FUN_001546d0; an instance's max
    /// and HP, FUN_00224140)
    pub hitpoints: f32,
    /// combat-target target-type (the TTYPE_ enum, 0x3bf294: 0 HUMAN, 1 FERAL, 2 FLINT, 3 SEER,
    /// 4 FIREHOUND, 5 HOUND, 6 SCIENTIST; type+0x20). What reads it isn't traced
    pub target_type: i64,
    /// its vulnerabilities (h_142be76f, `<vulnerability Type modifier>`): (damage-type, factor);
    /// x1 for every type not listed (FUN_00154740 fills type+0x78 + 4 x Type)
    pub vulnerabilities: Vec<(i64, f32)>,
    /// the combat-target's `<shield>`, when its shield-type isn't 0 (FUN_001546d0 makes a shield
    /// object then)
    pub shield: Option<Shield>,
    /// damage-multiplier (h_e7e9fcd3, type+0x220): a direct hit's factor with the struck bone's
    /// (FUN_00125940); 0.62 the campaign squad, 1.8 the deathmatch Shrike Hounds (every
    /// definition gives it; 1 without)
    pub damage_multiplier: f32,
    /// max-stamina (type+0x224): stamina and its max at spawn (FUN_0012f440)
    pub max_stamina: f32,
    /// height-offset (type+0x2ac, FUN_0012cd10): -2.3 on the Shadoon head, 0 on the rest
    pub height_offset: f32,
    /// turn-rate (h_1182c05d) in degrees a second (the loader keeps it in radians, type+0x2d0)
    pub turn_rate: f32,
    /// h_e78efbb2 (name not recovered; type+0x1a8 bit 0): true on the squad, every multiplayer
    /// type and the controllable allies; the stamina update runs for it (FUN_0011a840) and the
    /// spawn registers it with the session (FUN_0012f440)
    pub player_type: bool,
    /// enable-ragdoll-death (h_f397278e; type+0x34c bit 0): the death code queues the ragdoll
    /// only when it's set (FUN_00115d50 at 0x115e1f). False on the Seer Priests and the 1189-HP
    /// Shadoon, the types with death clips. (Every definition gives it; read as true without,
    /// a guess)
    pub ragdoll_death: bool,
    /// its `<ai>` (FUN_00188470; type+0x358, FUN_0001ff70)
    pub ai: TypeAi,
    /// `<inventory weapon-name>`: its weapons (type+0x2d4 / +0x2d8), empty names left out
    pub weapons: Vec<u32>,
    /// `<inventory><item count name>` (type+0x2dc + 8k): (item, count), empty names left out
    pub items: Vec<(u32, i64)>,
    /// `<limit-weapon one two>`: the weapon class each slot holds (the first number; -1 none)
    pub weapon_classes: [i64; 2],
    /// `<speeds speed>`: crouch walk, walk, run, dash (m/s; type+0x6f8..0x704, read by
    /// FUN_0012c6c0); the demo moves by root motion
    pub speeds: [f32; 4],
    /// `<cylinder-extents>` (FUN_00190290; type+0x1d4..0x1e0)
    pub capsule: Capsule,
    /// the blood decals, up to three each: the hit list (h_fb4bfc68) and the second
    /// (h_efc2a874); a decal of a list is picked at random (FUN_00124af0, FUN_00124f60). Names of
    /// `Game::decals`
    pub decals: [Vec<u32>; 2],
    /// `<death-effect effect-name hardpoint-name>` (type+0x350 / +0x354; FUN_00115d50 at
    /// 0x115e66): the priests' h_0046526a, the Supermutant's h_e19a8348; 0 none
    pub death_effect: (u32, u32),
    /// effects attached to the body (`<base><h_18b6ab72>`: effect, hardpoint): the hounds'
    /// gashound_bdy; empty entries left out. What starts them isn't traced
    pub attached_effects: Vec<(u32, u32)>,
    /// footstep type (h_072ab4af) and `<movement-sounds jump-sound>` (0 none)
    pub footstep_type: i64,
    pub jump_sound: u32,
    /// its `<combat-skill skill-set stamina-cost stamina-recovery>` with max-stamina and
    /// h_e78efbb2 (`SkillData`; the squad's are read by name too): the stamina update
    /// (FUN_0011a840) refills `stamina-recovery` a second for a type with h_e78efbb2, the
    /// deathmatch Shrikes' 5 / 4 (#129: their stamina shield's)
    pub skill: Option<SkillData>,
    /// the cash its death pays or costs (see `Award`, #221)
    pub award: Award,
}

/// A type's cash for its death (`<base><h_1edc28b5 amount h_ffc62853 h_06961311 h_f61ecbaf
/// h_199ff965 h_e925f48b>`: FUN_0018cfb0 parses it into the parse record's +0x11c..+0x124,
/// FUN_00154a20 copies it to type+0x124..+0x12c). FUN_00157070 pays it on the character's
/// CBT_DEAD (and on CBT_DAMAGED with `on_hit`), in a campaign: `amount` x (1 + 0.1 x the
/// difficulty), keyed in the cash record by the type's display name (type+0x120: its
/// h_1dd3a49d, "NO NICE NAME" h_1ac9db24 without one). The tutorial's: the Red Hand 100 to the
/// squad ($60 on STANDARD: take12's end screen "24 x Red Hand +$1440"), the four squad types
/// -5000 to their own squad ("1 x Tex -$3000"), the three Confed marine types -1000; every other
/// type 0.
#[derive(Clone, Copy, Debug, Default)]
pub struct Award {
    /// amount (type+0x124)
    pub amount: f32,
    /// h_ffc62853 (type+0x128): x the hit's damage, or at a death the character's +0x50 (no type
    /// in the data sets it)
    pub per_damage: bool,
    /// h_06961311 (type+0x129): paid on every hit, not only the death (none sets it)
    pub on_hit: bool,
    /// h_f61ecbaf (type+0x12a): paid to the campaign's squad, whoever killed it (the Red Hand's)
    pub to_squad: bool,
    /// h_199ff965 (type+0x12b): paid to its own squad when it's on the squad's side (the squad's
    /// and the marines' death penalties)
    pub to_own: bool,
    /// h_e925f48b (type+0x12c, an enum: 2 on the characters, 1 on the rest): not read by
    /// FUN_00157070
    pub kind: i64,
}

/// A character type's AI settings (`<ai>`, the parse record's +0x21c; FUN_0001ff70 copies them
/// to type+0x358 in another layout).
#[derive(Clone, Debug, Default)]
pub struct TypeAi {
    /// alignment-type: its team (the ALIGN_ enum, `TEAMS`; type+0x3b0), the team of a placement
    /// that gives none (0x10fa60). Every enemy species is 1 (SHRIKE)
    pub alignment: i64,
    /// personality-freewill, -follow, -formation, -guard, -wingman (`common/ai/personality`
    /// names), 0 none
    pub personalities: [u32; 5],
    /// vision-range (m) and half-fov (degrees)
    pub vision_range: f32,
    pub half_fov: f32,
    /// squad-leash-dist (h_1d50ffec): how far a squadmate strays from its leader (m)
    pub squad_leash: f32,
    /// h_12a04758 (the type parser FUN_00188470 puts it at the record's +0x98, FUN_0001ff70 at
    /// the AI settings' +0x7c): its AI's accuracy in percent, read by FUN_00024ae0 when the
    /// placement gives none (h_ec5337fb false). Every type has one: 50 on 145 of the 198, the Red
    /// Hand's 50, the Confed marines' 60
    pub accuracy: f32,
}

/// A combat-target's `<shield>` (FUN_00189c40 and FUN_00189a90 parse it, FUN_00154780 copies it
/// into the type): the stamina shield of the deathmatch Shrikes and Hunter Lord, the energy
/// shield of six bosses. Taken through the shield object's absorb (0x223690 stamina, 0x223400
/// energy) before the vulnerabilities (FUN_002232d0).
#[derive(Clone, Debug, Default)]
pub struct Shield {
    /// shield-type: the EST_ enum (0x3bf99c): 1 ENERGY_SHIELD, 2 STAMINA_SHIELD
    pub kind: i64,
    /// max-charge (the energy shield's pool: 500 on the Hunter Lord)
    pub max_charge: f32,
    /// active-effect and hit-effect (effect types), 0 none
    pub active_effect: u32,
    pub hit_effect: u32,
    /// `<recharge enabled>`'s three values (`delay`, h_1cf65a16, h_fe14a5b7: the record's +0x18,
    /// +0x1c, +0x20; what reads them isn't traced), None when not enabled
    pub recharge: Option<[f32; 3]>,
}

/// `<cylinder-extents radius height crouch-height><offset>` (h_0f7a74c1, name recovered by
/// hashing): the body's capsule (type+0x1d4 radius, +0x1d8 height, +0x1dc crouch-height, +0x1e0
/// offset; the stand-up clearance FUN_0012b440, the physics step FUN_0012cd10). The demo still
/// uses one cylinder for everyone (play.rs BODY_RADIUS, BODY_HEIGHT).
#[derive(Clone, Copy, Debug, Default)]
pub struct Capsule {
    pub radius: f32,
    pub height: f32,
    pub crouch_height: f32,
    pub offset: Vec3,
}

/// A motion set (characters/ms_<x>.xmb, see the module notes): what FUN_00131a10 reads into the
/// type.
#[derive(Clone, Debug, Default)]
pub struct MotionSet {
    /// its file's stem ("ms_lf"); `Game::motion_sets` is keyed by its hash
    pub file: String,
    /// `<motion-sequence name>`: the set's own name ("lf" for ms_lf and ms_dm_lf alike)
    pub name: u32,
    /// animation-set-name (h_fd06d1b2): the animation set its clips come from (type+0xcc). Seven
    /// play another's: ms_feral_colonist brutus, ms_mutant2 mutant, ms_mctavish militia, the
    /// four ms_dm_* their campaign set
    pub anim_set: u32,
    /// `<npc-transition-ready>` and `<pc-transition-ready>`, each (in-time, out-time) in seconds
    /// (FUN_00131a10 puts them at type+0x238 / +0x23c and +0x230 / +0x234): the gun's raise before
    /// a first shot (FUN_0010fce0: the ready timer starts at 4 + the in-time, and the fire command
    /// shoots once it's at 4) and the ready pose's blend in and out (FUN_0011a440), the pc pair
    /// for a character a human player controls, the npc pair for the AI. The squad's are pc 0.1 /
    /// 0.2 and npc 0.2 / 0.2 (Brutus's npc in-time 0.3); e40's Shrikes pc 0.01 / 0.2, npc 0.4 / 0.3
    pub npc_ready: (f32, f32),
    pub pc_ready: (f32, f32),
    /// `<sequence default-motion general-idle cautious-idle>`: MOTION names
    pub default_motion: u32,
    pub general_idle: u32,
    pub cautious_idle: u32,
    /// the MOTION table, in the file's order
    pub motions: Vec<Motion>,
    /// its `<overlay>`s (upper-body clips: reload, use_item, the skill toggles OMT_SPECIAL_ON 44
    /// and OFF 43 ...), in the file's order
    pub overlays: Vec<Overlay>,
    /// `<collision><bones><bone ..>` (FUN_00194d70; FUN_00130aa0 copies them to type+0x188)
    pub hit_bones: Vec<HitBone>,
}

/// A MOTION of a motion set (attributes: FUN_001953a0; its record FUN_00195780).
#[derive(Clone, Debug, Default)]
pub struct Motion {
    /// its name (a hash; "slide" is the one written out)
    pub name: u32,
    /// motion-type: the MT_ enum (`MOTION_TYPES`)
    pub motion_type: i64,
    /// script and ready-script (with the gun up): their two slots each, in order, "" for an
    /// empty one (the parser fills two 0x1c-byte strings of each, FUN_00195710: `stunned
    /// stunned2` both, most MOTIONs the first; `ready-script=" "` is two empty ones)
    pub scripts: Vec<String>,
    pub ready_scripts: Vec<String>,
    /// prefix-type (FUN_001879e0): MS_PREFIX_NONE 0 (the script is the clip's whole name),
    /// MS_PREFIX_WEAPON 1 and MS_PREFIX_WEAPON_NUMBER 2 ("Sc_" + stance + script; table 0x40cae8)
    pub prefix: i64,
    /// next-motion (a MOTION name, 0 none), transition (s) and loop
    pub next: u32,
    pub transition: f32,
    pub looping: bool,
}

/// An `<overlay>` of a motion set: an upper-body clip (FUN_001314c0 builds its clip names).
#[derive(Clone, Debug, Default)]
pub struct Overlay {
    pub name: u32,
    /// motion-type: the OMT_ enum (1 alert, 26 / 27 the looks, 38 reload, 43 / 44 the skill's
    /// off / on, 47 use_item ...)
    pub motion_type: i64,
    pub script: String,
    /// prefix-type: 0 the script is the whole name, else "Sc_" + stance + script
    pub prefix: i64,
}

/// A hit bone: a motion set's `<collision><bones><bone name mass Damage recoil>` (FUN_00194d70;
/// the type's 0x14-byte records +0x188: bone, mass, Damage, the part, recoil).
#[derive(Clone, Copy, Debug, Default)]
pub struct HitBone {
    /// the bone's name hash ("Bip01 Head" ...)
    pub bone: u32,
    /// mass (h_1a19ff7a, name recovered by hashing): 1 on every bone of the data's
    pub mass: f32,
    /// Damage: the factor a direct hit on it takes (FUN_00125940): the head x5 (x3 hounds,
    /// Brutus, ferals), upper arms x0.8, forearms and calves x0.5
    pub damage: f32,
    /// recoil: true on the head and arms (meaning not traced)
    pub recoil: bool,
}

/// A placed character (levels-<lvl>.xmb `<character-object>`): its type with this instance's
/// overrides. The game makes the character from the type (its vtable +8, 0x12f3e0), then feeds it
/// the placement's attributes and children (its property handler 0x112eb0).
#[derive(Clone, Debug, Default)]
pub struct Placement {
    /// the archive whose level file places it
    pub archive: String,
    /// object-type-name (h_ff8051ee, name recovered by hashing): its type, a key of
    /// `Game::character_types`
    pub kind: u32,
    /// its own name: what a spawn trigger names as its template (h_1299668b, 0x209250)
    pub name: u32,
    pub transform: Mat4,
    /// its `<ai>` (FUN_00188100; the whole record goes to char+0x1b8, FUN_00110cd0)
    pub ai: PlacementAi,
    /// `<inventory><weapon name count>`: two slots (the second empty for every enemy), empty
    /// names left out; and `<h_10dd6db2 name count>`: ammo and grenades
    pub weapons: Vec<(u32, i64)>,
    pub items: Vec<(u32, i64)>,
    /// `<h_0f77963d inventory-drop>`: what it leaves when it dies (FUN_00156390 at 0x156666)
    pub drops: Vec<u32>,
    /// h_17a554bf: a character a player can take control of (FUN_00156390: object +0x70 bit 4)
    pub controllable: bool,
    /// h_072837bb, the accuracy bonus (-100..100, char+0x49c), when h_ec5337fb is set
    pub accuracy_bonus: Option<i64>,
}

/// A placement's `<ai>` (FUN_00188100's record, copied to char+0x1b8 by FUN_00110cd0).
#[derive(Clone, Debug, Default)]
pub struct PlacementAi {
    /// alignment-type (char+0x1d4): its team, ALIGN_NONE (-1) to leave it to the type (0x10fa60)
    pub alignment: i64,
    /// personality-freewill, -follow, -formation, -guard, -wingman (`TypeAi`'s order), 0 none
    pub personalities: [u32; 5],
    /// vision (m, char+0x1d8) and goal-id (char+0x1e4)
    pub vision: f32,
    pub goal_id: i64,
    /// h_11cddd9f (char+0x1b9; name not recovered): its own `vision` is its sight distance, not its
    /// type's vision-range (the sensor's getter 0x10fb90). True on 88 of the data's placements:
    /// the tutorial's sniper-lesson target (vision 10) and one start marine (80)
    pub vision_own: bool,
    /// h_1b2327a0, field-of-view (name recovered by hashing; char+0x1e0): how far off its look
    /// direction (degrees, either way) it sees (the sensor's fov test 0x3d300): 60 on the Red
    /// Hand, 90 on the Confed marines and the squad
    pub field_of_view: f32,
}

impl CharacterType {
    /// Where it's defined, for lists: its archive ("e34"), with the library when that has
    /// another name ("mp_common/sdm_shrike").
    pub fn place(&self) -> String {
        if self.library.eq_ignore_ascii_case(&self.archive) { self.archive.clone() } else { format!("{}/{}", self.archive, self.library) }
    }

    /// The factor it takes damage of `damage_type` by (its vulnerabilities; 1 for a type they
    /// don't list: FUN_00154740 starts all 12 at 1.0). What `Game::damage_factor` gives the
    /// squad by name.
    pub fn vulnerability(&self, damage_type: i64) -> f32 {
        self.vulnerabilities.iter().find(|v| v.0 == damage_type).map_or(1.0, |v| v.1)
    }
}

impl Placement {
    /// Its team (0x10fa60): its own alignment-type, or its type's when that's ALIGN_NONE.
    pub fn team(&self, t: Option<&CharacterType>) -> i64 {
        if self.ai.alignment != ALIGN_NONE { self.ai.alignment } else { t.map_or(ALIGN_NONE, |t| t.ai.alignment) }
    }
}

impl MotionSet {
    /// Its MOTIONs of motion type `mt`, in the file's order.
    pub fn motions_of(&self, mt: i64) -> impl Iterator<Item = &Motion> {
        self.motions.iter().filter(move |m| m.motion_type == mt)
    }

    /// The MOTION named `name`.
    pub fn motion(&self, name: u32) -> Option<&Motion> {
        self.motions.iter().find(|m| m.name == name)
    }
}

impl Motion {
    /// A clip name for `script`: "Sc_" + stance + script + suffix when the MOTION is prefixed,
    /// else script + suffix (FUN_00130ba0's formats "Sc_%s%s%s" at 0x40caf0 and "%s%s" at
    /// 0x40dd4c).
    pub fn clip_name(&self, script: &str, stance: usize, suffix: usize) -> String {
        if self.prefix != 0 {
            format!("Sc_{}{script}{}", STANCES[stance.min(2)], SUFFIXES[suffix.min(5)])
        } else {
            format!("{script}{}", SUFFIXES[suffix.min(5)])
        }
    }

    /// The clip of one of this MOTION's slots, by FUN_00130c50's rule: `variant` is the slot (0
    /// or 1: `stunned` / `stunned2`), `ready` its ready-script's (the gun up), in `stance` (1 w1_,
    /// 2 w2_) with `suffix`.
    ///  - A slot whose script and ready-script are both empty is empty (0x130f1a-0x130f26).
    ///  - The name is made of the ready-script for a ready slot, of the script for a script's,
    ///    and of the script too when the ready-script is empty (FUN_00130ba0: `test dl,dl` at
    ///    0x130bb3 and `cmp byte ptr [eax],0` at 0x130bb7 send both to the script in EDI): so
    ///    the 90 MOTIONs with an empty ready-script (stand2crouch, crouch2stand and
    ///    throw_grenade in 15 sets, the hounds' gas attacks, ...: character-types.md) play their
    ///    script with the gun up too.
    ///  - When the animation set has no clip of that name (`has` tells, by the name's hash), the
    ///    suffix's fallbacks (0x38db80) in turn, then the name with no stance and no suffix (the
    ///    third call zeroes both: `xor esi,esi` at 0x131058, `xor ecx,ecx` at 0x131061).
    ///
    /// The name found, or None.
    pub fn clip(&self, variant: usize, ready: bool, stance: usize, suffix: usize, has: impl Fn(u32) -> bool) -> Option<String> {
        let script = self.scripts.get(variant).map_or("", String::as_str);
        let ready_script = self.ready_scripts.get(variant).map_or("", String::as_str);
        if script.is_empty() && ready_script.is_empty() {
            return None;
        }
        let script = if ready && !ready_script.is_empty() { ready_script } else { script };
        let mut k = suffix.min(5);
        loop {
            let n = self.clip_name(script, stance, k);
            if has(h(&n)) {
                return Some(n);
            }
            if k == 0 {
                break;
            }
            k = SUFFIX_FALLBACK[k];
        }
        Some(self.clip_name(script, 0, 0)).filter(|n| has(h(n)))
    }
}

/// A float attribute (an int one read as a float).
fn num(e: &Element, k: u32) -> Option<f32> {
    e.attr(k).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32)))
}

/// An int attribute.
fn int(e: &Element, k: u32) -> Option<i64> {
    e.attr(k).and_then(|v| v.as_i64())
}

/// A name attribute (a string id or a string), 0 when it's missing or empty.
fn name(e: &Element, k: u32) -> u32 {
    e.attr(k).and_then(|v| v.as_hash()).filter(|&x| x != 0 && x != h("")).unwrap_or(0)
}

/// A string attribute as written, trimmed ("" when it's missing or not a string).
fn text(e: &Element, k: u32) -> String {
    e.attr(k).and_then(|v| v.as_str()).map(|s| s.trim().to_string()).unwrap_or_default()
}

/// Every value of attribute `k` on `e`: a list attribute is stored as the attribute repeated, one
/// value per entry (the parsers advance a cursor per entry: FUN_00190250 the speeds,
/// FUN_00195710 a MOTION's scripts).
fn all(e: &Element, k: u32) -> impl Iterator<Item = &Value> {
    e.attrs.iter().filter(move |(n, _)| *n == k).map(|(_, v)| v)
}

/// A list of strings by slot (a MOTION's script and ready-script): one entry per value of
/// attribute `k`, in order, an empty one kept as "" so the slots stay in their places (the
/// decoder's XML writes them space-joined: `script="idle_stop "` is "idle_stop" and "").
fn strings(e: &Element, k: u32) -> Vec<String> {
    all(e, k).flat_map(|v| match v {
        Value::List(l) => l.iter().map(|x| x.as_str().unwrap_or("").trim().to_string()).collect(),
        v => vec![v.as_str().unwrap_or("").trim().to_string()],
    }).collect()
}

/// A `<x name count>` list's entries (a type's items, a placement's weapons and ammo), empty
/// names left out.
fn counted<'a>(list: impl Iterator<Item = &'a Element>) -> Vec<(u32, i64)> {
    list.filter_map(|e| {
        let n = name(e, h("name"));
        (n != 0).then(|| (n, int(e, h("count")).unwrap_or(0)))
    }).collect()
}

/// A character type from its objecttypes entry `e` (`<h_e275fb80 motion-archetype ..>`), read
/// from `archive`'s library `library`; None when it has no `<base mesh-name><base name>`.
pub fn parse_type(e: &Element, archive: &str, library: &str) -> Option<CharacterType> {
    let part = e.child(h("base"))?;
    let key = name(part.child(h("base"))?, h("name"));
    if key == 0 {
        return None;
    }
    let combat = part.child(h("combat-target"));
    let vulnerabilities = combat.and_then(|c| c.child(h("vulnerabilities"))).map(|l| l.children.iter()
        .filter_map(|v| Some((int(v, h("Type"))?, num(v, h("modifier"))?))).collect()).unwrap_or_default();
    let shield = combat.and_then(|c| c.child(h("shield"))).filter(|s| int(s, h("shield-type")).unwrap_or(0) != 0).map(|s| {
        let r = s.child(h("recharge")).filter(|r| int(r, h("enabled")).unwrap_or(0) != 0);
        Shield {
            kind: int(s, h("shield-type")).unwrap_or(0),
            max_charge: num(s, h("max-charge")).unwrap_or(0.0),
            active_effect: name(s, h("active-effect")),
            hit_effect: name(s, h("hit-effect")),
            recharge: r.map(|r| H_SHIELD_RECHARGE.map(|k| num(r, k).unwrap_or(0.0))),
        }
    });
    let ai = e.child(h("ai")).map(|a| TypeAi {
        alignment: int(a, h("alignment-type")).unwrap_or(ALIGN_NONE),
        personalities: ["personality-freewill", "personality-follow", "personality-formation", "personality-guard", "personality-wingman"]
            .map(|k| name(a, h(k))),
        vision_range: num(a, h("vision-range")).unwrap_or(0.0),
        half_fov: num(a, h("half-fov")).unwrap_or(0.0),
        squad_leash: num(a, h("squad-leash-dist")).unwrap_or(0.0),
        accuracy: num(a, H_AI_ACCURACY).unwrap_or(0.0),
    }).unwrap_or(TypeAi { alignment: ALIGN_NONE, ..Default::default() });
    let inventory = e.child(h("inventory"));
    let limit = e.child(h("limit-weapon"));
    let class = |k: &str| limit.and_then(|l| l.attr(h(k))).and_then(|v| v.ints().first().copied()).unwrap_or(-1);
    let speeds: Vec<f32> = e.child(h("speeds")).map(|s| all(s, h("speed")).flat_map(|v| v.floats()).collect()).unwrap_or_default();
    let cyl = e.child(h("cylinder-extents"));
    let offset = cyl.and_then(|c| c.child(h("offset"))).and_then(|o| o.text.as_ref()).map(|t| t.floats())
        .filter(|f| f.len() >= 3).map_or(Vec3::ZERO, |f| Vec3::new(f[0], f[1], f[2]));
    let pair = |x: Option<&Element>| x.map_or((0, 0), |x| (name(x, h("effect-name")), name(x, h("hardpoint-name"))));
    let motion = e.attr(h("motion-archetype"));
    Some(CharacterType {
        key,
        archive: archive.to_string(),
        library: library.to_string(),
        label: name(part, H_LABEL),
        mesh: name(part, h("mesh-name")),
        motion_archetype: motion.and_then(|v| v.as_str()).map(|s| s.trim().to_string())
            .unwrap_or_else(|| format!("h_{:08x}", motion.and_then(|v| v.as_hash()).unwrap_or(0))),
        motion_set: motion.and_then(|v| match v { Value::Str(s) => Some(h(s.trim())), v => v.as_hash() }).unwrap_or(0),
        face_set: e.child(h("face")).map_or(0, |f| name(f, h("viseme-file"))),
        chatter: text(e, h("chatter-library")),
        hitpoints: combat.and_then(|c| num(c, h("hitpoints"))).unwrap_or(0.0),
        target_type: combat.and_then(|c| int(c, h("target-type"))).unwrap_or(0),
        vulnerabilities,
        shield,
        damage_multiplier: num(e, h("damage-multiplier")).unwrap_or(1.0),
        max_stamina: num(e, h("max-stamina")).unwrap_or(0.0),
        height_offset: num(e, h("height-offset")).unwrap_or(0.0),
        turn_rate: num(e, h("turn-rate")).unwrap_or(0.0),
        player_type: int(e, H_PLAYER_TYPE).unwrap_or(0) != 0,
        ragdoll_death: int(e, h("enable-ragdoll-death")).unwrap_or(1) != 0,
        ai,
        weapons: inventory.map(|i| weapon::hashes(i, h("weapon-name")).into_iter().filter(|&w| w != h("")).collect()).unwrap_or_default(),
        items: inventory.map(|i| counted(i.children_named(h("item")))).unwrap_or_default(),
        weapon_classes: [class("one"), class("two")],
        speeds: [0, 1, 2, 3].map(|i| speeds.get(i).copied().unwrap_or(0.0)),
        capsule: Capsule {
            radius: cyl.and_then(|c| num(c, h("radius"))).unwrap_or(0.0),
            height: cyl.and_then(|c| num(c, h("height"))).unwrap_or(0.0),
            crouch_height: cyl.and_then(|c| num(c, h("crouch-height"))).unwrap_or(0.0),
            offset,
        },
        decals: H_DECALS.map(|k| weapon::hashes(e, k).into_iter().filter(|&d| d != h("")).take(3).collect()),
        death_effect: pair(e.child(h("death-effect"))),
        attached_effects: part.child(H_ATTACHED).map(|l| l.children_named(H_ATTACHED_EFFECT).map(|x| pair(Some(x)))
            .filter(|&(fx, _)| fx != 0).collect()).unwrap_or_default(),
        footstep_type: int(e, H_FOOTSTEP).unwrap_or(-1),
        jump_sound: e.child(h("movement-sounds")).map_or(0, |m| name(m, h("jump-sound"))),
        skill: skill_data(e),
        award: part.child(H_AWARD).map(|a| Award {
            amount: num(a, h("amount")).unwrap_or(0.0),
            per_damage: int(a, H_AWARD_PER_DAMAGE).unwrap_or(0) != 0,
            on_hit: int(a, H_AWARD_ON_HIT).unwrap_or(0) != 0,
            to_squad: int(a, H_AWARD_TO_SQUAD).unwrap_or(0) != 0,
            to_own: int(a, H_AWARD_TO_OWN).unwrap_or(0) != 0,
            kind: int(a, H_AWARD_KIND).unwrap_or(0),
        }).unwrap_or_default(),
    })
}

/// A motion set from its file's root (`<motion-sequence>`), `file` its stem ("ms_lf").
pub fn parse_motion_set(root: &Element, file: &str) -> MotionSet {
    let all = root.walk();
    let seq = all.iter().find(|e| e.name == h("sequence"));
    let ready = |k: u32| root.child(k).map_or((0.0, 0.0), |b| (num(b, H_IN_TIME).unwrap_or(0.0), num(b, h("out-time")).unwrap_or(0.0)));
    MotionSet {
        file: file.to_string(),
        name: name(root, h("name")),
        anim_set: name(root, h("animation-set-name")),
        npc_ready: ready(H_NPC_READY),
        pc_ready: ready(H_PC_READY),
        default_motion: seq.map_or(0, |s| name(s, h("default-motion"))),
        general_idle: seq.map_or(0, |s| name(s, h("general-idle"))),
        cautious_idle: seq.map_or(0, |s| name(s, h("cautious-idle"))),
        motions: all.iter().filter(|e| e.name == h("MOTION")).map(|m| Motion {
            name: name(m, h("name")),
            motion_type: int(m, h("motion-type")).unwrap_or(0),
            scripts: strings(m, h("script")),
            ready_scripts: strings(m, h("ready-script")),
            prefix: int(m, h("prefix-type")).unwrap_or(0),
            next: name(m, h("next-motion")),
            transition: num(m, h("transition")).unwrap_or(0.0),
            looping: int(m, h("loop")).unwrap_or(0) != 0,
        }).collect(),
        overlays: all.iter().filter(|e| e.name == h("overlay")).map(|o| Overlay {
            name: name(o, h("name")),
            motion_type: int(o, h("motion-type")).unwrap_or(0),
            script: strings(o, h("script")).into_iter().next().unwrap_or_default(),
            prefix: int(o, h("prefix-type")).unwrap_or(0),
        }).collect(),
        hit_bones: root.child(h("collision")).and_then(|c| c.child(h("bones"))).map(|b| b.children_named(h("bone")).filter_map(|b| {
            Some(HitBone {
                bone: b.attr(h("name"))?.as_hash()?,
                mass: num(b, h("mass")).unwrap_or(1.0),
                damage: num(b, h("Damage")).unwrap_or(1.0),
                recoil: int(b, h("recoil")).unwrap_or(0) != 0,
            })
        }).collect()).unwrap_or_default(),
    }
}

/// The placed characters of a level file (`<character-object>`), read from `archive`.
pub fn parse_placements(root: &Element, archive: &str) -> Vec<Placement> {
    root.walk().into_iter().filter(|e| e.name == h("character-object")).filter_map(|e| {
        let t = e.child(h("transform")).and_then(|t| t.text.as_ref()).map(|t| t.floats()).unwrap_or_default();
        let ai = e.child(h("ai"));
        let inventory = e.child(h("inventory"));
        Some(Placement {
            archive: archive.to_string(),
            kind: name(e, h("object-type-name")),
            name: name(e, h("name")),
            transform: level::transform(&t)?,
            ai: PlacementAi {
                alignment: ai.and_then(|a| int(a, h("alignment-type"))).unwrap_or(ALIGN_NONE),
                personalities: ["personality-freewill", "personality-follow", "personality-formation", "personality-guard", "personality-wingman"]
                    .map(|k| ai.map_or(0, |a| name(a, h(k)))),
                vision: ai.and_then(|a| num(a, h("vision"))).unwrap_or(0.0),
                goal_id: ai.and_then(|a| int(a, h("goal-id"))).unwrap_or(0),
                vision_own: ai.and_then(|a| int(a, H_VISION_OWN)).unwrap_or(0) != 0,
                field_of_view: ai.and_then(|a| num(a, H_FIELD_OF_VIEW)).unwrap_or(0.0),
            },
            weapons: inventory.map(|i| counted(i.children_named(h("weapon")))).unwrap_or_default(),
            items: inventory.map(|i| counted(i.children_named(H_AMMO_ITEM))).unwrap_or_default(),
            drops: e.child(H_DROPS).map(|d| weapon::hashes(d, h("inventory-drop")).into_iter().filter(|&x| x != h("")).collect()).unwrap_or_default(),
            controllable: int(e, H_CONTROLLABLE).unwrap_or(0) != 0,
            accuracy_bonus: (int(e, H_ACCURACY_ON).unwrap_or(0) != 0).then(|| int(e, H_ACCURACY).unwrap_or(0)),
        })
    }).collect()
}

/// The character types of an objecttypes document, library `library` of `archive` (its
/// `<character>` list), in the document's order.
pub(crate) fn parse_types(root: &Element, archive: &str, library: &str) -> Vec<CharacterType> {
    root.children_named(h("character")).flat_map(|l| l.children_named(H_ENTRY))
        .filter_map(|e| parse_type(e, archive, library)).collect()
}

impl Game {
    /// Adds the character types of an objecttypes document, library `library` of `archive` (its
    /// `<character>` list): a name already loaded keeps its first definition (FUN_00160da0).
    pub(crate) fn add_character_types(&mut self, root: &Element, archive: &str, library: &str) {
        for t in parse_types(root, archive, library) {
            self.add_character_type(t);
        }
    }

    /// Adds one character type, unless a type of its name is loaded already (FUN_00160da0: the
    /// first library with the name wins).
    pub(crate) fn add_character_type(&mut self, t: CharacterType) {
        if !self.character_types.contains_key(&t.key) {
            self.character_type_order.push(t.key);
            self.character_types.insert(t.key, t);
        }
    }

    /// Adds an archive's motion sets (characters/ms_<x>.xmb), keyed by the hash of the file's
    /// stem; one already loaded is kept (every copy of a file is the same). A file that doesn't
    /// decode is skipped, and so is one whose stem ("ms_lf") `keep` refuses. The stems added,
    /// for what's derived from them (`Game::skill_overlays`).
    pub(crate) fn add_motion_sets(&mut self, ar: &Archive, keep: impl Fn(&str) -> bool) -> Vec<String> {
        let mut added = vec![];
        for n in ar.find("ms_", ".xmb") {
            let stem = n.trim_end_matches(".xmb");
            if self.motion_sets.contains_key(&h(stem)) || !keep(stem) {
                continue;
            }
            let Some(root) = ar.get(&n).and_then(|d| bxml::parse(d, &self.schemas).ok()) else { continue };
            self.motion_sets.insert(h(stem), parse_motion_set(&root, stem));
            added.push(stem.to_string());
        }
        added
    }

    /// Adds the placed characters of a level file read from `archive` (each archive has one
    /// level file): once, though an archive may be loaded again (the test map's levels and the
    /// map, `Game::load_level`).
    pub(crate) fn add_placements(&mut self, root: &Element, archive: &str) {
        if !self.placements.iter().any(|p| p.archive == archive) {
            self.placements.extend(parse_placements(root, archive));
        }
    }

    /// The character type `key`, if a loaded archive defines it.
    pub fn character_type(&self, key: u32) -> Option<&CharacterType> {
        self.character_types.get(&key)
    }

    /// A character type's display name (its string), else its key as h_xxxxxxxx.
    pub fn type_label(&self, key: u32) -> String {
        self.character_types.get(&key).and_then(|t| self.strings.get(&t.label)).cloned().unwrap_or_else(|| format!("h_{key:08x}"))
    }

    /// The character type a test hook names (`s`, any case): its key as h_xxxxxxxx, its
    /// `<base name>` (brutus, mp_brutus), its display name (the first loaded of that name), or
    /// `name@archive` (that archive's or that objecttypes library's: `@m09_a`, `@sdm_shrike`).
    /// None if no loaded type matches.
    pub fn find_character_type(&self, s: &str) -> Option<u32> {
        let s = s.trim();
        if let Some(k) = s.strip_prefix("h_").and_then(|x| u32::from_str_radix(x, 16).ok()) {
            return self.character_types.contains_key(&k).then_some(k);
        }
        if self.character_types.contains_key(&h(s)) {
            return Some(h(s));
        }
        let (label, archive) = match s.rsplit_once('@') {
            Some((l, a)) => (l.trim(), Some(a.trim())),
            None => (s, None),
        };
        self.character_type_order.iter().copied().find(|k| {
            let t = &self.character_types[k];
            self.strings.get(&t.label).is_some_and(|n| n.eq_ignore_ascii_case(label))
                && archive.is_none_or(|a| t.archive.eq_ignore_ascii_case(a) || t.library.eq_ignore_ascii_case(a))
        })
    }

    /// The world material a model is made of (#110 leads section 4): the id (the low 5 bits,
    /// `collision::MATERIAL_ID`) most of its parts' collision triangles carry, read through each
    /// PART's archetype's surface. 21 FLESH_HUMAN (the squad but Flint, humans, ferals), 22
    /// MUTANT, 23 SHRIKE, 24 SYNTHETIC (Flint, Kingman), 25 SEER, 26 FIREHOUND. None when the
    /// loaded levels lack its parts' collision.
    pub fn flesh(&self, mesh: u32) -> Option<u8> {
        let (arch, _) = self.archetype(mesh)?;
        let parts: Vec<u32> = arch.walk().into_iter().filter(|e| e.name == h("PART")).map(|p| name(p, h("archetype-name"))).collect();
        let mut count: HashMap<u8, usize> = HashMap::new();
        for surface in parts.into_iter().filter_map(|p| self.archetype_physics(p)) {
            for (_, m) in self.collision(surface) {
                *count.entry(m & MATERIAL_ID).or_default() += 1;
            }
        }
        count.into_iter().max_by_key(|&(id, n)| (n, id)).map(|(id, _)| id)
    }

    /// A character type's hit bones: its motion set's `<collision>` list (FUN_00130aa0; the
    /// game's type+0x188). None when the type or its motion set isn't loaded.
    pub fn hit_bones(&self, key: u32) -> Option<&[HitBone]> {
        let t = self.character_types.get(&key)?;
        self.motion_sets.get(&t.motion_set).map(|m| m.hit_bones.as_slice())
    }

    /// Whether a type's model (its compound archetype) is in the loaded archives.
    pub fn has_model(&self, t: &CharacterType) -> bool {
        self.archetype(t.mesh).is_some()
    }

    /// Whether the animation set `set` has a clip whose name hashes to `clip`.
    pub fn has_clip(&self, set: u32, clip: u32) -> bool {
        self.anim_sets.get(&set).is_some_and(|a| a.iter().any(|c| c.name == clip))
    }

    /// The root-height of a type's default motion's clip (FUN_00131a10 at 0x131b2f: type+0x1b0):
    /// how high its root sits over the floor at rest; DEFAULT_ROOT_HEIGHT when the clip has none.
    pub fn default_root_height(&self, t: &CharacterType) -> f32 {
        let Some(ms) = self.motion_sets.get(&t.motion_set) else { return DEFAULT_ROOT_HEIGHT };
        let clip = ms.motion(ms.default_motion).and_then(|m| m.clip(0, false, 1, 0, |c| self.has_clip(ms.anim_set, c)));
        clip.and_then(|n| self.anim_sets.get(&ms.anim_set)?.iter().find(|a| a.name == h(&n))?.root_height).unwrap_or(DEFAULT_ROOT_HEIGHT)
    }

    /// The character types report (BF_DUMP_TYPES): per type (in load order) its key, archive,
    /// name, model (loaded: bones, LOD0 vertices), motion set, animation set and face set, flesh,
    /// HP and factors, shield, team, speeds, capsule, the ragdoll switch, weapons, decals and
    /// chatter; then a line per motion type with each MOTION's clips (stance w1_: script /
    /// ready-script, "!" when the set lacks it) and the idles' root-heights; then each loaded
    /// placement with its resolved team; then the totals and the squad's check (its four loaded
    /// by type match `Character::load` by name). `models`: load each type's model (slower).
    pub fn character_type_report(&self, models: bool, squad: &[&str]) -> Vec<String> {
        let mut out = vec![];
        let (mut no_model, mut no_motion, mut no_anims, mut missing_clips) = (0, 0, 0, 0);
        let team = |a: i64| TEAMS.get((a + 1).max(0) as usize).copied().unwrap_or("?");
        for &k in &self.character_type_order {
            let t = &self.character_types[&k];
            let ms = self.motion_sets.get(&t.motion_set);
            let set = ms.map_or(0, |m| m.anim_set);
            let clips = self.anim_sets.get(&set).map_or(0, |a| a.len());
            let model = if models {
                match Character::load_type(self, t, 0) {
                    Ok(c) => format!("{} bones, {} LOD0 vertices, {} face clips", c.bones.len(), c.geosets.iter().map(|g| g.positions.len()).sum::<usize>(), c.face_anims.len()),
                    Err(e) => { no_model += 1; format!("NOT LOADED: {e}") }
                }
            } else {
                String::from("(not loaded)")
            };
            if ms.is_none() { no_motion += 1; }
            if clips == 0 { no_anims += 1; }
            let hashes = |v: &[u32]| v.iter().map(|x| format!("h_{x:08x}")).collect::<Vec<_>>().join(" ");
            let weapons: Vec<String> = t.weapons.iter().map(|w| self.weapons.get(w).map_or(format!("h_{w:08x}"), |d| d.label.clone())).collect();
            let chatter = t.chatter.trim_end_matches("_chatter");
            out.push(format!("type h_{k:08x} [{}] \"{}\" mesh h_{:08x}: {model}", t.place(), self.type_label(k), t.mesh));
            out.push(format!("  motion set {} -> anim set h_{set:08x} ({clips} clips){}, face h_{:08x}{}, flesh {:?}, default root-height {:.3}",
                t.motion_archetype, if ms.is_none() { " NOT LOADED" } else { "" }, t.face_set,
                if t.face_set != 0 && !self.anim_sets.contains_key(&t.face_set) { " (not loaded)" } else { "" },
                self.flesh(t.mesh), self.default_root_height(t)));
            out.push(format!("  hp {} target-type {} vulnerabilities {:?} shield {} damage-multiplier {} | team {} {} | stamina {} turn-rate {} player-type {} ragdoll-death {}",
                t.hitpoints, t.target_type, t.vulnerabilities,
                t.shield.as_ref().map_or("-".into(), |s| format!("type {} charge {} hit-effect h_{:08x} recharge {:?}", s.kind, s.max_charge, s.hit_effect, s.recharge)),
                t.damage_multiplier, t.ai.alignment, team(t.ai.alignment), t.max_stamina, t.turn_rate, t.player_type, t.ragdoll_death));
            out.push(format!("  speeds {:?} capsule r {} h {} crouch {} offset {:.2} height-offset {} | weapons {:?} classes {:?} items {} | decals [{}] [{}] | death-effect h_{:08x} attached {} | chatter {}{} | footsteps {} jump h_{:08x}",
                t.speeds, t.capsule.radius, t.capsule.height, t.capsule.crouch_height, t.capsule.offset, t.height_offset, weapons, t.weapon_classes,
                t.items.len(), hashes(&t.decals[0]), hashes(&t.decals[1]), t.death_effect.0, t.attached_effects.len(),
                if chatter.is_empty() { "-" } else { chatter }, if !chatter.is_empty() && !self.chatter.contains_key(chatter) { " (not loaded)" } else { "" },
                t.footstep_type, t.jump_sound));
            let Some(ms) = ms else { continue };
            let has = |c: u32| self.has_clip(ms.anim_set, c);
            let seconds = |n: &str| self.anim_sets.get(&ms.anim_set).and_then(|a| a.iter().find(|c| c.name == h(n))).map(|c| (c.duration, c.root_height));
            let mut types: Vec<i64> = ms.motions.iter().map(|m| m.motion_type).collect();
            types.sort();
            types.dedup();
            for mt in types {
                let mut line = format!("    {mt:2} {:<22}", MOTION_TYPES.get(mt as usize).copied().unwrap_or("?"));
                for m in ms.motions_of(mt) {
                    // each slot that isn't empty: its script's clip / its ready-script's (the
                    // script's when the ready-script is empty, `Motion::clip`)
                    for v in 0..m.scripts.len().max(m.ready_scripts.len()) {
                        let slot = |l: &[String]| l.get(v).map_or("", |s| s.as_str()).to_string();
                        let (script, ready) = (slot(&m.scripts), slot(&m.ready_scripts));
                        if script.is_empty() && ready.is_empty() {
                            continue;
                        }
                        let show = |is_ready: bool| -> String {
                            match m.clip(v, is_ready, 1, 0, has) {
                                Some(n) => {
                                    let (d, r) = seconds(&n).unwrap_or((0.0, None));
                                    let r = if mt == MT_IDLE { r.map_or(String::from(" root -"), |r| format!(" root {r:.3}")) } else { String::new() };
                                    format!("{n} {d:.2}s{r}")
                                }
                                None => format!("!{}", m.clip_name(if is_ready && !ready.is_empty() { &ready } else { &script }, 1, 0)),
                            }
                        };
                        let (a, b) = (show(false), show(true));
                        missing_clips += a.starts_with('!') as usize + b.starts_with('!') as usize;
                        line += &format!(" | {a} / {b}");
                    }
                }
                out.push(line);
            }
        }
        let mut differ = 0;
        let mut unresolved = 0;
        for p in &self.placements {
            let t = self.character_types.get(&p.kind);
            if t.is_none() { unresolved += 1; }
            let own = t.map_or(ALIGN_NONE, |t| t.ai.alignment);
            if p.ai.alignment != ALIGN_NONE && p.ai.alignment != own { differ += 1; }
            let at = p.transform.w_axis.truncate();
            out.push(format!("placement [{}] h_{:08x} type h_{:08x} {}: \"{}\" team {} {} (own {}, type's {}) at {:.1} {:.1} {:.1} weapons {:?} drops {}{}",
                p.archive, p.name, p.kind, if t.is_some() { "ok" } else { "UNRESOLVED" }, self.type_label(p.kind), p.team(t), team(p.team(t)),
                p.ai.alignment, own, at.x, at.y, at.z, p.weapons.iter().map(|(w, _)| self.weapons.get(w).map_or(format!("h_{w:08x}"), |d| d.label.clone())).collect::<Vec<_>>(),
                p.drops.len(), if p.controllable { " controllable" } else { "" }));
        }
        out.push(format!("{} character types ({} models not loaded, {} without a motion set, {} without clips; {} MOTION clips missing), {} motion sets, {} placements ({} unresolved, {} with their own team)",
            self.character_types.len(), no_model, no_motion, no_anims, missing_clips, self.motion_sets.len(), self.placements.len(), unresolved, differ));
        // the squad's type against the tables keyed by its name: the same HP, damage factors,
        // multiplier, weapons, decals and leash, and its own motion set (ms_<name>, whose hit
        // bones and skill overlays the name-keyed lookups had), so play can read them by type
        // without a change. (On a squad deathmatch map the damage-multiplier by name is sdm_bf's
        // 1, `load_squad_library`, not the campaign type's 0.62: play_damage.rs keeps it.)
        for &s in squad {
            let Some(t) = self.character_type(h(s)) else { continue };
            let mut differ: Vec<String> = vec![];
            if self.character_hitpoints.get(s) != Some(&t.hitpoints) { differ.push("hitpoints".into()); }
            if (0..12).any(|k| self.damage_factor(s, k) != t.vulnerability(k)) { differ.push("damage factors".into()); }
            if let Some(m) = self.character_damage_multiplier.get(s).filter(|&&m| m != t.damage_multiplier) {
                differ.push(format!("damage-multiplier ({m} by name, {} by type)", t.damage_multiplier));
            }
            if t.motion_set != h(&format!("ms_{s}")) { differ.push("motion set".into()); }
            if self.character_weapons.get(s) != Some(&t.weapons) { differ.push("weapons".into()); }
            if self.character_decals.get(s).copied() != Some((t.decals[0].first().copied().unwrap_or(0), t.decals[1].first().copied().unwrap_or(0))) { differ.push("decals".into()); }
            if self.squad_leash.get(s) != Some(&t.ai.squad_leash) { differ.push("squad-leash-dist".into()); }
            out.push(format!("squad {s}: type's data {}", if differ.is_empty() { "== the name-keyed tables'".to_string() } else { format!("DIFFERS in {}", differ.join(", ")) }));
        }
        // the squad by type against Character::load by name (its stem): the same model, clips and face
        for &s in squad {
            let by_type = self.character_type(h(s)).ok_or("no type".to_string()).and_then(|t| Character::load_type(self, t, 0));
            let by_name = Character::load(self, s, 0);
            let sum = |c: &Character| (c.bones.clone(), c.geosets.iter().map(|g| (g.material, g.positions.len(), g.indices.len())).collect::<Vec<_>>(),
                                       c.anims.iter().map(|a| a.name).collect::<Vec<_>>(), c.face_anims.iter().map(|a| a.name).collect::<Vec<_>>());
            out.push(match (by_type, by_name) {
                (Ok(a), Ok(b)) => format!("squad {s}: by type {} by name ({} bones, {} clips, {} face clips)", if sum(&a) == sum(&b) { "==" } else { "DIFFERS FROM" },
                                          a.bones.len(), a.anims.len(), a.face_anims.len()),
                (a, b) => format!("squad {s}: by type {:?}, by name {:?}", a.err(), b.err()),
            });
        }
        out
    }
}

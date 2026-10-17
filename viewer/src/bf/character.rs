//! Characters straight from the level archives. Port of char_render.py.
//!
//!  objecttypes  character: motion-archetype ms_<name> + base mesh-name (compound archetype)
//!  objects      compound: PART list (bones, parent-first), joints (parent-part, parent-point),
//!               per-part inverse bind (local = R v + t, R stored row-major), LOD skin meshes
//!  .ivd         32-byte skinned vertices; indices as raw u16 strips/lists or NV2A push buffers
//!  materials    skin material (type h_eb58ab52) has no textures: the next material does
//!  animations   archetype-set <name> -> animations -> target bone -> channel in .chnl

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bevy::math::{Mat3, Mat4, Quat, Vec3, Vec4};

use super::ale;
use super::archive::Archive;
use super::audio::{self, SoundBank, Surface};
use super::weapon::{self, Hardpoint, WeaponDef};
use super::bxml::{self, Element, SchemaSet, Value};
use super::character_type::{CharacterType, MotionSet, Placement};
use super::personality::{self, Personality};
use super::hash::h;
use super::texture::{self, TexInfo};

const SKIN_WRAPPER: u32 = 0xEB58_AB52;
/// The animated liquid shader (toxic rivers): its colour is "texture-1".
const LIQUID_SHADER: u32 = 0xF124_A774;
/// The self-lit shader's (h_f539fe8c: skies, pickup icons) one texture, "h_e01baa40" (also
/// the name of its colour constant).
const GLOW_TEXTURE: u32 = 0xE01B_AA40;
/// Terrain vertices (h_ef44f398; the game's vertex format table, slot 8: SHORT2 position, packed
/// normal, two float2 uvs, NORMSHORT3, 2 bytes). Its vertex shader (default.xbe 0x3d50f8) splits
/// the first short into row and column (row * 65 + column) and scales both and the height
/// from shader constants that are set at run time; these values were fitted against the
/// level: the second uv set is exactly the 2 m grid / 10, heights land on 26-54 m (the level's
/// terrain range) and placed objects sit on the ground.
const TERRAIN_VERTEX: u32 = 0xEF44_F398;
const TERRAIN_ROW: u32 = 65;
const TERRAIN_CELL: f32 = 2.0;
const TERRAIN_HEIGHT: f32 = 1.0 / 16.0;
const H_WRAP_COUNT: u32 = 0xEDCA_4B16;
/// A wrapper material's opacity, 0-255 (e.g. 50 for a pickup's glow shell).
pub const H_WRAP_ALPHA: u32 = 0xE59D_69A0;

/// How a texture is sampled past its edges, per axis: a material's texture entry carries its
/// `address-u` / `address-v` (`TAM_WRAP`, `TAM_CLAMP` or `TAM_MIRROR`). The gates' leaves run
/// their texture 0..2 across with TAM_MIRROR: each leaf shows its picture and its mirror image.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Address {
    #[default]
    Wrap,
    Clamp,
    Mirror,
}

impl Address {
    fn of(mode: u32) -> Address {
        match mode {
            m if m == h("TAM_CLAMP") => Address::Clamp,
            m if m == h("TAM_MIRROR") => Address::Mirror,
            _ => Address::Wrap,
        }
    }
}
const DT_QUAT: u32 = 0x0667_7BCB;
const DT_VEC: u32 = 0xF36D_8810;
const DT_XFORM: u32 = 0x0690_1441;
const FT_KEYED: u32 = 0x00AD_833C;
const LIST_PRIM: u32 = 0x0E7A_B726;
const H_INVBIND: u32 = 0xE4B8_22AF;
const H_JOINT: u32 = 0x151A_CE78;
const H_PALETTE: u32 = 0xF133_0520;
const H_DRAW: u32 = 0xFDE5_0E9F;
/// A mesh geoset's bounding sphere: its radius attribute and its centre child (the names of an
/// archetype's and an effect type's alike; `Character::bounds`).
const H_GEOSET_RADIUS: u32 = 0x1AB0_9530;
const H_GEOSET_CENTRE: u32 = H_ARCHETYPE_CENTRE;
const H_INTERVAL: u32 = 0xF63F_0F49;
/// First clip of every '<name>face' set: closed, relaxed mouth. Human faces are modelled
/// mid-speech, so the game always layers a face clip; this one is the neutral default.
const NEUTRAL_FACE: u32 = 0xEFF8_AC8D;
/// Face clips shorter than this are held poses (the neutral face is two keys, 0.03 s).
const STATIC_FACE: f32 = 0.1;
/// game-options `<difficulty-settings>` (name recovered by hashing): the campaign's difficulty
/// values (`Game::difficulty_settings`)
const H_DIFFICULTY_SETTINGS: u32 = 0xFFC1_BECB;

/// combat-target's damage-type factor list (h_142be76f; the shield holds an empty one of its own)
/// and an entry's factor (`<h_1d403525 Type=4 h_04653d86=0.05>`: Flint's for the Gas, common
/// objecttypes). Names unknown, read from the layout.
const H_DAMAGE_FACTORS: u32 = 0x142B_E76F;
const H_DAMAGE_FACTOR: u32 = 0x0465_3D86;
/// An object type's debris list and its entries' attributes (see `Debris`), and an effect
/// type's damage list (`<h_fb0a5f1d><Damage>`, see `AreaDamage`). Names unknown.
const H_DEBRIS_LIST: u32 = 0x197C_AF14;
const H_DEBRIS: u32 = 0x19C8_DF19;
const H_DEBRIS_MAIN: u32 = 0x1C1E_17FE;
const H_DEBRIS_DELAY: u32 = 0x1B6A_0EDE;
const H_DEBRIS_LIFE: u32 = 0xF2E4_E1A3;
const H_EFFECT_DAMAGE: u32 = 0xFB0A_5F1D;
/// An archetype's centre point (`<h_e99750a9>x y z</h_e99750a9>`), see `Game::archetype_centre`.
const H_ARCHETYPE_CENTRE: u32 = 0xE997_50A9;
/// Collision: the objects file's surface table, its entries (name, h_e9e44859 = offset in the
/// .ipn) and the attribute an archetype / terrain block / blocker names its surface with.
pub(crate) const H_PHYSICS_FILE: u32 = 0x104D_B1CD;
pub(crate) const H_PHYSICS: u32 = 0xF3F0_67AD;
const H_PHYSICS_OFFSET: u32 = 0xE9E4_4859;
pub const H_PHYSICS_NAME: u32 = 0x051A_C63E;

fn hash_of(v: Option<&Value>) -> u32 {
    v.and_then(|v| v.as_hash()).unwrap_or(0)
}
fn int_of(e: &Element, name: u32) -> i64 {
    e.attr(name).and_then(|v| v.as_i64()).unwrap_or(0)
}

/// A zone's movie list (`list`: its `pre-movies` or `post-movies` children), each movie with its
/// subtitles (see `Subtitle`). A number stored as text is read as a number.
fn campaign_movies(zone: &Element, list: u32) -> Vec<CampaignMovie> {
    let num = |e: &Element, k: u32| e.attr(k).and_then(|v| v.as_f32().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok())));
    zone.children_named(list).map(|m| CampaignMovie {
        file: m.attr(h("bink-file")).and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        subtitles: m.children_named(H_SUBTITLE).map(|s| {
            let style = s.child(h("style"));
            Subtitle {
                text: hash_of(s.attr(h("text-string"))),
                start: num(s, H_SUBTITLE_START).unwrap_or(0.0),
                shown: num(s, h("display-time")).unwrap_or(0.0),
                letter: style.and_then(|st| num(st, H_LETTER_TIME)).unwrap_or(0.0),
                colours: style.map(|st| st.children_named(H_SUBTITLE_COLOUR).filter_map(|c| {
                    let text = c.text.as_ref()?;
                    let mut ints = text.ints();
                    if ints.is_empty() {
                        ints = text.as_str().map(|s| s.split_whitespace().filter_map(|x| x.parse().ok()).collect()).unwrap_or_default();
                    }
                    let v: Vec<u8> = ints.into_iter().map(|i| i.clamp(0, 255) as u8).collect();
                    (v.len() >= 4).then(|| [v[0], v[1], v[2], v[3]])
                }).collect()).unwrap_or_default(),
            }
        }).collect(),
    }).collect()
}

pub struct Channel {
    pub data_type: u32,
    pub keyed: bool,
    pub interval: f32,
    pub frames: usize,
    pub offset: usize,
    pub size: usize,
    pub blob: Arc<Vec<u8>>,
}

/// An animation archetype's own ground speed attribute (see `AnimDef::move_speed`).
const H_MOVE_SPEED: u32 = 0xF962_8571;

pub struct AnimDef {
    pub name: u32,
    pub duration: f32,
    /// (bone name hash, channel name hash)
    pub targets: Vec<(u32, u32)>,
    /// timed events (footsteps, drop_weapon, ...), see `Game::events`
    pub event_channel: u32,
    /// its `root-height` (m): how high the root bone sits over the floor in this clip, where
    /// FUN_0023d4f0 puts a character's root node, the point a missile homes on (play_missile's
    /// `root_point`); None for the data's -100000 (the model's own height then)
    pub root_height: Option<f32>,
    /// its own ground speed (m/s), attribute h_f9628571 (name not recovered): the clip record's
    /// +8 (FUN_00286710's attribute handler at 0x286f0f, copied at 0x286a6b), which FUN_0012c6c0
    /// divides a walk's speed by for the clip's play rate. On the squad's walks it is the root
    /// channel's distance over the clip's length (Tex's Sc_w1_rp_walk 1.616: 1.83 m in 1.133 s);
    /// 5.9e-05 on Tex's Sc_w1_rp_f_side_walk_l, whose root doesn't move. None when absent.
    pub move_speed: Option<f32>,
}

/// Everything loaded from data/common.tgz (+ textures found in level archives on demand).
pub struct Game {
    pub schemas: SchemaSet,
    objects: Vec<(Element, Arc<Vec<u8>>)>,
    pub materials: HashMap<u32, HashMap<u32, u32>>,
    /// the shader type (Material Type) that renders each material
    material_types: HashMap<u32, u32>,
    /// constants (param -> values) of the material that renders each material
    pub material_constants: HashMap<u32, HashMap<u32, Vec<f32>>>,
    /// each material's textures' address modes (texture -> [u, v])
    material_address: HashMap<u32, HashMap<u32, [Address; 2]>>,
    textures: HashMap<u32, (TexInfo, Arc<Vec<u8>>)>,
    pending_texture_archives: Vec<PathBuf>,
    pub channels: HashMap<u32, Channel>,
    pub anim_sets: HashMap<u32, Vec<AnimDef>>,
    /// (name, compound archetype hash)
    pub characters: Vec<(String, u32)>,
    /// character name -> (footstep type h_072ab4af, jump sound id)
    pub character_audio: HashMap<String, (i64, u32)>,
    /// character name -> hitpoints (combat-target)
    pub character_hitpoints: HashMap<String, f32>,
    /// character name -> damage-type factors: (damage-type, factor) from its combat-target's
    /// own h_142be76f list (`<h_1d403525 Type h_04653d86>`; not the shield's list). Common
    /// objecttypes: Flint takes type 4 (the Gas) x0.05 and type 6 (the Energy) x2; Brutus, Hawk
    /// and Tex list none (x1)
    pub character_damage_factors: HashMap<String, Vec<(i64, f32)>>,
    pub sounds: SoundBank,
    /// world materials (surface sound sets) of the loaded level
    pub surfaces: Vec<Surface>,
    /// English string table (iD hash -> text)
    pub strings: HashMap<u32, String>,
    /// weapon definitions of the loaded objecttypes (common + level)
    pub weapons: HashMap<u32, WeaponDef>,
    /// character name -> starting inventory weapons
    pub character_weapons: HashMap<String, Vec<u32>>,
    /// character name -> the weapon class (weapon-type: 0 light, 1 medium, 2 heavy) each of its
    /// two weapon slots holds, from `<limit-weapon one="CLASS COUNT" two=..>`; the motion
    /// sets Sc_w1_* / Sc_w2_* are the stances for those classes (Brutus: medium, heavy)
    pub character_weapon_classes: HashMap<String, [i64; 2]>,
    /// character name -> chatter line tag -> voice line ids (common/characters/<name>_chatter)
    pub chatter: HashMap<String, HashMap<u32, Vec<u32>>>,
    /// character name -> chatter line tag -> the tag others answer it with (a block's
    /// `response` / response_id: e.g. a death cry -> "<name> is down!")
    pub chatter_response: HashMap<String, HashMap<u32, u32>>,
    /// character name -> the AI's squad-leash-dist (m): how far a squadmate strays from the leader
    pub squad_leash: HashMap<String, f32>,
    /// character name -> the breathing heard while it looks through a scope (`<h_12501487
    /// snipe-sound>`, beside its radar-range; a 2D sound of the common bank)
    pub character_snipe_sound: HashMap<String, u32>,
    /// character name -> the scope's two sounds beside it (h_00dd1fe0, h_02c94d34: ~0.3 s each,
    /// taken as going into the scope and coming out; Flint has her own pair)
    pub character_scope_sounds: HashMap<String, (u32, u32)>,
    /// character name -> its camera offsets per mode (`<h_0d41e5f1>`: offset-walk .. offset-dead,
    /// offset-snipe; see `CameraOffsets`)
    pub character_camera: HashMap<String, CameraOffsets>,
    /// character name -> HUD icons: portrait (h_f9d94425) and the skull shown when dead (h_00c51907)
    pub character_icons: HashMap<String, (u32, u32)>,
    /// character name -> blood decals: per hit (h_fb4bfc68) and the pool where the body lies
    /// (h_efc2a874), names into `decals`
    pub character_decals: HashMap<String, (u32, u32)>,
    /// character name -> its special skill and stamina (see `SkillData`): the campaign squad's,
    /// or squad deathmatch's once `load_squad_library` has put them in their place
    pub character_skills: HashMap<String, SkillData>,
    /// character name -> its motion set's skill toggle overlays (see `SkillOverlays`; from the
    /// parsed `motion_sets`). (A motion set's hit bones are its `MotionSet::hit_bones`, a type's
    /// `Game::hit_bones`.)
    pub skill_overlays: HashMap<String, SkillOverlays>,
    /// character name -> its type's `damage-multiplier` (h_e7e9fcd3, the type's +0x220): a
    /// direct hit's factor with the struck bone's (FUN_00125940). The campaign squad 0.62, the
    /// Shrikes 1; on a squad deathmatch map the sdm_bf squad's 1 (`load_squad_library`)
    pub character_damage_multiplier: HashMap<String, f32>,
    /// the campaign's difficulty values [easy, normal, hard]: common/game-options-en.xmb
    /// `<difficulty-settings>` (h_ffc1becb) `<h_1b8535f0 easy="-4" normal="0" hard="4">`. The
    /// SELECT DIFFICULTY menu (FUN_000f13c0) stores STANDARD's easy, HARD's normal or BRUTAL's
    /// hard in the game flag campaign/difficulty-setting, the n of every hit's step on a
    /// character (0x125600; play_damage.rs). None if the file doesn't have them
    pub difficulty_settings: Option<[i64; 3]>,
    /// game-options `<reticule>`: the crosshair's lock reach and box (see `ReticuleOptions`)
    pub reticule: ReticuleOptions,
    /// game-options `<combat-skills>`: the skills' own tuning (see `CombatSkills`)
    pub combat_skills: CombatSkills,
    /// game-options' HUD `<main h_e72ec57a h_1d9f07c5>`: the screen's safe margins across and
    /// down, 48 and 36 HUD units (7.5% of 640 x 480): the HUD keeps its widgets 3 units inside
    /// them (FUN_000b2770 and its three neighbours; Flint's targeting marker, play_targeting.rs)
    pub hud_safe: [f32; 2],
    /// character name -> chatter line tag -> what the game's chatter gate reads of its block (see
    /// `ChatterBlock`; play_skills.rs's skill lines go through it)
    pub chatter_blocks: HashMap<String, HashMap<u32, ChatterBlock>>,
    /// ground decals of objecttypes `<decal>`, by name
    pub decals: HashMap<u32, DecalDef>,
    /// object type name -> its mesh archetype (objecttypes `<base mesh-name>` around `<base name>`)
    pub object_meshes: HashMap<u32, u32>,
    /// liquid types (objecttypes `<h_fa2f5452>`): type name -> its settings
    pub liquids: HashMap<u32, LiquidType>,
    /// inventory item types (objecttypes `<inventory>`): type name -> its settings
    pub items: HashMap<u32, ItemType>,
    /// the loaded levels' level files (levels-<name>.xmb: terrain, placed objects, sky, fog)
    pub levels: Vec<Element>,
    /// the game's level list (common campaign-bf.xmb), in its order
    pub campaign: Vec<CampaignLevel>,
    /// the same file's missions: the squad, the extra marines, each mission's zones and
    /// objectives (#221, `campaign::Campaign`)
    pub missions: super::campaign::Campaign,
    /// a new campaign's game flags (the file campaign-bf names, common/default-gameflags.xmb:
    /// #221, `gameflags::GameFlags`); empty if it isn't there
    pub default_flags: super::gameflags::GameFlags,
    /// animated textures (textures-*.xmb <animated-texture>): name -> the sheet texture, frames
    /// per second (h_e6375a47) and each frame's (u0, v0, u1, v1) on the sheet
    pub flipbooks: HashMap<u32, Flipbook>,
    /// collision surfaces (objects-<lvl>.ipn) by name: the blob and the surface's offset
    physics: HashMap<u32, (Arc<Vec<u8>>, usize)>,
    /// the particle effect library (common/effects-common.ale), effects keyed by name hash
    pub effects: ale::Library,
    /// effect types (objecttypes `<effect>`): type name -> the ALE effects it plays (name hashes,
    /// keys of `effects`)
    pub effect_types: HashMap<u32, Vec<u32>>,
    /// effect types again, with their light effect and sounds (see `EffectType`)
    pub effect_type_defs: HashMap<u32, EffectType>,
    /// object type -> the effect type it shows while idle (`<events><event state="1"
    /// h_ed0c9fac=..>`: the power-ups' spinning icons)
    pub idle_effects: HashMap<u32, u32>,
    /// object types (objecttypes `<h_e275fb80><base name object-type ..>`): type name -> its
    /// object-type, combat target and what it breaks into (see `ObjectType`)
    pub object_types: HashMap<u32, ObjectType>,
    /// character types (#128): every objecttypes `<character>` entry by its `<base name>` hash,
    /// the first loaded archive's (see `character_type::CharacterType`)
    pub character_types: HashMap<u32, CharacterType>,
    /// `character_types`' keys in the order they were loaded (common's squad first)
    pub character_type_order: Vec<u32>,
    /// motion sets (characters/ms_<x>.xmb of every loaded archive) by the hash of the file's
    /// stem, a type's motion-archetype (see `character_type::MotionSet`)
    pub motion_sets: HashMap<u32, MotionSet>,
    /// the loaded levels' placed characters (`<character-object>`, see
    /// `character_type::Placement`)
    pub placements: Vec<Placement>,
    /// the AI personalities (common/ai/personality.xmb, #129) by name hash (see
    /// `personality::Personality`)
    pub personalities: HashMap<u32, Personality>,
    /// names the loaded levels' cutscene scripts give their entities (scripts/*.gscr,
    /// `entity_name = "..."`): the triggers they signal ("trigger_spawn8"), the spawn copies
    /// ("tex_1", "TUTORIAL_terrorist_5"), cameras, by name hash. The level files name everything
    /// by hash; the trigger runtime's log (play_triggers.rs) reads names from here
    pub script_names: HashMap<u32, String>,
    /// the loaded levels' cutscene scripts (scripts/<name>.gscr, `gscr::Script`) by the hash of
    /// their name, what a script-trigger's `script-name` holds (FUN_00174180 registers each so);
    /// a later level's replaces an earlier one's of the same name (#222)
    pub scripts: HashMap<u32, super::gscr::Script>,
}

/// An object type's `<base object-type>`: 0 a compound of loose pieces (archetype-type 9, the
/// debris that flies apart), 2 a game object (scenery, archetype-type 7), 17 an effect object
/// (archetype-type 6: effects, a light, sounds and `<Damage>` areas). The game's OT_ name table
/// (0x3be87c) names them OT_BODY, OT_STATIC_GEOMETRY and OT_EFFECT.
pub const OBJECT_COMPOUND: i64 = 0;
pub const OBJECT_GAME: i64 = 2;
pub const OBJECT_EFFECT: i64 = 17;

/// An object type (objecttypes `<h_e275fb80>` entries), as far as breaking it goes:
///
///   <h_e275fb80 mesh-name ..><base name=T object-type=2 archetype-type=7/>
///     <combat-target hitpoints=H><h_142be76f><h_1d403525 Type=K h_04653d86=F/>..</h_142be76f>
///     <h_197caf14><h_19c8df19 archetype-name=D h_1c1e17fe=main h_1b6a0ede=delay h_f2e4e1a3=life/>..
///
/// A game object whose h_197caf14 list isn't empty breaks when its hitpoints run out: the game's
/// take-damage (FUN_002232d0, the combat-target's vtable slot 0) takes value x factor[Type] off
/// them and at 0 sends message 0x4e; the object (FUN_00157260) queues its debris list then.
#[derive(Clone, Debug, Default)]
pub struct ObjectType {
    /// `<base object-type>` (OBJECT_COMPOUND, OBJECT_GAME, OBJECT_EFFECT, ...)
    pub object_type: i64,
    /// combat-target hitpoints (40 on most; the radiation barrel and the missile rack 1, the
    /// supply crate 25)
    pub hitpoints: f32,
    /// the combat-target's own h_142be76f list: (damage-type, factor), x1 for types it doesn't
    /// list (the record at type +0x60 + Type x 4 that FUN_002232d0 multiplies by)
    pub factors: Vec<(i64, f32)>,
    /// h_197caf14: what it breaks into, in the list's order
    pub debris: Vec<Debris>,
    /// its radar-contact by default (H_RADAR_CONTACT on the game-archetype record that holds the
    /// `<base name object-type>`; the type's +0x130, set on the object at its set-up, 0x155058):
    /// 0 for the scenery, 7 for the squad's memory chip h_f50f94f9, 6 for the bosses
    pub radar_contact: u8,
}

/// An object's radar-contact (name recovered by hashing as `radar-contact-type`; byte +0x7a of
/// the object, set by FUN_0015b310): RCONTACT_ (name table 0x3beaac) 0 NONE, 1 ALIGNMENT (every
/// character), 2 OBJECTIVE, 3 OBJECTIVE_NO_CURSOR, 4 OBJECTIVE_SECONDARY, 5 INVISIBLE, 6
/// BOSS_ALIGNMENT, 7 IMPORTANT_PICKUP. On an objecttypes game-archetype record (the type's
/// default) and on a level's placements (the object's own). The crosshair turns yellow over a 2
/// or a 4 (0xc59d7, play_reticle.rs); the radar reads it too (#160).
pub const H_RADAR_CONTACT: u32 = 0x1905_F381;

/// game-options `<reticule>` (common/game-options-en.xmb; line 125 of the decoded
/// decompiled/xml/common/game-options-en.xml; its attribute handler
/// 0x19d7d0 fills the settings the app's vtable +0x48 returns): the crosshair's lock rules
/// (play_reticle.rs). The defaults are the shipped file's.
#[derive(Clone, Copy, Debug)]
pub struct ReticuleOptions {
    /// h_0c3eb4ff (settings +0x18c, 0x19d80d): 0.7, the lock's reach as a share of the gun's
    /// (FUN_000bf7a0 and FUN_00232590 multiply FUN_00232520 by it)
    pub lock_share: f32,
    /// h_f079aebd (settings +0x178, 0x19d8ba): 0.14, the lock box's size as a share of the
    /// screen's height; 0xc50b0 puts it in the target selection's +0x30, and x 0.75 (0x3a510c, the
    /// 4:3 aspect) in its +0x2c for the width
    pub box_share: f32,
}

/// game-options `<combat-skills>` (h_f7bc2603, name recovered by hashing; line 158 of the decoded
/// decompiled/xml/common/game-options-en.xml): one block per skill, parsed by FUN_0019e1f0 into
/// the settings the app's vtable +0x48 returns, which each skill's constructor copies. What the
/// demo's skills use of it (play_skills.rs); the defaults are the shipped file's.
#[derive(Clone, Copy, Debug)]
pub struct CombatSkills {
    /// Tex's block h_04aa86aa, h_1469a0df: his drain's factor while the gun in hand's clip is
    /// empty, 0.5 (settings +0x25c, which his constructor FUN_0013d440 copies to skill+0x24 at
    /// 0x13d475 and his drain 0x13d720 multiplies by; take05 measured -2.22/s, then -1.12/s)
    pub berserk_empty_drain: f32,
    /// Brutus's block h_f5dc6aac (Spirit of Vengar, see `VengarSkill`)
    pub vengar: VengarSkill,
    /// Hawk's block h_0ec919f1 (her Stealth, `StealthSkill`)
    pub stealth: StealthSkill,
    /// Flint's block h_0725cdb1 (her Advanced Targeting, `Tracking`)
    pub tracking: Tracking,
}

impl Default for CombatSkills {
    fn default() -> Self {
        CombatSkills { berserk_empty_drain: 0.5, vengar: VengarSkill::default(), stealth: StealthSkill::default(), tracking: Tracking::default() }
    }
}

/// Hawk's `<combat-skills>` block h_0ec919f1 (line 161 of the decoded game-options-en.xml: `<h_0ec919f1
/// h_00894f2a="1" h_f726177e="4" h_1d194665="0.25" h_1883630e="h_f6b07f87"/>`): her skill's
/// constructor FUN_0013cf10 copies the settings' +0x28c, +0x290 and +0x294 into the skill (the
/// offsets below). Only stealth-level-recovery-rate (h_f726177e) is named, by hashing. The
/// fourth, h_1883630e (settings +0x298, her vtable +0x134), names the Power Blade's item type
/// h_f6b07f87, with which in hand her drain is x5 (0x13d030): the demo has no Power Blade. The
/// defaults are the shipped file's.
#[derive(Clone, Copy, Debug)]
pub struct StealthSkill {
    /// h_00894f2a, +0x28c -> skill+0x34: her drain's factor while the gun of the second weapon
    /// slot is in hand (the inventory's +0x1cc at 2, 0x13d046; FUN_0014d090 reads 1 there as slot
    /// 1 in hand): 1
    pub second_slot_drain: f32,
    /// stealth-level-recovery-rate, h_f726177e, +0x290 -> skill+0x2c: how fast a second the
    /// stealth factor (skill+0x28) climbs back from the 0 a shot sets it to (FUN_0013d1a0): 4, so
    /// a shot shows her to the AI for a quarter second (#35 takes 12, 13: 0 -> 1 in 0.25-0.3 s)
    pub level_recovery: f32,
    /// h_1d194665, +0x294 -> skill+0x30: the floor of her noise and footsteps' factor, `floor +
    /// (1 - floor) x her visibility` (her +0xe8, 0x13d210): 0.25, a quarter as loud stealthed
    pub noise_floor: f32,
}

impl Default for StealthSkill {
    fn default() -> Self {
        StealthSkill { second_slot_drain: 1.0, level_recovery: 4.0, noise_floor: 0.25 }
    }
}

/// Brutus's `<combat-skills>` block h_f5dc6aac (game-options-en.xml line 160): FUN_0019e210 puts
/// each attribute at its settings offset, and his skill's constructor FUN_0013bef0 copies them to
/// the skill object (the offsets below). The names weren't recovered but `vengar-damage-multiplier`
/// (h_f4f602c6); the uses are read from the code that takes each field (play_skills.rs `Vengar`,
/// play_vengar.rs). The defaults are the shipped file's.
#[derive(Clone, Copy, Debug)]
pub struct VengarSkill {
    /// h_eaa78fdf, settings +0x260, skill+0x20: hit points regenerated a second while on, up to
    /// the max (his update 0x13c0b0): 1 (take07 +1.01/s)
    pub regen: f32,
    /// `vengar-damage-multiplier`, h_f4f602c6, +0x264, skill+0x24: what his skill's +0xd8 (0x13c140)
    /// gives the damage handler 0x125600 (at 0x125747) while on: 0.85 (take27: 0.84 per hit)
    pub damage_taken: f32,
    /// h_0461a627, +0x268, skill+0x28: the stamina a charge hit takes (put in skill+0x2c at
    /// 0x13c474, drained with the next frame's drain 0x13c570): 100
    pub charge_cost: f32,
    /// h_f5dae098, +0x26c, skill+0x30: the upward part of a charge kill's knock-back (the push
    /// vector's y; its x and z are Brutus's velocity, 0x13c4c9-0x13c4fd): 2
    pub knock_up: f32,
    /// h_0e406b26, +0x270, skill+0x34, and h_fa20e794, +0x274, skill+0x38: FUN_000e00b0's third and
    /// fifth arguments on a charge kill (2 and 75; a weapon kill passes twice its damage and the
    /// damage record's +0x54 there, 0x11605b). What they do to the body isn't traced
    pub knock_weight: f32,
    pub knock_force: f32,
    /// h_e26c9e84, +0x278, skill+0x3c: the charge's reach, m (the query's radius and its distance
    /// test, FUN_0013c1c0): 1.5
    pub reach: f32,
    /// h_f3d82857, +0x27c, skill+0x40: the charge's half-angle off his facing, degrees (its cosine,
    /// x pi/180 at 0x3a4fe0): 30
    pub cone: f32,
    /// h_14935eb6, +0x288, skill+0x44: a charge hit's damage, dealt as DTYPE_EXPLOSION (10) through
    /// FUN_00223160: 200 (take08: 280 on an enemy, the difficulty step)
    pub damage: f32,
    /// h_0571f62e, +0x280 (the skill's vtable +0x134): the effect type started on a victim,
    /// h_0dcd4fc7 (the sound h_e6a4d091, and ALE `vengar_hit` at a creation-time of 6.9e28 s, so
    /// never: `EffectType::starts`); 0 none
    pub hit_effect: u32,
    /// h_e5417809, +0x284 (vtable +0x138): the effect type started on a victim killed: none for
    /// Brutus (""), 0
    pub kill_effect: u32,
}

impl Default for VengarSkill {
    fn default() -> Self {
        VengarSkill { regen: 1.0, damage_taken: 0.85, charge_cost: 100.0, knock_up: 2.0, knock_weight: 2.0, knock_force: 75.0, reach: 1.5,
                      cone: 30.0, damage: 200.0, hit_effect: 0x0DCD_4FC7, kill_effect: 0 }
    }
}

/// Flint's `<combat-skills>` block h_0725cdb1 (line 162 of the decoded game-options-en.xml): its
/// attribute handler 0x19e450 stores each in the settings, and her skill's constructor
/// FUN_0013c9e0 copies them into the skill (0x13ca13-0x13cab9). The defaults are the shipped
/// file's. (Its h_1b947b90, the tracking-reticule-speed 2 at settings +0x2b0 -> skill+0x38, only
/// moves the hidden crosshair, play_skills.rs `Targeting`; h_f192e10e names the marker's HUD
/// widget, h_121cf7bb, settings +0x2b4: both left out.)
#[derive(Clone, Copy, Debug)]
pub struct Tracking {
    /// tracking-weapon-type (h_108b56f9, settings +0x29c -> skill+0x44, parsed as a WTYPE_ name or
    /// number, FUN_00186e80): 1, WTYPE_RIFLE. Switched on with a gun of this class in hand, the
    /// toggle marks skill+0x48 and asks for motion type 45 (0x13cbbc-0x13cbd5)
    pub weapon_type: i64,
    /// h_108f0925 (+0x2a4 -> skill+0x24) and h_081520f0 (+0x2a0 -> skill+0x28): 0.4 and 0.4,
    /// the crosshair's lock box as shares of the screen's width and height while on (0x13cd30
    /// writes them into the target selection's +0x2c / +0x30; the marker's wide box is the same,
    /// 0xb4b14 / 0xb4b49)
    pub box_shares: [f32; 2],
    /// h_18bf34d4 (+0x2a8 -> skill+0x2c): 0.15, the chance of a doubled roll while on (0x13cc70)
    pub double_chance: f32,
    /// h_fe103e1b (+0x2ac -> skill+0x30): 2, that roll's factor
    pub double_factor: f32,
    /// h_191dafb6 (+0x2b8 -> skill+0x34): 1.5, the roll's factor while scoped, on or off
    pub scope_factor: f32,
    /// h_ec0b2194 (+0x2bc -> skill+0x3c): 1, WTYPE_RIFLE, the class whose range the skill
    /// lengthens (0x13ce80)
    pub range_weapon_type: i64,
    /// h_f82a974c (+0x2c0 -> skill+0x40): 1.25, that range's factor
    pub range_factor: f32,
}

impl Default for Tracking {
    fn default() -> Self {
        Tracking { weapon_type: 1, box_shares: [0.4, 0.4], double_chance: 0.15, double_factor: 2.0, scope_factor: 1.5,
                   range_weapon_type: 1, range_factor: 1.25 }
    }
}

/// game-options' `<combat-skills>` element and Tex's block in it, and the block's attribute read
/// as `CombatSkills::berserk_empty_drain` (names not recovered but the first).
const H_COMBAT_SKILLS: u32 = 0xF7BC_2603;
const H_BERSERKER_BLOCK: u32 = 0x04AA_86AA;
const H_BERSERK_EMPTY_DRAIN: u32 = 0x1469_A0DF;
/// Brutus's block in it (`VengarSkill`) and its attributes, by the settings offset FUN_0019e210
/// gives each.
const H_VENGAR_BLOCK: u32 = 0xF5DC_6AAC;
const H_VENGAR_REGEN: u32 = 0xEAA7_8FDF;
const H_VENGAR_DAMAGE_TAKEN: u32 = 0xF4F6_02C6;
const H_VENGAR_CHARGE_COST: u32 = 0x0461_A627;
const H_VENGAR_KNOCK_UP: u32 = 0xF5DA_E098;
const H_VENGAR_KNOCK_WEIGHT: u32 = 0x0E40_6B26;
const H_VENGAR_KNOCK_FORCE: u32 = 0xFA20_E794;
const H_VENGAR_REACH: u32 = 0xE26C_9E84;
const H_VENGAR_CONE: u32 = 0xF3D8_2857;
const H_VENGAR_DAMAGE: u32 = 0x1493_5EB6;
const H_VENGAR_HIT_EFFECT: u32 = 0x0571_F62E;
const H_VENGAR_KILL_EFFECT: u32 = 0xE541_7809;
/// Hawk's block in `<combat-skills>` and its attributes (`StealthSkill`; stealth-level-recovery-rate
/// recovered by hashing, the rest not).
const H_STEALTH_BLOCK: u32 = 0x0EC9_19F1;
const H_STEALTH_SECOND_SLOT: u32 = 0x0089_4F2A;
const H_STEALTH_RECOVERY: u32 = 0xF726_177E;
const H_STEALTH_NOISE: u32 = 0x1D19_4665;
/// Flint's block in `<combat-skills>` and its attributes (`Tracking`; tracking-weapon-type and
/// weapon-range-modifier recovered by hashing, the rest not).
const H_TRACKING_BLOCK: u32 = 0x0725_CDB1;
const H_TRACKING_WEAPON_TYPE: u32 = 0x108B_56F9;
const H_TRACKING_BOX_X: u32 = 0x108F_0925;
const H_TRACKING_BOX_Y: u32 = 0x0815_20F0;
const H_DOUBLE_CHANCE: u32 = 0x18BF_34D4;
const H_DOUBLE_FACTOR: u32 = 0xFE10_3E1B;
const H_SCOPE_FACTOR: u32 = 0x191D_AFB6;
const H_RANGE_WEAPON_TYPE: u32 = 0xEC0B_2194;
const H_RANGE_FACTOR: u32 = 0xF82A_974C;

/// game-options' HUD `<main>` block (line 122 of the decoded game-options-en.xml, in the HUD's
/// h_06c98fbe beside `<reticule>`): its attribute handler (0x19d5ef, 0x19d61e) stores h_1d9f07c5
/// at settings +0x128 and h_e72ec57a at +0x12c, which the HUD copies to its +0x1374 / +0x1378
/// (0xb8b06-0xb8b0f; its constructor's own 30 / 40 at 0xb8aae are overwritten): the screen's
/// safe margins in HUD units, down and across (`Game::hud_safe`).
const H_HUD_SAFE_Y: u32 = 0x1D9F_07C5;
const H_HUD_SAFE_X: u32 = 0xE72E_C57A;

impl Default for ReticuleOptions {
    fn default() -> Self {
        ReticuleOptions { lock_share: 0.7, box_share: 0.14 }
    }
}

impl ObjectType {
    /// The factor it takes damage of `damage_type` by (1 if its list doesn't name the type).
    pub fn factor(&self, damage_type: i64) -> f32 {
        self.factors.iter().find(|f| f.0 == damage_type).map_or(1.0, |f| f.1)
    }
}

/// One entry of an object type's debris list (`<h_19c8df19>`; its parser FUN_00187fc0 fills a
/// 16-byte record: +0 archetype-name, +4 h_1c1e17fe, +8 h_1b6a0ede, +0xc h_f2e4e1a3).
#[derive(Clone, Copy, Debug, Default)]
pub struct Debris {
    /// the object type it spawns (archetype-name): a compound of pieces, a game object that
    /// stays (the missile rack's stand) or an effect object
    pub kind: u32,
    /// h_1c1e17fe: the first entry has it (true on the compound of pieces). The spawn
    /// (FUN_0015ac90) hands this one the hit's push (the object's +0xe8..+0x100)
    pub main: bool,
    /// h_1b6a0ede (s): read by the debris countdown FUN_0015af20 (see play_scenery.rs: an entry
    /// spawns once the countdown from the list's longest delay is down to its own delay, so the
    /// longest comes first)
    pub delay: f32,
    /// h_f2e4e1a3: 9999 on every entry (a lifetime, not seen used)
    pub life: f32,
}

/// An effect object's `<h_fb0a5f1d><Damage ..>` (the parser FUN_00191170 fills a 0x1c-byte
/// record: +0 amount, +4 Type, +8 h_ed582b3c, +0xc h_f724cb8c, +0x10 duration, +0x14 range,
/// +0x18 falloff). The radiation barrel's h_1aee9c6b: amount 80, Type 3, duration 0.5, range 3,
/// falloff 0; the missile rack's h_e7d56d93: 200, Type 10, 0.5, 8, falloff 3.
#[derive(Clone, Copy, Debug, Default)]
pub struct AreaDamage {
    pub amount: f32,
    /// the DTYPE_ enum (default.xbe 0x3bd9d0): 1 BALLISTIC .. 3 BIOREACTIVE, 10 EXPLOSION
    pub damage_type: i64,
    /// h_ed582b3c (2 on the barrel and rack, 4 on others; meaning not traced)
    pub mode: i64,
    /// h_f724cb8c (0 on every one; read as a delay, as on the effects' sounds)
    pub delay: f32,
    pub duration: f32,
    pub range: f32,
    /// the DFALL_ enum (default.xbe 0x3bda70): 0 NONE, 1 LINEAR, 2 EXPONENTIAL, 3 HALF_LIFE
    pub falloff: i64,
}

/// A level of the game's list (campaign-bf.xmb): its archive (data/<file>.tgz), name and
/// description (string ids, see `Game::strings`) and its preview picture (a texture).
#[derive(Clone, Debug)]
pub struct CampaignLevel {
    pub file: String,
    /// every archive of the level, in its order (its `<zone file-name>`s, lower case; a
    /// mission's parts, e.g. M01_a and M01_b; `file` is the first)
    pub zones: Vec<String>,
    pub name: u32,
    pub description: u32,
    pub preview: u32,
    /// the planet's name (string id, h_1dff9e2d)
    pub planet: u32,
    /// the kind of level (Type): LEVEL_DEATHMATCH, LEVEL_SQUAD_DEATHMATCH or a mission
    pub kind: u32,
    /// recommended players (dm-data h_0398b8bf min, max; 0 0 on the squad deathmatch maps)
    pub players: (i64, i64),
    /// the level's id (`<level h_046391fd>`: the level parser FUN_001a4000 makes its holder,
    /// FUN_001a3eb0 stores it): a mission's name in the game flags, `h("m00")` for the tutorial
    /// and `h("m01")` for Rescue Brutus, `h("e01")` for Cemetery Gates (the names recovered by
    /// hashing)
    pub id: u32,
    /// each zone's movie lists, in `zones` order (the zone parser FUN_001a3d40)
    pub movies: Vec<ZoneMovies>,
}

/// A zone's movie lists (campaign-bf.xmb `<zone>`, its children read by FUN_001a3d40): the
/// `pre-movies` the game plays before the zone loads and the `post-movies` after it, in order.
#[derive(Clone, Debug, Default)]
pub struct ZoneMovies {
    /// before the zone loads (the zone's +0x40)
    pub pre: Vec<CampaignMovie>,
    /// after it (+0x44)
    pub post: Vec<CampaignMovie>,
}

/// One movie of a zone's list: its Bink file in data/movies (`bink-file`, FUN_0018d750; the
/// names as the list writes them, e.g. "mb01.bik" for data/movies/MB01.bik) and its subtitles in
/// order (`<h_e5d32e85>`, FUN_0018d7b0).
#[derive(Clone, Debug)]
pub struct CampaignMovie {
    pub file: String,
    pub subtitles: Vec<Subtitle>,
}

/// A movie's subtitle: FUN_0018d690 reads its attributes into a 0x20-byte record, FUN_0018d610
/// its `<style>`. The movie shows them one after another (the sequence FUN_0010c060 walks): each
/// takes `start` + `shown` seconds from the end of the one before (the first from the movie's
/// start) and is on screen for the last `shown` of them.
#[derive(Clone, Debug)]
pub struct Subtitle {
    /// the text (`text-string`, a string table id; record +0)
    pub text: u32,
    /// h_f724cb8c (record +4): seconds from the end of the previous subtitle to this one
    pub start: f32,
    /// `display-time` (record +8): seconds on screen
    pub shown: f32,
    /// the style's h_1927c9d0 (record +0xc): seconds a letter takes as the text types out
    pub letter: f32,
    /// the style's h_05387560 colours, RGBA, in order (FUN_001a6e30 appends each): the text's
    /// `^c1` picks the first, `^c2` the second
    pub colours: Vec<[u8; 4]>,
}

/// campaign-bf.xmb's names (no string recovered) for a level's id, a subtitle element, its start
/// and its style's seconds a letter and colour (FUN_001a4000, FUN_0018d7b0, FUN_0018d690,
/// FUN_0018d610); the lists' other names are known strings (`pre-movies`, `bink-file`,
/// `text-string`, `display-time`, `style`).
const H_LEVEL_ID: u32 = 0x0463_91FD;
const H_SUBTITLE: u32 = 0xE5D3_2E85;
const H_SUBTITLE_START: u32 = 0xF724_CB8C;
const H_LETTER_TIME: u32 = 0x1927_C9D0;
const H_SUBTITLE_COLOUR: u32 = 0x0538_7560;

/// Level list kinds (campaign-bf.xmb `Type`): the deathmatch arenas (mp*) and the squad
/// deathmatch maps (sdm_*); missions are h_17506391.
pub const LEVEL_DEATHMATCH: u32 = 0x1207_85EF;
pub const LEVEL_SQUAD_DEATHMATCH: u32 = 0xEFB1_8D28;

/// An animated texture: frames laid out on one sheet texture.
#[derive(Clone, Debug)]
pub struct Flipbook {
    pub texture: u32,
    pub fps: f32,
    pub frames: Vec<[f32; 4]>,
}

/// The objecttypes list of liquid types, and the placed liquids' element (levels-*.xmb).
const H_LIQUIDS: u32 = 0xFA2F_5452;
pub const H_LIQUID_OBJECT: u32 = 0x0436_6A6A;

/// A character type's camera block (objecttypes `<h_0d41e5f1>`): one offset per camera mode,
/// each a 3-float list stored as a child element. The objecttypes field parser `FUN_0018fa80`
/// copies the seven into the character type as 12-byte vectors from +0x1b0, in this order:
/// walk 0x1b0, run 0x1bc, dash 0x1c8, ready 0x1d4, `h_e42d50a5` 0x1e0, dead 0x1ec, snipe 0x1f8.
/// The squad's walk .. `h_e42d50a5` differ per character (Brutus `3.4 7 0`, Flint `3.2 1 0`),
/// dead is `3.5 5 0` for all four and snipe `0.001 0 0`. What the three numbers mean isn't in
/// the data; the death camera reads `dead` as (height, distance back, unused).
#[derive(Clone, Copy, Debug, Default)]
pub struct CameraOffsets {
    pub walk: [f32; 3],
    pub run: [f32; 3],
    pub dash: [f32; 3],
    pub ready: [f32; 3],
    /// h_e42d50a5: the fifth mode (name not recovered; the same as walk for the squad)
    pub h_e42d50a5: [f32; 3],
    /// offset-dead (hash f13fb0c4): the death camera's framing of the body
    pub dead: [f32; 3],
    pub snipe: [f32; 3],
}

/// The camera block of a character type, and the element names of two of its offsets
/// (`offset-dead` is hash f13fb0c4; `h_e42d50a5` has no recovered name).
const H_CAMERA_BLOCK: u32 = 0x0D41_E5F1;
const H_OFFSET_DEAD: u32 = 0xF13F_B0C4;
const H_OFFSET_5: u32 = 0xE42D_50A5;

/// A squad character's special skill and stamina, from its objecttypes character entry
/// (`common/objecttypes-common.xml` for the campaign squad, `mp_common/objecttypes-sdm_bf.xml`
/// for squad deathmatch's, see `Game::load_squad_library`):
///
///   <h_e275fb80 motion-archetype="ms_tex" h_e78efbb2="true" max-stamina="100" ..>
///     <combat-skill skill-set="3" stamina-cost="2.2" stamina-recovery="1.8">
///       <sounds start-skill="h_ed7492f4" end-skill="h_e00d91fa" ../>
///
/// (`stamina-cost` and `stamina-recovery` are hashes 105c13aa / f2a2beb9 in the file, their
/// names recovered by hashing.) default.xbe: the parser FUN_00190330 keeps max-stamina, the
/// type loader 0x12ed40 copies it to the runtime type, the spawn (FUN_0012f440) sets stamina and
/// its max from it and the skill's cost (skill+4); FUN_0011a840 drains and refills.
#[derive(Clone, Debug, Default)]
pub struct SkillData {
    /// the skill (`skill-set`, default.xbe's CSKILL_ name table 0x3bd814): 1 Brutus's Spirit of
    /// Vengar, 2 Flint's Advanced Targeting, 3 Tex's Berserker, 4 Hawk's Stealth
    pub skill_set: i64,
    /// `max-stamina`: the stamina at spawn and its cap (0x12f46e); 100 for the squad
    pub max_stamina: f32,
    /// `stamina-cost`: stamina per second while the skill is on (the base drain, cost x dt)
    pub cost: f32,
    /// `stamina-recovery`: stamina per second while it's off (FUN_0011a840)
    pub recovery: f32,
    /// `start-skill` / `end-skill`: the sounds at the switch on and off (type+0x328 / +0x330); 0
    /// none
    pub start_sound: u32,
    pub end_sound: u32,
    /// h_1629a90a (name not recovered), the fourth sound of `<sounds>` (type+0x334): what Brutus's
    /// charge plays at each hit (FUN_0013c1c0 at 0x13c40c, through FUN_0010e7a0), h_e4c94dfe; 0 for
    /// the other three
    pub hit_sound: u32,
    /// h_e78efbb2: the stamina update runs for the type at all (type+0x1a8 bit 0, tested at
    /// 0x11a8b8); true for the four
    pub stamina_update: bool,
}

/// The skill data on an objecttypes character entry `e` (see `SkillData`), if it has a
/// `<combat-skill>` (every character type has one: `CharacterType::skill`).
pub(crate) fn skill_data(e: &Element) -> Option<SkillData> {
    let c = e.child(h("combat-skill"))?;
    let num = |x: &Element, k: u32| x.attr(k).and_then(|v| v.as_f32()).unwrap_or(0.0);
    let sound = |k: u32| c.child(h("sounds")).and_then(|s| s.attr(k)).and_then(|v| v.as_hash())
        .filter(|&x| x != 0 && x != h("")).unwrap_or(0);
    Some(SkillData {
        skill_set: int_of(c, h("skill-set")),
        max_stamina: num(e, h("max-stamina")),
        cost: num(c, h("stamina-cost")),
        recovery: num(c, h("stamina-recovery")),
        start_sound: sound(h("start-skill")),
        end_sound: sound(h("end-skill")),
        hit_sound: sound(H_SKILL_HIT_SOUND),
        stamina_update: e.attr(H_STAMINA_UPDATE).and_then(|v| v.as_i64()).unwrap_or(0) != 0,
    })
}

/// The character type's flag that turns its stamina update on (name not recovered).
const H_STAMINA_UPDATE: u32 = 0xE78E_FBB2;
/// `<combat-skill><sounds>`'s fourth sound (name not recovered; `SkillData::hit_sound`).
const H_SKILL_HIT_SOUND: u32 = 0x1629_A90A;

/// What the game's chatter gate (FUN_00140a60) reads of a `<chatter_line>` block (h_f74e6a16,
/// `common/characters/<name>_chatter.xml`), with the defaults the parser sets (FUN_0019cd70; the
/// header's attributes FUN_0019ca10, the variants FUN_0019c5d0, copied into the runtime record by
/// FUN_00136ab0). For the skill lines: probability 50, replay-delay 10 / 20 / 30 s, priority 1 /
/// 1 / 5 (on, off with stamina left, off at 0). todo/35-special-skills/answer-120-chatter.md.
#[derive(Clone, Debug)]
pub struct ChatterBlock {
    /// `probability` (h_ead503b2, name recovered by hashing): the gate's roll in % (FUN_001374c0:
    /// 100 or more always passes); 100 when not given
    pub probability: i64,
    /// `replay-delay` (h_ea21ae4b, recovered by hashing): seconds the speaker can't say the tag
    /// again once said (char+0x7f4, FUN_00126460); 0 when not given
    pub replay_delay: f32,
    /// `priority` (h_e146dc86, recovered by hashing); 5 when not given
    pub priority: i64,
    /// the condition kind (h_ff361b85, not recovered; the runtime record's +0x58): which value
    /// the variants' ranges test (FUN_00137360); h_1409c600 (its value 0) when not given
    pub condition: u32,
    /// h_f1cb4e63 (not recovered; default true): a step of priority less beyond 15 m of the
    /// player (FUN_001409c0); h_19be8951 (default true): not said beyond 45 m (FUN_001409a0)
    pub far_step: bool,
    pub far_cut: bool,
    /// the sound variants: id and the condition value's range (h_0ab96f95 min, 0 when not given;
    /// h_16087610 max, 100), in the file's order
    pub variants: Vec<(u32, i64, i64)>,
}

/// A chatter block's condition kind when it gives none (FUN_0019cd70's default; FUN_00137360
/// gives it the value 0).
pub const CHATTER_CONDITION_NONE: u32 = 0x1409_C600;
/// The block header's and the variants' attributes whose names weren't recovered (see
/// `ChatterBlock`).
const H_CHATTER_CONDITION: u32 = 0xFF36_1B85;
const H_CHATTER_FAR_STEP: u32 = 0xF1CB_4E63;
const H_CHATTER_FAR_CUT: u32 = 0x19BE_8951;
const H_CHATTER_MIN: u32 = 0x0AB9_6F95;
const H_CHATTER_MAX: u32 = 0x1608_7610;

/// A motion set's skill toggle overlays (`ms_<name>.xmb`, `<overlay motion-type=..>`): [on, off],
/// the entries of OMT_SPECIAL_ON (44) and OMT_SPECIAL_OFF (43), each its script and whether the
/// stance's prefix goes before it (`prefix-type` 1: `Sc_w1_` / `Sc_w2_` + script; 0: the script
/// is the clip's whole name). Tex `Sc_w1_bz_start` / `Sc_w1_bz_stop` (prefix 0); Brutus `spirit`
/// on only; Hawk `activate_stealth` both ways; Flint none.
pub type SkillOverlays = [Option<(String, bool)>; 2];
/// The overlays' motion types (OMT_SPECIAL_ON, OMT_SPECIAL_OFF).
const OMT_SPECIAL_ON: i64 = 44;
const OMT_SPECIAL_OFF: i64 = 43;

/// An inventory item type (objecttypes `<inventory>` entries: `<h_e275fb80 class-type=2
/// function-type=..>` around `<base mesh-name><base name=T>`). `function` is the game's IFSET_
/// enum (default.xbe name table at 0x3be3cc): 5 IFSET_GENERIC_HEALING (the Medkit: carried,
/// used later), 14 IFSET_AMMO_BOX, 19 IFSET_POWERUP_MEDKIT (Healing Garo Fruit: taken at once),
/// 20-23 the power-ups. `health` (h_0a811e94) and `stamina` (h_11884f2e) are what it restores:
/// Medkit 60, Healing Garo Fruit 40, HEALTH POWER 50 health, STAMINA POWER 50 stamina (and 0
/// health: so not a respawn time, as first guessed). `kind` (h_e5f51266) tells the type-19
/// items apart: fruit 1, medkit 2, health power 3, stamina power 4.
#[derive(Clone, Debug, Default)]
pub struct ItemType {
    pub function: i64,
    pub stack_limit: i64,
    pub amount: i64,
    pub health: f32,
    pub stamina: f32,
    pub kind: i64,
    /// the item it puts in the inventory when picked up (pickup-archetype), 0: itself. The
    /// placed Medkit h_f5123ace (60) gives h_192d5337, a Medkit of 80: what's carried and used.
    pub gives: u32,
    pub sound: u32,
    /// the sound of using it (h_153f90c9 on its <base>: the Medkit's h_eb368082; the fruit's is
    /// its pickup sound), 0 if none
    pub use_sound: u32,
    pub label: String,
    /// its HUD icon (h_e5ec3f1f), 0 if none
    pub icon: u32,
    /// the string shown when it's picked up (h_1dd3a49d on its <base>: the memory chip's
    /// h_091ce603 "Memory Chip Recovered!"), 0 if none
    pub message: u32,
}

/// A liquid type: a pool's surface is its object's collision plane, which characters sink
/// through. The game reads, besides these, five factors (h_049350bb .. h_e12e7b9d: 0.1 1 0.4
/// 0.4 0.1 on every liquid but one) and four sounds (entering, leaving, ..).
#[derive(Clone, Copy, Debug, Default)]
pub struct LiquidType {
    /// liquid-type: 1 water, 2 lava (sdm_e13), 4 toxic (sdm_m03's river);
    /// 3 hurts only deep (0 10 30), 5 is silent and harmless
    pub kind: i64,
    /// damage per second, from the shallowest depth to under (h_06f6a40d, h_eba69d33,
    /// h_e0489ea0: water 0 0 0, lava 50 50 30, toxic 50 50 50); see `play`'s use of them
    pub damage: [f32; 3],
    /// its effects (ALE effect name hashes): a shot striking it (h_08f6cd94, env_<x>l_projectile),
    /// a small splash (h_e4791abc, env_<x>l_smsplash), a big one (h_07ab4143, env_<x>l_mdsplash),
    /// the ring round a wader (h_f458daca, env_<x>l_wadering); <x> is x water, s lava, f toxic,
    /// t type 3, i type 0
    pub effects: [u32; 4],
    /// its four sounds (h_e259380a, h_0ed21145, h_0c46dff8, h_ed055bce: unnamed; taken in the
    /// effects' order: water's are 0.40, 0.61, 0.47 and 0.61 s)
    pub sounds: [u32; 4],
}

/// A liquid type's effect slots (`LiquidType::effects` / `sounds`).
pub const LIQUID_SHOT: usize = 0;
pub const LIQUID_SPLASH: usize = 1;
pub const LIQUID_BIG_SPLASH: usize = 2;
pub const LIQUID_WADE: usize = 3;

/// An effect type (objecttypes `<effect>`, object-type 17): what plays where it's started.
///
///   <h_e275fb80 ..><base name=T/>
///     <alchemy-effects><effect effect-name=E ../> x 8</alchemy-effects>
///     <light-effect effect-name=L ../>
///     <sounds><Sound sound-id=S h_f724cb8c=.. play-length=.. enable-looping=../></sounds>
///
/// The light effect is an ALE effect too (light_explosion, light_phosphor: a "light_" effect
/// lights the scene rather than drawing sprites). The Frag's explosion h_1fe9ae17: six ALE
/// effects (exp-lrg-*, exp-fire-add), light_explosion and the blast 145f09e5 (play-length 2).
#[derive(Clone, Debug, Default)]
pub struct EffectType {
    /// ALE effects (name hashes, keys of `Game::effects`)
    pub effects: Vec<u32>,
    /// each effect's `creation-time` (s after the type starts, the alchemy entry's +4 by
    /// FUN_00188a90), by `effects`' order: 0 for 30 of common's 37 entries, 2 for six; the
    /// Vengar hit's (h_0dcd4fc7) is 6.9e28, so its `vengar_hit` never shows (#35 takes 08, 22: no
    /// effect at the hit)
    pub starts: Vec<f32>,
    /// the light effect (an ALE effect name hash), 0 if none
    pub light: u32,
    /// sounds: (sound id, h_f724cb8c, play-length s, enable-looping). h_f724cb8c is read as a
    /// delay in seconds (an inference: 0 on most, 0.2 on the Light grenade's ignition)
    pub sounds: Vec<(u32, f32, f32, bool)>,
    /// `<h_fb0a5f1d><Damage ..>`: the damage areas it sets up where it's started (the
    /// exploding scenery's effects, see `AreaDamage`); empty on most
    pub damage: Vec<AreaDamage>,
    /// its bounding sphere: the centre `<h_e99750a9>` in the effect's own frame and the radius
    /// h_1ab09530 (FUN_00219ed0 reads them for the effect's distance LOD, ale_fx's LOD_TAN). The
    /// Therm Sweeper's explosion h_173132f0: (-1.99, -3.21, -1.67), 25 m; its trail h_194dc78e
    /// radius 0. None if the type has neither
    pub bounds: Option<(Vec3, f32)>,
}

/// A ground decal (objecttypes `<decal>`): one of its textures at random (white shapes), tinted
/// `color` (RGBA 0-1), `width` x `height` m, turned at random within +-`rotation` degrees; it stays
/// `life` s, fading out over the last `fade` s. (life / fade / rotation are inferred from the
/// values: footprints 30 / 2 / 0, blood 40 / 20 and 20 / 10 with 180.)
#[derive(Clone, Debug, Default)]
pub struct DecalDef {
    pub textures: Vec<u32>,
    pub width: f32,
    pub height: f32,
    pub color: [f32; 4],
    pub life: f32,
    pub fade: f32,
    pub rotation: f32,
}

impl Game {
    pub fn load(data_dir: &Path) -> Result<Self, String> {
        let common = Archive::open(&data_dir.join("common.tgz"), |_| true)?;
        let mut schemas = SchemaSet::default();
        for n in common.find("", ".xsb") {
            schemas.add(common.get(&n).unwrap()).map_err(|e| format!("{n}: {e}"))?;
        }
        let mut g = Game::empty(schemas);
        g.load_archive(&common, "common")?;
        let mut others: Vec<PathBuf> = std::fs::read_dir(data_dir)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "tgz") && !p.ends_with("common.tgz"))
            .collect();
        others.sort();
        g.pending_texture_archives = others;
        Ok(g)
    }

    /// A Game with nothing loaded but `schemas` (common's, to parse the documents with) and no
    /// archives to search for textures: what `Game::load` starts from, and the test map's
    /// scenery (#185, bf/pieces.rs) loads one level archive's pieces into.
    pub(crate) fn empty(schemas: SchemaSet) -> Game {
        Game {
            schemas,
            objects: vec![],
            materials: HashMap::new(),
            material_types: HashMap::new(),
            material_constants: HashMap::new(),
            material_address: HashMap::new(),
            textures: HashMap::new(),
            pending_texture_archives: vec![],
            channels: HashMap::new(),
            anim_sets: HashMap::new(),
            characters: vec![],
            character_audio: HashMap::new(),
            character_hitpoints: HashMap::new(),
            character_damage_factors: HashMap::new(),
            sounds: SoundBank::default(),
            surfaces: vec![],
            strings: HashMap::new(),
            weapons: HashMap::new(),
            character_weapons: HashMap::new(),
            character_weapon_classes: HashMap::new(),
            chatter: HashMap::new(),
            chatter_response: HashMap::new(),
            squad_leash: HashMap::new(),
            character_snipe_sound: HashMap::new(),
            character_scope_sounds: HashMap::new(),
            character_camera: HashMap::new(),
            character_icons: HashMap::new(),
            character_decals: HashMap::new(),
            character_skills: HashMap::new(),
            skill_overlays: HashMap::new(),
            chatter_blocks: HashMap::new(),
            decals: HashMap::new(),
            object_meshes: HashMap::new(),
            liquids: HashMap::new(),
            items: HashMap::new(),
            levels: vec![],
            flipbooks: HashMap::new(),
            campaign: vec![],
            missions: Default::default(),
            default_flags: Default::default(),
            physics: HashMap::new(),
            effects: ale::Library::default(),
            effect_types: HashMap::new(),
            effect_type_defs: HashMap::new(),
            character_damage_multiplier: HashMap::new(),
            difficulty_settings: None,
            reticule: ReticuleOptions::default(),
            combat_skills: CombatSkills::default(),
            hud_safe: [48.0, 36.0],
            idle_effects: HashMap::new(),
            object_types: HashMap::new(),
            character_types: HashMap::new(),
            character_type_order: vec![],
            motion_sets: HashMap::new(),
            placements: vec![],
            personalities: HashMap::new(),
            script_names: HashMap::new(),
            scripts: HashMap::new(),
        }
    }

    /// Every texture `other` has found so far (its index entries; the data is shared, not
    /// copied), as this one's, for a Game made with `empty`: a level's materials can name one of
    /// common's (the static scenery of m01_a, m05_a and m05_b does 16 times, the rest none).
    pub(crate) fn share_textures(&mut self, other: &Game) {
        for (k, v) in &other.textures {
            self.textures.entry(*k).or_insert_with(|| v.clone());
        }
    }

    /// Whether an objects file loaded has the archetype `name` (`archetype`).
    pub fn has_archetype(&self, name: u32) -> bool {
        self.archetype(name).is_some()
    }

    fn parse(&self, ar: &Archive, name: &str) -> Result<Option<Element>, String> {
        match ar.get(name) {
            Some(d) => bxml::parse(d, &self.schemas).map(Some).map_err(|e| format!("{name}: {e}")),
            None => Ok(None),
        }
    }

    /// Load the voice lines every level shares (data/ml-sounds/<lang>/common-<lang>.tgz: squad
    /// chatter and barks). Returns how many lines were added.
    pub fn load_voices(&mut self, data_dir: &Path, lang: &str) -> Result<usize, String> {
        let path = data_dir.join("ml-sounds").join(lang).join(format!("common-{lang}.tgz"));
        let ar = Archive::open(&path, |n| n.ends_with(".xwb"))?;
        let mut n = 0;
        for name in ar.find("", ".xwb") {
            if let Some(d) = ar.get(&name) {
                n += self.sounds.add_voices(Arc::new(d.to_vec()));
            }
        }
        Ok(n)
    }

    /// Read an archive's data (`archive`: its name, e.g. "common" or "e34", kept with the
    /// character types it defines). Entries already loaded win.
    fn load_archive(&mut self, ar: &Archive, archive: &str) -> Result<(), String> {
        self.load_motion_sets(ar, |_| true);
        // the AI personalities (common's ai/personality.xmb: the enemies' hit reactions, #129)
        if self.personalities.is_empty() {
            if let Some(root) = self.parse(ar, "personality.xmb")? {
                self.personalities = personality::parse_personalities(&root);
            }
        }
        self.load_chatter(ar, |_| true)?;
        self.load_archive_rest(ar, archive)
    }

    /// An archive's motion sets whose file stem ("ms_lf") passes `keep` (character_type.rs
    /// `add_motion_sets`), and each new one's skill toggle overlays by its name without "ms_":
    /// its `<overlay motion-type=44|43 prefix-type script>` (the first archive with the set wins).
    pub(crate) fn load_motion_sets(&mut self, ar: &Archive, keep: impl Fn(&str) -> bool) {
        for stem in self.add_motion_sets(ar, keep) {
            let Some(ms) = self.motion_sets.get(&h(&stem)) else { continue };
            let mut overlays: SkillOverlays = [None, None];
            for o in &ms.overlays {
                let slot = match o.motion_type { OMT_SPECIAL_ON => 0, OMT_SPECIAL_OFF => 1, _ => continue };
                if !o.script.is_empty() {
                    overlays[slot] = Some((o.script.clone(), o.prefix == 1));
                }
            }
            self.skill_overlays.entry(stem.trim_start_matches("ms_").to_string()).or_insert(overlays);
        }
    }

    /// An archive's chatter files (characters/<who>_chatter.xmb) whose `who` passes `keep`:
    /// <h_f74e6a16> blocks of a line_tag and its sound_id alternatives (a file already loaded
    /// from another archive is kept: the levels' copies of the enemies' files).
    pub(crate) fn load_chatter(&mut self, ar: &Archive, keep: impl Fn(&str) -> bool) -> Result<(), String> {
        for n in ar.find("", "_chatter.xmb") {
            let who = n.rsplit('/').next().unwrap_or(&n).trim_end_matches("_chatter.xmb").to_string();
            if self.chatter.contains_key(&who) || !keep(&who) {
                continue;
            }
            let Some(root) = self.parse(ar, &n)? else { continue };
            let mut lines: HashMap<u32, Vec<u32>> = HashMap::new();
            let mut responses: HashMap<u32, u32> = HashMap::new();
            let mut blocks: HashMap<u32, ChatterBlock> = HashMap::new();
            for block in root.walk().into_iter().filter(|e| e.name == 0xF74E_6A16) {
                let all = block.walk();
                let Some(tag) = all.iter().find_map(|e| e.attr(h("line_tag"))).and_then(|v| v.as_hash()) else { continue };
                let ids = all.iter().filter_map(|e| e.attr(h("sound_id")).and_then(|v| v.as_hash()));
                lines.entry(tag).or_default().extend(ids);
                if let Some(r) = all.iter().find_map(|e| e.attr(h("response_id"))).and_then(|v| v.as_hash()) {
                    responses.insert(tag, r);
                }
                // the gate's values: the header beside the line_tag, and each sound_id with its
                // range (a later block with the same tag is dropped, as FUN_001352a0 does)
                let Some(head) = all.iter().find(|e| e.attr(h("line_tag")).is_some()) else { continue };
                let int = |e: &Element, k: u32, or: i64| e.attr(k).and_then(|v| v.as_i64()).unwrap_or(or);
                let variants = all.iter().filter_map(|e| {
                    let id = e.attr(h("sound_id")).and_then(|v| v.as_hash())?;
                    Some((id, int(e, H_CHATTER_MIN, 0), int(e, H_CHATTER_MAX, 100)))
                }).collect();
                blocks.entry(tag).or_insert(ChatterBlock {
                    probability: int(head, h("probability"), 100),
                    replay_delay: head.attr(h("replay-delay")).and_then(|v| v.as_f32()).unwrap_or(0.0),
                    priority: int(head, h("priority"), 5),
                    condition: head.attr(H_CHATTER_CONDITION).and_then(|v| v.as_hash()).unwrap_or(CHATTER_CONDITION_NONE),
                    far_step: int(head, H_CHATTER_FAR_STEP, 1) != 0,
                    far_cut: int(head, H_CHATTER_FAR_CUT, 1) != 0,
                    variants,
                });
            }
            self.chatter.insert(who.clone(), lines);
            self.chatter_response.insert(who.clone(), responses);
            self.chatter_blocks.insert(who, blocks);
        }
        Ok(())
    }

    /// The rest of `load_archive`, after the motion sets, the personalities and the chatter.
    fn load_archive_rest(&mut self, ar: &Archive, archive: &str) -> Result<(), String> {
        // the level list: <level><h_1b8535f0 h_07f2f670=NAME texture-name=PREVIEW ...>
        // <zone file-name=ARCHIVE/><dm-data description=TEXT/>
        if let Some(root) = self.parse(ar, "campaign-bf.xmb")? {
            for l in root.walk().into_iter().filter(|e| e.name == h("level")) {
                let Some(file) = l.child(h("zone")).and_then(|z| z.attr(h("file-name"))).and_then(|v| v.as_str()) else { continue };
                let head = l.children.first();
                let id = |e: Option<&Element>, k: u32| e.and_then(|e| e.attr(k)).and_then(|v| v.as_hash()).unwrap_or(0);
                let movies = l.children_named(h("zone")).map(|z| ZoneMovies {
                    pre: campaign_movies(z, h("pre-movies")),
                    post: campaign_movies(z, h("post-movies")),
                }).collect();
                self.campaign.push(CampaignLevel {
                    id: id(Some(l), H_LEVEL_ID),
                    movies,
                    file: file.to_lowercase(),
                    zones: l.children_named(h("zone")).filter_map(|z| z.attr(h("file-name")).and_then(|v| v.as_str())).map(|f| f.to_lowercase()).collect(),
                    name: id(head, 0x07F2_F670),
                    description: id(l.child(h("dm-data")), h("description")),
                    preview: id(head, h("texture-name")),
                    planet: id(head, 0x1DFF_9E2D),
                    kind: id(head, h("Type")),
                    players: l.child(h("dm-data")).and_then(|d| d.child(0x0398_B8BF)).map_or((0, 0), |p| {
                        let n = |k: &str| p.attr(h(k)).and_then(|v| v.ints().first().copied()).unwrap_or(0);
                        (n("min"), n("max"))
                    }),
                });
            }
            // the missions (#221), and a new campaign's game flags from the file the root names
            // (h_1a468238 "default-gameflags"); a flags file that doesn't decode leaves them
            // empty, not an error
            self.missions = super::campaign::Campaign::from_root(&root);
            let file = format!("{}.xmb", self.missions.flags_file);
            if let Ok(Some(flags)) = self.parse(ar, &file) {
                self.default_flags = super::gameflags::GameFlags::from_root(&flags);
            }
        }
        // the campaign's difficulty values: <difficulty-settings><h_1b8535f0 easy normal hard>
        // (a game options file that doesn't decode leaves them unset, not an error)
        if let Ok(Some(root)) = self.parse(ar, "game-options-en.xmb") {
            let int = |v: &Value| v.as_i64().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok()));
            let values = root.walk().into_iter().filter(|e| e.name == H_DIFFICULTY_SETTINGS).flat_map(|e| e.children.iter())
                .find_map(|e| Some([int(e.attr(h("easy"))?)?, int(e.attr(h("normal"))?)?, int(e.attr(h("hard"))?)?]));
            if values.is_some() {
                self.difficulty_settings = values;
            }
            // the crosshair's lock options: <reticule h_0c3eb4ff h_f079aebd ..> (ReticuleOptions)
            if let Some(r) = root.walk().into_iter().find(|e| e.name == h("reticule")) {
                let num = |k: u32| r.attr(k).and_then(|v| v.as_f32().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok())));
                let d = ReticuleOptions::default();
                self.reticule = ReticuleOptions {
                    lock_share: num(0x0C3E_B4FF).unwrap_or(d.lock_share),
                    box_share: num(0xF079_AEBD).unwrap_or(d.box_share),
                };
            }
            // the skills' tuning: <h_f7bc2603><h_04aa86aa h_1469a0df="0.5"/>.. (CombatSkills)
            let skills = root.walk().into_iter().find(|e| e.name == H_COMBAT_SKILLS);
            if let Some(tex) = skills.and_then(|c| c.child(H_BERSERKER_BLOCK)) {
                let num = |k: u32| tex.attr(k).and_then(|v| v.as_f32().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok())));
                self.combat_skills.berserk_empty_drain = num(H_BERSERK_EMPTY_DRAIN).unwrap_or(self.combat_skills.berserk_empty_drain);
            }
            // Brutus's: <h_f5dc6aac h_eaa78fdf="1" h_f4f602c6="0.85" ..> (VengarSkill); an effect
            // type given as "" is none
            if let Some(b) = skills.and_then(|c| c.child(H_VENGAR_BLOCK)) {
                let num = |k: u32| b.attr(k).and_then(|v| v.as_f32().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok())));
                let effect = |k: u32| b.attr(k).map(|x| x.as_hash().filter(|&x| x != h("")).unwrap_or(0));
                let v = &mut self.combat_skills.vengar;
                v.regen = num(H_VENGAR_REGEN).unwrap_or(v.regen);
                v.damage_taken = num(H_VENGAR_DAMAGE_TAKEN).unwrap_or(v.damage_taken);
                v.charge_cost = num(H_VENGAR_CHARGE_COST).unwrap_or(v.charge_cost);
                v.knock_up = num(H_VENGAR_KNOCK_UP).unwrap_or(v.knock_up);
                v.knock_weight = num(H_VENGAR_KNOCK_WEIGHT).unwrap_or(v.knock_weight);
                v.knock_force = num(H_VENGAR_KNOCK_FORCE).unwrap_or(v.knock_force);
                v.reach = num(H_VENGAR_REACH).unwrap_or(v.reach);
                v.cone = num(H_VENGAR_CONE).unwrap_or(v.cone);
                v.damage = num(H_VENGAR_DAMAGE).unwrap_or(v.damage);
                v.hit_effect = effect(H_VENGAR_HIT_EFFECT).unwrap_or(v.hit_effect);
                v.kill_effect = effect(H_VENGAR_KILL_EFFECT).unwrap_or(v.kill_effect);
            }
            // Hawk's: <h_0ec919f1 h_00894f2a="1" h_f726177e="4" h_1d194665="0.25" ..> (StealthSkill)
            if let Some(b) = skills.and_then(|c| c.child(H_STEALTH_BLOCK)) {
                let num = |k: u32| b.attr(k).and_then(|v| v.as_f32().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok())));
                let s = &mut self.combat_skills.stealth;
                s.second_slot_drain = num(H_STEALTH_SECOND_SLOT).unwrap_or(s.second_slot_drain);
                s.level_recovery = num(H_STEALTH_RECOVERY).unwrap_or(s.level_recovery);
                s.noise_floor = num(H_STEALTH_NOISE).unwrap_or(s.noise_floor);
            }
            // Flint's: <h_0725cdb1 h_108b56f9="1" .. h_f82a974c="1.25"/> (Tracking)
            if let Some(b) = skills.and_then(|c| c.child(H_TRACKING_BLOCK)) {
                let num = |k: u32| b.attr(k).and_then(|v| v.as_f32().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok())));
                let int = |k: u32| b.attr(k).and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok())));
                let t = &mut self.combat_skills.tracking;
                t.weapon_type = int(H_TRACKING_WEAPON_TYPE).unwrap_or(t.weapon_type);
                t.box_shares = [num(H_TRACKING_BOX_X).unwrap_or(t.box_shares[0]), num(H_TRACKING_BOX_Y).unwrap_or(t.box_shares[1])];
                t.double_chance = num(H_DOUBLE_CHANCE).unwrap_or(t.double_chance);
                t.double_factor = num(H_DOUBLE_FACTOR).unwrap_or(t.double_factor);
                t.scope_factor = num(H_SCOPE_FACTOR).unwrap_or(t.scope_factor);
                t.range_weapon_type = int(H_RANGE_WEAPON_TYPE).unwrap_or(t.range_weapon_type);
                t.range_factor = num(H_RANGE_FACTOR).unwrap_or(t.range_factor);
            }
            // the HUD's safe margins: <main h_1d9f07c5="36" h_e72ec57a="48"> (Game::hud_safe)
            if let Some(m) = root.walk().into_iter().find(|e| e.name == h("main") && e.attr(H_HUD_SAFE_X).is_some()) {
                let num = |k: u32| m.attr(k).and_then(|v| v.as_f32().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok())));
                self.hud_safe = [num(H_HUD_SAFE_X).unwrap_or(self.hud_safe[0]), num(H_HUD_SAFE_Y).unwrap_or(self.hud_safe[1])];
            }
        }
        if let Some(root) = self.parse(ar, "string-table-en.xmb")? {
            for e in root.walk() {
                if let (Some(id), Some(text)) = (e.attr(h("iD")).and_then(|v| v.as_hash()), e.attr(h("value")).and_then(|v| v.as_str())) {
                    self.strings.entry(id).or_insert_with(|| text.to_string());
                }
            }
        }
        for n in ar.find("objects-", ".xmb") {
            if let Some(root) = self.parse(ar, &n)? {
                let ivd = Arc::new(ar.get(&n.replace(".xmb", ".ivd")).unwrap_or(&[]).to_vec());
                let ipn = Arc::new(ar.get(&n.replace(".xmb", ".ipn")).unwrap_or(&[]).to_vec());
                self.add_objects(root, ivd, ipn);
            }
        }
        for n in ar.find("", ".ale") {
            if let (true, Some(d)) = (self.effects.effects.is_empty(), ar.get(&n)) {
                self.effects = ale::Library::parse(d);
            }
        }
        for n in ar.find("levels-", ".xmb") {
            if let Some(root) = self.parse(ar, &n)? {
                self.add_placements(&root, archive);
                self.levels.push(root);
            }
        }
        for n in ar.find("materials-", ".xmb") {
            let Some(root) = self.parse(ar, &n)? else { continue };
            self.add_materials(&root, |_| true);
        }
        self.load_textures(ar)?;
        self.load_sounds(ar)?;
        for n in ar.find("animations-", ".xmb") {
            let Some(root) = self.parse(ar, &n)? else { continue };
            let blob = Arc::new(ar.get(&n.replace(".xmb", ".chnl")).unwrap_or(&[]).to_vec());
            self.add_animations(&root, blob, None);
        }
        for n in ar.find("objecttypes", ".xmb") {
            let Some(root) = self.parse(ar, &n)? else { continue };
            self.add_character_types(&root, archive, n.trim_start_matches("objecttypes-").trim_end_matches(".xmb"));
            for w in weapon::parse_weapons(&root, &self.strings) {
                self.weapons.entry(w.name).or_insert(w);
            }
            // object types: <... mesh-name=M ...><base name=T .../> (the mesh-name sits on a
            // <base> or on the type's own element)
            for b in root.walk().into_iter().filter(|e| e.attr(h("mesh-name")).is_some()) {
                let mesh = hash_of(b.attr(h("mesh-name")));
                if let (true, Some(inner)) = (mesh != 0 && mesh != h(""), b.child(h("base"))) {
                    let name = hash_of(inner.attr(h("name")));
                    if name != 0 {
                        self.object_meshes.entry(name).or_insert(mesh);
                    }
                }
            }
            // inventory items: <inventory><h_e275fb80 function-type=.. ..><base mesh-name><base name=T/>
            for t in root.children_named(h("inventory")).flat_map(|l| l.children.iter()) {
                let name = hash_of(t.child(h("base")).and_then(|b| b.child(h("base"))).and_then(|b| b.attr(h("name"))));
                let int = |k: &str| t.attr(h(k)).and_then(|v| v.ints().first().copied()).unwrap_or(0);
                let id = |k: u32| hash_of(t.attr(k)).ne(&h("")).then(|| hash_of(t.attr(k))).unwrap_or(0);
                if name != 0 {
                    self.items.entry(name).or_insert(ItemType {
                        function: int("function-type"),
                        stack_limit: int("stack-limit"),
                        amount: int("pickup-amount"),
                        health: t.attr(0x0A81_1E94).and_then(|v| v.as_f32()).unwrap_or(0.0),
                        stamina: t.attr(0x1188_4F2E).and_then(|v| v.as_f32()).unwrap_or(0.0),
                        kind: int_of(t, 0xE5F5_1266),
                        gives: id(h("pickup-archetype")),
                        sound: id(h("pickup-sound")),
                        use_sound: t.child(h("base")).map(|b| hash_of(b.attr(0x153F_90C9))).filter(|&x| x != 0 && x != h("")).unwrap_or(0),
                        label: self.strings.get(&hash_of(t.attr(h("stringtable-name")))).cloned().unwrap_or_default(),
                        icon: id(0xE5EC_3F1F),
                        message: t.child(h("base")).map(|b| hash_of(b.attr(0x1DD3_A49D))).filter(|&x| x != 0 && x != h("")).unwrap_or(0),
                    });
                }
            }
            // liquid types: <h_fa2f5452><.. liquid-type=K h_06f6a40d=D..><base ..><base name=T/>
            for t in root.children_named(H_LIQUIDS).flat_map(|l| l.children.iter()) {
                let name = hash_of(t.child(h("base")).and_then(|b| b.child(h("base"))).and_then(|b| b.attr(h("name"))));
                let f = |k: u32| t.attr(k).and_then(|v| v.as_f32()).unwrap_or(0.0);
                if name != 0 {
                    let effect = |k: u32| hash_of(t.attr(k)).ne(&h("")).then(|| hash_of(t.attr(k))).unwrap_or(0);
                    let sound = |k: u32| hash_of(t.child(k).and_then(|s| s.attr(h("sound-id"))));
                    self.liquids.entry(name).or_insert(LiquidType {
                        kind: t.attr(h("liquid-type")).and_then(|v| v.ints().first().copied()).unwrap_or(0),
                        damage: [f(0x06F6_A40D), f(0xEBA6_9D33), f(0xE048_9EA0)],
                        effects: [effect(0x08F6_CD94), effect(0xE479_1ABC), effect(0x07AB_4143), effect(0xF458_DACA)],
                        sounds: [sound(0xE259_380A), sound(0x0ED2_1145), sound(0x0C46_DFF8), sound(0xED05_5BCE)],
                    });
                }
            }
            self.add_effect_types(&root);
            // object types: <h_e275fb80><base name=T object-type=K/><combat-target ..>
            // <h_197caf14 debris list> (see `ObjectType`)
            for t in root.walk() {
                let Some(b) = t.child(h("base")).filter(|b| b.attr(h("object-type")).is_some()) else { continue };
                let name = hash_of(b.attr(h("name")));
                if name == 0 || name == h("") || self.object_types.contains_key(&name) {
                    continue;
                }
                let num = |e: &Element, k: u32| e.attr(k).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32)));
                let combat = t.child(h("combat-target"));
                let factors = combat.and_then(|c| c.child(H_DAMAGE_FACTORS)).map(|l| l.children.iter()
                    .filter_map(|f| Some((num(f, h("Type"))? as i64, num(f, H_DAMAGE_FACTOR)?))).collect()).unwrap_or_default();
                let debris = t.child(H_DEBRIS_LIST).map(|l| l.children_named(H_DEBRIS).filter_map(|d| {
                    let kind = hash_of(d.attr(h("archetype-name")));
                    (kind != 0 && kind != h("")).then(|| Debris {
                        kind,
                        main: d.attr(H_DEBRIS_MAIN).and_then(|v| v.as_i64()).unwrap_or(0) != 0,
                        delay: num(d, H_DEBRIS_DELAY).unwrap_or(0.0),
                        life: num(d, H_DEBRIS_LIFE).unwrap_or(0.0),
                    })
                }).collect()).unwrap_or_default();
                self.object_types.insert(name, ObjectType {
                    object_type: b.attr(h("object-type")).and_then(|v| v.as_i64()).unwrap_or(-1),
                    hitpoints: combat.and_then(|c| num(c, h("hitpoints"))).unwrap_or(0.0),
                    factors,
                    debris,
                    radar_contact: t.attr(H_RADAR_CONTACT).and_then(|v| v.as_i64()).map_or(0, |v| v.clamp(0, 255) as u8),
                });
            }
            // idle effects: a type's <events><event state="1" h_ed0c9fac=EFFECT_TYPE/>
            for e in root.walk().into_iter().filter(|e| e.child(h("events")).is_some()) {
                let Some(b) = e.child(h("base")) else { continue };
                let name = hash_of(b.child(h("base")).or(Some(b)).and_then(|b| b.attr(h("name"))));
                let idle = e.child(h("events")).into_iter().flat_map(|ev| ev.children_named(h("event")))
                    .find(|ev| ev.attr(h("state")).and_then(|v| v.as_i64()) == Some(1))
                    .map(|ev| hash_of(ev.attr(0xED0C_9FAC))).filter(|&x| x != 0 && x != h(""));
                if let (true, Some(fx)) = (name != 0, idle) {
                    self.idle_effects.entry(name).or_insert(fx);
                }
            }
            self.add_decals(&root);
            for e in root.walk() {
                let Some(ms) = e.attr(h("motion-archetype")) else { continue };
                let Some(mesh) = e.child(h("base")).map(|b| hash_of(b.attr(h("mesh-name")))) else { continue };
                let name = ms.as_str().map(|s| s.trim_start_matches("ms_").to_string())
                    .unwrap_or_else(|| format!("h_{:08x}", hash_of(Some(ms))));
                // (some levels' copies of a character leave it empty: the first one set wins)
                if let Some(snipe) = e.walk().into_iter().filter_map(|x| x.attr(h("snipe-sound"))).filter_map(|v| v.as_hash())
                    .find(|&x| x != 0 && x != h("")) {
                    self.character_snipe_sound.entry(name.clone()).or_insert(snipe);
                }
                if let Some(x) = e.walk().into_iter().find(|x| x.attr(h("snipe-sound")).is_some_and(|v| v.as_hash().is_some_and(|s| s != 0 && s != h("")))) {
                    let id = |k: u32| x.attr(k).and_then(|v| v.as_hash()).filter(|&s| s != 0 && s != h("")).unwrap_or(0);
                    self.character_scope_sounds.entry(name.clone()).or_insert((id(0x00DD_1FE0), id(0x02C9_4D34)));
                }
                if !self.characters.iter().any(|(n, _)| *n == name) {
                    let footstep = e.attr(0x072A_B4AF).and_then(|v| v.as_i64()).unwrap_or(-1);
                    let jump = e.walk().into_iter().find(|x| x.name == h("movement-sounds"))
                        .and_then(|m| m.attr(h("jump-sound"))).and_then(|v| v.as_hash()).unwrap_or(0);
                    self.character_audio.insert(name.clone(), (footstep, jump));
                    if let Some(hp) = e.walk().into_iter().find(|x| x.name == h("combat-target")).and_then(|c| c.attr(h("hitpoints")))
                        .and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32))) {
                        self.character_hitpoints.insert(name.clone(), hp);
                    }
                    if let Some(c) = e.walk().into_iter().find(|x| x.name == h("combat-target")) {
                        let factors = c.child(H_DAMAGE_FACTORS).map(|l| l.children.iter().filter_map(|f| {
                            let num = |k: u32| f.attr(k).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32)));
                            Some((num(h("Type"))? as i64, num(H_DAMAGE_FACTOR)?))
                        }).collect()).unwrap_or_default();
                        self.character_damage_factors.insert(name.clone(), factors);
                    }
                    if let Some(m) = e.attr(0xE7E9_FCD3).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32))) {
                        self.character_damage_multiplier.insert(name.clone(), m);
                    }
                    let icon = |k: u32| e.attr(k).and_then(|v| v.as_hash()).unwrap_or(0);
                    self.character_icons.insert(name.clone(), (icon(0xF9D9_4425), icon(0x00C5_1907)));
                    let decal = |k: u32| weapon::hashes(e, k).into_iter().find(|&x| x != h("")).unwrap_or(0);
                    self.character_decals.insert(name.clone(), (decal(0xFB4B_FC68), decal(0xEFC2_A874)));
                    if let Some(s) = skill_data(e) {
                        self.character_skills.insert(name.clone(), s);
                    }
                    // <ai ... squad-leash-dist (hash 1d50ffec, recovered from the game's name hash)>
                    if let Some(d) = e.walk().into_iter().find(|x| x.name == h("ai")).and_then(|a| a.attr(0x1D50_FFEC))
                        .and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32))) {
                        self.squad_leash.insert(name.clone(), d);
                    }
                    let inventory = e.walk().into_iter().find(|x| x.name == h("inventory"))
                        .map(|i| weapon::hashes(i, h("weapon-name"))).unwrap_or_default();
                    self.character_weapons.insert(name.clone(), inventory);
                    if let Some(l) = e.walk().into_iter().find(|x| x.name == h("limit-weapon")) {
                        let class = |k: &str| l.attr(h(k)).and_then(|v| v.ints().first().copied()).unwrap_or(-1);
                        self.character_weapon_classes.insert(name.clone(), [class("one"), class("two")]);
                    }
                    if let Some(c) = e.walk().into_iter().find(|x| x.name == H_CAMERA_BLOCK) {
                        let v = |k: u32| c.child(k).and_then(|o| o.text.as_ref()).map(|t| t.floats()).filter(|f| f.len() >= 3)
                            .map_or([0.0; 3], |f| [f[0], f[1], f[2]]);
                        self.character_camera.insert(name.clone(), CameraOffsets {
                            walk: v(h("offset-walk")), run: v(h("offset-run")), dash: v(h("offset-dash")),
                            ready: v(h("offset-ready")), h_e42d50a5: v(H_OFFSET_5), dead: v(H_OFFSET_DEAD),
                            snipe: v(h("offset-snipe")),
                        });
                    }
                    self.characters.push((name, mesh));
                }
            }
        }
        Ok(())
    }

    /// An objecttypes file's effect types (first loaded wins): `effect_types`, their ALE
    /// effects, and `effect_type_defs`, the whole definition.
    pub(crate) fn add_effect_types(&mut self, root: &Element) {
        // effect types: <effect><.. ><base name=T/><alchemy-effects><effect effect-name=E/>..
        for t in root.children_named(h("effect")).flat_map(|l| l.children.iter()) {
            let name = hash_of(t.child(h("base")).and_then(|b| b.attr(h("name"))));
            let list: Vec<u32> = t.child(h("alchemy-effects")).map(|a| a.children_named(h("effect"))
                .map(|e| hash_of(e.attr(h("effect-name")))).filter(|&x| x != 0 && x != h("")).collect()).unwrap_or_default();
            if name != 0 && !list.is_empty() {
                self.effect_types.entry(name).or_insert(list.clone());
            }
            if name != 0 {
                let light = hash_of(t.child(h("light-effect")).and_then(|l| l.attr(h("effect-name"))));
                let sounds = t.child(h("sounds")).map(|s| s.children_named(h("Sound")).filter_map(|e| {
                    let id = hash_of(e.attr(h("sound-id")));
                    let f = |k: u32| e.attr(k).and_then(|v| v.as_f32()).unwrap_or(0.0);
                    let looping = e.attr(h("enable-looping")).and_then(|v| v.as_i64()).unwrap_or(0) != 0;
                    (id != 0 && id != h("")).then_some((id, f(0xF724_CB8C), f(h("play-length")), looping))
                }).collect()).unwrap_or_default();
                let damage = t.child(H_EFFECT_DAMAGE).map(|d| d.children_named(h("Damage")).map(|e| {
                    let f = |k: u32| e.attr(k).and_then(|v| v.as_f32()).unwrap_or(0.0);
                    let i = |k: u32| e.attr(k).and_then(|v| v.as_i64()).unwrap_or(0);
                    AreaDamage { amount: f(h("amount")), damage_type: i(h("Type")), mode: i(0xED58_2B3C), delay: f(0xF724_CB8C),
                                 duration: f(h("duration")), range: f(h("range")), falloff: i(h("falloff")) }
                }).collect()).unwrap_or_default();
                let radius = t.attr(0x1AB0_9530).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32)));
                let centre = t.child(0xE997_50A9).and_then(|c| c.text.as_ref()).map(|x| x.floats()).filter(|f| f.len() >= 3)
                    .map(|f| Vec3::new(f[0], f[1], f[2]));
                let bounds = (radius.is_some() || centre.is_some()).then(|| (centre.unwrap_or(Vec3::ZERO), radius.unwrap_or(0.0).max(0.0)));
                // each listed effect's creation-time (the entry's +4, FUN_00188a90), in the list's
                // order
                let starts: Vec<f32> = t.child(h("alchemy-effects")).map(|a| a.children_named(h("effect"))
                    .filter(|e| { let x = hash_of(e.attr(h("effect-name"))); x != 0 && x != h("") })
                    .map(|e| e.attr(h("creation-time")).and_then(|v| v.as_f32()).unwrap_or(0.0)).collect()).unwrap_or_default();
                self.effect_type_defs.entry(name).or_insert(EffectType {
                    effects: list, starts, light: if light == h("") { 0 } else { light }, sounds, damage, bounds,
                });
            }
        }
    }

    /// An objecttypes file's decal types (first loaded wins): `<decal><.. width height
    /// h_e2ab97b1=TEXTURES h_1c2f0cfb=COLOUR ..><base name=T/>`.
    pub(crate) fn add_decals(&mut self, root: &Element) {
        for d in root.children_named(h("decal")).flat_map(|l| l.children.iter()) {
            let name = hash_of(d.child(h("base")).and_then(|b| b.attr(h("name"))));
            let f = |k: u32| d.attr(k).and_then(|v| v.as_f32()).unwrap_or(0.0);
            let color = d.child(0x1C2F_0CFB).and_then(|c| c.text.as_ref()).map(|t| t.floats()).unwrap_or_default();
            let color = if color.len() >= 4 { [0, 1, 2, 3].map(|i| color[i] / 255.0) } else { [1.0; 4] };
            self.decals.entry(name).or_insert(DecalDef {
                textures: weapon::hashes(d, 0xE2AB_97B1).into_iter().filter(|&x| x != h("")).collect(),
                width: f(h("width")), height: f(h("height")), color,
                life: f(0x1998_70EC), fade: f(0xFE4E_1D82), rotation: f(0xF1C6_C7B5),
            });
        }
    }

    /// An objects file (objects-<lvl>.xmb, its vertex data `ivd` and its collision `ipn`): its
    /// archetypes and meshes, looked up in the order the files were added (`archetype`, `mesh`),
    /// and its collision surfaces' table, `<h_104db1cd data-file=..ipn><h_f3f067ad name off>`
    /// (a surface already loaded wins).
    pub(crate) fn add_objects(&mut self, root: Element, ivd: Arc<Vec<u8>>, ipn: Arc<Vec<u8>>) {
        for table in root.children_named(H_PHYSICS_FILE) {
            for s in table.children_named(H_PHYSICS) {
                let (name, off) = (hash_of(s.attr(h("name"))), s.attr(H_PHYSICS_OFFSET).and_then(|v| v.as_i64()));
                if let (true, Some(off)) = (name != 0 && !ipn.is_empty(), off) {
                    self.physics.entry(name).or_insert((ipn.clone(), off as usize));
                }
            }
        }
        self.objects.push((root, ivd));
    }

    /// A materials file's materials whose name passes `keep`. A skin wrapper (SKIN_WRAPPER with
    /// no textures) takes the textures, type and constants of the next material of the file
    /// that has some, within its wrapper count, keeping its own opacity.
    pub(crate) fn add_materials(&mut self, root: &Element, keep: impl Fn(u32) -> bool) {
        // (name, type, textures param->texture, constants param->values, wrapper count,
        // address modes texture->[u, v])
        type Consts = HashMap<u32, Vec<f32>>;
        let mut seq: Vec<(u32, u32, HashMap<u32, u32>, Consts, usize, HashMap<u32, [Address; 2]>)> = vec![];
        for m in root.walk().into_iter().filter(|e| e.name == h("Material")) {
            let mut tex = HashMap::new();
            let mut consts = HashMap::new();
            let mut address = HashMap::new();
            for e in m.walk() {
                if e.name == h("texture") {
                    let name = hash_of(e.attr(h("texture-name")));
                    tex.insert(hash_of(e.attr(h("param-id"))), name);
                    address.insert(name, [h("address-u"), h("address-v")].map(|k| Address::of(hash_of(e.attr(k)))));
                }
                if e.name == h("constant") {
                    // vector constants are repeated value attributes
                    let v: Vec<f32> = e.attrs.iter().filter(|(k, _)| *k == h("value")).flat_map(|(_, v)| v.floats()).collect();
                    consts.insert(hash_of(e.attr(h("param-id"))), v);
                }
            }
            let count = consts.get(&H_WRAP_COUNT).and_then(|v| v.first().copied()).unwrap_or(1.0) as usize;
            seq.push((hash_of(m.attr(h("name"))), hash_of(m.attr(h("Type"))), tex, consts, count, address));
        }
        for i in 0..seq.len() {
            if !keep(seq[i].0) {
                continue;
            }
            let (name, mut ty, mut tex, mut consts, count, mut address) = seq[i].clone();
            if ty == SKIN_WRAPPER && tex.is_empty() {
                if let Some(next) = seq[i + 1..].iter().take(count.max(1)).find(|s| !s.2.is_empty()) {
                    ty = next.1;
                    tex = next.2.clone();
                    address = next.5.clone();
                    // the wrapper's own opacity (0-255) stays with the material
                    let alpha = consts.get(&H_WRAP_ALPHA).cloned();
                    consts = next.3.clone();
                    if let Some(a) = alpha {
                        consts.insert(H_WRAP_ALPHA, a);
                    }
                }
            }
            // first loaded wins, unless it found no textures and this one did (a skin
            // wrapper is resolved by the material after it, which differs per archive)
            if self.materials.get(&name).is_none_or(|t| t.is_empty() && !tex.is_empty()) {
                self.material_types.insert(name, ty);
                self.materials.insert(name, tex);
                self.material_constants.insert(name, consts);
                self.material_address.insert(name, address);
            }
        }
    }

    /// An animations file's animation sets (`<archetype-set>`: its clips, each a list of bone
    /// targets and their data channels, an event channel, duration, root-height, move speed) and
    /// its data channels (keyed by name, their data in `blob`, the .chnl beside it); the first
    /// loaded of each wins. `sets`: only those animation sets, and only the channels their clips
    /// use (None: everything).
    pub(crate) fn add_animations(&mut self, root: &Element, blob: Arc<Vec<u8>>, sets: Option<&std::collections::HashSet<u32>>) {
        let mut used = std::collections::HashSet::new();
        for e in root.walk().into_iter().filter(|e| e.name == h("archetype-set")) {
            let set_name = hash_of(e.children.first().and_then(|c| c.attr(h("name"))));
            if sets.is_some_and(|s| !s.contains(&set_name)) {
                continue;
            }
            let mut anims = vec![];
            for a in e.children_named(h("archetype")) {
                let head = a.children.first();
                let targets: Vec<(u32, u32)> = a.children_named(h("target")).filter_map(|t| {
                    let p = t.children.first()?;
                    Some((hash_of(p.attr(h("name"))), hash_of(p.attr(h("data-channel")))))
                }).collect();
                let event_channel = hash_of(head.and_then(|p| p.attr(h("event-channel"))));
                used.extend(targets.iter().map(|t| t.1));
                used.insert(event_channel);
                anims.push(AnimDef {
                    name: hash_of(head.and_then(|p| p.attr(h("name")))),
                    duration: head.and_then(|p| p.attr(h("duration"))).and_then(|v| v.as_f32()).unwrap_or(1.0),
                    targets,
                    event_channel,
                    root_height: head.and_then(|p| p.attr(h("root-height"))).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32))).filter(|&r| r > -1000.0),
                    move_speed: head.and_then(|p| p.attr(H_MOVE_SPEED)).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32))),
                });
            }
            self.anim_sets.entry(set_name).or_insert(anims);
        }
        for e in root.walk() {
            let Some(frames) = e.attr(h("num-frames")).and_then(|v| v.as_i64()) else { continue };
            let name = hash_of(e.attr(h("name")));
            if sets.is_some() && !used.contains(&name) {
                continue;
            }
            self.channels.entry(name).or_insert(Channel {
                data_type: hash_of(e.attr(h("data-type"))),
                keyed: hash_of(e.attr(h("frame-type"))) == FT_KEYED,
                interval: e.attr(H_INTERVAL).and_then(|v| v.as_f32()).unwrap_or(1.0 / 30.0),
                frames: frames as usize,
                offset: int_of(e, h("offset")) as usize,
                size: int_of(e, h("size")) as usize,
                blob: blob.clone(),
            });
        }
    }

    fn load_sounds(&mut self, ar: &Archive) -> Result<(), String> {
        for n in ar.find("sounds-", ".xmb") {
            let Some(root) = self.parse(ar, &n)? else { continue };
            let mem = Arc::new(ar.get(&n.replace(".xmb", ".mem")).unwrap_or(&[]).to_vec());
            self.sounds.add(&root, mem);
        }
        Ok(())
    }

    /// Put a multiplayer squad library's skill data and damage-multiplier in place of the
    /// campaign squad's: squad deathmatch plays the Brute Force squad of `library` (game-options
    /// `<squad h_fed0689a="mp_tex mp_brutus mp_hawk mp_flint" library-name="sdm_bf">`), whose
    /// types are in `mp_common/objecttypes-<library>.xmb`, keyed here by motion set like the
    /// campaign's. The skill data (#35 take21: on sdm_e34 Tex drains 6.55/s, sdm_bf's 6.5) and
    /// the damage-multiplier h_e7e9fcd3 (sdm_bf's 1 for all four, the campaign's 0.62: a direct
    /// hit's factor, FUN_00125940) are taken; the rest of that squad (hitpoints, chatter) stays
    /// the campaign's in the demo. How many skill entries it read.
    pub fn load_squad_library(&mut self, data_dir: &Path, library: &str) -> Result<usize, String> {
        let file = format!("objecttypes-{library}.xmb");
        let ar = Archive::open_until(&data_dir.join("mp_common.tgz"), |n| n == file, 1)?;
        let Some(root) = self.parse(&ar, &file)? else { return Ok(0) };
        let mut n = 0;
        for e in root.walk() {
            let Some(ms) = e.attr(h("motion-archetype")) else { continue };
            let name = ms.as_str().map(|s| s.trim_start_matches("ms_").to_string())
                .unwrap_or_else(|| format!("h_{:08x}", hash_of(Some(ms))));
            if let Some(m) = e.attr(0xE7E9_FCD3).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32))) {
                self.character_damage_multiplier.insert(name.clone(), m);
            }
            if let Some(s) = skill_data(e) {
                self.character_skills.insert(name, s);
                n += 1;
            }
        }
        Ok(n)
    }

    /// Load a level's objects (weapon models), materials, weapon definitions, sound bank and
    /// world materials (surface footstep / landing sounds), its characters' motion sets and
    /// chatter (characters/ms_*.xmb, *_chatter.xmb: #128) and its placed characters. Entries
    /// already loaded win.
    pub fn load_level(&mut self, data_dir: &Path, level: &str) -> Result<(), String> {
        let ar = Archive::open(&data_dir.join(format!("{level}.tgz")), |n| {
            ["objects-", "materials-", "objecttypes-", "sounds-", "world-materials", "levels-", "animations-", "ms_"].iter().any(|p| n.starts_with(p))
                || n.ends_with("_chatter.xmb") || n.ends_with(".gscr")
        })?;
        self.load_archive(&ar, level)?;
        // the cutscene scripts' entity names (Lua 4 source: `entity_name = "trigger_spawn8",`),
        // and the scripts themselves, by their names (the file's, without .gscr: #222)
        for f in ar.find("", ".gscr") {
            let text = String::from_utf8_lossy(ar.get(&f).unwrap_or(&[])).into_owned();
            for part in text.split("entity_name").skip(1) {
                let Some(name) = part.split('"').nth(1) else { continue };
                self.script_names.entry(h(name)).or_insert_with(|| name.to_string());
            }
            let stem = f.trim_end_matches(".gscr");
            match super::gscr::Script::parse(stem, &text) {
                Ok(s) => {
                    self.script_names.entry(h(stem)).or_insert_with(|| stem.to_string());
                    self.scripts.insert(h(stem), s);
                }
                Err(e) => eprintln!("{level}: script {f}: {e}"),
            }
        }
        if let Some(root) = self.parse(&ar, "world-materials.xmb")? {
            self.surfaces = audio::surfaces(&root);
        }
        Ok(())
    }

    /// Attach a level's streamed sounds (door and gate sounds, ...): the wave bank in
    /// data/ml-sounds/<lang>/<level>-<lang>.tgz. Returns how many sounds it holds.
    pub fn load_level_streams(&mut self, data_dir: &Path, level: &str, lang: &str) -> Result<usize, String> {
        let path = data_dir.join("ml-sounds").join(lang).join(format!("{level}-{lang}.tgz"));
        let ar = Archive::open(&path, |n| n.ends_with(".xwb"))?;
        let mut n = 0;
        for name in ar.find("", ".xwb") {
            if let Some(d) = ar.get(&name) {
                n += self.sounds.add_stream(Arc::new(d.to_vec()));
            }
        }
        Ok(n)
    }

    /// Attach the streamed files (resource-type 1) of every sound bank of an archive to their own
    /// language wave banks: `sounds-<bank>.xmb` streams from data/ml-sounds/<lang>/<bank>-<lang>.tgz
    /// (mp_common's `sounds-dm` from dm-en.tgz, `sounds-sdm_shrike` from sdm_shrike-en.tgz: the
    /// deathmatch types' voice lines, their death lines among them, #129). Each bank is read again
    /// (what's loaded already wins) for its streamed files, which `SoundBank::add` keeps for the
    /// last bank only. Returns how many files were attached.
    pub fn load_bank_streams(&mut self, data_dir: &Path, archive: &str, lang: &str) -> Result<usize, String> {
        let ar = Archive::open(&data_dir.join(format!("{archive}.tgz")), |n| n.starts_with("sounds-"))?;
        let mut n = 0;
        for name in ar.find("sounds-", ".xmb") {
            let Some(root) = self.parse(&ar, &name)? else { continue };
            let mem = Arc::new(ar.get(&name.replace(".xmb", ".mem")).unwrap_or(&[]).to_vec());
            self.sounds.add(&root, mem);
            let bank = name.rsplit('/').next().unwrap_or(&name).trim_start_matches("sounds-").trim_end_matches(".xmb");
            let path = data_dir.join("ml-sounds").join(lang).join(format!("{bank}-{lang}.tgz"));
            let Ok(waves) = Archive::open(&path, |x| x.ends_with(".xwb")) else { continue };
            for x in waves.find("", ".xwb") {
                if let Some(d) = waves.get(&x) {
                    n += self.sounds.add_stream(Arc::new(d.to_vec()));
                }
            }
        }
        Ok(n)
    }

    /// A level's music (data/sounds/<level>.xwb, which the game streams by level name; sdm_e34
    /// holds one stereo track, caspian_action20) as WAV bytes: (track name, WAV) per entry.
    pub fn load_music(data_dir: &Path, level: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
        Self::load_xwb(&data_dir.join("sounds").join(format!("{level}.xwb")))
    }

    /// Every wave of a wave bank file (e.g. media/Wave.xwb, the menu's sounds), decoded to WAV
    /// files, with their names.
    pub fn load_xwb(path: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
        let d = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(Self::xwb_waves(&d))
    }

    /// The waves of a level's language bank (ml-sounds/<lang>/<level>-<lang>.tgz), decoded to
    /// WAV files with their names: e.g. splash_screen's menu_dub1, the front end's music (its
    /// data/sounds/splash_screen.xwb is an empty stub).
    pub fn load_language_waves(data_dir: &Path, level: &str, lang: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
        let path = data_dir.join("ml-sounds").join(lang).join(format!("{level}-{lang}.tgz"));
        let ar = Archive::open(&path, |n| n.ends_with(".xwb"))?;
        Ok(ar.find("", ".xwb").iter().filter_map(|n| ar.get(n)).flat_map(Self::xwb_waves).collect())
    }

    fn xwb_waves(d: &[u8]) -> Vec<(String, Vec<u8>)> {
        audio::xwb_entries(d).into_iter().filter_map(|(name, fmt, range)| {
            let (tag, channels, rate) = (fmt & 3, (fmt >> 2) & 7, (fmt >> 5) & 0x3ff_ffff);
            let data = d.get(range)?;
            let pcm = match tag {
                1 => audio::adpcm_decode(data, channels as usize),
                0 => data.chunks_exact(2).map(|b| i16::from_le_bytes([b[0], b[1]])).collect(),
                _ => return None,
            };
            Some((name, audio::wav_bytes_channels(rate.max(8000), channels.max(1) as u16, &pcm)))
        }).collect()
    }

    /// The factor character `name` takes damage of `damage_type` by (its combat-target's list,
    /// see `character_damage_factors`; 1 when it lists none for the type).
    pub fn damage_factor(&self, name: &str, damage_type: i64) -> f32 {
        self.character_damage_factors.get(name).and_then(|l| l.iter().find(|f| f.0 == damage_type)).map_or(1.0, |f| f.1)
    }

    /// The type of a placed object that breaks: a game object (object-type 2) whose debris list
    /// (h_197caf14) isn't empty. 467 game types have one; most of them 40 hp.
    pub fn breakable(&self, kind: u32) -> Option<&ObjectType> {
        self.object_types.get(&kind).filter(|t| t.object_type == OBJECT_GAME && !t.debris.is_empty())
    }

    /// An archetype's `<h_e99750a9>` point (its own frame), or for a compound without one its
    /// first part's (the missile rack h_fbdcd828: its root part h_e3860cda's -0.18 1.16 0). Read
    /// as the physics body's centre, where the game places the object: the radiation barrel's
    /// (0 0.435 0) sits in its middle. Not every archetype has one.
    pub fn archetype_centre(&self, archetype: u32) -> Option<Vec3> {
        let (a, _) = self.archetype(archetype)?;
        let read = |e: &Element| e.child(H_ARCHETYPE_CENTRE).and_then(|c| c.text.as_ref()).map(|t| t.floats()).filter(|f| f.len() >= 3)
            .map(|f| Vec3::new(f[0], f[1], f[2]));
        read(a).or_else(|| {
            let first = a.child(0xE487_418A)?.children_named(h("PART")).next()?.attr(h("archetype-name"))?.as_hash()?;
            read(self.archetype(first)?.0)
        })
    }

    /// A collision surface's triangles (corners in its owner's frame, material), see
    /// `bf::collision`. Empty if the loaded levels lack it.
    pub fn collision(&self, name: u32) -> Vec<([Vec3; 3], u8)> {
        self.physics.get(&name).map(|(d, off)| super::collision::surface(d, *off)).unwrap_or_default()
    }

    /// The collision surface an archetype names (`<h_f3f067ad h_051ac63e=N>`), if any.
    pub fn archetype_physics(&self, archetype: u32) -> Option<u32> {
        let (a, _) = self.archetype(archetype)?;
        a.child(H_PHYSICS).map(|p| hash_of(p.attr(H_PHYSICS_NAME))).filter(|&n| n != 0 && n != h("") && self.physics.contains_key(&n))
    }

    /// A float channel's frames (sampled, one f32 per frame), e.g. a door leaf's travel.
    pub fn channel_floats(&self, channel: u32) -> Option<(f32, Vec<f32>)> {
        let c = self.channels.get(&channel)?;
        let d = c.blob.get(c.offset..c.offset + c.size)?;
        let v: Vec<f32> = d.chunks_exact(4).take(c.frames.max(1)).map(|b| f32::from_le_bytes(b.try_into().unwrap())).collect();
        (!c.keyed && !v.is_empty()).then_some((c.interval, v))
    }

    /// Merge another level's sound bank only (sound ids are shared between levels; entries
    /// already loaded win). Missions only carry the squad members they use, so e.g. e01 lacks
    /// Hawk's footsteps; multiplayer levels have all four.
    pub fn load_extra_sounds(&mut self, data_dir: &Path, level: &str) -> Result<(), String> {
        let ar = Archive::open(&data_dir.join(format!("{level}.tgz")), |n| n.starts_with("sounds-"))?;
        self.load_sounds(&ar)
    }

    /// A level's sound bank alone, read with `schemas` (a copy of `Game::schemas`: for a thread
    /// that has no `Game`): its sounds and their Types, e.g. to tell a music bank's track from its
    /// ambience bed (`SoundBank::sound_type`).
    pub fn sound_bank_of(schemas: &SchemaSet, data_dir: &Path, level: &str) -> Result<SoundBank, String> {
        let ar = Archive::open(&data_dir.join(format!("{level}.tgz")), |n| n.starts_with("sounds-"))?;
        let mut bank = SoundBank::default();
        for n in ar.find("sounds-", ".xmb") {
            let Some(d) = ar.get(&n) else { continue };
            let root = bxml::parse(d, schemas).map_err(|e| format!("{n}: {e}"))?;
            bank.add(&root, Arc::new(ar.get(&n.replace(".xmb", ".mem")).unwrap_or(&[]).to_vec()));
        }
        Ok(bank)
    }

    /// A level's level file (levels-<level>.xmb) alone, read with `schemas` (see
    /// `sound_bank_of`); the rest of its archive isn't unpacked. None if it has none.
    pub fn level_file_of(schemas: &SchemaSet, data_dir: &Path, level: &str) -> Result<Option<Element>, String> {
        let ar = Archive::open_until(&data_dir.join(format!("{level}.tgz")), |n| n.starts_with("levels-") && n.ends_with(".xmb"), 1)?;
        let Some(n) = ar.find("levels-", ".xmb").into_iter().next() else { return Ok(None) };
        let Some(d) = ar.get(&n) else { return Ok(None) };
        bxml::parse(d, schemas).map(Some).map_err(|e| format!("{n}: {e}"))
    }

    /// An archive's texture libraries (textures-*.xmb, each with its .tex kept whole): their index
    /// and animated textures; entries already loaded win.
    pub(crate) fn load_textures(&mut self, ar: &Archive) -> Result<(), String> {
        for n in ar.find("textures-", ".xmb") {
            let Some(root) = self.parse(ar, &n)? else { continue };
            let blob = Arc::new(ar.get(&n.replace(".xmb", ".tex")).unwrap_or(&[]).to_vec());
            for t in texture::index(&root) {
                self.textures.entry(t.name).or_insert((t, blob.clone()));
            }
            for a in root.walk().into_iter().filter(|e| e.name == h("animated-texture")) {
                let head = a.children.first();
                let name = hash_of(head.and_then(|p| p.attr(h("name"))));
                let fps = head.and_then(|p| p.attr(0xE637_5A47)).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32))).unwrap_or(15.0);
                let frames = a.child(h("frames")).map(|f| f.children_named(h("frame")).filter_map(|fr| {
                    let p = fr.children.first()?;
                    let g = |n: &str| p.attr(h(n)).and_then(|v| v.as_f32().or(v.as_i64().map(|i| i as f32)));
                    Some([g("u0")?, g("v0")?, g("u1")?, g("v1")?])
                }).collect()).unwrap_or_default();
                let sheet = a.walk().into_iter().find(|e| e.name == h("frame-texture"))
                    .map(|f| hash_of(f.children.first().and_then(|p| p.attr(h("name"))))).unwrap_or(0);
                if sheet != 0 {
                    self.flipbooks.entry(name).or_insert(Flipbook { texture: sheet, fps, frames });
                }
            }
        }
        Ok(())
    }

    /// RGBA8 of a texture; squad skins live in every level archive rather than common, so
    /// missing textures are searched for in level archives (texture files only) on demand.
    /// Load one level archive's textures now (e.g. the tutorial's, for a HUD picture only it
    /// holds), instead of waiting for `texture_rgba` to reach it: the pending archives go in name
    /// order, and the tutorial's is last. Nothing if `name` is already known.
    pub fn load_textures_for(&mut self, data_dir: &Path, level: &str, name: u32) {
        if self.textures.contains_key(&name) {
            return;
        }
        let path = data_dir.join(format!("{level}.tgz"));
        if let Ok(ar) = Archive::open(&path, |n| n.starts_with("textures-")) {
            let _ = self.load_textures(&ar);
            self.pending_texture_archives.retain(|p| *p != path);
        }
    }

    pub fn texture_rgba(&mut self, name: u32) -> Option<(u32, u32, Vec<u8>)> {
        while !self.textures.contains_key(&name) && !self.pending_texture_archives.is_empty() {
            let p = self.pending_texture_archives.remove(0);
            if let Ok(ar) = Archive::open(&p, |n| n.starts_with("textures-")) {
                let _ = self.load_textures(&ar);
            }
        }
        let (t, blob) = self.textures.get(&name)?;
        texture::decode(t, blob).map(|px| (t.w, t.h, px))
    }

    /// A material's colour texture: its "color" parameter, or for shaders without one their
    /// numbered textures: the animated liquid (h_f124a774: sdm_m03's toxic river) shows
    /// "texture-1" over a dark reflection map in "texture-0"; the water (h_0f8904b9) and others
    /// "texture-0"; the self-lit shader (skies) its h_e01baa40.
    pub fn material_texture(&self, material: u32) -> Option<u32> {
        let t = self.materials.get(&material)?;
        let numbered = if self.material_type(material) == LIQUID_SHADER { ["texture-1", "texture-0"] } else { ["texture-0", "texture-1"] };
        let named = |k: u32| t.get(&k).copied().filter(|&x| x != 0 && x != h(""));
        named(h("color")).or_else(|| numbered.iter().find_map(|n| named(h(n)))).or_else(|| named(GLOW_TEXTURE))
    }

    /// The shader type that renders a material (e.g. h_02dca948: alpha-tested foliage, fences).
    pub fn material_type(&self, material: u32) -> u32 {
        self.material_types.get(&material).copied().unwrap_or(0)
    }

    /// Whether the loaded levels define a material at all (some placeholder meshes name one
    /// that no level has: the game draws nothing).
    pub fn has_material(&self, material: u32) -> bool {
        self.materials.contains_key(&material)
    }

    /// A material constant's values (e.g. H_WRAP_ALPHA, or a glow colour h_e01baa40).
    pub fn material_constant(&self, material: u32, param: u32) -> Option<&[f32]> {
        self.material_constants.get(&material)?.get(&param).map(|v| v.as_slice())
    }

    /// A material's alpha (cut-out) texture, if it names one.
    pub fn material_alpha_texture(&self, material: u32) -> Option<u32> {
        self.materials.get(&material)?.get(&h("alpha")).copied().filter(|&t| t != 0 && t != h(""))
    }

    /// A material's texture for a parameter (e.g. a terrain layer's "alpha" mask).
    /// How a material samples one of its textures past the texture's edges (see `Address`).
    pub fn texture_address(&self, material: u32, texture: u32) -> [Address; 2] {
        self.material_address.get(&material).and_then(|a| a.get(&texture)).copied().unwrap_or_default()
    }

    pub fn material_param_texture(&self, material: u32, param: u32) -> Option<u32> {
        self.materials.get(&material)?.get(&param).copied().filter(|&t| t != 0 && t != h(""))
    }

    /// Specular settings of a skin material (BF_CS_rt "Color-Specular"). The parameter names are
    /// hashed; from their values they read as specular colour (h_16e7f952), level (h_e1664805,
    /// 0.2-0.5) and glossiness (h_e7604658, 0.1-1.0). None if the material has no specular.
    pub fn material_specular(&self, material: u32) -> Option<Specular> {
        let c = self.material_constants.get(&material)?;
        let level = *c.get(&0xE166_4805)?.first()?;
        let gloss = c.get(&0xE760_4658).and_then(|v| v.first().copied()).unwrap_or(0.3);
        let tint = c.get(&0x16E7_F952).cloned().unwrap_or_default();
        let tint = match tint.len() {
            0 => [1.0; 3],
            1 | 2 => [tint[0]; 3],
            _ => [tint[0], tint[1], tint[2]],
        };
        Some(Specular { tint, level, gloss })
    }

    /// An animation's events: (time, event name hash), e.g. 0xE2A3FFF9 "drop_weapon". Stored as
    /// f32 time + u32 hash per event; every clip starts with 0xFAEA99C9 and ends with 0x03670002.
    pub fn events(&self, channel: u32) -> Vec<(f32, u32)> {
        let Some(c) = self.channels.get(&channel) else { return vec![] };
        let Some(d) = c.blob.get(c.offset..c.offset + c.size) else { return vec![] };
        d.chunks_exact(8).map(|e| (f32::from_le_bytes(e[..4].try_into().unwrap()), u32::from_le_bytes(e[4..].try_into().unwrap()))).collect()
    }

    pub(crate) fn archetype(&self, name: u32) -> Option<(&Element, &Arc<Vec<u8>>)> {
        self.objects.iter().find_map(|(root, ivd)| {
            root.children_named(h("archetype-library"))
                .flat_map(|lib| lib.children_named(h("archetype")))
                .find(|a| hash_of(a.attr(h("name"))) == name)
                .map(|a| (a, ivd))
        })
    }

    pub(crate) fn mesh(&self, name: u32) -> Option<(&Element, &Arc<Vec<u8>>)> {
        self.objects.iter().find_map(|(root, ivd)| {
            root.children_named(h("mesh-library"))
                .flat_map(|lib| lib.children_named(h("mesh")))
                .find(|m| hash_of(m.attr(h("name"))) == name)
                .map(|m| (m, ivd))
        })
    }
}

pub struct Specular {
    pub tint: [f32; 3],
    pub level: f32,
    pub gloss: f32,
}

pub struct Geoset {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub joints: Vec<[u16; 4]>,
    pub weights: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
    pub material: u32,
}

pub struct Character {
    pub bones: Vec<u32>,
    pub parent: Vec<Option<usize>>,
    /// bind pose: bone -> model, row-vector form (v_model = v_local * rows + pos)
    pub bind_rows: Vec<Mat3>,
    pub bind_pos: Vec<Vec3>,
    pub parent_point: Vec<Vec3>,
    pub child_point: Vec<Vec3>,
    /// inverse bind matrices in column form, exactly as stored (local = R v + t)
    pub inverse_bind: Vec<Mat4>,
    pub geosets: Vec<Geoset>,
    pub anims: Vec<AnimDef>,
    /// '<name>face' set: lip-sync / expression clips for the face bones
    pub face_anims: Vec<AnimDef>,
    pub default_face: Option<usize>,
    /// hardpoints of the bones' part archetypes: name -> (bone, hardpoint in the bone's frame)
    pub hardpoints: HashMap<u32, (usize, Hardpoint)>,
    /// per bone, its part archetype's collision surface (`<h_f3f067ad h_051ac63e>`, in
    /// objects-*.ipn): triangles in the bone's own frame, empty for a part without one. On the
    /// squad's models exactly the eleven bones of the motion set's `<collision>` list have one
    /// (Bip01, Spine2, Head, thighs, calves, upper arms, forearms): the bodies FUN_00114010
    /// makes a character's hit parts of (FUN_00114d00 builds each from the part's node), which
    /// a shot strikes (FUN_00115000 gives the part's bone)
    pub part_collision: Vec<Vec<[Vec3; 3]>>,
    /// its bounding sphere in the model's frame (centre, radius): the union (FUN_002364a0, the
    /// smallest sphere round both) of its skin mesh's geosets' spheres, each `<geoset
    /// h_1ab09530>` radius round its `<h_e99750a9>` centre (the e34 Shrike Heavy's 1.24 m). The
    /// game's object sphere (FUN_00022bc0 -> FUN_00236670) is the union of the model's pieces',
    /// the posed bones' parts' too (not added: the skin's dominates). None without them
    pub bounds: Option<(Vec3, f32)>,
}

/// FUN_002364a0: the smallest sphere round sphere `a` and sphere `b` (centre, radius).
pub fn sphere_union(a: (Vec3, f32), b: (Vec3, f32)) -> (Vec3, f32) {
    let d = a.0.distance(b.0);
    if (b.1 - a.1).abs() < d {
        let k = (b.1 - a.1) / d;
        ((a.0 + b.0 + (b.0 - a.0) * k) * 0.5, (d + a.1 + b.1) * 0.5)
    } else if a.1 < b.1 {
        b
    } else {
        a
    }
}

/// Rows-convention rotation (child rows = L * parent rows) -> quaternion of the column matrix L^T.
fn quat_of_rows(l: Mat3) -> Quat {
    Quat::from_mat3(&l.transpose()).normalize()
}

/// glam Mat3 from row-major values interpreted as row vectors.
fn rows(v: &[f32]) -> Mat3 {
    Mat3::from_cols(Vec3::new(v[0], v[3], v[6]), Vec3::new(v[1], v[4], v[7]), Vec3::new(v[2], v[5], v[8]))
}

impl Character {
    /// A character by its motion set's stem (`Game::characters`: "brutus", the first definition
    /// with that motion set): its mesh, the animation set of its name and the face set
    /// `<name>face`. Right for the squad, whose motion sets name their own sets; the other types
    /// load with `load_type`.
    pub fn load(game: &Game, name: &str, lod: usize) -> Result<Self, String> {
        let &(_, compound) = game.characters.iter().find(|(n, _)| n == name)
            .ok_or_else(|| format!("unknown character {name}"))?;
        Self::load_model(game, compound, h(name), h(&format!("{name}face")), lod)
    }

    /// A character type's model (#128): its mesh (`<base mesh-name>`), the animation set its
    /// motion set names (animation-set-name: the type's +0xcc, FUN_00131a10; ms_feral_colonist
    /// plays brutus, ms_dm_lf lf, ...) and its `<face viseme-file>` set (none for Shrikes and
    /// hounds). For the squad this is what `load` gives (BF_DUMP_TYPES checks it).
    pub fn load_type(game: &Game, t: &CharacterType, lod: usize) -> Result<Self, String> {
        let anims = game.motion_sets.get(&t.motion_set).map_or(0, |m| m.anim_set);
        Self::load_model(game, t.mesh, anims, t.face_set, lod)
    }

    /// A character model: the compound archetype `compound` (its PARTs the skeleton, its LOD
    /// meshes the skin), with the clips of animation set `anim_set` and the face clips of set
    /// `face_set` (0: none).
    fn load_model(game: &Game, compound: u32, anim_set: u32, face_set: u32, lod: usize) -> Result<Self, String> {
        let (arch, _) = game.archetype(compound).ok_or("compound archetype not found")?;
        let all = arch.walk();

        // skeleton
        let parts: Vec<(u32, u32)> = all.iter().filter(|e| e.name == h("PART"))
            .map(|p| (hash_of(p.attr(h("part-name"))), hash_of(p.attr(h("archetype-name"))))).collect();
        let bones: Vec<u32> = parts.iter().map(|p| p.0).collect();
        let index: HashMap<u32, usize> = bones.iter().enumerate().map(|(i, b)| (*b, i)).collect();
        let (mut bind_rows, mut bind_pos, mut inverse_bind) = (vec![], vec![], vec![]);
        let mut hardpoints = HashMap::new();
        let mut part_collision = vec![];
        for (bi, &(_, an)) in parts.iter().enumerate() {
            let (pa, _) = game.archetype(an).ok_or("part archetype not found")?;
            for (k, hp) in weapon::parse_hardpoints(pa) {
                hardpoints.entry(k).or_insert((bi, hp));
            }
            part_collision.push(game.archetype_physics(an).map(|s| game.collision(s).into_iter().map(|(t, _)| t).collect()).unwrap_or_default());
            let v = pa.child(H_INVBIND).and_then(|e| e.text.as_ref()).map(|t| t.floats()).unwrap_or_default();
            if v.len() < 12 {
                return Err("bad part matrix".into());
            }
            // stored: local = R v + t, R row-major (column-vector form)
            let r_col = Mat3::from_cols(Vec3::new(v[0], v[3], v[6]), Vec3::new(v[1], v[4], v[7]), Vec3::new(v[2], v[5], v[8]));
            let t = Vec3::new(v[9], v[10], v[11]);
            inverse_bind.push(Mat4::from_cols(r_col.x_axis.extend(0.0), r_col.y_axis.extend(0.0), r_col.z_axis.extend(0.0), t.extend(1.0)));
            let rr = rows(&v[..9]);                 // the same numbers as a rows matrix
            bind_rows.push(rr);
            bind_pos.push(-row_mul(t, rr));         // bone origin in the model: -t @ R
        }
        let n = bones.len();
        let (mut parent, mut parent_point, mut child_point) = (vec![None; n], vec![Vec3::ZERO; n], vec![Vec3::ZERO; n]);
        for j in all.iter().filter(|e| e.name == H_JOINT) {
            let c = *index.get(&hash_of(j.attr(h("child-part")))).ok_or("joint child")?;
            parent[c] = index.get(&hash_of(j.attr(h("parent-part")))).copied();
            if let Some(d) = j.children.first() {
                let vec3 = |name: &str| d.child(h(name)).and_then(|e| e.text.as_ref()).map(|t| t.floats())
                    .filter(|f| f.len() >= 3).map(|f| Vec3::new(f[0], f[1], f[2])).unwrap_or(Vec3::ZERO);
                parent_point[c] = vec3("parent-point");
                child_point[c] = vec3("child-point");
            }
        }
        // Free face joints (brows, cheeks, nose, lids...) store both points as zero; their
        // offset comes from the bind matrices. Left at zero they collapse onto the head bone
        // whenever no face channel moves them, which caves in the forehead.
        for c in 0..n {
            if let Some(p) = parent[c] {
                if parent_point[c] == Vec3::ZERO && child_point[c] == Vec3::ZERO {
                    parent_point[c] = row_mul(bind_pos[c] - bind_pos[p], bind_rows[p].transpose());
                }
            }
        }

        // mesh (LOD)
        let lods: Vec<u32> = all.iter().filter(|e| e.name == h("mesh"))
            .filter_map(|e| e.attr(h("mesh-name")).and_then(|v| v.as_hash())).collect();
        let lod_name = *lods.get(lod.min(lods.len().saturating_sub(1))).ok_or("no skin mesh")?;
        let (mesh, ivd) = game.mesh(lod_name).ok_or("skin mesh not found")?;
        let palettes: Vec<Vec<usize>> = mesh.children_named(H_PALETTE)
            .map(|p| p.children.first().and_then(|e| e.text.as_ref()).map(|t| t.ints().into_iter().map(|x| x as usize).collect()).unwrap_or_default())
            .collect();
        let mut geosets = vec![];
        let mut rigid = vec![];
        let mut bounds: Option<(Vec3, f32)> = None;
        for g in mesh.walk().into_iter().filter(|e| e.name == h("geoset")) {
            let rec = g.walk().into_iter().find(|e| e.name == H_DRAW).ok_or("geoset without draw record")?;
            let (gs, is_rigid) = read_geoset(rec, ivd, &palettes, hash_of(g.attr(h("material-name"))))?;
            rigid.push(is_rigid);
            geosets.push(gs);
            // its bounding sphere: <geoset h_1ab09530=radius><h_e99750a9>centre</h_e99750a9>
            let radius = g.attr(H_GEOSET_RADIUS).and_then(|v| v.as_f32());
            let centre = g.child(H_GEOSET_CENTRE).and_then(|e| e.text.as_ref()).map(|t| t.floats()).filter(|f| f.len() >= 3)
                .map(|f| Vec3::new(f[0], f[1], f[2]));
            if let (Some(r), Some(c)) = (radius.filter(|&r| r > 0.0), centre) {
                bounds = Some(bounds.map_or((c, r), |b| sphere_union(b, (c, r))));
            }
        }
        attach_rigid(&mut geosets, &rigid);

        let copy = |set: u32| -> Vec<AnimDef> {
            game.anim_sets.get(&set).map(|a| a.iter().map(|d| AnimDef {
                name: d.name, duration: d.duration, targets: d.targets.clone(), event_channel: d.event_channel, root_height: d.root_height,
                move_speed: d.move_speed }).collect()).unwrap_or_default()
        };
        let anims = copy(anim_set);
        let face_anims = if face_set == 0 { vec![] } else { copy(face_set) };
        let default_face = face_anims.iter().position(|a| a.name == NEUTRAL_FACE)
            .or(if face_anims.is_empty() { None } else { Some(0) });
        Ok(Character { bones, parent, bind_rows, bind_pos, parent_point, child_point,
                       inverse_bind, geosets, anims, face_anims, default_face, hardpoints, part_collision, bounds })
    }

    /// Index of the animation whose (hashed) name is `script`, e.g. "Sc_w1_run".
    pub fn anim_by_name(&self, script: &str) -> Option<usize> {
        let target = h(script);
        self.anims.iter().position(|a| a.name == target)
    }

    /// Local transform (rotation, translation) of every bone at bind.
    pub fn bind_local(&self) -> Vec<(Quat, Vec3)> {
        (0..self.bones.len()).map(|i| match self.parent[i] {
            None => (quat_of_rows(self.bind_rows[i]), self.bind_pos[i]),
            Some(p) => {
                let l = self.bind_rows[i] * self.bind_rows[p].transpose();   // rows: child = L * parent
                (quat_of_rows(l), self.parent_point[i] - row_mul(self.child_point[i], l))
            }
        }).collect()
    }

    /// Local transforms at time t of animation `anim`, with face clip `face` (looping on its
    /// own clock `face_t`) layered over the face bones. Bones without a channel keep bind.
    pub fn pose(&self, game: &Game, anim: usize, t: f32, face: Option<usize>, face_t: f32) -> Vec<(Quat, Vec3)> {
        let mut local = self.bind_local();
        let index: HashMap<u32, usize> = self.bones.iter().enumerate().map(|(i, b)| (*b, i)).collect();
        let mut apply = |a: &AnimDef, time: f32, skip_root: bool| {
            for &(bone, ch) in &a.targets {
                let (Some(&bi), Some(c)) = (index.get(&bone), game.channels.get(&ch)) else { continue };
                if skip_root && self.parent[bi].is_none() {
                    continue;
                }
                let (q, v) = sample(c, time);
                if let Some(q) = q {
                    local[bi].0 = q;
                }
                if let Some(v) = v {
                    local[bi].1 = v;
                }
            }
        };
        if let Some(a) = self.anims.get(anim) {
            apply(a, t, false);
        }
        if let Some(f) = face.and_then(|i| self.face_anims.get(i)) {
            // the neutral face is a two-key 0.03 s pose, not an animation: looping it would
            // flip the mouth between its keys ~30 times a second, so hold its first key
            let ft = if f.duration < STATIC_FACE { 0.0 } else { face_t % f.duration.max(1e-6) };
            apply(f, ft, true);
        }
        local
    }

    /// World (model-space) transforms from local ones.
    pub fn world(&self, local: &[(Quat, Vec3)]) -> Vec<Mat4> {
        let mut w: Vec<Mat4> = Vec::with_capacity(local.len());
        for (i, (q, t)) in local.iter().enumerate() {
            let m = Mat4::from_rotation_translation(*q, *t);
            w.push(match self.parent[i] { Some(p) => w[p] * m, None => m });
        }
        w
    }
}

/// Row vector times a rows-convention matrix: v @ R.
fn row_mul(v: Vec3, r: Mat3) -> Vec3 {
    // `r` holds the rows matrix as a normal math matrix, so v @ R == R^T v
    r.transpose() * v
}

pub(crate) fn read_geoset(rec: &Element, ivd: &[u8], palettes: &[Vec<usize>], material: u32) -> Result<(Geoset, bool), String> {
    let cnt = int_of(rec, 0x0FDF_830E) as u64;
    let ioff = int_of(rec, 0xFBB8_81DA) as usize;
    let nv = int_of(rec, 0x0C36_8BC2) as usize;
    let vb = int_of(rec, 0x0CDF_FA25) as usize;
    let pal = int_of(rec, 0x0BDC_6E74);
    // vertex layouts: 32 bytes (position, packed normal, uv, 4 weights, 4 palette slots), the
    // static 24-byte one (h_1fe75072: no skin data), or terrain (h_ef44f398, see TERRAIN_*)
    let vtype = hash_of(rec.attr(h("vertex-type")));
    let terrain = vtype == TERRAIN_VERTEX;
    let stride = if vtype == 0x1FE7_5072 { 24 } else { 32 };
    let max_w = if stride == 24 || terrain { 0 } else { int_of(rec, 0xE80D_AE68) };
    let rd32 = |o: usize| u32::from_le_bytes(ivd[o..o + 4].try_into().unwrap());

    let (is_list, idx): (bool, Vec<u32>) = if cnt >> 16 != 0 {
        // NV2A push buffer: SET_BEGIN_END(prim) ARRAY_ELEMENT16/32 ... SET_BEGIN_END(0)
        let (mut o, mut prim, mut idx) = (ioff, 0u32, vec![]);
        loop {
            let v = rd32(o);
            o += 4;
            let (count, method) = (((v >> 18) & 0x7FF) as usize, v & 0x1FFC);
            let args: Vec<u32> = (0..count).map(|i| rd32(o + 4 * i)).collect();
            o += 4 * count;
            match method {
                0x17FC if args.first() == Some(&0) => break,
                0x17FC => prim = args[0],
                0x1800 => args.iter().for_each(|a| { idx.push(a & 0xFFFF); idx.push(a >> 16); }),
                0x1808 => idx.extend(args),
                _ => {}
            }
        }
        (prim == 5, idx)
    } else {
        let n = (cnt & 0xFFFF) as usize;
        (hash_of(rec.attr(0x1A7E_EC9F)) == LIST_PRIM,
         (0..n).map(|i| u16::from_le_bytes([ivd[ioff + 2 * i], ivd[ioff + 2 * i + 1]]) as u32).collect())
    };
    let mut indices = vec![];
    if is_list {
        indices.extend_from_slice(&idx[..idx.len() / 3 * 3]);
    } else {
        for i in 0..idx.len().saturating_sub(2) {
            let (a, b, c) = (idx[i], idx[i + 1], idx[i + 2]);
            if a != b && b != c && a != c {
                if i % 2 == 0 { indices.extend([a, b, c]) } else { indices.extend([b, a, c]) }
            }
        }
    }

    let palette = palettes.get(pal.max(0) as usize).cloned().unwrap_or_default();
    let mut g = Geoset { positions: vec![], normals: vec![], uvs: vec![], joints: vec![], weights: vec![], indices, material };
    let f32at = |o: usize| f32::from_le_bytes(ivd[o..o + 4].try_into().unwrap());
    let sx = |x: u32, bits: u32| { let x = x as i32; if x >= 1 << (bits - 1) { x - (1 << bits) } else { x } };
    for i in 0..nv {
        let o = vb + i * stride;
        if terrain {
            // SHORT2 a, b: |a| = row * 65 + column on the 2 m grid, signs give x / z, |b| the
            // height in 1/16 m; then a packed normal and the colour layer's uv
            let a = i16::from_le_bytes([ivd[o], ivd[o + 1]]) as i32;
            let b = i16::from_le_bytes([ivd[o + 2], ivd[o + 3]]) as i32;
            let (row, col) = (a.unsigned_abs() / TERRAIN_ROW, a.unsigned_abs() % TERRAIN_ROW);
            let sign = |v: i32| if v >= 0 { 1.0 } else { -1.0 };
            g.positions.push([sign(a) * col as f32 * TERRAIN_CELL, b.unsigned_abs() as f32 * TERRAIN_HEIGHT,
                              sign(b) * row as f32 * TERRAIN_CELL]);
            let p = rd32(o + 4);
            let n = Vec3::new(sx(p & 0x7FF, 11) as f32 / 1023.0, sx((p >> 11) & 0x7FF, 11) as f32 / 1023.0,
                              sx((p >> 22) & 0x3FF, 10) as f32 / 511.0).normalize_or_zero();
            g.normals.push(n.into());
            g.uvs.push([f32at(o + 8), f32at(o + 12)]);
            g.joints.push([0; 4]);
            g.weights.push([1.0, 0.0, 0.0, 0.0]);
            continue;
        }
        g.positions.push([f32at(o), f32at(o + 4), f32at(o + 8)]);
        let p = rd32(o + 12);   // NORMPACKED3: 11/11/10-bit signed
        let n = Vec3::new(sx(p & 0x7FF, 11) as f32 / 1023.0, sx((p >> 11) & 0x7FF, 11) as f32 / 1023.0,
                          sx((p >> 22) & 0x3FF, 10) as f32 / 511.0).normalize_or_zero();
        g.normals.push(n.into());
        g.uvs.push([f32at(o + 16), f32at(o + 20)]);
        let mut j = [0u16; 4];
        let mut w = [0f32; 4];
        if max_w > 0 {
            for k in 0..4 {
                w[k] = ivd[o + 24 + k] as f32 / 255.0;
                let slot = (ivd[o + 28 + k] as i32 - 1) / 5;
                j[k] = if w[k] > 0.0 { palette.get(slot.max(0) as usize).copied().unwrap_or(0) as u16 } else { 0 };
            }
            let s: f32 = w.iter().sum();
            if s > 0.0 { w.iter_mut().for_each(|x| *x /= s) } else { w[0] = 1.0 }
        } else {
            w[0] = 1.0;
        }
        g.joints.push(j);
        g.weights.push(w);
    }
    Ok((g, max_w == 0))
}

/// Rigid geosets (hair, helmets) carry no bone data: attach each to the bone dominating the
/// nearest skinned vertices (majority vote), as char_render.py does.
fn attach_rigid(geosets: &mut [Geoset], rigid: &[bool]) {
    let mut pts: Vec<(Vec3, u16)> = vec![];
    for (g, r) in geosets.iter().zip(rigid) {
        if !r {
            for i in 0..g.positions.len() {
                let k = (0..4).max_by(|&a, &b| g.weights[i][a].total_cmp(&g.weights[i][b])).unwrap();
                pts.push((Vec3::from(g.positions[i]), g.joints[i][k]));
            }
        }
    }
    if pts.is_empty() {
        return;
    }
    for (g, r) in geosets.iter_mut().zip(rigid) {
        if !r {
            continue;
        }
        let mut votes: HashMap<u16, usize> = HashMap::new();
        for p in &g.positions {
            let p = Vec3::from(*p);
            let best = pts.iter().min_by(|a, b| a.0.distance_squared(p).total_cmp(&b.0.distance_squared(p))).unwrap();
            *votes.entry(best.1).or_default() += 1;
        }
        let bone = votes.into_iter().max_by_key(|(_, c)| *c).map(|(b, _)| b).unwrap_or(0);
        g.joints.iter_mut().for_each(|j| *j = [bone, 0, 0, 0]);
        g.weights.iter_mut().for_each(|w| *w = [1.0, 0.0, 0.0, 0.0]);
    }
}

fn quat_at(b: &[u8]) -> Quat {
    let c = |i: usize| i16::from_le_bytes([b[2 * i], b[2 * i + 1]]) as f32 / 32767.0;
    let (x, y, z) = (c(0), c(1), c(2));
    Quat::from_xyzw(x, y, z, (1.0 - x * x - y * y - z * z).max(0.0).sqrt())
}

fn vec_at(b: &[u8]) -> Vec3 {
    let c = |i: usize| f32::from_le_bytes(b[4 * i..4 * i + 4].try_into().unwrap());
    Vec3::new(c(0), c(1), c(2))
}

fn nlerp(a: Quat, b: Quat, f: f32) -> Quat {
    let b = if a.dot(b) < 0.0 { -b } else { b };
    Quat::from_vec4(Vec4::from(a).lerp(Vec4::from(b), f)).normalize()
}

/// Sample a channel: (rotation, translation) where present.
pub fn sample(c: &Channel, t: f32) -> (Option<Quat>, Option<Vec3>) {
    let n = c.frames.max(1);
    let per = c.size / n;
    let raw = &c.blob[c.offset..c.offset + c.size];
    let frame = |k: usize| &raw[k * per..(k + 1) * per];
    let (i, j, f, off) = if c.keyed {
        let times: Vec<f32> = (0..n).map(|k| f32::from_le_bytes(frame(k)[..4].try_into().unwrap())).collect();
        let i = times.partition_point(|&x| x <= t).saturating_sub(1).min(n - 1);
        let j = (i + 1).min(n - 1);
        let f = if i == j { 0.0 } else { ((t - times[i]) / (times[j] - times[i]).max(1e-9)).clamp(0.0, 1.0) };
        (i, j, f, 4)
    } else {
        let x = t / c.interval;
        let i = (x.max(0.0) as usize).min(n - 1);
        (i, (i + 1).min(n - 1), (x - i as f32).clamp(0.0, 1.0), 0)
    };
    let (a, b) = (&frame(i)[off..], &frame(j)[off..]);
    match c.data_type {
        DT_QUAT => (Some(nlerp(quat_at(a), quat_at(b), f)), None),
        DT_VEC => (None, Some(vec_at(a).lerp(vec_at(b), f))),
        DT_XFORM => (Some(nlerp(quat_at(&a[12..]), quat_at(&b[12..]), f)), Some(vec_at(a).lerp(vec_at(b), f))),
        _ => (None, None),
    }
}

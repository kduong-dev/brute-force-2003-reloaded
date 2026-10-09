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
const H_INTERVAL: u32 = 0xF63F_0F49;
/// First clip of every '<name>face' set: closed, relaxed mouth. Human faces are modelled
/// mid-speech, so the game always layers a face clip; this one is the neutral default.
const NEUTRAL_FACE: u32 = 0xEFF8_AC8D;
/// Face clips shorter than this are held poses (the neutral face is two keys, 0.03 s).
const STATIC_FACE: f32 = 0.1;

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
const H_PHYSICS_FILE: u32 = 0x104D_B1CD;
const H_PHYSICS: u32 = 0xF3F0_67AD;
const H_PHYSICS_OFFSET: u32 = 0xE9E4_4859;
pub const H_PHYSICS_NAME: u32 = 0x051A_C63E;

fn hash_of(v: Option<&Value>) -> u32 {
    v.and_then(|v| v.as_hash()).unwrap_or(0)
}
fn int_of(e: &Element, name: u32) -> i64 {
    e.attr(name).and_then(|v| v.as_i64()).unwrap_or(0)
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

pub struct AnimDef {
    pub name: u32,
    pub duration: f32,
    /// (bone name hash, channel name hash)
    pub targets: Vec<(u32, u32)>,
    /// timed events (footsteps, drop_weapon, ...), see `Game::events`
    pub event_channel: u32,
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
}

/// An object type's `<base object-type>`: 0 a compound of loose pieces (archetype-type 9, the
/// debris that flies apart), 2 a game object (scenery, archetype-type 7), 17 an effect object
/// (archetype-type 6: effects, a light, sounds and `<Damage>` areas).
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
    pub name: u32,
    pub description: u32,
    pub preview: u32,
    /// the planet's name (string id, h_1dff9e2d)
    pub planet: u32,
    /// the kind of level (Type): LEVEL_DEATHMATCH, LEVEL_SQUAD_DEATHMATCH or a mission
    pub kind: u32,
    /// recommended players (dm-data h_0398b8bf min, max; 0 0 on the squad deathmatch maps)
    pub players: (i64, i64),
}

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
    /// the light effect (an ALE effect name hash), 0 if none
    pub light: u32,
    /// sounds: (sound id, h_f724cb8c, play-length s, enable-looping). h_f724cb8c is read as a
    /// delay in seconds (an inference: 0 on most, 0.2 on the Light grenade's ignition)
    pub sounds: Vec<(u32, f32, f32, bool)>,
    /// `<h_fb0a5f1d><Damage ..>`: the damage areas it sets up where it's started (the
    /// exploding scenery's effects, see `AreaDamage`); empty on most
    pub damage: Vec<AreaDamage>,
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
        let mut g = Game {
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
            decals: HashMap::new(),
            object_meshes: HashMap::new(),
            liquids: HashMap::new(),
            items: HashMap::new(),
            levels: vec![],
            flipbooks: HashMap::new(),
            campaign: vec![],
            physics: HashMap::new(),
            effects: ale::Library::default(),
            effect_types: HashMap::new(),
            effect_type_defs: HashMap::new(),
            idle_effects: HashMap::new(),
            object_types: HashMap::new(),
        };
        g.load_archive(&common)?;
        let mut others: Vec<PathBuf> = std::fs::read_dir(data_dir)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "tgz") && !p.ends_with("common.tgz"))
            .collect();
        others.sort();
        g.pending_texture_archives = others;
        Ok(g)
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

    fn load_archive(&mut self, ar: &Archive) -> Result<(), String> {
        // chatter: <h_f74e6a16> blocks of a line_tag and its sound_id alternatives
        for n in ar.find("", "_chatter.xmb") {
            let Some(root) = self.parse(ar, &n)? else { continue };
            let who = n.rsplit('/').next().unwrap_or(&n).trim_end_matches("_chatter.xmb").to_string();
            let mut lines: HashMap<u32, Vec<u32>> = HashMap::new();
            let mut responses: HashMap<u32, u32> = HashMap::new();
            for block in root.walk().into_iter().filter(|e| e.name == 0xF74E_6A16) {
                let all = block.walk();
                let Some(tag) = all.iter().find_map(|e| e.attr(h("line_tag"))).and_then(|v| v.as_hash()) else { continue };
                let ids = all.iter().filter_map(|e| e.attr(h("sound_id")).and_then(|v| v.as_hash()));
                lines.entry(tag).or_default().extend(ids);
                if let Some(r) = all.iter().find_map(|e| e.attr(h("response_id"))).and_then(|v| v.as_hash()) {
                    responses.insert(tag, r);
                }
            }
            self.chatter.insert(who.clone(), lines);
            self.chatter_response.insert(who, responses);
        }
        // the level list: <level><h_1b8535f0 h_07f2f670=NAME texture-name=PREVIEW ...>
        // <zone file-name=ARCHIVE/><dm-data description=TEXT/>
        if let Some(root) = self.parse(ar, "campaign-bf.xmb")? {
            for l in root.walk().into_iter().filter(|e| e.name == h("level")) {
                let Some(file) = l.child(h("zone")).and_then(|z| z.attr(h("file-name"))).and_then(|v| v.as_str()) else { continue };
                let head = l.children.first();
                let id = |e: Option<&Element>, k: u32| e.and_then(|e| e.attr(k)).and_then(|v| v.as_hash()).unwrap_or(0);
                self.campaign.push(CampaignLevel {
                    file: file.to_lowercase(),
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
                let ivd = ar.get(&n.replace(".xmb", ".ivd")).unwrap_or(&[]).to_vec();
                // the collision surfaces' table: <h_104db1cd data-file=..ipn><h_f3f067ad name off>
                let ipn = Arc::new(ar.get(&n.replace(".xmb", ".ipn")).unwrap_or(&[]).to_vec());
                for table in root.children_named(H_PHYSICS_FILE) {
                    for s in table.children_named(H_PHYSICS) {
                        let (name, off) = (hash_of(s.attr(h("name"))), s.attr(H_PHYSICS_OFFSET).and_then(|v| v.as_i64()));
                        if let (true, Some(off)) = (name != 0 && !ipn.is_empty(), off) {
                            self.physics.entry(name).or_insert((ipn.clone(), off as usize));
                        }
                    }
                }
                self.objects.push((root, Arc::new(ivd)));
            }
        }
        for n in ar.find("", ".ale") {
            if let (true, Some(d)) = (self.effects.effects.is_empty(), ar.get(&n)) {
                self.effects = ale::Library::parse(d);
            }
        }
        for n in ar.find("levels-", ".xmb") {
            if let Some(root) = self.parse(ar, &n)? {
                self.levels.push(root);
            }
        }
        for n in ar.find("materials-", ".xmb") {
            let Some(root) = self.parse(ar, &n)? else { continue };
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
        self.load_textures(ar)?;
        self.load_sounds(ar)?;
        for n in ar.find("animations-", ".xmb") {
            let Some(root) = self.parse(ar, &n)? else { continue };
            let blob = Arc::new(ar.get(&n.replace(".xmb", ".chnl")).unwrap_or(&[]).to_vec());
            for e in root.walk() {
                if let Some(frames) = e.attr(h("num-frames")).and_then(|v| v.as_i64()) {
                    let name = hash_of(e.attr(h("name")));
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
                if e.name == h("archetype-set") {
                    let set_name = hash_of(e.children.first().and_then(|c| c.attr(h("name"))));
                    let mut anims = vec![];
                    for a in e.children_named(h("archetype")) {
                        let head = a.children.first();
                        let targets = a.children_named(h("target")).filter_map(|t| {
                            let p = t.children.first()?;
                            Some((hash_of(p.attr(h("name"))), hash_of(p.attr(h("data-channel")))))
                        }).collect();
                        anims.push(AnimDef {
                            name: hash_of(head.and_then(|p| p.attr(h("name")))),
                            duration: head.and_then(|p| p.attr(h("duration"))).and_then(|v| v.as_f32()).unwrap_or(1.0),
                            targets,
                            event_channel: hash_of(head.and_then(|p| p.attr(h("event-channel")))),
                        });
                    }
                    self.anim_sets.entry(set_name).or_insert(anims);
                }
            }
        }
        for n in ar.find("objecttypes", ".xmb") {
            let Some(root) = self.parse(ar, &n)? else { continue };
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
                    self.effect_type_defs.entry(name).or_insert(EffectType {
                        effects: list, light: if light == h("") { 0 } else { light }, sounds, damage,
                    });
                }
            }
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
                    let icon = |k: u32| e.attr(k).and_then(|v| v.as_hash()).unwrap_or(0);
                    self.character_icons.insert(name.clone(), (icon(0xF9D9_4425), icon(0x00C5_1907)));
                    let decal = |k: u32| weapon::hashes(e, k).into_iter().find(|&x| x != h("")).unwrap_or(0);
                    self.character_decals.insert(name.clone(), (decal(0xFB4B_FC68), decal(0xEFC2_A874)));
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

    fn load_sounds(&mut self, ar: &Archive) -> Result<(), String> {
        for n in ar.find("sounds-", ".xmb") {
            let Some(root) = self.parse(ar, &n)? else { continue };
            let mem = Arc::new(ar.get(&n.replace(".xmb", ".mem")).unwrap_or(&[]).to_vec());
            self.sounds.add(&root, mem);
        }
        Ok(())
    }

    /// Load a level's sound bank and world materials (surface footstep / landing sounds).
    /// Load a level's objects (weapon models), materials, weapon definitions, sound bank and
    /// world materials (surface footstep / landing sounds). Entries already loaded win.
    pub fn load_level(&mut self, data_dir: &Path, level: &str) -> Result<(), String> {
        let ar = Archive::open(&data_dir.join(format!("{level}.tgz")), |n| {
            ["objects-", "materials-", "objecttypes-", "sounds-", "world-materials", "levels-", "animations-"].iter().any(|p| n.starts_with(p))
        })?;
        self.load_archive(&ar)?;
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

    fn load_textures(&mut self, ar: &Archive) -> Result<(), String> {
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
    pub fn load(game: &Game, name: &str, lod: usize) -> Result<Self, String> {
        let &(_, compound) = game.characters.iter().find(|(n, _)| n == name)
            .ok_or_else(|| format!("unknown character {name}"))?;
        let (arch, _) = game.archetype(compound).ok_or("compound archetype not found")?;
        let all = arch.walk();

        // skeleton
        let parts: Vec<(u32, u32)> = all.iter().filter(|e| e.name == h("PART"))
            .map(|p| (hash_of(p.attr(h("part-name"))), hash_of(p.attr(h("archetype-name"))))).collect();
        let bones: Vec<u32> = parts.iter().map(|p| p.0).collect();
        let index: HashMap<u32, usize> = bones.iter().enumerate().map(|(i, b)| (*b, i)).collect();
        let (mut bind_rows, mut bind_pos, mut inverse_bind) = (vec![], vec![], vec![]);
        let mut hardpoints = HashMap::new();
        for (bi, &(_, an)) in parts.iter().enumerate() {
            let (pa, _) = game.archetype(an).ok_or("part archetype not found")?;
            for (k, hp) in weapon::parse_hardpoints(pa) {
                hardpoints.entry(k).or_insert((bi, hp));
            }
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
        for g in mesh.walk().into_iter().filter(|e| e.name == h("geoset")) {
            let rec = g.walk().into_iter().find(|e| e.name == H_DRAW).ok_or("geoset without draw record")?;
            let (gs, is_rigid) = read_geoset(rec, ivd, &palettes, hash_of(g.attr(h("material-name"))))?;
            rigid.push(is_rigid);
            geosets.push(gs);
        }
        attach_rigid(&mut geosets, &rigid);

        let copy = |set: u32| -> Vec<AnimDef> {
            game.anim_sets.get(&set).map(|a| a.iter().map(|d| AnimDef {
                name: d.name, duration: d.duration, targets: d.targets.clone(), event_channel: d.event_channel }).collect()).unwrap_or_default()
        };
        let anims = copy(h(name));
        let face_anims = copy(h(&format!("{name}face")));
        let default_face = face_anims.iter().position(|a| a.name == NEUTRAL_FACE)
            .or(if face_anims.is_empty() { None } else { Some(0) });
        Ok(Character { bones, parent, bind_rows, bind_pos, parent_point, child_point,
                       inverse_bind, geosets, anims, face_anims, default_face, hardpoints })
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

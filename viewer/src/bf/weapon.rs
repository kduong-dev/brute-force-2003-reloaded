//! Weapons: definitions from objecttypes-<level>.xmb, models from objects-<level>.xmb, and the
//! hardpoints that join a weapon to a character's hand.
//!
//!   <h_e275fb80 weapon-type=.. h_e93a46d6=RATE h_e33f08b8="FIRE SOUNDS" ammo-count=..>  weapon
//!     <base ... stringtable-name=LABEL> <base mesh-name=ARCHETYPE> <base name=WEAPON object-type=20/>
//!     <bullet h_113d3e3c=SPEED range=.. effect-name=FLIGHT h_08d28037=HIT> ...
//!     <h_f738f302 h_e41cb542=MUZZLE HARDPOINT/>
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
const H_MUZZLE_REF: u32 = 0xF738_F302;
const H_MUZZLE_REF_NAME: u32 = 0xE41C_B542;
/// item base: HUD icon texture (+ its size, h_e2bc2c81 x h_f71189bb, 128 x 64 for guns)
const H_HUD_ICON: u32 = 0xE5EC_3F1F;
const H_RELOAD_SOUND: u32 = 0x0CBF_5B9C;
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
/// object base: the effects attached to the object (8 slots of `<h_081398d6 effect-name
/// hardpoint-name>`); a grenade's first is its trail (the Frag's h_10a5508f: grenade_trail and
/// the hiss 10318b29)
const H_ATTACHED_EFFECTS: u32 = 0x18B6_AB72;
/// Damage: h_04ea9251, seconds the damage is dealt over (the Gas's 5.5; 0 for the rest - an
/// inference from the Gas recording's ~5.7 s of steady damage)
const H_DAMAGE_TIME: u32 = 0x04EA_9251;
const H_PARTS: u32 = 0xE487_418A;
const H_MESHES: u32 = 0xEA78_3362;
const H_JOINT: u32 = 0x151A_CE78;

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
    pub muzzle: u32,
    /// weapon handling sound (picked up / drawn)
    pub pickup_sound: u32,
    /// HUD icon texture (0 if none)
    pub icon: u32,
    /// seconds (0 in most definitions: the game times reloads by animation)
    pub reload_time: f32,
    pub reload_sound: u32,
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
    /// items: h_1ee2f4ed, how it's used (see H_USE_TYPE)
    pub use_type: i64,
    /// items: how many fit in the inventory (stack-limit: 10 for the squad's grenades)
    pub stack_limit: i64,
    /// the effect type attached to the object (h_18b6ab72's first slot; 0 if none)
    pub attached_effect: u32,
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
            muzzle: all.iter().find(|e| e.name == H_MUZZLE_REF).and_then(|e| e.attr(H_MUZZLE_REF_NAME))
                .and_then(|v| v.as_hash()).unwrap_or(MUZZLE),
            pickup_sound: find(h("pickup-sound")).and_then(|v| v.as_hash()).unwrap_or(0),
            icon: find(H_HUD_ICON).and_then(|v| v.as_hash()).unwrap_or(0),
            reload_time: f32_of(Some(&w), h("reload-time"), 0.0),
            reload_sound: hashes(w, H_RELOAD_SOUND).first().copied().unwrap_or(0),
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
            use_type: find(H_USE_TYPE).and_then(|v| v.ints().first().copied()).unwrap_or(0),
            stack_limit: find(h("stack-limit")).and_then(|v| v.ints().first().copied()).unwrap_or(0),
            attached_effect: all.iter().find(|e| e.name == H_ATTACHED_EFFECTS)
                .and_then(|a| a.children.first()).and_then(|e| e.attr(h("effect-name"))).and_then(|v| v.as_hash())
                .filter(|&x| x != h("")).unwrap_or(0),
        });
    }
    out
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
}

/// Joint type of a sliding (prismatic) joint: door and gate leaves.
const SLIDING_JOINT: u32 = 0xFD8F_670C;

pub struct WeaponModel {
    pub parts: Vec<WeaponPart>,
    pub hardpoints: HashMap<u32, Hardpoint>,
}

impl WeaponModel {
    pub fn load(game: &Game, archetype: u32) -> Result<Self, String> {
        let (arch, _) = game.archetype(archetype).ok_or_else(|| format!("weapon archetype h_{archetype:08x} not found"))?;
        let mut hardpoints = parse_hardpoints(arch);
        let mut parts = vec![];
        let part_list: Vec<(u32, u32)> = arch.child(H_PARTS).map(|p| p.children_named(h("PART"))
            .map(|e| (e.attr(h("part-name")).map(part_hash).unwrap_or(0), e.attr(h("archetype-name")).and_then(|v| v.as_hash()).unwrap_or(0)))
            .collect()).unwrap_or_default();
        if part_list.is_empty() {
            parts.push(WeaponPart { name: 0, physics: game.archetype_physics(archetype), geosets: meshes(game, arch)?, offset: Vec3::ZERO, rotation: Quat::IDENTITY, spin_axis: None, slide_axis: None });
        } else {
            for (part, part_arch) in part_list {
                let Some((pa, _)) = game.archetype(part_arch) else { continue };
                for (k, v) in parse_hardpoints(pa) {
                    hardpoints.entry(k).or_insert(v);
                }
                // joint to the parent (compounds here are one level deep: root + children)
                let joint = arch.walk().into_iter().filter(|e| e.name == H_JOINT)
                    .find(|j| j.attr(h("child-part")).map(part_hash) == Some(part));
                let (mut offset, mut rotation, mut spin_axis, mut slide_axis) = (Vec3::ZERO, Quat::IDENTITY, None, None);
                if let Some(d) = joint.and_then(|j| j.children.first()) {
                    let v = |n: &str| d.child(h(n)).and_then(|e| e.text.as_ref()).map(|t| t.floats())
                        .filter(|f| f.len() >= 3).map(|f| Vec3::new(f[0], f[1], f[2])).unwrap_or(Vec3::ZERO);
                    // the part sits at parent-point + child-point, both in the parent's frame (a
                    // building set lays its pieces out by the child-point: a hangar's roof bays
                    // 5 m apart, its bunks in pairs), and the joint's ORIENTATION (w x y z; e.g. a
                    // wall set's mirrored gate leaves) turns the part about its own origin. Read
                    // as parent-point - ORIENTATION * child-point, Bulgar's compound h_0baf091a
                    // came out jumbled: roof bays outside the building, bunks through the walls
                    // (issue #12). Each of its 19 pieces lies inside the building only this way.
                    if let Some(q) = d.child(h("ORIENTATION")).and_then(|e| e.text.as_ref()).map(|t| t.floats()).filter(|q| q.len() >= 4) {
                        rotation = Quat::from_xyzw(q[1], q[2], q[3], q[0]).normalize();
                    }
                    offset = v("parent-point") + v("child-point");
                    let axis = v("axis");
                    if axis.length_squared() > 0.5 {
                        if d.attr(h("Type")).and_then(|t| t.as_hash()) == Some(SLIDING_JOINT) {
                            slide_axis = Some(rotation * axis.normalize());
                        } else {
                            spin_axis = Some(axis.normalize());
                        }
                    }
                }
                parts.push(WeaponPart { name: part, physics: game.archetype_physics(part_arch), geosets: meshes(game, pa)?, offset, rotation, spin_axis, slide_axis });
            }
        }
        Ok(WeaponModel { parts, hardpoints })
    }
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

/// Geosets of a mesh (objects `<mesh name=...>`), e.g. a level's terrain or sky.
pub fn mesh_geosets(game: &Game, best: u32) -> Result<Vec<Geoset>, String> {
    let (mesh, ivd) = game.mesh(best).ok_or_else(|| format!("mesh h_{best:08x} not found"))?;
    let mut out = vec![];
    for g in mesh.walk().into_iter().filter(|e| e.name == h("geoset")) {
        let material = g.attr(h("material-name")).and_then(|v| v.as_hash()).unwrap_or(0);
        // one or more draw records; records over the same vertex buffer (terrain patches) are
        // merged: each uses the buffer's first `n` vertices
        let mut merged: Option<(i64, Geoset)> = None;
        for rec in g.walk().into_iter().filter(|e| e.name == 0xFDE5_0E9F) {
            let vb = rec.attr(0x0CDF_FA25).and_then(|v| v.as_i64()).unwrap_or(-1);
            let mut part = read_geoset(rec, ivd, &[], material)?.0;
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

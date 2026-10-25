//! Pickups from spawn-triggers (#278; viewer/docs/pickups.md, "Pickups from spawn-triggers").
//! Every pickup the multiplayer maps place (sdm_e34's 54, mp1's 29 ...) and some of the
//! campaign's (e34's supply drops, m02_a's caches) is a spawn-trigger's template: out of the
//! world (FUN_000dfe90; level_scene.rs keeps its model hidden, `SpawnTemplate`), and each spawn
//! of its trigger (FUN_002099e0 -> FUN_00209620) puts a copy of it at the trigger's transform
//! (FUN_000dfd90's clone placed by its vtable +0x50, the game object's place 0x159b70). The
//! triggers (play_triggers.rs `spawn_copy`) ask for each copy here, and `make_copies` makes it
//! from the template's model in the same frame:
//!  - a type without a collision body (its `collision-type` CT_NONE: the power-ups) stays where
//!    the trigger puts it, with its idle effect: sdm_e34's STAMINA POWER at (-84.07, 29.78,
//!    60.58), where todo/35-special-skills take20's live object lies, 2 m from its placement.
//!    Taking a power-up is #15's;
//!  - any other item (CT_CONVEX: a body the world's physics drops) lies on the ground under the
//!    trigger as the level's own pickups do (play_pickups.rs `LatePickup`): medkits, fruit and DNA
//!    canisters can be taken;
//!  - a weapon lies on its side on the ground under the trigger (`lying`): todo/129-shrikes
//!    take38 has mp1's walkway guns flat on the ground under their spawn glows. It's a lying gun
//!    with its type's full clip (play_weapon_pickups.rs `LyingGun`), offered by #17's prompt and
//!    taken by the swap, its clip by the walk-over.
//!
//! A copy goes by the name its trigger gives it (a re-armable trigger's copies get made-up ones,
//! as the game gives them none): taking it sends PICKUP_OBJECT from that name (FUN_0011c170's
//! walk-over, FUN_00117660's grab_weapon), and it's gone for good (`SpawnCopy`). What comes back
//! is the data's: the trigger's OPERATED goes through a subject relay (play_triggers.rs `relay`)
//! that makes a timer listen to the copy, the PICKUP_OBJECT starts the timer, and its pulse
//! spawns the next copy and, beside most, the spawn flash's effect (a second spawn-trigger whose
//! template is an effect object). sdm_e34's STAMINA POWER's timer h_0021e47f runs 30 s (take20:
//! back ~31 s after it was taken), its medkits' 10 or 45 s, its guns' 20 or 45 s.
//!
//! BF_PICKUP_LOG=1 (or BF_TRIGGER_LOG / BF_WEAPON_LOG) prints each copy made.

use super::*;
use bf_viewer::ale_fx::{AleAssets, AleEffect};
use bf_viewer::bf::character::CT_NONE;
use bf_viewer::bf::hash::h;
use bf_viewer::level_scene::{LevelMaterial, Placed, SpawnTemplate};
use bevy::pbr::NotShadowCaster;

/// How far above a trigger the ground under its copy is looked for (m): the floor below this,
/// so one a little under the trigger's point still counts (the multiplayer maps' pickup triggers
/// stand ~0.5 m over the ground; the demo's choice).
const GROUND_ABOVE: f32 = 0.5;

/// A pickup copy a spawn-trigger made this frame (play_triggers.rs `spawn_copy`, FUN_00209620):
/// its template (the placement h_1299668b names), its own name (the trigger's h_14f15a34, or a
/// made-up one), the template's element (inventory-object, weapon-object) and object type, the
/// trigger's transform, and the trigger's name for the log.
pub struct PickupCopy {
    pub template: u32,
    pub copy: u32,
    pub tag: u32,
    pub kind: u32,
    pub at: Mat4,
    pub by: String,
}

/// The pickup copies the triggers asked for and `make_copies` hasn't made yet.
#[derive(Resource, Default)]
pub struct PickupCopies(pub Vec<PickupCopy>);

/// A spawn-trigger's copy of a pickup (see the module notes): taken, it's removed from the world
/// for good (FUN_0011c170 removes an item it takes; FUN_00117660's grab_weapon the gun,
/// FUN_000db3e0), where the level's own pickups come back after play_pickups.rs's guessed
/// RESPAWN: the trigger's own chain makes its next copy.
#[derive(Component)]
pub struct SpawnCopy;

/// The copies are made after the triggers' frame (play_triggers.rs `run`, before `doors`) and
/// before the characters move, so the pickups and lying guns have them in the same frame.
pub fn plugin(app: &mut App) {
    app.init_resource::<PickupCopies>()
        .add_systems(OnEnter(AppState::Playing), (|mut commands: Commands| commands.insert_resource(PickupCopies::default())).after(setup))
        .add_systems(Update, make_copies.after(doors).before(update_player).run_if(in_state(AppState::Playing)));
}

/// A weapon copy's turn (see the module notes): the trigger's `rotation` turned onto the gun's
/// side, as play_testtools.rs lays a gun down (a quarter turn about its length: modelled as held,
/// upright, it would stand on its grip).
fn on_side(rotation: Quat) -> Quat {
    rotation * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)
}

/// The pose a weapon copy rests in: the trigger's transform `at` turned onto its side (`on_side`),
/// its lowest point (`low`, below its origin so turned) on the ground `ground` under the trigger.
/// The drop's physics isn't read (FUN_0014a1d0 -> FUN_00236a50): this is where take38's guns
/// lie, not how they get there.
fn lying(at: Transform, low: f32, ground: f32) -> Transform {
    Transform::from_translation(Vec3::new(at.translation.x, ground - low, at.translation.z)).with_rotation(on_side(at.rotation))
}

/// Makes the copies the triggers asked for this frame (see the module notes): the template's
/// hidden model (`SpawnTemplate`'s parts, the level's materials) under a new root named as the
/// copy, then what its kind needs.
#[allow(clippy::too_many_arguments)]
fn make_copies(mut commands: Commands, mut copies: ResMut<PickupCopies>, mut game: ResMut<GameData>,
               templates: Query<(Entity, &Placed), With<SpawnTemplate>>, children: Query<&Children>,
               parts: Query<(&Mesh3d, &Transform, &MeshMaterial3d<LevelMaterial>)>, meshes: Res<Assets<Mesh>>,
               (mut ale, mut images, mut materials): (Option<ResMut<AleAssets>>, ResMut<Assets<Image>>, ResMut<Assets<StandardMaterial>>)) {
    if copies.0.is_empty() {
        return;
    }
    let log = ["BF_PICKUP_LOG", "BF_TRIGGER_LOG", "BF_WEAPON_LOG"].iter().any(|v| std::env::var(v).is_ok());
    for c in std::mem::take(&mut copies.0) {
        // the template's model: its parts as level_scene.rs placed them under its root
        let template = templates.iter().find(|(_, p)| p.name == c.template).map(|(e, _)| e);
        let model: Vec<(Handle<Mesh>, Transform, Handle<LevelMaterial>)> = template.and_then(|e| children.get(e).ok()).into_iter()
            .flat_map(|kids| kids.iter()).filter_map(|k| parts.get(k).ok()).map(|(m, t, mat)| (m.0.clone(), *t, mat.0.clone())).collect();
        let at = Transform::from_matrix(c.at);
        let fixed = game.0.object_types.get(&c.kind).is_some_and(|t| t.collision == CT_NONE);
        let weapon = c.tag == h("weapon-object");
        let place = if weapon && !fixed {
            let side = on_side(at.rotation);
            let low = model.iter().filter_map(|(m, t, _)| Some((meshes.get(m)?, t)))
                .filter_map(|(m, t)| match m.attribute(Mesh::ATTRIBUTE_POSITION) {
                    Some(bevy::render::mesh::VertexAttributeValues::Float32x3(v)) => Some(v.iter().map(|p| (side * t.transform_point(Vec3::from(*p))).y)
                        .fold(f32::MAX, f32::min)),
                    _ => None,
                }).fold(f32::MAX, f32::min);
            let ground = floor_y(at.translation.x, at.translation.z, at.translation.y + GROUND_ABOVE);
            lying(at, if low.is_finite() { low } else { 0.0 }, ground)
        } else {
            at
        };
        let root = commands.spawn((place, Visibility::default(), Name::new(format!("copy h_{:08x} of h_{:08x}", c.copy, c.template)),
                                   Placed { tag: c.tag, kind: c.kind, name: c.copy }, SpawnCopy)).id();
        for (mesh, t, material) in &model {
            commands.spawn((Mesh3d(mesh.clone()), *t, NotShadowCaster, MeshMaterial3d(material.clone()), ChildOf(root)));
        }
        let what = if weapon {
            let def = game.0.weapons.get(&c.kind);
            let clip = def.map_or(0, |d| d.ammo);
            commands.entity(root).insert(weapon_pickups::LyingGun { kind: c.kind, clip, name: c.copy });
            format!("a lying {} ({clip} rounds)", def.map_or_else(|| format!("h_{:08x}", c.kind), |d| d.label.clone()))
        } else if fixed {
            // its idle effect (a power-up's icon: `spawn_idle_effects` starts the placed ones')
            let list = game.0.idle_effects.get(&c.kind).and_then(|t| game.0.effect_types.get(t)).cloned().unwrap_or_default();
            let mut n = 0;
            if let Some(ale) = ale.as_deref_mut() {
                for (k, effect) in list.into_iter().enumerate() {
                    let Some(fx) = ale.load(&mut game.0, &mut images, &mut materials, effect) else { continue };
                    let seed = 0x9E37_79B9 ^ c.copy.wrapping_mul(0x85EB_CA6B) ^ (k as u32);
                    commands.spawn((Transform::IDENTITY, Visibility::default(), AleEffect::new(fx, 0.0, seed), Name::new(format!("idle effect h_{effect:08x}")),
                                    ChildOf(root)));
                    n += 1;
                }
            }
            format!("where it's put (no collision body), {n} idle effects")
        } else {
            commands.entity(root).insert(pickups::LatePickup);
            "on the ground under it".to_string()
        };
        if log {
            println!("{}: pickup copy h_{:08x} of h_{:08x} (type h_{:08x}, {} parts) at {:.2}: {what}", c.by, c.copy, c.template, c.kind, model.len(),
                     place.translation);
        }
        if template.is_none() && log {
            println!("{}: no model for the template h_{:08x} (it has no archetype)", c.by, c.template);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A weapon copy lies on its side, its lowest point on the ground under the trigger, turned
    /// the trigger's way about the vertical (take38's guns flat on the walkway).
    #[test]
    fn a_weapon_copy_lies_on_its_side() {
        let at = Transform::from_xyz(-87.0, 29.51, 9.48).with_rotation(Quat::from_rotation_y(0.7));
        let t = lying(at, -0.04, 29.0);
        assert!((t.translation.y - 29.04).abs() < 1e-5);
        assert_eq!((t.translation.x, t.translation.z), (-87.0, 9.48));
        // its up axis lies flat, its length (z) stays as the trigger turns it
        assert!((t.rotation * Vec3::Y).y.abs() < 1e-5);
        assert!((t.rotation * Vec3::Z - at.rotation * Vec3::Z).length() < 1e-5);
    }
}

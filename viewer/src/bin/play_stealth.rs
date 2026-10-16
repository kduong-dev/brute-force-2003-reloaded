//! Hawk's Stealth beyond her own character (#123): how her body looks as it fades. Her skill's
//! own part (the toggle, her invisibility controller, her visibility, the costs of a shot and an
//! item) is play_skills.rs's `Stealth`; who sees her, play.rs's `nearest_hostile` (the AI) and
//! play_reticle.rs (the crosshair). Sources: the spec and takes in `todo/35-special-skills/`
//! (spec.md Part 4: takes 10-14, 18, 19) and the functions cited (viewer/docs/code-map.md,
//! "Skills and stamina"); the page is viewer/docs/skills.md.
//!
//!  - **The game**: her model carries an invisibility material controller (kind 2, `Cloak`;
//!    FUN_00210060 gives the type "hawk" one). While it's active the renderer draws her in the
//!    see-through list (FUN_000e7850: the active controllers' kinds & 0xc2, or their body alpha
//!    under 1, FUN_00210a20), at the controller's body alpha (+0x14), and its material
//!    BF_InvisMaterial (class 0xf3bf04d1, set up at 0xa1a40 / 0xa1ba0) over her: a copy of the
//!    picture (stage 0), alpha-blended (0x302 / 0x303) through her silhouette with an offset that
//!    wobbles (FUN_00212430: 0.15 x her visibility + 0.1 x sin(2.5 t), and v x sin(5 t)), tinted
//!    (0.9, 1, 1) at the material's alpha (+0x50). Fading out, the material's alpha rises as x^2
//!    over the first half (x the share of the 1.0 s), the body's falls as (1 - x)^2 over the
//!    second; fading in, the reverse.
//!  - **Here**: her body (the model and the guns it carries, not the unlit muzzle flash) is
//!    drawn at `skills::opacity`, 1 less the controller's level: a fit to the takes, where her
//!    presence on screen falls and rises with the level (take11, see `skills::opacity`) and she's
//!    gone from a level of about 0.95. As a whole: each part below 1 is blended over the picture
//!    (AlphaMode::Blend, with a copy of its texture whose alpha is 255: the squad's skins keep a
//!    specular mask in theirs, scene.rs `material`), and a depth-only copy of each part (`Depth`)
//!    puts her nearest surface in the depth buffer first, so only that surface is blended, not
//!    the arms and the gun behind her back. The alpha is `srgb_alpha`'s, which gives the console's
//!    blend in sRGB values where bevy blends in linear light. The refraction isn't drawn: fully
//!    stealthed she's invisible here, where the game's picture keeps a faint ripple of the ground
//!    in her outline (take11 4.5-7.3 s, hardly seen at 3-10 m; a shot, which shows her to the AI,
//!    only strengthens it, by its offset's 0.15 x v). The close-up shot H1 and the material's
//!    shaders are its follow-up.
//!  - **Warm-up**: bevy compiles a pipeline the first time something is drawn with it and skips
//!    the draw until it's ready, so her first fade would lose her for a few frames. As her model
//!    first shows, each part gets an invisible blended copy for WARM_UP s (alpha 0) and its depth
//!    copy is drawn meanwhile (the depth her own body writes anyway): the pipelines are ready
//!    before she ever fades.
//!  - Her fading parts and their depth copies are drawn after every other see-through thing
//!    (FADE_SORT), so the copies take nothing from what lies behind her; one between her and the
//!    camera (smoke, a liquid's surface) is drawn under her during a fade, not over: the demo's,
//!    for the second each fade lasts.
//!  - A part she takes up after her model shows (a grenade in her hand, whose material its type
//!    shares with every such grenade) is faded with a copy of its material on its own entity
//!    (`Faded::foreign`), its own put back once it leaves her (thrown) or she's whole.
//!  - The Vengar view draws everyone as a heat figure (play_vengar.rs `heat_figures`), a
//!    stealthed Hawk too: the game draws them with every controller but kind 0x100 on
//!    (FUN_000e7d40's passes 9 and 10, the mask 0xfffffeff), so hers would apply there; what that
//!    looks like isn't filmed.
//!
//! Test hooks: play_skills.rs's (`BF_TEST_SKILL`, `BF_TEST_STAMINA`, `BF_SKILL_LOG`, which also
//! prints each body going see-through and back, and its copies made).

use super::*;
use std::collections::HashSet;
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline, MaterialPlugin, NotShadowCaster},
    render::{
        mesh::{skinning::SkinnedMesh, MeshVertexBufferLayoutRef},
        render_resource::{AsBindGroup, ColorWrites, RenderPipelineDescriptor, SpecializedMeshPipelineError},
        view::NoFrustumCulling,
    },
};

/// How long (s) a body's blended warm-up copies are drawn once its model shows (see the module's
/// notes): long enough for bevy to build the pipelines on its worker threads (#122's heat
/// figures took two frames at 60 fps).
const WARM_UP: f32 = 1.0;
/// The power that turns the share of her shown (the console's blend in sRGB values) into bevy's
/// alpha in linear light, for a dark body over a lighter picture: `srgb_alpha`. The HUD's fades
/// use the same 2.2 the other way round (play_reticle.rs `FADE_SRGB`: a light overlay on dark).
const SRGB_POWER: f32 = 2.2;
/// How much nearer (m) her see-through parts sort among the see-through things than they are
/// (bevy sorts those back to front by their origin's depth in the view plus the material's
/// `depth_bias`): past every other one, the decals' 40 (play_fx.rs), the liquids' and the DNA
/// chip's 1000 (level_scene.rs, play_dna.rs), so her depth copies, drawn just before her, take
/// nothing from what lies behind her: drawn after them, a scorch on the ground lost its part in
/// her outline. The cost, the chip's: a see-through thing between her and the camera (smoke, a
/// liquid's surface) is drawn under her during a fade, not over. Bevy also hands `depth_bias` to
/// the pipeline as a constant depth bias: at this value it moves her fragments' depth by about
/// 2e-4 of itself toward the camera (under a millimetre at 3 m), past nothing of her own (her
/// surfaces are centimetres apart); her depth copies set theirs back to 0 (`Depth`), so her
/// nearest surface meets theirs.
const FADE_SORT: f32 = 2000.0;
/// How far (m) the depth copies sort behind her own parts: all of them are drawn before any of
/// hers, whose origins (the root at her feet, the guns at her hands and back) lie within this of
/// each other.
const DEPTH_AHEAD: f32 = 2.0;

/// A part's depth-only copy: bevy's standard material, drawn among the see-through things
/// (AlphaMode::Blend, sorted by FADE_SORT less DEPTH_AHEAD) with a pipeline that writes depth and
/// no colour.
type DepthMaterial = ExtendedMaterial<StandardMaterial, Depth>;

/// `DepthMaterial`'s extension: nothing of its own but its pipeline (see `specialize`).
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
struct Depth {}

impl MaterialExtension for Depth {
    /// No colour written, the depth written (a blended pipeline writes none), and no depth bias
    /// (bevy makes the base's `depth_bias`, here the sort's, the pipeline's too): the copy only
    /// puts its own depth in the depth buffer, the same her part's fragments then meet.
    fn specialize(_pipeline: &MaterialExtensionPipeline, descriptor: &mut RenderPipelineDescriptor, _layout: &MeshVertexBufferLayoutRef,
                  _key: MaterialExtensionKey<Self>) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = descriptor.fragment.as_mut() {
            for target in fragment.targets.iter_mut().flatten() {
                target.write_mask = ColorWrites::empty();
            }
        }
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            depth.depth_write_enabled = true;
            depth.bias.constant = 0;
        }
        Ok(())
    }
}

/// Marks a part's depth-only copy, and whose (the model's root).
#[derive(Component)]
struct DepthCopy {
    root: Entity,
}

/// Marks a part's blended warm-up copy, and how long it's left (s).
#[derive(Component)]
struct WarmCopy {
    left: f32,
}

/// What a fade changed of an own part's material, to put back once the body is whole: the
/// character's (the model's root), and the material's texture, alpha mode and sort bias.
struct Kept {
    root: Entity,
    texture: Option<Handle<Image>>,
    mode: AlphaMode,
    bias: f32,
}

/// The fade's bookkeeping, emptied as a map starts (its characters are new):
///  - `own`: the materials of the parts the copies were made for, the character's own (each
///    spawned with its model, scene.rs `spawn_model` / `spawn_static`), and `made`: those a fade
///    has made see-through (`Kept`);
///  - `foreign`: the parts taken up since that a fade has given a see-through copy of their
///    material instead (a grenade in her hand: its type's material is every such grenade's,
///    play_grenade.rs `GrenadeKit::spawn_model`), by entity, with the character, the part's own
///    material and the copy; their own is put back once the body is whole or they've left it
///    (the grenade thrown, which flies seen: #35 take13 7.0-7.6 s);
///  - the opaque copies of the textures, by the original's id; the bodies whose copies are made,
///    and how long their warm-up has left (s); the one depth-only material.
#[derive(Resource, Default)]
struct Faded {
    own: HashSet<AssetId<StandardMaterial>>,
    made: HashMap<AssetId<StandardMaterial>, Kept>,
    foreign: HashMap<Entity, (Entity, Handle<StandardMaterial>, Handle<StandardMaterial>)>,
    textures: HashMap<AssetId<Image>, Handle<Image>>,
    prepared: HashMap<Entity, f32>,
    depth: Option<Handle<DepthMaterial>>,
}

/// The depth-only material, and the fade after the skills have stepped (so a switch shows in its
/// frame).
pub fn plugin(app: &mut App) {
    app.add_plugins(MaterialPlugin::<DepthMaterial>::default())
        .init_resource::<Faded>()
        .add_systems(OnEnter(AppState::Playing), |mut commands: Commands| commands.insert_resource(Faded::default()))
        .add_systems(Update, (prepare_bodies, fade_bodies).chain().after(skills::update_skills).run_if(in_state(AppState::Playing)));
}

/// The share of her shown, `p` (the console blends in sRGB values: the picture's value x (1 - p)
/// + hers x p), as bevy's alpha (it blends in linear light): for a dark body over a lighter
/// picture the same result needs 1 - (1 - p)^2.2. Measured with play_stealth's own captures
/// against take11's (skills.md § Hawk's Stealth).
fn srgb_alpha(p: f32) -> f32 {
    1.0 - (1.0 - p.clamp(0.0, 1.0)).powf(SRGB_POWER)
}

/// Material `m` made see-through at `alpha`: its texture's opaque copy (`opaque_copy`), blended,
/// sorted after every other see-through thing (FADE_SORT).
fn see_through(m: &mut StandardMaterial, images: &mut Assets<Image>, textures: &mut HashMap<AssetId<Image>, Handle<Image>>, alpha: f32) {
    if let Some(copy) = m.base_color_texture.as_ref().and_then(|t| opaque_copy(images, textures, t)) {
        m.base_color_texture = Some(copy);
    }
    m.alpha_mode = AlphaMode::Blend;
    m.depth_bias = FADE_SORT;
    m.base_color.set_alpha(alpha);
}

/// The copies of a cloaking character's parts (`skills::cloaks`), made once its model shows: a
/// depth-only copy of each (`DepthCopy`, hidden but in the warm-up and the fades) and a blended
/// warm-up copy (`WarmCopy`, alpha 0), each beside its part with the same mesh and skin. The
/// warm-up copies go when their time is out.
#[allow(clippy::too_many_arguments)]
fn prepare_bodies(mut commands: Commands, time: Res<Time>, player: Res<Player>, squad: Res<Squad>, children: Query<&Children>,
                  parts: Query<(&MeshMaterial3d<StandardMaterial>, &Mesh3d, &Transform, &ChildOf, Option<&SkinnedMesh>), (Without<WarmCopy>, Without<DepthCopy>)>,
                  mut warm: Query<(Entity, &mut WarmCopy)>, mut materials: ResMut<Assets<StandardMaterial>>, mut depths: ResMut<Assets<DepthMaterial>>,
                  mut images: ResMut<Assets<Image>>, mut faded: ResMut<Faded>) {
    let dt = frame_dt(&time);
    for (e, mut w) in &mut warm {
        w.left -= dt;
        if w.left <= 0.0 {
            commands.entity(e).despawn();
        }
    }
    for left in faded.prepared.values_mut() {
        *left = (*left - dt).max(0.0);
    }
    let log = std::env::var("BF_SKILL_LOG").is_ok();
    for u in std::iter::once(&*player).chain(squad.0.iter()) {
        let Some(l) = &u.loaded else { continue };
        if !skills::cloaks(u) || faded.prepared.contains_key(&l.root) {
            continue;
        }
        let depth = match &faded.depth {
            Some(d) => d.clone(),
            None => {
                let d = depths.add(DepthMaterial { base: StandardMaterial { alpha_mode: AlphaMode::Blend, depth_bias: FADE_SORT - DEPTH_AHEAD, ..default() },
                                                   extension: Depth {} });
                faded.depth = Some(d.clone());
                d
            }
        };
        let mut made = 0;
        for e in children.iter_descendants(l.root) {
            let Ok((part, mesh, tr, parent, skin)) = parts.get(e) else { continue };
            let Some(own) = materials.get(&part.0).filter(|m| !m.unlit).cloned() else { continue };
            // (the warm-up copy: the material as a fade leaves it, at alpha 0)
            let mut blend = own.clone();
            see_through(&mut blend, &mut images, &mut faded.textures, 0.0);
            let blend = materials.add(blend);
            faded.own.insert(part.0.id());
            for warm_copy in [false, true] {
                let mut c = commands.spawn((mesh.clone(), *tr, Visibility::Inherited, NoFrustumCulling));
                if let Some(s) = skin {
                    c.insert(s.clone());
                }
                if warm_copy {
                    // (where it's drawn doesn't matter, so not under her: the Vengar view's heat
                    // figures, play_vengar.rs, take every standard material there. Casting its
                    // shadow, as the faded part does: that pipeline too; at alpha 0 the shadow
                    // pass discards it all. The map's end removes it with the rest it spawned)
                    c.insert((MeshMaterial3d(blend.clone()), WarmCopy { left: WARM_UP }));
                } else {
                    c.insert((MeshMaterial3d(depth.clone()), DepthCopy { root: l.root }, NotShadowCaster, ChildOf(parent.parent())));
                }
            }
            made += 1;
        }
        // (its parts come the frame after its spawn: wait for them)
        if made > 0 {
            faded.prepared.insert(l.root, WARM_UP);
            if log {
                println!("t={:.2} {}: depth and warm-up copies of {made} parts", u.sim_time, unit_name(u));
            }
        }
    }
}

/// Each character's body at its opacity (see the module's notes). An own part not yet
/// see-through gets its texture's opaque copy and the blend, then the alpha; a part taken up
/// since, a see-through copy of its material (`Faded::foreign`). Once the body is whole again
/// every own part made see-through on it is put back (by its material, so a part the Vengar view
/// has swapped meanwhile comes back as it was too), and every foreign part gets its own material
/// back, as does one that has left the body. Its depth copies are drawn while it's partly there,
/// and in its warm-up.
#[allow(clippy::too_many_arguments)]
fn fade_bodies(mut commands: Commands, player: Res<Player>, squad: Res<Squad>, children: Query<&Children>, parents: Query<&ChildOf>,
               parts: Query<&MeshMaterial3d<StandardMaterial>, (Without<WarmCopy>, Without<DepthCopy>)>,
               mut copies: Query<(&DepthCopy, &mut Visibility)>, mut materials: ResMut<Assets<StandardMaterial>>,
               mut images: ResMut<Assets<Image>>, mut faded: ResMut<Faded>) {
    let log = std::env::var("BF_SKILL_LOG").is_ok();
    let Faded { own, made, foreign, textures, prepared, .. } = &mut *faded;
    // which bodies' depth copies are drawn this frame, and which are partly or wholly gone
    let mut depth_on: Vec<Entity> = prepared.iter().filter(|(_, &left)| left > 0.0).map(|(&root, _)| root).collect();
    let mut fading: Vec<Entity> = vec![];
    for u in std::iter::once(&*player).chain(squad.0.iter()) {
        let Some(l) = &u.loaded else { continue };
        let shown = skills::opacity(u);
        if shown >= 1.0 {
            let back: Vec<AssetId<StandardMaterial>> = made.iter().filter(|(_, k)| k.root == l.root).map(|(id, _)| *id).collect();
            for id in &back {
                let Some(k) = made.remove(id) else { continue };
                if let Some(m) = materials.get_mut(*id) {
                    m.base_color_texture = k.texture;
                    m.alpha_mode = k.mode;
                    m.depth_bias = k.bias;
                    m.base_color.set_alpha(1.0);
                }
            }
            if log && !back.is_empty() {
                println!("t={:.2} {}: body whole again ({} materials)", u.sim_time, unit_name(u), back.len());
            }
            continue;
        }
        fading.push(l.root);
        if shown > 0.0 {
            depth_on.push(l.root);
        }
        let alpha = srgb_alpha(shown);
        let mut new = 0;
        for e in children.iter_descendants(l.root) {
            let Ok(part) = parts.get(e) else { continue };
            // a foreign part's copy: at this alpha
            if let Some((_, _, copy)) = foreign.get(&e) {
                if materials.get(copy).is_some_and(|m| (m.base_color.alpha() - alpha).abs() >= 1e-4) {
                    if let Some(m) = materials.get_mut(copy) {
                        m.base_color.set_alpha(alpha);
                    }
                }
                continue;
            }
            let id = part.0.id();
            let known = made.contains_key(&id);
            // (the muzzle flash, unlit, stays; a part at this alpha already is left alone, so an
            // unchanged one isn't sent to the GPU again)
            match materials.get(id) {
                Some(m) if !known && m.unlit => continue,
                Some(m) if known && (m.base_color.alpha() - alpha).abs() < 1e-4 => continue,
                None => continue,
                _ => {}
            }
            if !own.contains(&id) {
                // taken up since: a copy of its own on this entity
                let Some(mut copy) = materials.get(id).cloned() else { continue };
                see_through(&mut copy, &mut images, textures, alpha);
                let copy = materials.add(copy);
                commands.entity(e).insert(MeshMaterial3d(copy.clone()));
                foreign.insert(e, (l.root, part.0.clone(), copy));
                new += 1;
                continue;
            }
            let Some(m) = materials.get_mut(id) else { continue };
            if known {
                m.base_color.set_alpha(alpha);
            } else {
                made.insert(id, Kept { root: l.root, texture: m.base_color_texture.clone(), mode: m.alpha_mode, bias: m.depth_bias });
                see_through(m, &mut images, textures, alpha);
                new += 1;
            }
        }
        if log && new > 0 {
            println!("t={:.2} {}: body see-through ({new} materials), shown {shown:.2}, alpha {alpha:.2}", u.sim_time, unit_name(u));
        }
    }
    // foreign parts back to their own once their body is whole (or gone) or they've left it
    let done: Vec<Entity> = foreign.iter().filter(|(&e, (root, _, _))| !fading.contains(root) || !descends(e, *root, &parents))
        .map(|(&e, _)| e).collect();
    for e in done {
        let Some((_, own_material, _)) = foreign.remove(&e) else { continue };
        if let Ok(mut part) = commands.get_entity(e) {
            part.insert(MeshMaterial3d(own_material));
        }
        if log {
            println!("foreign part {e:?}: its own material back");
        }
    }
    for (copy, mut vis) in &mut copies {
        let want = if depth_on.contains(&copy.root) { Visibility::Inherited } else { Visibility::Hidden };
        if *vis != want {
            *vis = want;
        }
    }
}

/// Whether `e` is under `root` in the hierarchy.
fn descends(e: Entity, root: Entity, parents: &Query<&ChildOf>) -> bool {
    let mut at = e;
    while let Ok(p) = parents.get(at) {
        at = p.parent();
        if at == root {
            return true;
        }
    }
    false
}

/// The copy of texture `t` with every alpha at 255 (the skin's alpha is a specular mask, not its
/// cover: blended, the material's alpha alone fades it), made once per texture.
fn opaque_copy(images: &mut Assets<Image>, copies: &mut HashMap<AssetId<Image>, Handle<Image>>, t: &Handle<Image>) -> Option<Handle<Image>> {
    if let Some(c) = copies.get(&t.id()) {
        return Some(c.clone());
    }
    let mut image = images.get(t)?.clone();
    // (the skins are Rgba8UnormSrgb: four bytes a texel, the alpha last)
    for texel in image.data.as_mut()?.chunks_exact_mut(4) {
        texel[3] = 255;
    }
    let copy = images.add(image);
    copies.insert(t.id(), copy.clone());
    Some(copy)
}

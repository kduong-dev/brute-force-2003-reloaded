//! Hawk's Stealth beyond her own character (#123, #198): how her body looks as it fades and
//! while she's stealthed; and the follow camera's fade of the squad's bodies (#126,
//! play_camera.rs `body_alpha`: the controlled character see-through with the camera close, a
//! squadmate between the camera and it by its depth), drawn the same way. Her skill's own part
//! (the toggle, her material controllers, her visibility, the costs of a shot and an item) is
//! play_skills.rs's `Stealth`; who sees her, play_brain.rs's `Sensor` (the AI) and
//! play_reticle.rs (the crosshair). Sources: the spec and takes in `todo/35-special-skills/`
//! (spec.md Part 4: takes 10-14, 18, 19), the leads and takes in `todo/198-hawk-invis/` and the
//! functions cited (viewer/docs/code-map.md, "Skills and stamina"); the page is
//! viewer/docs/skills.md.
//!
//! The game draws her through two of her model's material controllers (play_skills.rs):
//!  - **Her skin** (kind 1, `Skin`): her model with its own materials at the controller's alpha
//!    (0x213120), which fades out linearly over the second her stealth fades in and back as it
//!    fades out (`skills::opacity`; take11's presence). While it's under 1 she's drawn in the
//!    sorted transparent pass (FUN_000e7850, FUN_000e7d40 case 6) twice, depth then colour, so
//!    only her nearest surface is blended. Here each part below 1 is blended over the picture
//!    (AlphaMode::Blend, with a copy of its texture whose alpha is 255: the squad's skins keep a
//!    specular mask in theirs, scene.rs `material`), and a depth-only copy of each part (`Depth`)
//!    puts her nearest surface in the depth buffer first, as the game's first pass does. The
//!    alpha is `srgb_alpha`'s, which gives the console's blend in sRGB values where bevy blends
//!    in linear light. Fully stealthed it's off and her skin isn't drawn at all.
//!  - **Her refraction** (kind 2, `Cloak`): while it's active the renderer copies the back buffer,
//!    after the sorted transparent pass and the decals (FUN_000e81e0: FUN_000e4da0 at 0xe877a), and
//!    draws her whole model and the guns she holds with BF_InvisMaterial over the picture
//!    (FUN_000e5240 at 0xe87ab): each surface shows the copy at its own place on screen, shifted
//!    by the controller's offset x n.z x (n.x, n.y) of the screen (n its normal in world space),
//!    tinted (0.9 - 0.05 n.z, 1, 1) and blended at the controller's alpha (src/invis.wgsl). Its
//!    offset is 0.15 x her visibility + 0.1 x sin(2.5 t) while she's stealthed: the outline's
//!    slow wobble, which a shot strengthens for a quarter second; it fades in under her skin and
//!    out over it in two halves (`Cloak::update`). Here each part gets a twin (`Twin`) drawn with
//!    `InvisMaterial` while the cloak is active: bevy draws a material that reads the view's
//!    transmission texture in its Transmissive3d phase and copies the picture into that texture
//!    first; `RefractionOrder` moves the phase after the see-through things, where the game makes
//!    its copy, so the copy holds the terrain's blended layers, the decals, her fading skin and
//!    any smoke, as the game's does. The streaky texture the material loads (h_ff43a70b) is never
//!    bound, nor drawn here.
//!  - **Warm-up**: bevy compiles a pipeline the first time something is drawn with it and skips
//!    the draw until it's ready, so her first fade would lose her for a few frames. As her model
//!    first shows, each part gets an invisible blended copy for WARM_UP s (alpha 0), and its depth
//!    copy and its twin (alpha 0) are drawn meanwhile: the pipelines are ready before she fades.
//!  - Her fading parts and their depth copies are drawn after every other see-through thing
//!    (FADE_SORT), so the copies take nothing from what lies behind her; one between her and the
//!    camera (smoke, a liquid's surface) is drawn under her during a fade, not over: the demo's,
//!    for the second each fade lasts.
//!  - A part she takes up after her model shows (a grenade in her hand, whose material its type
//!    shares with every such grenade) is faded with a copy of its material on its own entity
//!    (`Faded::foreign`), its own put back once it leaves her (thrown) or she's whole; it gets a
//!    twin while it's hers. The gun she drops at death gets its own material back in the frame
//!    it leaves her hand (the game draws a held item through its holder's controller,
//!    FUN_000e51e0, and a dropped one has none), while her body fades back in.
//!  - The Vengar view draws everyone as a heat figure (play_vengar.rs `heat_figures`), a
//!    stealthed Hawk too: the game draws them with every controller but kind 0x100 on
//!    (FUN_000e7d40's passes 9 and 10, the mask 0xfffffeff), and in that view kind 2 draws with the
//!    vision material (FUN_000e7320), so there's no refraction there: the twins are hidden. In
//!    the controlled character's scope her model is hidden, the twins with it (take05).
//!
//! Test hooks: play_skills.rs's (`BF_TEST_SKILL`, `BF_TEST_STAMINA`, `BF_TEST_SQUAD_SKILL`,
//! `BF_TEST_STEALTH_FLASH`, `BF_SKILL_LOG`, which also prints each body going see-through and
//! back, and its copies and twins made).

use super::*;
use std::collections::HashSet;
use bevy::{
    asset::weak_handle,
    core_pipeline::core_3d::graph::{Core3d, Node3d},
    pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline, MaterialPipeline, MaterialPipelineKey, MaterialPlugin,
          NotShadowCaster},
    render::{
        mesh::{skinning::SkinnedMesh, MeshVertexBufferLayoutRef},
        render_graph::{RenderGraph, RenderGraphError},
        render_resource::{AsBindGroup, BlendState, ColorWrites, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError},
        view::NoFrustumCulling,
        RenderApp,
    },
};

/// How long (s) a body's blended warm-up copies, and its refraction twins at alpha 0, are drawn
/// once its model shows (see the module's notes): long enough for bevy to build the pipelines on
/// its worker threads (#122's heat figures took two frames at 60 fps).
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
/// The red of the refraction's colour (FUN_00212430: 0.9 at 0x3a50b8 while the controller's byte
/// +0x20 is set, which its factory sets and nothing clears; 1.0 without), the shader's c26.x.
const INVIS_RED: f32 = 0.9;
/// The refraction's shader (src/invis.wgsl).
const INVIS_SHADER: Handle<Shader> = weak_handle!("3c9e1f47-8a2b-4d6c-b5e0-71f4a9d2c8e6");

/// The game's invisibility material, BF_InvisMaterial (class 0xf3bf04d1, its draw 0xa1ba0;
/// src/invis.wgsl), one for each cloaking body, set from its cloak each frame (`refract_bodies`).
/// It reads the view's transmission texture, so bevy draws it in its Transmissive3d phase (its
/// alpha mode left opaque: a blended one would go to Transparent3d), with a copy of the picture
/// made just before (`RefractionOrder`).
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
struct InvisMaterial {
    /// x the lookup's offset (c24.w: `Cloak::offset`), y the alpha (c26.w: `Cloak::alpha`), z the
    /// colour's red (c26.x: INVIS_RED)
    #[uniform(0)]
    params: Vec4,
    /// xy the game's screen in the demo's view (`game_screen`), which the shift is a share of
    #[uniform(1)]
    screen: Vec4,
}

/// The game's screen in the demo's view: the half-width and the half-height of the game's 4:3
/// view at unit depth (its follow camera's 70 degrees across, play_targeting.rs
/// `GAME_TAN_ACROSS`, and 3 / 4 of that down) over the demo's half-height (play.rs `FOV_Y`, the
/// game's own 55.4 degrees down since #269), unzoomed (a zoom divides both views alike): (4/3, 1),
/// the game's 4:3 screen within the demo's taller-than-it-is-wide share. The material shifts its
/// lookup by a share of the game's screen (NDC); scaled by these, the copy moves by the same angle
/// as in the takes, so by the same amount against her body and what's behind her.
fn game_screen() -> Vec2 {
    Vec2::new(targeting::GAME_TAN_ACROSS, targeting::GAME_TAN_ACROSS * 0.75) / (FOV_Y * 0.5).tan()
}

impl Material for InvisMaterial {
    fn fragment_shader() -> ShaderRef {
        INVIS_SHADER.into()
    }

    /// The copy of the picture it samples, bevy's view transmission texture.
    fn reads_view_transmission_texture(&self) -> bool {
        true
    }

    /// Blended by its alpha (0xa1a40: SRCALPHA / INVSRCALPHA, the device's +0xc8 / +0xdc), tested
    /// against the depth and culled as her skin is (the material sets no depth or cull state: the
    /// pass's defaults), not writing depth (the pass's depth write isn't traced: a guess, which
    /// only decides which of her overlapping surfaces' shifts shows).
    fn specialize(_pipeline: &MaterialPipeline<Self>, descriptor: &mut RenderPipelineDescriptor, _layout: &MeshVertexBufferLayoutRef,
                  _key: MaterialPipelineKey<Self>) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = descriptor.fragment.as_mut() {
            for target in fragment.targets.iter_mut().flatten() {
                target.blend = Some(BlendState::ALPHA_BLENDING);
            }
        }
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            depth.depth_write_enabled = false;
        }
        Ok(())
    }
}

/// Marks a part's refraction twin: a child of the part with its mesh (and skin), drawn with its
/// body's `InvisMaterial`; and whose body it is (the model's root).
#[derive(Component)]
struct Twin {
    root: Entity,
}

/// Bevy's Transmissive3d pass moved after its Transparent3d one in the 3D graph: the game copies
/// the picture for the refraction after the sorted transparent pass and the decals (FUN_000e81e0:
/// case 6 at 0xe86f6, the decals at 0xe873c, the copy FUN_000e4da0 at 0xe877a, the refraction list
/// FUN_000e5240 at 0xe87ab), where bevy's order copies it before them, which would leave the
/// terrain's blended layers, the decals and her fading skin out of the copy and draw them over
/// her outline unshifted. Only `InvisMaterial` reads the copy in the demo, so nothing else moves.
struct RefractionOrder;

impl Plugin for RefractionOrder {
    fn build(&self, app: &mut App) {
        let Some(render) = app.get_sub_app_mut(RenderApp) else { return };
        let mut graph = render.world_mut().resource_mut::<RenderGraph>();
        let Some(core) = graph.get_sub_graph_mut(Core3d) else {
            // (without it the refraction would quietly keep bevy's order: say so)
            warn!("the refraction's pass isn't moved: no Core3d render graph");
            return;
        };
        if let Err(e) = transmissive_last(core) {
            warn!("the refraction's pass isn't moved: {e}");
        }
    }
}

/// The 3D graph's main passes as opaque, transparent, transmissive (bevy's: opaque,
/// transmissive, transparent).
fn transmissive_last(core: &mut RenderGraph) -> Result<(), RenderGraphError> {
    core.remove_node_edge(Node3d::MainTransmissivePass, Node3d::MainTransparentPass)?;
    core.try_add_node_edge(Node3d::MainOpaquePass, Node3d::MainTransparentPass)?;
    core.try_add_node_edge(Node3d::MainTransparentPass, Node3d::MainTransmissivePass)?;
    core.try_add_node_edge(Node3d::MainTransmissivePass, Node3d::EndMainPass)
}

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

/// Marks a copy of a body's part drawn with a material of its own (a depth copy, `DepthCopy`, or
/// a refraction twin, `Twin`), which sits under the body's root: play.rs's `lamp_characters`
/// passes over it, as it has no standard material to light (without this a cloaking body's
/// lamp light was sent to the GPU again every frame once it changed, #219).
#[derive(Component)]
pub struct BodyCopy;

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
///    and how long their warm-up has left (s); the one depth-only material;
///  - the refraction: each cloaking body's `InvisMaterial`, by its model's root, and the parts
///    given a twin (`Twin`), with their twins.
#[derive(Resource, Default)]
struct Faded {
    own: HashSet<AssetId<StandardMaterial>>,
    made: HashMap<AssetId<StandardMaterial>, Kept>,
    foreign: HashMap<Entity, (Entity, Handle<StandardMaterial>, Handle<StandardMaterial>)>,
    textures: HashMap<AssetId<Image>, Handle<Image>>,
    prepared: HashMap<Entity, f32>,
    depth: Option<Handle<DepthMaterial>>,
    invis: HashMap<Entity, Handle<InvisMaterial>>,
    twins: HashMap<Entity, Entity>,
}

/// The depth-only and the invisibility materials (the latter in no prepass or shadow: its twins
/// cast none), the refraction's pass order, and the fade and the refraction after the skills have
/// stepped (so a switch shows in its frame).
pub fn plugin(app: &mut App) {
    app.world_mut().resource_mut::<Assets<Shader>>().insert(&INVIS_SHADER, Shader::from_wgsl(include_str!("../invis.wgsl"), file!()));
    app.add_plugins((MaterialPlugin::<DepthMaterial>::default(),
                     MaterialPlugin::<InvisMaterial> { prepass_enabled: false, shadows_enabled: false, ..default() }, RefractionOrder))
        .init_resource::<Faded>()
        .add_systems(OnEnter(AppState::Playing), |mut commands: Commands| commands.insert_resource(Faded::default()))
        .add_systems(Update, (prepare_bodies, fade_bodies, refract_bodies).chain().after(skills::update_skills).run_if(in_state(AppState::Playing)));
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

/// Whether `u`'s body can be drawn see-through: Hawk's cloak (`skills::cloaks`), or the follow
/// camera's fade of the squad's four (any of them can be the one it follows, or stand between it
/// and that one: play_camera.rs `body_alpha`). The game's depth fade takes any character between
/// the camera and the one it follows (FUN_00116340); the demo the squad's, whose bodies these
/// copies are made for (an NPC or enemy there is drawn whole).
fn fades(u: &Player) -> bool {
    skills::cloaks(u) || (u.character < SQUAD && u.npc.is_none())
}

/// The copies of a fading character's parts (`fades`), made once its model shows: a
/// depth-only copy of each (`DepthCopy`, hidden but in the warm-up and the fades) and a blended
/// warm-up copy (`WarmCopy`, alpha 0), each beside its part with the same mesh and skin. The
/// warm-up copies go when their time is out. (Its refraction twins come from `refract_bodies`,
/// drawn at alpha 0 for the same warm-up.)
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
        if !fades(u) || faded.prepared.contains_key(&l.root) {
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
                    c.insert((MeshMaterial3d(depth.clone()), DepthCopy { root: l.root }, BodyCopy, NotShadowCaster, ChildOf(parent.parent())));
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
        // a gun dropped at death (play.rs `update_weapons` takes it off the body) has its own
        // material back at once: the game draws a held item through its holder's invisibility
        // controller (FUN_000e7850 asks FUN_000e51e0 for one on the item or on the character
        // holding it), and a dropped one has no holder, so it lies on the floor fully seen from
        // the drop while her body fades back in (#197; todo/197-ai-sight/answer.md §5.1)
        if l.weapon_dropped {
            for w in l.weapons.iter().filter(|w| !descends(w.entity, l.root, &parents)) {
                for e in std::iter::once(w.entity).chain(children.iter_descendants(w.entity)) {
                    let Ok(part) = parts.get(e) else { continue };
                    let id = part.0.id();
                    if !made.get(&id).is_some_and(|k| k.root == l.root) {
                        continue;
                    }
                    let Some(k) = made.remove(&id) else { continue };
                    if let Some(m) = materials.get_mut(id) {
                        m.base_color_texture = k.texture;
                        m.alpha_mode = k.mode;
                        m.depth_bias = k.bias;
                        m.base_color.set_alpha(1.0);
                    }
                    if log {
                        println!("t={:.2} {}: dropped gun's part opaque again", u.sim_time, unit_name(u));
                    }
                }
            }
        }
        // her skin's controller x the follow camera's (FUN_00116480 draws at the spawn fade x the
        // camera's alpha, and each controller multiplies its own in, 0x213120; the guns she holds
        // and carries too, 0x212d60: todo/126 answers.md Q6)
        let shown = skills::opacity(u) * camera::body_alpha(u, &player);
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

/// Each cloaking body's refraction (see the module's notes): its material set from its cloak this
/// frame (`skills::refraction`); its twins shown while the cloak is active, and in the warm-up at
/// alpha 0, but not in the Vengar view (the controlled character's vision mode 2), whose heat pass
/// draws her instead; a twin for each of its parts that hasn't one (those it takes up later too:
/// a grenade in her hand), and none for a part that has left it (the grenade thrown).
#[allow(clippy::too_many_arguments)]
fn refract_bodies(mut commands: Commands, player: Res<Player>, squad: Res<Squad>, children: Query<&Children>, parents: Query<&ChildOf>,
                  parts: Query<(&Mesh3d, &MeshMaterial3d<StandardMaterial>, Option<&SkinnedMesh>), (Without<WarmCopy>, Without<DepthCopy>)>,
                  mut twins: Query<(Entity, &Twin, &ChildOf, &mut Visibility)>, standard: Res<Assets<StandardMaterial>>,
                  mut invis: ResMut<Assets<InvisMaterial>>, mut faded: ResMut<Faded>) {
    let log = std::env::var("BF_SKILL_LOG").is_ok();
    let vengar = skills::vengar_on(&player);
    let Faded { prepared, invis: bodies, twins: given, .. } = &mut *faded;
    // each body's material now, and the bodies whose twins are drawn
    let mut drawn: Vec<(Entity, Handle<InvisMaterial>, &Player)> = vec![];
    for u in std::iter::once(&*player).chain(squad.0.iter()) {
        let Some(l) = &u.loaded else { continue };
        // (a body the camera fades without a cloak has no refraction)
        if !skills::cloaks(u) {
            continue;
        }
        let Some(&warm) = prepared.get(&l.root) else { continue };
        let now = skills::refraction(u);
        let (alpha, offset) = now.unwrap_or((0.0, 0.0));
        let params = Vec4::new(offset, alpha, INVIS_RED, 0.0);
        let handle = bodies.entry(l.root).or_insert_with(|| invis.add(InvisMaterial { params, screen: game_screen().extend(0.0).extend(0.0) })).clone();
        // (only when it changes: a changed material is sent to the GPU again)
        if invis.get(&handle).is_some_and(|m| m.params != params) {
            if let Some(m) = invis.get_mut(&handle) {
                m.params = params;
            }
        }
        if (now.is_some() || warm > 0.0) && !vengar {
            drawn.push((l.root, handle, u));
        }
    }
    // the twins made: shown with their body; gone with their part, or once the part has left it
    let mut kept = HashSet::new();
    for (twin, t, part, mut vis) in &mut twins {
        if !descends(part.parent(), t.root, &parents) {
            commands.entity(twin).despawn();
            continue;
        }
        kept.insert(twin);
        let want = if drawn.iter().any(|(root, ..)| *root == t.root) { Visibility::Inherited } else { Visibility::Hidden };
        if *vis != want {
            *vis = want;
        }
    }
    given.retain(|_, twin| kept.contains(twin));
    // a twin for each part of a drawn body without one (not the muzzle flash: unlit, not hers)
    for (root, handle, u) in drawn {
        let mut new = 0;
        for e in children.iter_descendants(root) {
            if given.contains_key(&e) {
                continue;
            }
            let Ok((mesh, material, skin)) = parts.get(e) else { continue };
            if standard.get(&material.0).is_none_or(|m| m.unlit) {
                continue;
            }
            let mut c = commands.spawn((Twin { root }, BodyCopy, mesh.clone(), MeshMaterial3d(handle.clone()), Transform::IDENTITY, Visibility::Inherited,
                                        NoFrustumCulling, NotShadowCaster, ChildOf(e)));
            if let Some(s) = skin {
                c.insert(s.clone());
            }
            given.insert(e, c.id());
            new += 1;
        }
        if log && new > 0 {
            println!("t={:.2} {}: refraction twins of {new} parts", u.sim_time, unit_name(u));
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

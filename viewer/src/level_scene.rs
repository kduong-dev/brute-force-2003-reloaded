//! A decoded `Level` as Bevy entities: terrain layers (blended by their masks), the placed
//! objects and the sky. Shared by the level viewer (bf_level) and the playable demo.

use std::collections::{HashMap, HashSet};

use bevy::{
    asset::{weak_handle, RenderAssetUsages},
    pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline, NotShadowCaster},
    prelude::*,
    render::{
        mesh::{Indices, MeshVertexBufferLayoutRef, PrimitiveTopology, VertexAttributeValues},
        render_resource::{AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError, TextureFormat},
        storage::ShaderStorageBuffer,
    },
};

use crate::bf::character::{Address, Game, Geoset, H_WRAP_ALPHA};
use crate::bf::hash::h;
use crate::bf::level::{Level, LevelLight, SkySpin, LU_GEOSET, LU_INDOOR};
use crate::bf::texture::Mips;
use crate::bf::triggers::{act, Class, AC_PLAY, AC_RESUME, AM_BACKWARD, AM_FORWARD};
use crate::bf::weapon::{PartJoint, WeaponModel, HINGE_JOINT, TRANSFORM_JOINT};
use crate::scene::mip_image;

/// A level's material: bevy's standard one, lit also by the level's point lights (light-objects
/// of a point type, `LevelLamp`) as the game lights its scenery: colour x (1 - distance / range)
/// x N.L, per pixel here (per vertex there), unshadowed, nothing past the range (src/lamps.wgsl).
/// Bevy's own point lights fall off as 1 / d^2 and couldn't follow that: a light matched at mid
/// range was far too bright beside it and too dim towards its edge. It takes the lamps of its
/// light pair only, as the wrapper material's draw gathers a scenery object's (FUN_0009f0b0: the
/// lamps flagged for the pair its light-usage picks), and an indoor one (`Lamps::indoor`) the
/// level's indoor pair in place of bevy's directional lights, which are the outdoor pair
/// (`spawn_lighting`). The terrain takes no lamps here: it's drawn unlit, by its baked light map
/// (`spawn_level`). The game adds lamps to it in a pass of their own per block (FUN_00091070:
/// additive, three a pass) from a list whose filling isn't traced, so which lamps light the
/// ground there, and by which pair if any, is unknown (#181's).
pub type LevelMaterial = ExtendedMaterial<StandardMaterial, Lamps>;

/// The level's lights, for `LevelMaterial`: one buffer shared by all its materials
/// (`light_buffer`: the pairs, then the point lights). A terrain layer's material also carries
/// its colour and detail textures (`terrain_detail`) and its bump map (`terrain_bump`).
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
#[bind_group_data(LampsKey)]
pub struct Lamps {
    #[storage(100, read_only)]
    pub lamps: Handle<ShaderStorageBuffer>,
    /// A terrain layer's depth-test bias (the pipeline's constant depth bias), where its base
    /// material's `depth_bias` is used to sort it (see TERRAIN_LAYER_SORT): bevy takes both
    /// from that one field.
    pub layer_bias: Option<i32>,
    /// Lit by the level's indoor pair and its lamps (the shader def INDOOR_PAIR), where the others
    /// take the outdoor pair and its lamps: what the game's wrapper material picks by the object's
    /// light-usage (`indoor_lit`).
    pub indoor: bool,
    /// A terrain layer's detail constants, the game's c20 (`terrain_detail`); zero for the
    /// other materials.
    #[uniform(101)]
    pub detail: Vec4,
    /// A terrain layer's colour texture, sampled here in place of the standard material's
    /// (which a terrain layer leaves without one): see `terrain_detail`.
    #[texture(102)]
    #[sampler(103)]
    pub colour: Option<Handle<Image>>,
    /// A terrain layer's detail texture (its colour texture where it names none).
    #[texture(104)]
    #[sampler(105)]
    pub detail_texture: Option<Handle<Image>>,
    /// A terrain layer's bump map (its material's normal map, h_ed0b519d), its normals in R G B
    /// (`Game::bump_mips`): see `terrain_bump`.
    #[texture(106)]
    #[sampler(107)]
    pub bump_texture: Option<Handle<Image>>,
    /// The light a terrain layer's bump map takes (`terrain_bump`); zero for the other materials.
    #[uniform(108)]
    pub bump: TerrainBump,
}

/// What a `Lamps` material's pipeline depends on: the layer's own depth-test bias, if any,
/// whether it takes the indoor pair, whether it mixes in a detail texture (a terrain layer: the
/// shader def TERRAIN_DETAIL) and whether it lights a bump map (TERRAIN_BUMP).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LampsKey {
    layer_bias: Option<i32>,
    indoor: bool,
    detail: bool,
    bump: bool,
}

impl From<&Lamps> for LampsKey {
    fn from(lamps: &Lamps) -> Self {
        LampsKey { layer_bias: lamps.layer_bias, indoor: lamps.indoor, detail: lamps.colour.is_some(), bump: lamps.bump_texture.is_some() }
    }
}

/// The light a terrain layer's bump map takes (`terrain_bump`, src/lamps.wgsl under
/// TERRAIN_BUMP). (In a module of its own: the derive's own checks read as unused.)
mod terrain_bump_params {
    #![allow(dead_code)]
    use bevy::{prelude::*, render::render_resource::ShaderType};

    #[derive(Clone, Copy, Debug, Default, Reflect, ShaderType)]
    pub struct TerrainBump {
        /// Toward the light the bump map takes, as the game hands it to vertex program 0x38 (c16:
        /// the column of the light's transform, not normalised); w the bump map's tiling over the
        /// colour's uv (c20.z, the detail's).
        pub key: Vec4,
        /// That light's colour (pixel shaders 0x1a / 0x1b's c0, see `terrain_bump`).
        pub key_colour: Vec4,
    }
}
pub use terrain_bump_params::TerrainBump;

const LAMP_SHADER: Handle<Shader> = weak_handle!("6b0f6c1e-5a3d-4f0e-9a51-1d2c7e4b8a90");

impl MaterialExtension for Lamps {
    fn fragment_shader() -> ShaderRef {
        LAMP_SHADER.into()
    }

    /// A terrain layer keeps its own depth-test bias (the base material set it from the sort
    /// bias, see TERRAIN_LAYER_SORT), an indoor material takes the indoor pair (INDOOR_PAIR) and a
    /// terrain layer samples its textures as the game's terrain pass does (TERRAIN_DETAIL, see
    /// `terrain_detail`) and lights its bump map (TERRAIN_BUMP, `terrain_bump`).
    fn specialize(_pipeline: &MaterialExtensionPipeline, descriptor: &mut RenderPipelineDescriptor, _layout: &MeshVertexBufferLayoutRef,
                  key: MaterialExtensionKey<Self>) -> Result<(), SpecializedMeshPipelineError> {
        if let (Some(bias), Some(depth)) = (key.bind_group_data.layer_bias, descriptor.depth_stencil.as_mut()) {
            depth.bias.constant = bias;
        }
        if let (true, Some(fragment)) = (key.bind_group_data.indoor, descriptor.fragment.as_mut()) {
            fragment.shader_defs.push("INDOOR_PAIR".into());
        }
        if let (true, Some(fragment)) = (key.bind_group_data.detail, descriptor.fragment.as_mut()) {
            fragment.shader_defs.push("TERRAIN_DETAIL".into());
        }
        if let (true, Some(fragment)) = (key.bind_group_data.bump, descriptor.fragment.as_mut()) {
            fragment.shader_defs.push("TERRAIN_BUMP".into());
        }
        Ok(())
    }
}

/// The terrain material class (h_0ec4edf0, factory 0x91330; every terrain layer's material).
const TERRAIN_MATERIAL: u32 = 0x0EC4_EDF0;
/// Its texture slots past `color` and `alpha` (0x90b10): the detail texture (h_17fc684a,
/// "detail" by its hash) and a P8 bump map (h_ed0b519d, TERRAIN_BUMP_TEXTURE). On all 141
/// terrain materials of the game's levels the detail texture is the colour texture.
const TERRAIN_DETAIL_TEXTURE: u32 = 0x17FC_684A;
/// Its constants (the table at 0x90e40) that the detail reads, over the 141 terrain materials of
/// the game's levels: the detail's tiling over the colour's uv (h_08e8e4c3, +0x54; 0-16, 112 of
/// them 3-5, the others 1 (2), 2 (4), 2.4, 2.5 (3), 2.8, 6 (8), 7 (5), 9 (2) and 16 (2); 3.5 / 3
/// / 2.5 on the tutorial's three layers), how far from the eye it reaches (h_fe22cc04, +0x5c, m;
/// 0-35, 117 of them 15-25, two 1 m; the tutorial's 20, 20, 15) and its share at the eye
/// (h_153264c6, +0x50; 0-1, 105 of them 0.8-0.9, 0.4 on three, 0.7 on one). e05's h_fa6eaf30
/// has 0 for all three: no detail. Without them the class's factory defaults (0x90ee0): 0.8, 16
/// and 0 (no detail).
const DETAIL_SCALE: u32 = 0x08E8_E4C3;
const DETAIL_FADE: u32 = 0xFE22_CC04;
const DETAIL_SHARE: u32 = 0x1532_64C6;
const DETAIL_DEFAULTS: (f32, f32, f32) = (0.8, 16.0, 0.0);

/// A terrain layer's detail, as the game draws its terrain (#257): material `id`'s constants
/// as the game's c20, with its detail texture; None for a material of another class, one whose
/// detail has no share (w 0) or with BF_NO_DETAIL=1 (to compare).
///
/// The terrain material's begin (FUN_00090f50) binds its detail texture (else its colour) at
/// stage 2 and sets c20 = (1 / s^2, s^2, k, w) from the constants above: s the fade distance, k
/// the scale, w the share. The terrain's base pass, vertex program 0x38 (0x3d8bf8; it reads c20
/// and c23.w = 1/16, the height scale the begin writes), passes the colour's uv as it is
/// (oT0 = v9), the mask's (oT1 = v10), the detail's and the bump map's at k times it
/// (oT2 = oT3 = v9 x c20.z), and the detail's share in oD1.w = w (1 - d^2 / s^2), d the vertex's
/// distance from c24, 0 past s (c24: the point the draw item's +0x34 points at, FUN_00091070;
/// the eye, by this use). Its pixel shaders 0x1a (a layer: its alpha the mask's) and 0x1b (the
/// ground) mix the two by that share, lerp(t0, t2, v1.a), before lighting it. So within s of the
/// eye the ground shows its texture k times finer, fading to the colour's own tiling at s; the
/// demo drew the colour's tiling everywhere. (The bump map, lit through oD1.rgb: `terrain_bump`.)
/// No code was found that selects 0x38 / 0x1a / 0x1b; the rest of the data and the takes fit
/// them (levels.md, Terrain layers). Drawn by lamps.wgsl under TERRAIN_DETAIL, per pixel (the
/// console's share is per vertex, 2 m apart).
pub fn terrain_detail(game: &Game, id: u32) -> Option<(Vec4, u32)> {
    if game.material_type(id) != TERRAIN_MATERIAL || std::env::var("BF_NO_DETAIL").is_ok() {
        return None;
    }
    let num = |n: u32, or: f32| game.material_constant(id, n).and_then(|v| v.first().copied()).unwrap_or(or);
    let (k, s, w) = (num(DETAIL_SCALE, DETAIL_DEFAULTS.0), num(DETAIL_FADE, DETAIL_DEFAULTS.1), num(DETAIL_SHARE, DETAIL_DEFAULTS.2));
    let texture = game.material_param_texture(id, TERRAIN_DETAIL_TEXTURE).or_else(|| game.material_texture(id))?;
    // (FUN_00090f50: 1 / s^2 is 0 when s^2 is no more than FLT_EPSILON, 0x3a4ffc)
    let s2 = s * s;
    (w > 0.0).then_some((Vec4::new(if s2 > f32::EPSILON { 1.0 / s2 } else { 0.0 }, s2, k, w), texture))
}

/// The terrain material's bump map: its texture slot h_ed0b519d ("normalmap" by its hash; +0x18
/// of the class's slots, 0x90b10 / 0x90bf0), a P8 texture of normals (`Game::bump_mips`). 140 of
/// the 141 terrain materials of the game's levels name one, all P8: 132 of 256 x 256 (the
/// tutorial's with 9 levels), 5 of 128 x 128 and 3 a flat 2 x 2 (e40's rock h_f5b37660, e05's
/// h_fa6eaf30); mp7's h_1c7f8bd5 names none.
const TERRAIN_BUMP_TEXTURE: u32 = 0xED0B_519D;

/// A terrain layer's bump map, lit as the game's terrain base pass lights it (#271): material
/// `id`'s normal map with the constants its light comes from (`TerrainBump`); None for a material
/// of another class, one without a normal map or with BF_NO_BUMP=1 (to compare). (`spawn_level`
/// draws none either without the normal palette: no default.xbe beside the data.)
///
/// Vertex program 0x38 (0x3d8bf8) lights a terrain vertex by the ambient and slot 1's light of
/// the pair the shaders hold (`Level::slots`: c18 toward it, c19 its colour), oD0.rgb = (c23 +
/// max(0, n . c18) c19 c21) x the vertex's baked light (v12.y, TERRAIN_FRAME), and hands slot 0's
/// light (c16) to the bump map: oD1.rgb = c16 turned into the vertex's tangent frame
/// (`Geoset::frames`), x 0.5 + 0.5, and oD0.w = the baked light where n . c16 >= 0, else 0. Its
/// pixel shaders 0x1a (a layer) and 0x1b (the ground) add (2 t3 - 1) . (2 v1 - 1), the bump map's
/// normal (sampled at oT3 = the colour's uv x c20.z, the detail's tiling, not faded) against that
/// light, clamped at 0 (the combiner's unsigned input), x v0.a x c0 to v0, and light the colour
/// and detail mix with the sum (`terrain_detail`). The demo lights the ground by the baked light
/// alone (`spawn_level`), which already holds that light as a flat normal gives it, so the shader
/// (lamps.wgsl, TERRAIN_BUMP) adds the term's change from the flat one: the ground's light x (1 +
/// c0 (max(0, N . c16) - max(0, n . c16))) where n . c16 >= 0, N the bump map's normal and n the
/// vertex's, in the console's gamma terms. That's the console's own change, the baked light
/// standing for its v0.a; where the bump map is flat the ground is as before. (Taken over the
/// whole vertex light instead, (v0 + c0 max(0, N . c16)) / (v0 + c0 max(0, n . c16)), it blew
/// slopes the key light grazes up to white: the console's v0 there is dark where the demo's baked
/// light isn't, #181. The console's clamp of the sum at 1 is left out.)
///
/// No code was found that selects program 0x38 or pixel shaders 0x1a / 0x1b (levels.md "The
/// bump map"); they're the only shaders that read the terrain's normal map and its tangent frame,
/// and the takes show the relief they give: e40's sand (todo/49-bower-20 take06), m03_a's (take01)
/// and the tutorial's hillsides (todo/28-tutorial take03). The pair is the outdoor one, the one
/// each view's draw starts with (0xda2aa). c0 (stage 2's C0: pixel constant 0, by the shaders' C0
/// mapping 0xfffff0ff; their own PSConstant0 is 0) is set nowhere for them: taken as slot 0's
/// colour, the light the bump map stands for (a guess, not fitted; levels.md "The bump map" has
/// what it gives against the takes).
pub fn terrain_bump(game: &Game, level: &Level, id: u32) -> Option<(TerrainBump, u32)> {
    if game.material_type(id) != TERRAIN_MATERIAL || std::env::var("BF_NO_BUMP").is_ok() {
        return None;
    }
    let texture = game.material_param_texture(id, TERRAIN_BUMP_TEXTURE)?;
    let k = game.material_constant(id, DETAIL_SCALE).and_then(|v| v.first().copied()).unwrap_or(DETAIL_DEFAULTS.0);
    // slot 0 of the outdoor pair by FUN_000dcc10's rule on the lights' types (`Level::slots`,
    // #288: the key light whichever the file lists first; the lava's v0 is the same light's,
    // `Level::objects_key`); toward the light is minus `dir`, at the column's length; an empty
    // slot is black (no bump)
    let [key, _] = level.slots(false);
    let bump = TerrainBump {
        key: key.map_or(Vec3::ZERO, |l| -l.dir * l.strength).extend(k),
        key_colour: key.map_or(Vec3::ZERO, |l| Vec3::from(l.color)).extend(1.0),
    };
    Some((bump, texture))
}

/// Added to a terrain layer's sort distance: bevy sorts transparent meshes back to front by
/// their origin's depth in the view plus the material's `depth_bias`, and a layer's origin is
/// the world's (its mesh is in world coordinates). Where that origin is nearer the camera than
/// an effect, the layer drew after it, painting the ground over explosions, muzzle flashes and
/// decals: take04's cliff blasts in #73 vanished, and their scorch showed only where the rock
/// layer's mask let it through, as straight-edged wedges. A layer is part of the ground, so it
/// sorts first, before every other transparent thing; the layers keep their order among
/// themselves (+10 each, as before). Its depth-test bias stays as it was (`Lamps::layer_bias`).
const TERRAIN_LAYER_SORT: f32 = -1.0e6;

/// A liquid's surface, drawn as the game's two liquid material classes draw theirs (#4;
/// src/liquid.wgsl): BF_LavaMaterial (h_f124a774: lava, sludge, toxic waste, chloride), whose two
/// blend modes pick one of its pixel shaders (`lava_passes`), and BF_OceanMaterialBase
/// (h_0f8904b9: water), two layers averaged and lit by the level's lights. The extension makes the
/// colour; bevy's standard material carries the rest: the fog, the blending, the depth bias. The
/// code read is todo/146-liquids/leads.md §6 and answer-spec.md Q2, Q3, Q7, Q8, Q10, Q11.
pub type LiquidMaterial = ExtendedMaterial<StandardMaterial, LiquidSurface>;

/// A liquid surface's textures and constants (`LiquidMaterial`, `liquid_materials`).
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct LiquidSurface {
    #[uniform(100)]
    pub params: LiquidParams,
    /// `texture-0`'s frames, a layer each (a plain texture is one), as stored (`frames_image`).
    /// None: bevy's white fallback, which is what the console binds for a slot left empty
    /// (FUN_00062280 binds its 1 x 1 opaque white default texture for a null one: the e40 ocean's
    /// texture-0, answer-spec Q2)
    #[texture(101, dimension = "2d_array")]
    #[sampler(102)]
    pub layer0: Option<Handle<Image>>,
    /// `texture-1`'s frames, likewise (the water's shimmer: 34)
    #[texture(103, dimension = "2d_array")]
    #[sampler(104)]
    pub layer1: Option<Handle<Image>>,
    /// the water's alpha ramp (`water_ramp`): 128 x 1, clamped
    #[texture(105)]
    #[sampler(106)]
    pub ramp: Option<Handle<Image>>,
    /// where the material's clock starts (s): BF_LavaMaterial's `lifetime` (+0x38, its clock:
    /// 0x91ba0 adds dt to it), 0 for the water (+0x94 from 0, 0x92f50); see `LiquidClock`
    pub start: f32,
}

/// See src/liquid.wgsl. (In a module of its own: the derive's own checks read as unused.)
mod liquid_params {
    #![allow(dead_code)]
    use bevy::{prelude::*, render::render_resource::ShaderType};

    /// A liquid surface's constants (src/liquid.wgsl `Liquid`).
    #[derive(Clone, Copy, Debug, Default, Reflect, ShaderType)]
    pub struct LiquidParams {
        /// the game's pixel shader it draws with (`level_scene`'s PS_*: BF_LavaMaterial's 0x27 -
        /// 0x2c, the water's 0x2f; 0 the BF_LIQUID_DEBUG white)
        pub shader: u32,
        /// how its output is blended (`level_scene`'s BLEND_*), for what it writes as alpha
        pub blend: u32,
        /// pixel shader 0x2c's layer: 0 texture-0, 1 texture-1
        pub layer: u32,
        /// the material's clock (s): `LiquidSurface::start` + the level's `LiquidClock`
        pub clock: f32,
        /// per layer: 1 / its scale (u, v) and its translate (u, v; repeats per second)
        pub coords0: Vec4,
        pub coords1: Vec4,
        /// per layer: its frame count and frame rate (per second), and 1 where its translate is
        /// divided by its scale (the water's vertex program 0x45), else 0
        pub anim0: Vec4,
        pub anim1: Vec4,
        /// the water's `color` and `color-1` (pixel shader 0x2f's c0 and c1); BF_LavaMaterial's v0
        /// in `color0` (pixel shader 0x2b's tint of texture-1)
        pub color0: Vec4,
        pub color1: Vec4,
    }
}
pub use liquid_params::LiquidParams;

const LIQUID_SHADER_HANDLE: Handle<Shader> = weak_handle!("2c41a7d3-8e5b-4b6f-b0c2-93d1f5e7a604");

impl MaterialExtension for LiquidSurface {
    fn fragment_shader() -> ShaderRef {
        LIQUID_SHADER_HANDLE.into()
    }
}

/// The game time (s) the level's liquid materials have run for, set to 0 as `spawn_level` makes
/// them: what their clocks count from (`LiquidSurface::start`). The game steps every material's
/// update with the frame's dt from the main tick, skipped while the game is paused (0x1ea830 ->
/// the material library's +0x4c, 0x25cf30: the water's +0x94, the lava's `lifetime`), and the
/// texture library's animators with the same dt (+0x1c, 0x286060: the water's shimmer), so one
/// clock runs both. bf_play runs it on the game's clock (paused under a menu, fixed steps under
/// BF_CAPTURE), bf_level on bevy's.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct LiquidClock(pub f32);

/// The level materials (`LevelMaterial`, `LiquidMaterial`), their shaders, and the liquids' clock
/// (`LiquidClock`).
pub fn plugin(app: &mut App) {
    let mut shaders = app.world_mut().resource_mut::<Assets<Shader>>();
    shaders.insert(&LAMP_SHADER, Shader::from_wgsl(include_str!("lamps.wgsl"), file!()));
    shaders.insert(&LIQUID_SHADER_HANDLE, Shader::from_wgsl(include_str!("liquid.wgsl"), file!()));
    app.add_plugins((MaterialPlugin::<LevelMaterial>::default(), MaterialPlugin::<LiquidMaterial>::default()))
        .init_resource::<LiquidClock>()
        .add_systems(PostUpdate, run_liquid_clocks);
}

/// Each liquid material's clock: its start + the level's `LiquidClock`, once the clock has moved.
fn run_liquid_clocks(clock: Res<LiquidClock>, mut liquids: ResMut<Assets<LiquidMaterial>>) {
    if !clock.is_changed() {
        return;
    }
    for (_, m) in liquids.iter_mut() {
        m.extension.params.clock = m.extension.start + clock.0;
    }
}

/// The game's liquid material classes: BF_LavaMaterial (lava, sludge, toxic waste, chloride) and
/// BF_OceanMaterialBase (water). (The e40 surf band's BF_OceanMaterialSurf, h_06591b8a, isn't
/// one of them here: it keeps the level material, out of #4.)
const LIQUID_LAYERS: u32 = 0xF124_A774;
const LIQUID_WATER: u32 = 0x0F89_04B9;
/// The water's ramp texture parameter (h_0b85cb09, its stage 2 slot; 0x925e0): the texture
/// 0x92eb0 fills from the ramp's keys (`water_ramp`).
const WATER_RAMP: u32 = 0x0B85_CB09;
/// BF_OceanMaterialBase's six ramp keys (+0x64..+0x78) and six values (+0x7c..+0x90), in the
/// parameter table's order (0x92a70; todo/146-liquids leads §6.3). Unnamed.
const RAMP_KEYS: [u32; 6] = [0x04F8_4A86, 0x0DFF_7A10, 0x16F6_2BAA, 0x1FF1_1B3C, 0xFB95_8E9F, 0xF292_BE09];
const RAMP_VALUES: [u32; 6] = [0x19DF_E035, 0x10D8_D0A3, 0x0BD1_8119, 0x02D6_B18F, 0xE6B2_242C, 0xEFB5_14BA];
/// How many texels 0x92eb0 writes into the water's ramp texture, and the step between their
/// keys (0x3a52e8: 1/127) and their scale (0x3a50e8: 255).
const RAMP_TEXELS: usize = 128;
const RAMP_STEP: f32 = 0.007_874_02;
/// The ramp's mode (FUN_00092df0 sets the `IRampSingle`'s vtable +0x34 to 4): Freelancer's
/// ease-in-out between keys (`ale::ease` 4).
const RAMP_EASE: u8 = 4;
/// Liquid surfaces are drawn as if this much nearer (bevy depth bias), over terrain just under them.
const LIQUID_DEPTH_BIAS: f32 = 1000.0;

/// BF_LavaMaterial's pixel shaders (FUN_00091bc0, FUN_00091e60; the combiners decoded by the
/// analyst, todo/146-liquids leads §6.2), on the texels as stored (t0 texture-0, t1 texture-1),
/// each fogged, none doubling: 0x27 t0 + t1; 0x28 lerp(t0, t1, t1.a), alpha t1.a; 0x29
/// t0 x t0.a + t1, alpha t0.a; 0x2a t0 + t1; 0x2b lerp(t0, t1 x v0, t1.a); 0x2c one layer as it
/// is. And the water's (0x92f70): 0x2f (t0 x color + t1 x color-1) / 2 x v0, alpha the ramp's.
pub const PS_ADD: u32 = 0x27;
pub const PS_OVER: u32 = 0x28;
pub const PS_SEE_THROUGH: u32 = 0x29;
pub const PS_SUM: u32 = 0x2A;
pub const PS_CRUST: u32 = 0x2B;
pub const PS_ONE: u32 = 0x2C;
pub const PS_WATER: u32 = 0x2F;
/// How a liquid's output is blended into the picture (the material's render states): none
/// (opaque), SRCALPHA / INVSRCALPHA, ONE / INVSRCALPHA, ONE / ONE.
pub const BLEND_NONE: u32 = 0;
pub const BLEND_ALPHA: u32 = 1;
pub const BLEND_PREMULTIPLIED: u32 = 2;
pub const BLEND_ADD: u32 = 3;

/// BF_LavaMaterial's passes by its `blend-mode` (+0x24), `blend-mode-1` (+0x34) and which of its
/// textures it has (FUN_00091fe0): (pixel shader, blending, layer) each.
///  - Both textures (FUN_00091bc0): blend-mode-1 2: 0x2c on texture-1, no blend state set; 0 / 0:
///    two passes (FUN_000920a0), texture-0 then texture-1 through 0x2c, each SRCALPHA /
///    INVSRCALPHA, the second at ZFUNC LESSEQUAL (m02_a's h_fa91cd3b); blend-mode 2: 0x2b for
///    blend-mode-1 0 (lava), 0x2a for 1 (sludge), else 0x2c (texture-0), no blend state; blend-mode
///    0: 0x29, ONE / INVSRCALPHA (m09 and m14's toxic waste); otherwise 0x28 (blend-mode-1 0) ONE /
///    INVSRCALPHA, or 0x27 ONE / ONE (m14_c's chloride).
///  - One texture (FUN_00091e60): 0x2c on it, blended by its own layer's mode: 0 SRCALPHA /
///    INVSRCALPHA, 1 ONE / ONE, anything else none (e13's h_eabb45fa, mode 2: opaque, answer-spec
///    Q11).
///  - Neither: nothing drawn (FUN_00091fe0 clears its draw bit).
///
/// "No blend state set" leaves the world pass's own: blending off (the state reset FUN_001ea890),
/// so opaque.
pub fn lava_passes(blend0: i64, blend1: i64, has0: bool, has1: bool) -> Vec<(u32, u32, u32)> {
    let own = |mode: i64| match mode {
        0 => BLEND_ALPHA,
        1 => BLEND_ADD,
        _ => BLEND_NONE,
    };
    match (has0, has1) {
        (false, false) => vec![],
        (true, false) => vec![(PS_ONE, own(blend0), 0)],
        (false, true) => vec![(PS_ONE, own(blend1), 1)],
        (true, true) => match (blend0, blend1) {
            (_, 2) => vec![(PS_ONE, BLEND_NONE, 1)],
            (0, 0) => vec![(PS_ONE, BLEND_ALPHA, 0), (PS_ONE, BLEND_ALPHA, 1)],
            (2, 0) => vec![(PS_CRUST, BLEND_NONE, 0)],
            (2, 1) => vec![(PS_SUM, BLEND_NONE, 0)],
            (2, _) => vec![(PS_ONE, BLEND_NONE, 0)],
            (0, _) => vec![(PS_SEE_THROUGH, BLEND_PREMULTIPLIED, 0)],
            (_, 0) => vec![(PS_OVER, BLEND_PREMULTIPLIED, 0)],
            _ => vec![(PS_ADD, BLEND_ADD, 0)],
        },
    }
}

/// The water's alpha ramp as the game makes it (#4): FUN_00092df0 builds an `IRampSingle` from
/// the material's six key / value pairs (`RAMP_KEYS`, `RAMP_VALUES`) in mode 4 (RAMP_EASE), and
/// 0x92eb0 fills the ramp texture with it, texel i = trunc(ramp(i x RAMP_STEP) x 255),
/// RAMP_TEXELS of them. The ramp past its first and last keys holds their values. Read this way
/// it gives the archives' own baked ramps byte for byte (sdm_e10's pond h_fa12ff06, sdm_e40's
/// pools' h_1efc6e62 and ocean's h_15a80001, A8 128 x 1; a straight line between keys is off by
/// up to 25): the ease-in-out is inferred from that match, the ramp class's evaluation wasn't
/// read.
pub fn water_ramp(keys: [f32; 6], values: [f32; 6]) -> [u8; RAMP_TEXELS] {
    let at = |x: f32| -> f32 {
        if x <= keys[0] {
            return values[0];
        }
        for k in 0..5 {
            if x <= keys[k + 1] {
                let f = if keys[k + 1] > keys[k] { (x - keys[k]) / (keys[k + 1] - keys[k]) } else { 1.0 };
                return values[k] + (values[k + 1] - values[k]) * crate::bf::ale::ease(RAMP_EASE, f);
            }
        }
        values[5]
    };
    let mut out = [0u8; RAMP_TEXELS];
    for (i, texel) in out.iter_mut().enumerate() {
        *texel = (at(i as f32 * RAMP_STEP) * 255.0).clamp(0.0, 255.0) as u8;
    }
    out
}

/// Water material `id`'s ramp (`water_ramp`) from its six key / value pairs (0 where one is
/// missing).
fn material_ramp(game: &Game, id: u32) -> [u8; RAMP_TEXELS] {
    let pairs = |names: [u32; 6]| names.map(|n| game.material_constant(id, n).and_then(|v| v.first().copied()).unwrap_or(0.0));
    water_ramp(pairs(RAMP_KEYS), pairs(RAMP_VALUES))
}

/// A texture's frames as one 2D array image, as stored (Rgba8Unorm: the console's combiners
/// take the texels' bytes as they are, so the shader's sums and products are on those values,
/// src/liquid.wgsl): an animated texture's (`Game::flipbooks`: each frame its own texture,
/// `Flipbook::frame_textures`, the water's shimmer h_e757ea7c) or a plain texture as one frame,
/// each with its mip chain, sampled past its edges as `material` says. With the frame count and
/// the texture's own frame rate (the animated texture's h_e6375a47, 30 on all of the data's; 0
/// for a plain one). None if a frame doesn't decode or the frames' sizes differ.
fn frames_image(game: &mut Game, images: &mut Assets<Image>, material: u32, texture: u32) -> Option<(Handle<Image>, u32, f32)> {
    let (names, fps) = match game.flipbooks.get(&texture) {
        Some(b) if !b.frame_textures.is_empty() => (b.frame_textures.clone(), b.fps),
        _ => (vec![texture], 0.0),
    };
    let frames: Vec<Mips> = names.iter().map(|&t| game.texture_mips(t)).collect::<Option<_>>()?;
    let top = (frames[0].top().w, frames[0].top().h);
    if frames.iter().any(|m| (m.top().w, m.top().h) != top) {
        return None;
    }
    // (every frame's levels that halve as wgpu wants, as `mip_image` keeps them)
    let levels = frames.iter().map(|m| m.levels.iter().enumerate()
        .take_while(|(n, l)| l.w == (top.0 >> n).max(1) && l.h == (top.1 >> n).max(1) && l.px.len() == (l.w * l.h * 4) as usize)
        .count().max(1)).min().unwrap_or(1);
    let mut img = Image::new_uninit(bevy::render::render_resource::Extent3d { width: top.0, height: top.1, depth_or_array_layers: frames.len() as u32 },
                                    bevy::render::render_resource::TextureDimension::D2, TextureFormat::Rgba8Unorm, RenderAssetUsages::default());
    img.texture_descriptor.mip_level_count = levels as u32;
    // (layer by layer, each with its levels: wgpu's layer-major order, bevy's upload)
    img.data = Some(frames.iter().flat_map(|m| m.levels[..levels].iter().flat_map(|l| l.px.iter().copied())).collect());
    img.texture_view_descriptor = Some(bevy::render::render_resource::TextureViewDescriptor {
        dimension: Some(bevy::render::render_resource::TextureViewDimension::D2Array),
        ..default()
    });
    img.sampler = bevy::image::ImageSampler::Descriptor(crate::scene::world_sampler(game.texture_address(material, texture)));
    Some((images.add(img), frames.len() as u32, fps))
}

/// The water's ramp image (`water_ramp`): white, its alpha the ramp, stored as is (linear), clamped
/// past its ends (0x93165: stage 2's ADDRESSU and ADDRESSV CLAMP).
fn ramp_image(images: &mut Assets<Image>, ramp: &[u8; RAMP_TEXELS]) -> Handle<Image> {
    let px: Vec<u8> = ramp.iter().flat_map(|&a| [255, 255, 255, a]).collect();
    images.add(mip_image(&Mips::single(RAMP_TEXELS as u32, 1, px), TextureFormat::Rgba8Unorm, [Address::Clamp; 2]))
}

/// The materials a liquid surface draws with, one per pass (see `LiquidMaterial`), if `id` is a
/// liquid material class's; `opaque_water`: a water surface whose ramp is 1 all over its mesh
/// (`spawn_level`: the ponds and pools, at model height 1), drawn opaque (the game blends it by
/// alpha 1: the same picture, and it writes depth as the game's does); `key` the level's lava v0
/// (`Level::objects_key`'s colour; black without one). Made once per (id, opaque_water)
/// (`cache`).
///
/// Water (BF_OceanMaterialBase, 0x92f70 with vertex program 0x45 and pixel shader 0x2f): its
/// layers at uv x 1 / (`scale-u`, `scale-v`) + frac(clock x (`translate-u`, `translate-v`) /
/// scale), and the same with the `-1` constants (c26, c27; the names and order answer-spec Q8);
/// `texture-1` (and `texture-0`, were it animated) plays at the material's `framerate-1`
/// (`framerate`), FUN_00092d10 giving each animator its rate (0x262680: the loop is frames /
/// rate, 34 frames: sdm_e10's pond 17 fps, 2 s; the pools 25; the ocean 30), looping on the game's
/// clock (answer-spec Q10); colour (t0 x `color` + t1 x `color-1`) / 2 x the vertex light, which
/// the shader takes from the level's lights (src/liquid.wgsl `water_light`); alpha the ramp at the
/// vertex's own height in its mesh (`water_mesh`), SRCALPHA / INVSRCALPHA (0x93165). No
/// reflection: no render target or reflection camera exists (leads §6.3), and take20 of #73
/// mirrors nothing in the e40 sea (spec §1.5).
///
/// BF_LavaMaterial (FUN_00091fe0, vertex program 0x42): each layer at uv x 1 / its
/// `texture-scale` (`-1`) + clock x its `translate-u`, `translate-v` (`-1`) (c25, c26: not divided
/// by the scale), the clock from its `lifetime` (h_0f85824e; parameter table 0x91a10:
/// texture-scale h_06262ee7 +0x18, translate h_fb0bff34 / h_e002ae8e +0x1c / +0x20, blend-mode
/// h_0f5ae13f +0x24, texture-scale-1 h_ff6226e8 +0x28, translate-1 h_0e41af5b / h_0e2b9302 +0x2c /
/// +0x30, blend-mode-1 h_f3c8bfa2 +0x34, lifetime +0x38), its passes by `lava_passes`, unlit
/// (no vertex light: pixel shader 0x2b's v0 is the constant `key`, FUN_00091fe0's slot 24). The
/// wrapper material's h_e59d69a0 (lava 60, sludge 200) is the stencil value its pixels write for
/// the game's glare pass (answer-spec Q4), not an opacity: left out with that pass.
///
/// Test hooks: BF_LIQUID_TEX=<hex> puts one texture on every liquid layer; BF_LIQUID_DEBUG=1 draws
/// every liquid surface solid white; BF_LIQUID_LOG=1 prints each material's constants.
#[allow(clippy::too_many_arguments)]
fn liquid_materials(game: &mut Game, images: &mut Assets<Image>, liquids: &mut Assets<LiquidMaterial>,
                    cache: &mut HashMap<(u32, bool), Vec<Handle<LiquidMaterial>>>, id: u32, opaque_water: bool, key: [f32; 3]) -> Option<Vec<Handle<LiquidMaterial>>> {
    let ty = game.material_type(id);
    if ty != LIQUID_LAYERS && ty != LIQUID_WATER {
        return None;
    }
    if let Some(m) = cache.get(&(id, opaque_water)) {
        return Some(m.clone());
    }
    let water = ty == LIQUID_WATER;
    let num = |game: &Game, n: &str, or: f32| game.material_constant(id, h(n)).and_then(|v| v.first().copied()).unwrap_or(or);
    let colour = |game: &Game, n: &str| game.material_constant(id, h(n)).filter(|v| v.len() >= 3).map_or(Vec4::ONE, |v| Vec4::new(v[0], v[1], v[2], 1.0));
    let forced = std::env::var("BF_LIQUID_TEX").ok().and_then(|v| u32::from_str_radix(&v, 16).ok());
    // a layer's frames (`frames_image`), its count and rate; none for an empty slot
    let mut layer = |game: &mut Game, param: &str| game.material_param_texture(id, h(param)).filter(|&t| t != 0 && t != h(""))
        .map(|t| forced.unwrap_or(t)).and_then(|t| frames_image(game, images, id, t));
    let (l0, l1) = (layer(game, "texture-0"), layer(game, "texture-1"));
    let coords = |game: &Game, scale: [&str; 2], translate: [&str; 2]| {
        let inv = |n: &str| { let s = num(game, n, 1.0); if s.abs() > 1e-6 { 1.0 / s } else { 0.0 } };
        Vec4::new(inv(scale[0]), inv(scale[1]), num(game, translate[0], 0.0), num(game, translate[1], 0.0))
    };
    // (frame count, rate: the material's own for the water, FUN_00092d10; else the texture's)
    let anim = |l: &Option<(Handle<Image>, u32, f32)>, rate: Option<f32>, divided: bool|
        Vec4::new(l.as_ref().map_or(1.0, |x| x.1 as f32), rate.or(l.as_ref().map(|x| x.2)).unwrap_or(0.0), if divided { 1.0 } else { 0.0 }, 0.0);
    let mut params = LiquidParams::default();
    let (passes, start) = if water {
        params.coords0 = coords(game, ["scale-u", "scale-v"], ["translate-u", "translate-v"]);
        params.coords1 = coords(game, ["scale-u-1", "scale-v-1"], ["translate-u-1", "translate-v-1"]);
        params.anim0 = anim(&l0, Some(num(game, "framerate", 30.0)), true);
        params.anim1 = anim(&l1, Some(num(game, "framerate-1", 30.0)), true);
        params.color0 = colour(game, "color");
        params.color1 = colour(game, "color-1");
        (vec![(PS_WATER, if opaque_water { BLEND_NONE } else { BLEND_ALPHA }, 0)], 0.0)
    } else {
        params.coords0 = coords(game, ["texture-scale", "texture-scale"], ["translate-u", "translate-v"]);
        params.coords1 = coords(game, ["texture-scale-1", "texture-scale-1"], ["translate-u-1", "translate-v-1"]);
        params.anim0 = anim(&l0, None, false);
        params.anim1 = anim(&l1, None, false);
        params.color0 = Vec3::from(key).extend(1.0);
        let modes = (num(game, "blend-mode", 0.0) as i64, num(game, "blend-mode-1", 0.0) as i64);
        (lava_passes(modes.0, modes.1, l0.is_some(), l1.is_some()), num(game, "lifetime", 0.0))
    };
    let ramp = water.then(|| ramp_image(images, &material_ramp(game, id)));
    if std::env::var("BF_LIQUID_LOG").is_ok() {
        eprintln!("liquid material h_{id:08x} ({}): texture-0 {:x?} ({} frames) texture-1 {:x?} ({} frames), clock from {start}, passes {passes:x?} {params:?}",
                  if water { "water" } else { "BF_LavaMaterial" }, game.material_param_texture(id, h("texture-0")), params.anim0.x,
                  game.material_param_texture(id, h("texture-1")), params.anim1.x);
        // (the ramp made from the keys against the archive's own texture in the ramp's slot)
        if let Some(t) = game.material_param_texture(id, WATER_RAMP).filter(|_| water) {
            let made = material_ramp(game, id);
            let stored: Option<Vec<u8>> = game.texture_rgba(t).map(|(_, _, px)| px.chunks_exact(4).map(|p| p[3]).collect());
            let differ = stored.as_ref().map(|a| a.iter().zip(made.iter()).filter(|(x, y)| x != y).count());
            eprintln!("  ramp from its keys {:?}: the archive's h_{t:08x} differs in {differ:?} of {RAMP_TEXELS} texels", made.iter().step_by(8).collect::<Vec<_>>());
        }
    }
    // BF_LIQUID_DEBUG: every liquid surface solid white (where are they drawn?)
    let debug = std::env::var("BF_LIQUID_DEBUG").is_ok();
    let passes = if debug { vec![(0, BLEND_NONE, 0)] } else { passes };
    let made: Vec<Handle<LiquidMaterial>> = passes.iter().enumerate().map(|(k, &(shader, blend, layer))| {
        let surface = LiquidSurface {
            params: LiquidParams { shader, blend, layer, clock: start, ..params },
            layer0: l0.as_ref().map(|x| x.0.clone()),
            layer1: l1.as_ref().map(|x| x.0.clone()),
            ramp: ramp.clone(),
            start,
        };
        let base = StandardMaterial {
            alpha_mode: match blend {
                BLEND_ALPHA => AlphaMode::Blend,
                BLEND_PREMULTIPLIED => AlphaMode::Premultiplied,
                BLEND_ADD => AlphaMode::Add,
                _ => AlphaMode::Opaque,
            },
            // (Singe's lake lies on terrain painted with lava veins, centimetres under it: seen
            // low, the two fought and the terrain's veins showed through the lava; a second pass
            // sorts after the first, FUN_000920a0)
            depth_bias: LIQUID_DEPTH_BIAS + k as f32,
            // one-sided, as the game's CULLMODE is left at its default for them: take03 of #146
            // shows no lava surface from under it (26-41 s, the camera under the lava)
            double_sided: false,
            cull_mode: Some(bevy::render::render_resource::Face::Back),
            ..default()
        };
        liquids.add(LiquidMaterial { base, extension: surface })
    }).collect();
    cache.insert((id, opaque_water), made.clone());
    Some(made)
}

/// A water part's mesh as BF_OceanMaterialBase draws it (#4), from its shared mesh (`mesh_of`:
/// the geoset's own positions) and `node`, the part's world matrix (the object's and the part's):
/// every vertex at the node's world height, as vertex program 0x45 puts slot 28's y in place of
/// the vertex's world y (FUN_00093210 sets it to the draw's world matrix's translation y,
/// [matrix]+0x28), and the vertex's own height in the mesh (its model-space y, the ramp's
/// coordinate: stage 2's u) kept as the second uv. So sdm_e10's pond is drawn at 15.22 (its mesh
/// sits 1 m higher: model height 1), the e40 pools at 27.0-27.1 (the collision tops there), and
/// the e40 ocean at 22.10 while its collision's top is 23.10 (leads §6.3). The positions are put
/// back in the part's frame, so it stays under its object. None for a mesh without float
/// positions or a node that can't be inverted.
fn water_mesh(mesh: &Mesh, node: Mat4) -> Option<Mesh> {
    let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else { return None };
    if node.determinant().abs() < 1e-9 {
        return None;
    }
    let back = node.inverse();
    let height = node.w_axis.y;
    let flat: Vec<[f32; 3]> = positions.iter().map(|&p| {
        let mut w = node.transform_point3(Vec3::from(p));
        w.y = height;
        back.transform_point3(w).to_array()
    }).collect();
    let own: Vec<[f32; 2]> = positions.iter().map(|p| [p[1], 0.0]).collect();
    let mut m = mesh.clone();
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, flat);
    m.insert_attribute(Mesh::ATTRIBUTE_UV_1, own);
    Some(m)
}

/// Whether a water mesh's ramp is 1 all over it (`water_mesh`'s model heights, the ramp's texels
/// between the lowest and the highest, linearly sampled): the opaque ponds and pools.
fn water_opaque(mesh: &Mesh, ramp: &[u8; RAMP_TEXELS]) -> bool {
    let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else { return false };
    let (lo, hi) = positions.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| (lo.min(p[1]), hi.max(p[1])));
    if lo > hi {
        return false;
    }
    // (the texels a linear sample at those heights reads: u x 128 - 0.5, clamped)
    let texel = |u: f32| (u.clamp(0.0, 1.0) * RAMP_TEXELS as f32 - 0.5).clamp(0.0, (RAMP_TEXELS - 1) as f32);
    let (a, b) = (texel(lo).floor() as usize, texel(hi).ceil() as usize);
    ramp[a..=b.min(RAMP_TEXELS - 1)].iter().all(|&t| t == 255)
}

/// The level's point light at `p` from every direction (linear colour: each lamp's colour x
/// (1 - distance / range)), for what isn't a `LevelMaterial` (the characters): the outdoor pair's
/// lamps, as a character's light-usage is LU_OUTDOOR (535 of the 542 placed ones; FUN_0009f0b0
/// takes the lamps of its pair, `LevelLamp`).
pub fn lamp_light_at(level: &Level, p: Vec3) -> Vec3 {
    if std::env::var("BF_NO_LAMPS").is_ok() {
        return Vec3::ZERO;
    }
    level.lamps.iter().filter(|l| l.outdoor).filter_map(|l| {
        let d = l.at.distance(p);
        (d < l.range).then(|| Vec3::from(l.color.map(|c| Color::srgb(c, c, c).to_linear().red)) * (1.0 - d / l.range))
    }).sum()
}

/// The level materials' lights (`Lamps`): `level`'s pairs and point lights, or none (the flat
/// test floor). What a map's scenery is lit by, the test tools' placed scenery too (#185).
pub fn lamps_of(level: Option<&Level>, buffers: &mut Assets<ShaderStorageBuffer>) -> Lamps {
    let buffer = match level {
        Some(l) => light_buffer(l),
        None => no_lights(),
    };
    Lamps { lamps: buffers.add(buffer), ..default() }
}

/// A light buffer with no lights in it (`light_buffer`'s layout: its pairs empty, one lamp that
/// reaches nowhere, as a buffer can't be empty).
fn no_lights() -> ShaderStorageBuffer {
    ShaderStorageBuffer::new(&[0u8; LIGHTS_HEADER + LAMP_SIZE], RenderAssetUsages::default())
}

/// `light_buffer`'s pairs (lamps.wgsl `Pairs`): each pair's two directions and two colours, a
/// vec4 each: 128 bytes.
const LIGHTS_HEADER: usize = 128;
/// A lamp in `light_buffer` (lamps.wgsl `Lamp`): its place and range, its linear colour and
/// falloff kind, its pairs and padding: 48 bytes.
const LAMP_SIZE: usize = 48;
/// A lamp's pairs in `light_buffer`, as the game's light keeps them in its +0x94: bit 1 the
/// indoor pair's (h_e02aba1c), bit 2 the outdoor pair's (h_1f416687).
const LAMP_INDOOR: u32 = 2;
const LAMP_OUTDOOR: u32 = 4;

/// The level's lights for `LevelMaterial`s (lamps.wgsl's `LevelLights`): first the pairs
/// (`Level::pair`), the indoor pair then the outdoor one, each slot the way toward its light (w 1
/// when there's one) then its linear colour (bevy's conversion of the sRGB colour
/// `spawn_lighting` gives it). The outdoor pair are bevy's directional lights: an indoor material
/// finds them by both (a vision light may come from the same way) and swaps them for the indoor
/// pair (lamps.wgsl `indoor_swap`). Then per lamp its place, range, linear colour, falloff kind
/// and pairs (`LAMP_INDOOR`, `LAMP_OUTDOOR`); one lamp that reaches nowhere when there are none
/// (or BF_NO_LAMPS).
pub fn light_buffer(level: &Level) -> ShaderStorageBuffer {
    let linear = |c: f32| Color::srgb(c, c, c).to_linear().red;
    let mut bytes: Vec<u8> = Vec::with_capacity(LIGHTS_HEADER + LAMP_SIZE * level.lamps.len().max(1));
    let floats = |b: &mut Vec<u8>, v: &[f32]| v.iter().for_each(|f| b.extend(f.to_le_bytes()));
    let toward = |l: Option<&&LevelLight>| l.map_or([0.0; 4], |l| { let to = -l.dir; [to.x, to.y, to.z, 1.0] });
    let colour = |l: Option<&&LevelLight>| l.map_or([0.0; 4], |l| { let [r, g, b] = l.lit_color().map(linear); [r, g, b, 0.0] });
    for pair in [level.pair(true), level.pair(false)] {
        for k in 0..2 {
            floats(&mut bytes, &toward(pair.get(k)));
        }
        for k in 0..2 {
            floats(&mut bytes, &colour(pair.get(k)));
        }
    }
    let mut n = 0;
    for l in level.lamps.iter().filter(|_| std::env::var("BF_NO_LAMPS").is_err()) {
        let [r, g, b] = l.color.map(linear);
        floats(&mut bytes, &[l.at.x, l.at.y, l.at.z, l.range, r, g, b, l.falloff as f32]);
        let pairs = (if l.indoor { LAMP_INDOOR } else { 0 }) | (if l.outdoor { LAMP_OUTDOOR } else { 0 });
        bytes.extend(pairs.to_le_bytes());
        bytes.resize(LIGHTS_HEADER + LAMP_SIZE * (n + 1), 0);
        n += 1;
    }
    if n == 0 {
        bytes.resize(LIGHTS_HEADER + LAMP_SIZE, 0);
    }
    ShaderStorageBuffer::new(&bytes, RenderAssetUsages::default())
}

/// Material types that are alpha-tested (grass, fences, grates, signs, cables); the others use
/// the texture's alpha for shine or glow.
pub const CUTOUT_TYPES: [u32; 2] = [0x02DC_A948, 0x1C5F_7AAB];
/// The glow shader: a colour texture (its alpha a shine mask) and a glow texture
/// (h_e01baa40) that lights itself: light strips and panels, lava cracks in rock. All 211 of
/// its materials have one.
const GLOW_SHADER: u32 = 0x0E76_58E4;
const GLOW_TEXTURE: u32 = 0xE01B_AA40;
/// Lit shader with a tinted glow (h_1df292a1, 851 materials): colour texture, glow texture
/// h_e01baa40 tinted by its constant of the same name, added as light. The gate panels' green
/// button (h_f87dca06: glow 0.276 0.914 0.220) is one.
const TINTED_GLOW_SHADER: u32 = 0x1DF2_92A1;
/// Untextured glow material (pickups' glass shells): a colour constant and the wrapper's opacity.
pub const GLOW_SHELL: u32 = 0xF539_FE8C;
/// A flat untextured glow (the medkit's cross, h_031724e6: a 12-vertex quad on the case's top,
/// wrapper opacity 50) isn't a see-through shell: it's drawn at full strength, as in
/// todo/medkits glow.png where the crosses are bright blue. Flat: thinner than this (m).
const GLOW_DECAL_FLAT: f32 = 0.005;
/// (the full-strength variant of a glow material, in the material cache)
const GLOW_DECAL_KEY: u32 = 0x61D0_DECA;

/// Whether `g` is a flat untextured glow (see GLOW_DECAL_FLAT).
pub fn is_glow_decal(game: &Game, g: &Geoset) -> bool {
    if game.material_texture(g.material).is_some() || game.material_type(g.material) != GLOW_SHELL || g.positions.is_empty() {
        return false;
    }
    let (lo, hi) = g.positions.iter().fold((Vec3::MAX, Vec3::MIN), |(l, h), p| (l.min(Vec3::from(*p)), h.max(Vec3::from(*p))));
    (hi - lo).min_element() < GLOW_DECAL_FLAT
}

/// A placed level object's root entity: its element (inventory-object, game-object, ...),
/// object type and its own name (what the level's triggers call it; 0 for one put down by the
/// test tools).
#[derive(Component, Clone, Copy)]
pub struct Placed {
    pub tag: u32,
    pub kind: u32,
    pub name: u32,
}

/// A door or gate, in level object order (the same order as `Arena::doors`): its leaves and how
/// long it takes to open (its archetype's animation: 1.21 s for sdm_e34's doors, 4.96 s for the
/// gate, 3.29 s for m09's Shrike doors). Its sounds are the level's sound-triggers' (bf_play's
/// play_sound_triggers.rs).
pub struct Door {
    /// the object's name (what the level's triggers call it)
    pub name: u32,
    pub centre: Vec3,
    pub leaves: Vec<Leaf>,
    pub duration: f32,
}

/// A door leaf: its entity, its place with its joint at rest (the door shut, as the level places
/// it: the joint's parent-point + child-point and ORIENTATION, where the game's part sits until
/// a clip sets its joint: FUN_00235c00 gives a new joint a zero value) and how its clip moves it.
pub struct Leaf {
    pub entity: Entity,
    pub closed: Transform,
    pub motion: LeafMotion,
}

/// How a door leaf moves. Its archetype's animation set (named as the archetype) has a clip per
/// door, or per leaf (the gates), with a target per moving part naming the channel that sets
/// its joint (FUN_00234dc0); FUN_00234f90 places the part from its joint by the joint's type.
pub enum LeafMotion {
    /// A sliding joint (h_fd8f670c): `axis` (object frame) times its float channel, metres a
    /// frame of `interval` s (sdm_e34's doors and gates, the tutorial's); or `travel` m eased
    /// without one.
    Slide { axis: Vec3, travel: f32, curve: Vec<f32>, interval: f32 },
    /// A transform joint or a hinge, moved by its channel (`Animated`): m09's and m14's Shrike
    /// doors, m05's, m08's and e10's doors, m01's hatches, e10's ramp (a hinge).
    Animated(Animated),
}

/// Fallback opening time (s) of a door without an animation.
const DOOR_TIME: f32 = 0.8;

impl Leaf {
    /// How far a sliding leaf has slid `t` s into the opening (0 for the others).
    pub fn slide(&self, t: f32, duration: f32) -> f32 {
        let LeafMotion::Slide { travel, curve, interval, .. } = &self.motion else { return 0.0 };
        if curve.is_empty() {
            let k = (t / duration).clamp(0.0, 1.0);
            return travel * k * k * (3.0 - 2.0 * k);
        }
        let f = (t / interval.max(1e-3)).max(0.0);
        let i = (f as usize).min(curve.len() - 1);
        let j = (i + 1).min(curve.len() - 1);
        curve[i] + (curve[j] - curve[i]) * (f - i as f32).min(1.0)
    }

    /// The leaf's place in its object `t` s into the opening (`duration` s long): a sliding leaf
    /// slid along its axis, the others where their channel puts them (`Animated::pose`).
    pub fn pose(&self, t: f32, duration: f32) -> Transform {
        match &self.motion {
            LeafMotion::Slide { axis, .. } => Transform { translation: self.closed.translation + *axis * self.slide(t, duration), ..self.closed },
            LeafMotion::Animated(a) => {
                let (translation, rotation) = a.pose(t);
                Transform { translation, rotation, ..self.closed }
            }
        }
    }

    /// How far the leaf is from its closed place `t` s into the opening (m) and how far it's
    /// turned (degrees), for the log.
    pub fn moved(&self, t: f32, duration: f32) -> (f32, f32) {
        let p = self.pose(t, duration);
        (p.translation.distance(self.closed.translation), p.rotation.angle_between(self.closed.rotation).to_degrees())
    }
}

/// A part on a transform joint or a hinge that a clip of its archetype moves
/// (`animated_parts`): its joint and the keys of the channel that moves it.
#[derive(Clone)]
pub struct Animated {
    pub joint: PartJoint,
    pub keys: AnimKeys,
}

/// A channel's keys, (time s from the clip's start, value), a sampled channel's frames alike
/// (character.rs `channel_xform_keys`, `channel_angle_keys`).
#[derive(Clone)]
pub enum AnimKeys {
    /// a transform joint's: translation and rotation
    Xform(Vec<(f32, Vec3, Quat)>),
    /// a hinge's: the angle, radians
    Angle(Vec<(f32, f32)>),
}

/// Where `t` falls among `n` keys at `time(k)`: the keys either side and how far from the first
/// to the second, as the channel readers find them (0x291630 keyed, FUN_00291060 sampled): the
/// first key before the first, the last from the last on.
fn key_span(n: usize, time: impl Fn(usize) -> f32, t: f32) -> (usize, usize, f32) {
    if n == 0 || t <= time(0) {
        return (0, 0, 0.0);
    }
    if t >= time(n - 1) {
        return (n - 1, n - 1, 0.0);
    }
    let i = (0..n).take_while(|&k| time(k) <= t).last().unwrap_or(0).min(n - 2);
    let (a, b) = (time(i), time(i + 1));
    (i, i + 1, ((t - a) / (b - a).max(1e-9)).clamp(0.0, 1.0))
}

impl Animated {
    /// The part's place in its object `t` s into the clip (FUN_00234f90, by the joint's type):
    /// - a transform joint (h_f35d5b3c): at its child-point plus the channel's translation,
    ///   turned by the channel's rotation after the joint's ORIENTATION. Between keys the
    ///   translation goes straight and the rotation by a normalised blend (0x2914d0 ->
    ///   FUN_00290720; FUN_00233b10, as glam's `Quat::lerp`).
    /// - a hinge (h_e94eb66a): turned by the channel's angle (between keys the short way round,
    ///   0x292230; then clamped to the joint's limits, FUN_00234dc0) about its axis
    ///   (FUN_0001fc70) after the ORIENTATION, with its child-point on its parent-point.
    ///
    /// The object's root part is the object's frame (every part this moves hangs from it:
    /// `animated_parts`).
    pub fn pose(&self, t: f32) -> (Vec3, Quat) {
        let j = &self.joint;
        match &self.keys {
            AnimKeys::Xform(k) => {
                let (a, b, f) = key_span(k.len(), |i| k[i].0, t);
                let Some((ka, kb)) = k.get(a).zip(k.get(b)) else { return (j.child_point, j.orientation) };
                (j.child_point + ka.1.lerp(kb.1, f), ka.2.lerp(kb.2, f) * j.orientation)
            }
            AnimKeys::Angle(k) => {
                let (a, b, f) = key_span(k.len(), |i| k[i].0, t);
                let Some((&(_, va), &(_, vb))) = k.get(a).zip(k.get(b)) else { return (j.parent_point - j.orientation * j.child_point, j.orientation) };
                let d = vb - va;
                let vb = if d < -std::f32::consts::PI { vb + std::f32::consts::TAU } else if d > std::f32::consts::PI { vb - std::f32::consts::TAU } else { vb };
                // (up to the min, then down to the max, as FUN_00234dc0 takes them)
                let angle = (va + (vb - va) * f).max(j.limits.0).min(j.limits.1);
                // (FUN_0001fc70 divides by the axis' length: a hinge without one doesn't turn)
                let turn = j.axis.try_normalize().map_or(Quat::IDENTITY, |axis| Quat::from_axis_angle(axis, angle));
                let rotation = turn * j.orientation;
                (j.parent_point - rotation * j.child_point, rotation)
            }
        }
    }
}

/// The parts of archetype `arch` its clips move on transform joints and hinges, by part name:
/// each part on a joint of either type that a clip of its animation set (named as the archetype)
/// names with a channel of the joint's kind, a transform channel for a transform joint, an angle
/// one for a hinge (FUN_002340f0 plays no other on them), the first such clip's channel. Empty
/// when one of them hangs from another part than the root: the demo places a compound's parts
/// one level deep (weapon.rs), so a part on a moving part (m09_b's AA gun h_0849d1d8: its gun on
/// its turning mount) couldn't follow it, and the object stays as placed.
pub fn animated_parts(game: &Game, arch: u32) -> HashMap<u32, Animated> {
    let clips = game.anim_sets.get(&arch).map(|a| a.as_slice()).unwrap_or_default();
    let mut out = HashMap::new();
    if clips.is_empty() {
        return out;
    }
    for (name, joint) in WeaponModel::joints(game, arch) {
        let Some(joint) = joint.filter(|j| j.kind == TRANSFORM_JOINT || j.kind == HINGE_JOINT) else { continue };
        let keys = clips.iter().flat_map(|c| c.targets.iter()).filter(|t| t.0 == name).find_map(|t| match joint.kind {
            TRANSFORM_JOINT => game.channel_xform_keys(t.1).filter(|k| !k.is_empty()).map(AnimKeys::Xform),
            _ => game.channel_angle_keys(t.1).filter(|k| !k.is_empty()).map(AnimKeys::Angle),
        });
        let Some(keys) = keys else { continue };
        if joint.parent != h("root") {
            return HashMap::new();
        }
        out.insert(name, Animated { joint, keys });
    }
    out
}

/// The placed objects an anim-trigger of the level plays once (the trigger's first object,
/// FUN_00204bb0; an OPERATE entry with AC_PLAY or AC_RESUME, AM_FORWARD or AM_BACKWARD): the
/// doors among the objects with parts on transform joints or hinges (`animated_parts`). The
/// others only loop (AM_FORWARD_LOOP: the fans and other props of e01, e16, e23, m03_a and the
/// tutorial, m08_b's dish; the door survey's 217 looping props), which the demo's runtime doesn't
/// run yet (it plays a loop once: triggers.md "Not done"), so they stay as placed, as they did.
pub fn played_once(level: &Level) -> HashSet<u32> {
    level.triggers.iter().filter(|t| matches!(t.class, Class::Anim))
        .filter(|t| t.entries.iter().any(|e| e.action == act::OPERATE && [AC_PLAY, AC_RESUME].contains(&e.anim_command())
            && [AM_FORWARD, AM_BACKWARD].contains(&e.anim_mode())))
        .filter_map(|t| t.objects.first().copied()).collect()
}

/// Whether a placed object named `name` is a door by its transform and hinge parts (`moving`:
/// its archetype's `animated_parts`): it has some, and an anim-trigger plays it once
/// (`played`: `played_once`). `spawn_level` and `Arena::new` both number the doors by this and
/// by the sliding parts, so their lists line up.
pub fn animates(moving: &HashMap<u32, Animated>, played: &HashSet<u32>, name: u32) -> bool {
    !moving.is_empty() && played.contains(&name)
}

/// A sky layer: where the sky sits with the camera at the map's centre (its `<position>`), how
/// far its own origin is lifted above that to draw the layers in order (`make_sky`), how the sky
/// turns (`SkySpin`) and its clock (s of game time since it was put up). The sky moves along with
/// the camera across the map (so its mountains stay on the horizon), not up and down; the game's
/// stays at its `<position>` (FUN_00177d50), a few degrees of horizon parallax across a map.
#[derive(Component)]
pub struct SkyLayer {
    /// the sky's origin, its `<position>`
    pub at: Vec3,
    /// how far this layer's origin sits above the sky's, unturned (`make_sky`)
    pub lift: Vec3,
    /// how the sky turns
    pub spin: SkySpin,
    /// s of game time on the sky's clock (`run`)
    pub time: f64,
}

impl SkyLayer {
    /// Its clock run by `dt` s of game time: the game turns the sky in the world step with the
    /// step's dt (FUN_00177ba0 from FUN_000d9a20), so it stops while the game's clock does.
    pub fn run(&mut self, dt: f32) {
        self.time += dt as f64;
    }
}

/// Keep the sky around the camera and turned by its clock (see `SkyLayer`, `SkySpin`); add it
/// to a level app's PostUpdate, before the transforms propagate. The whole model turns about the
/// sky's origin (FUN_00177ba0 sets the instance's frame, FUN_00177d50 draws it at its position):
/// each layer's mesh sits at its lifted origin less the lift, so its origin is put at the sky's
/// plus the lift turned, and every layer stays where the others are as the sky tumbles (m09,
/// m14, mp3); for the levels that turn about y alone the lift, along y, doesn't move.
pub fn follow_sky(camera: Query<&GlobalTransform, With<Camera3d>>, mut sky: Query<(&SkyLayer, &mut Transform)>) {
    let Some(eye) = camera.iter().next().map(|c| c.translation()) else { return };
    for (s, mut tr) in &mut sky {
        let turn = s.spin.rotation(s.time);
        tr.translation = s.at + turn * s.lift + Vec3::new(eye.x, 0.0, eye.z);
        tr.rotation = turn;
    }
}

/// The camera's look, as the console drew it: no tone mapping, and an exposure at which a
/// light of 1.0 (a white directional light of PI lux, or ambient brightness 1) shows a texture
/// as it is (Bevy scales light by 1 / (1.2 x 2^ev100)).
pub fn console_look() -> (bevy::core_pipeline::tonemapping::Tonemapping, bevy::render::camera::Exposure) {
    (bevy::core_pipeline::tonemapping::Tonemapping::None, bevy::render::camera::Exposure { ev100: (1.0f32 / 1.2).log2() })
}

/// Point lights tuned for Bevy's default camera exposure (ev100 9.7) keep their look under
/// `console_look` when scaled by this (1.2 x 2^9.7 / (1.2 x 2^-0.263)).
pub const POINT_LIGHT_SCALE: f32 = 1.0 / 997.0;
/// How far apart (m) the sky layers' origins are stacked, to draw them in order (see the sky
/// in `spawn_level`).
const SKY_LAYER_STEP: f32 = 1000.0;

/// The level's own lighting (instead of a made-up sun): the outdoor pair (`Level::pair`) as
/// directional lights, and the object ambient. The game lights each surface texture x (ambient
/// + one light's colour x N.L + the other's) in gamma space (vertex program 0x19: oD0 = the
/// material's ambient x c14 + (max(0, n.c16) c17 + max(0, n.c18) c19) x its colour, with oB0 the
/// same for the back face), so the colours (x the lights' strength, `LevelLight::lit_color`) are
/// given as sRGB (Bevy works in linear light) at an illuminance of PI (see `console_look`). The
/// outdoor pair lights characters (LU_OUTDOOR) and the scenery placed outdoors; the scenery
/// placed indoors takes the indoor pair in its own shader (`Lamps::indoor`). The game's lights
/// cast no shadow; here the brightest of the pair (by its colour's luma) casts the characters'
/// (`luma`). A level with no directional light at all gets a made-up sun; one with no outdoor
/// pair (mp2: both its lights are indoor) none.
pub fn spawn_lighting(commands: &mut Commands, level: &Level) {
    let pair = level.pair(false);
    let shadow = pair.iter().enumerate().max_by(|a, b| luma(a.1.lit_color()).total_cmp(&luma(b.1.lit_color()))).map(|(i, _)| i);
    for (i, l) in pair.iter().enumerate() {
        let [r, g, b] = l.lit_color();
        let at = Transform::default().looking_to(l.dir.normalize_or(Vec3::NEG_Y), Vec3::Y);
        commands.spawn((DirectionalLight { illuminance: std::f32::consts::PI, color: Color::srgb(r, g, b), shadows_enabled: shadow == Some(i), ..default() },
                        at, Name::new(format!("outdoor light h_{:08x}", l.name))));
    }
    if level.lights.is_empty() {
        commands.spawn((DirectionalLight { illuminance: std::f32::consts::PI, color: Color::srgb(0.85, 0.85, 0.8), shadows_enabled: true, ..default() },
                        Transform::default().looking_to(Vec3::new(-0.45, -0.75, -0.35), Vec3::Y)));
    }
    let [r, g, b] = level.ambient;
    commands.insert_resource(AmbientLight { color: Color::srgb(r, g, b), brightness: 1.0, ..default() });
    if std::env::var("BF_LIGHT_LOG").is_ok() {
        log_lights(level, shadow.and_then(|i| pair.get(i)).map(|l| l.name));
    }
    // (the point lights are the level materials' own: see `LevelMaterial`)
}

/// Which light casts the characters' shadows (`spawn_lighting`): the brightest of the outdoor
/// pair, by this luma of its colour (Rec. 601, as the game's glare measures a picture's,
/// FUN_000e5f60). The game's own lights cast none (FUN_000dcc10 hands the shaders a direction and
/// a colour, nothing else): the demo keeps one for the characters, a choice.
fn luma(c: [f32; 3]) -> f32 {
    0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2]
}

/// BF_LIGHT_LOG: every light-object of the level with its type and kind, the two pairs, the
/// ambients, and the light-objects that aren't drawn.
fn log_lights(level: &Level, shadow: Option<u32>) {
    let flags = |indoor: bool, outdoor: bool| match (indoor, outdoor) {
        (true, true) => "indoor+outdoor",
        (true, false) => "indoor",
        (false, true) => "outdoor",
        (false, false) => "no pair",
    };
    eprintln!("lights: {} directional, {} point, {} not drawn", level.lights.len(), level.lamps.len(), level.other_lights.len());
    for l in &level.lights {
        let to = -l.dir;
        eprintln!("  directional h_{:08x} type h_{:08x}  {:14}  colour {:.2} {:.2} {:.2} x {:.3}  toward {:5.2} {:5.2} {:5.2}{}", l.name, l.kind,
                  flags(l.indoor, l.outdoor), l.color[0], l.color[1], l.color[2], l.strength, to.x, to.y, to.z,
                  if shadow == Some(l.name) { "  (casts the characters' shadow)" } else { "" });
    }
    for l in &level.lamps {
        eprintln!("  point h_{:08x} type h_{:08x}  {:14}  at {:7.1} {:6.1} {:7.1}  colour {:.2} {:.2} {:.2}  range {:4.1}  falloff {}",
                  l.name, l.kind, flags(l.indoor, l.outdoor), l.at.x, l.at.y, l.at.z, l.color[0], l.color[1], l.color[2], l.range, l.falloff);
    }
    for l in &level.other_lights {
        eprintln!("  not drawn h_{:08x} type h_{:08x}: {}", l.name, l.kind, l.why);
    }
    let names = |indoor: bool| level.pair(indoor).iter().map(|l| format!("h_{:08x}", l.name)).collect::<Vec<_>>().join(" ");
    eprintln!("  outdoor pair: [{}]  indoor pair: [{}]", names(false), names(true));
    let slots = |indoor: bool| level.slots(indoor).map(|l| l.map_or("-".to_string(), |l| format!("h_{:08x}", l.name))).join(" / ");
    eprintln!("  slots 0 / 1 (FUN_000dcc10): outdoor {}, indoor {} (the outdoor slot 0 lights the terrain's bump map and is the lava's v0)",
              slots(false), slots(true));
    let [o, c, t] = [level.ambient, level.character_ambient, level.terrain_ambient];
    eprintln!("  ambient: object {:.3} {:.3} {:.3}, character {:.3} {:.3} {:.3} (not drawn: characters take the object's), terrain {:.3} {:.3} {:.3}",
              o[0], o[1], o[2], c[0], c[1], c[2], t[0], t[1], t[2]);
}

/// How the baked terrain light map lies on the grid (see `Level::baked_light`): blocks row by
/// row, cells row by row, x and z increasing; found by correlating it with the terrain's own
/// slope lighting (sdm_e34 0.26, sdm_m03 0.39; every other orientation near 0).
const TERRAIN_LIGHT_ORIENTATION: u8 = 0;

/// How much dimmer the terrain's lights are than the objects' (per channel, at most 1), for a
/// terrain without a baked light map (none of the game's 54 levels): its materials are tinted by
/// it, as bevy's directional lights light everything. Read as before #143: the terrain's light is
/// the brightest of the lights marked h_e02aba1c, the objects' the brightest of the outdoor pair
/// (the code shows h_e02aba1c is the indoor pair's mark; what lights the terrain is #181's).
fn terrain_tint(level: &Level) -> Color {
    let brightest = |l: &mut dyn Iterator<Item = &LevelLight>| l.max_by(|a, b| luma(a.color).total_cmp(&luma(b.color))).map(|l| l.color);
    match (brightest(&mut level.lights.iter().filter(|l| l.indoor)), brightest(&mut level.pair(false).into_iter())) {
        (Some(t), Some(o)) => Color::srgb((t[0] / o[0].max(0.01)).min(1.0), (t[1] / o[1].max(0.01)).min(1.0), (t[2] / o[2].max(0.01)).min(1.0)),
        _ => Color::WHITE,
    }
}

/// The colour of the lights marked h_e02aba1c with the terrain's ambient (read as the terrain's
/// own lights before #143; the code shows they're the indoor pair, and the terrain's light is
/// #181's), scaled to unit brightness and given in linear terms: the baked light map says how
/// bright the ground is; these lights give it their hue. (Bulgar's are blue-grey: 0.44 0.57
/// 0.65, ambient 0.13 0.16 0.24 - the captures' ground is grey, not sand-brown.) Kept as it was
/// calibrated (TERRAIN_HUE).
fn terrain_hue(level: &Level) -> [f32; 3] {
    let mut c = level.terrain_ambient;
    for l in level.lights.iter().filter(|l| l.indoor) {
        for k in 0..3 {
            c[k] += l.color[k];
        }
    }
    let lum = 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2];
    if lum < 1e-3 {
        return [1.0; 3];
    }
    c.map(|v| (1.0 + TERRAIN_HUE * (v / lum - 1.0)).powf(2.2))
}

/// How much of the terrain lights' hue the ground takes (1: all). Calibrated on a capture of
/// Bulgar's gate: its ground measures 73 81 79 where the untinted sand was 79 74 64.
const TERRAIN_HUE: f32 = 0.6;

/// Whether a geoset is drawn: placeholder meshes (power-ups' cube) name a material no level
/// defines; the game draws nothing for them.
pub fn visible(game: &Game, g: &Geoset) -> bool {
    game.has_material(g.material)
}

/// A texture repeating past its edges, with its mip levels (`scene::mip_image`).
fn image(images: &mut Assets<Image>, mips: &Mips) -> Handle<Image> {
    image_with(images, mips, [Address::Wrap; 2])
}

/// A texture sampled past its edges as its material says (`Game::texture_address`), with its
/// mip levels (`scene::mip_image`).
fn image_with(images: &mut Assets<Image>, mips: &Mips, address: [Address; 2]) -> Handle<Image> {
    images.add(mip_image(mips, TextureFormat::Rgba8UnormSrgb, address))
}

/// A level material: its colour texture; for the cut-out types (and `cutout`) the texture's
/// alpha, or its "alpha" texture's, cuts it out (a terrain layer's "alpha" is its blend mask).
fn material(game: &mut Game, images: &mut Assets<Image>, materials: &mut Assets<LevelMaterial>, lamps: &Lamps,
            cache: &mut HashMap<u32, Handle<LevelMaterial>>, id: u32, unlit: bool, cutout: bool) -> Handle<LevelMaterial> {
    if let Some(m) = cache.get(&id) {
        return m.clone();
    }
    let mut mat = StandardMaterial { base_color: Color::srgb(0.6, 0.6, 0.6), perceptual_roughness: 0.9, reflectance: 0.2,
                                     unlit, fog_enabled: !unlit, ..default() };
    // untextured glow shells (pickups' glass): their glow colour, see-through by the wrapper's
    // opacity, added to the picture
    if game.material_texture(id).is_none() && game.material_type(id) == GLOW_SHELL {
        let c = game.material_constant(id, 0xE01B_AA40).filter(|c| c.len() >= 3).map_or([0.1, 0.35, 0.8], |c| [c[0], c[1], c[2]]);
        let a = game.material_constant(id, H_WRAP_ALPHA).and_then(|a| a.first().copied()).unwrap_or(255.0) / 255.0;
        mat.base_color = Color::srgba(c[0], c[1], c[2], a);
        mat.alpha_mode = AlphaMode::Add;
        mat.unlit = true;
    }
    let kind = game.material_type(id);
    if kind == GLOW_SHADER || kind == TINTED_GLOW_SHADER {
        if let Some(t) = game.material_param_texture(id, GLOW_TEXTURE) {
            let address = game.texture_address(id, t);
            let tint = match kind {
                TINTED_GLOW_SHADER => game.material_constant(id, GLOW_TEXTURE).filter(|c| c.len() >= 3)
                    .map_or(LinearRgba::WHITE, |c| Color::srgb(c[0], c[1], c[2]).into()),
                _ => LinearRgba::WHITE,
            };
            if let Some(mips) = game.texture_mips(t) {
                mat.emissive = tint;
                mat.emissive_texture = Some(image_with(images, &mips, address));
            }
        }
    }
    if std::env::var("BF_MAT_LOG").is_ok() {
        let tex = game.material_texture(id);
        let avg = tex.and_then(|t| game.texture_rgba(t)).map(|(w, h, px)| {
            let n = (w * h) as f32;
            let s = px.chunks_exact(4).fold([0f32; 4], |a, p| [a[0] + p[0] as f32, a[1] + p[1] as f32, a[2] + p[2] as f32, a[3] + p[3] as f32]);
            (w, h, s.map(|v| (v / n) as u8))
        });
        eprintln!("material h_{id:08x} type h_{:08x} texture {:x?} alpha {:x?} {:?} glow {:?} wrap alpha {:?}", game.material_type(id), tex,
                  game.material_alpha_texture(id), avg, game.material_constant(id, GLOW_TEXTURE), game.material_constant(id, H_WRAP_ALPHA));
    }
    let address = game.material_texture(id).map_or_else(Default::default, |t| game.texture_address(id, t));
    if let Some(mut mips) = game.material_texture(id).and_then(|t| game.texture_mips(t)) {
        if cutout && CUTOUT_TYPES.contains(&game.material_type(id)) {
            if let Some(alpha) = game.material_alpha_texture(id).and_then(|a| game.texture_mips(a)) {
                mips.take_alpha(&alpha);
            }
            mat.alpha_mode = AlphaMode::Mask(0.5);
            mat.double_sided = true;
            mat.cull_mode = None;
        } else {
            mips.map_texels(|p| p[3] = 255);
        }
        mat.base_color = Color::WHITE;
        mat.base_color_texture = Some(image_with(images, &mips, address));
        // a self-lit shell with a picture (its h_e01baa40): added to the scene, like the
        // untextured ones
        if game.material_type(id) == GLOW_SHELL {
            let a = game.material_constant(id, H_WRAP_ALPHA).and_then(|a| a.first().copied()).unwrap_or(255.0) / 255.0;
            mat.base_color = Color::srgba(1.0, 1.0, 1.0, a);
            mat.alpha_mode = AlphaMode::Add;
            mat.unlit = true;
        }
    }
    let h = materials.add(LevelMaterial { base: mat, extension: lamps.clone() });
    cache.insert(id, h.clone());
    h
}

/// Grass: see-through cards (a cut-out material) standing upright with their normals lying
/// flat (Bulgar's clumps: every normal's y is 0). Lit by those, the cards caught almost none of
/// the high key light and their backs none at all: black. Like most games' grass they're lit as
/// the ground under them is, by normals pointing up (`upright_normals`).
fn is_grass(game: &Game, g: &Geoset) -> bool {
    CUTOUT_TYPES.contains(&game.material_type(g.material)) && !g.normals.is_empty()
        && g.normals.iter().all(|n| n[1].abs() < GRASS_FLAT)
}

/// How far from flat a grass card's normals may lean (|y|), see `is_grass`.
const GRASS_FLAT: f32 = 0.1;
/// (the grass variant of a material, in the material cache)
const GRASS_KEY: u32 = 0x6752_A55E;

fn upright_normals(g: &Geoset) -> Geoset {
    Geoset { normals: vec![[0.0, 1.0, 0.0]; g.normals.len()], positions: g.positions.clone(), uvs: g.uvs.clone(), joints: g.joints.clone(),
             weights: g.weights.clone(), indices: g.indices.clone(), material: g.material, indoor: g.indoor, frames: g.frames.clone() }
}

/// `flip`: reverse the winding (terrain triangles face the other way from objects'). Otherwise
/// each triangle is wound to face along its vertices' normals: mirrored meshes (same index order,
/// mirrored positions) would otherwise be culled from the front.
fn mesh_of(g: &Geoset, colors: Option<Vec<[f32; 4]>>, lift: f32, flip: bool) -> Mesh {
    let positions: Vec<[f32; 3]> = g.positions.iter().map(|p| [p[0], p[1] + lift, p[2]]).collect();
    let mut indices = g.indices.clone();
    if flip {
        indices.chunks_exact_mut(3).for_each(|t| t.swap(1, 2));
    } else if g.normals.len() == g.positions.len() {
        for t in indices.chunks_exact_mut(3) {
            let [a, b, c] = [t[0], t[1], t[2]].map(|i| Vec3::from(g.positions[i as usize]));
            let n: Vec3 = [t[0], t[1], t[2]].iter().map(|&i| Vec3::from(g.normals[i as usize])).sum();
            if (b - a).cross(c - a).dot(n) < 0.0 {
                t.swap(1, 2);
            }
        }
    }
    let mut m = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, g.normals.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, g.uvs.clone())
        .with_inserted_indices(Indices::U32(indices));
    if let Some(c) = colors {
        m.insert_attribute(Mesh::ATTRIBUTE_COLOR, c);
    }
    m
}

/// A terrain geoset's tangents for its bump map (`terrain_bump`): each vertex's tangent frame
/// (`Geoset::frames`, world to tangent) taken back to the world, its x axis (the way the colour's
/// u grows) with w 1, as bevy keeps a tangent: the frame's y axis is the normal x that axis (a
/// quaternion's frame is never mirrored), the way v falls. The shader makes it perpendicular to
/// the vertex normal, which the frame's z axis is within 8 degrees of on most vertices
/// (TERRAIN_FRAME).
fn terrain_tangents(g: &Geoset) -> Vec<[f32; 4]> {
    g.frames.iter().map(|q| (Quat::from_array(*q).inverse() * Vec3::X).extend(1.0).to_array()).collect()
}

/// A terrain layer's mask at world (x, z), bilinear: one mask over the whole terrain (`half` m
/// either side of the origin).
fn mask_at(mask: &(u32, u32, Vec<u8>), x: f32, z: f32, half: f32) -> f32 {
    let (w, h, px) = mask;
    let u = (x + half) / (2.0 * half) * *w as f32 - 0.5;
    let v = (1.0 - (z + half) / (2.0 * half)) * *h as f32 - 0.5;
    let at = |i: i32, j: i32| {
        let (i, j) = (i.clamp(0, *w as i32 - 1) as u32, j.clamp(0, *h as i32 - 1) as u32);
        px[((j * w + i) * 4 + 3) as usize] as f32 / 255.0
    };
    let (i, j) = (u.floor() as i32, v.floor() as i32);
    let (fu, fv) = (u - u.floor(), v - v.floor());
    let top = at(i, j) * (1.0 - fu) + at(i + 1, j) * fu;
    let bottom = at(i, j + 1) * (1.0 - fu) + at(i + 1, j + 1) * fu;
    top * (1.0 - fv) + bottom * fv
}

/// One drawn geoset of a placed object, made ready as `spawn_level` draws a level's objects: its
/// mesh and material, its part's placement in the object, the part's slide (a door leaf's axis
/// and travel), the part's name (what animation targets name), the geoset's material id and its
/// indoor flag (`Geoset::indoor`).
#[derive(Clone)]
pub struct ObjectPart {
    pub mesh: Handle<Mesh>,
    pub material: Handle<LevelMaterial>,
    pub at: Transform,
    pub slide: Option<(Vec3, f32)>,
    pub name: u32,
    pub material_id: u32,
    pub indoor: bool,
}

/// Whether a placed object's geoset is lit by the level's indoor pair: the wrapper material's
/// draw 0x9f3c0 picks the pair by the object's light-usage (its +0x44, 0x9f3ff-0x9f42f): LU_INDOOR
/// the indoor one, LU_OUTDOOR the outdoor one, LU_GEOSET the geoset's own flag (`indoor`). An
/// object without one (LU_NONE: 294 of m09_a's placements, 165 of m09_x's, 20 of m14_b's) uploads
/// no pair and keeps the last one drawn's; here the outdoor pair, the one a view's draw starts
/// with (FUN_000dcc10(0, ..) at 0xda2aa).
pub fn indoor_lit(light_usage: u8, indoor: bool) -> bool {
    match light_usage {
        LU_INDOOR => true,
        LU_GEOSET => indoor,
        _ => false,
    }
}

/// The indoor-lit variant of a level material (`Lamps::indoor`), made once per material
/// (`cache`, by the material's asset id).
fn indoor_material(materials: &mut Assets<LevelMaterial>, cache: &mut HashMap<AssetId<LevelMaterial>, Handle<LevelMaterial>>,
                   material: &Handle<LevelMaterial>) -> Handle<LevelMaterial> {
    if let Some(m) = cache.get(&material.id()) {
        return m.clone();
    }
    let Some(mut m) = materials.get(material).cloned() else { return material.clone() };
    m.extension.indoor = true;
    let h = materials.add(m);
    cache.insert(material.id(), h.clone());
    h
}

/// An archetype's drawn geosets, each part's most detailed mesh, as a level draws its placed
/// objects (`spawn_level`; the test tools' scenery from any level, #185): the materials no level
/// defines left out (`visible`), cut-outs and glows by their material type, grass lit from above,
/// flat glow decals at full strength. `cache` keeps the materials made, by id (and the grass and
/// glow-decal variants), across archetypes of one `game`.
pub fn object_parts(game: &mut Game, arch: u32, meshes: &mut Assets<Mesh>, materials: &mut Assets<LevelMaterial>, images: &mut Assets<Image>,
                    lamps: &Lamps, cache: &mut HashMap<u32, Handle<LevelMaterial>>) -> Vec<ObjectPart> {
    let mut parts = vec![];
    let Ok(m) = WeaponModel::load(game, arch) else { return parts };
    for p in &m.parts {
        // a sliding leaf travels its own length along its axis (fully clear)
        let slide = p.slide_axis.map(|axis| {
            let along = p.geosets.iter().flat_map(|g| &g.positions).map(|v| (p.rotation * Vec3::from(*v)).dot(axis));
            let (lo, hi) = along.fold((f32::MAX, f32::MIN), |(lo, hi), d| (lo.min(d), hi.max(d)));
            (axis, (hi - lo).max(0.0) + 0.1)
        });
        let shown: Vec<&Geoset> = p.geosets.iter().filter(|g| visible(game, g)).collect();
        for g in shown {
            let mut mat = material(game, images, materials, lamps, cache, g.material, false, true);
            let grass = is_grass(game, g);
            if grass {
                // both faces drawn, both lit from above (a double-sided material turns a back
                // face's normal down)
                let key = g.material ^ GRASS_KEY;
                mat = match cache.get(&key) {
                    Some(m) => m.clone(),
                    None => {
                        let mut m = materials.get(&mat).cloned().unwrap_or_default();
                        m.base.double_sided = false;
                        m.base.cull_mode = None;
                        let m = materials.add(m);
                        cache.insert(key, m.clone());
                        m
                    }
                };
            }
            if is_glow_decal(game, g) {
                let key = g.material ^ GLOW_DECAL_KEY;
                mat = match cache.get(&key) {
                    Some(m) => m.clone(),
                    None => {
                        let mut m = materials.get(&mat).cloned().unwrap_or_default();
                        m.base.base_color.set_alpha(1.0);
                        m.base.depth_bias = 1.0;
                        let m = materials.add(m);
                        cache.insert(key, m.clone());
                        m
                    }
                };
            }
            let mesh = if grass { mesh_of(&upright_normals(g), None, 0.0, false) } else { mesh_of(g, None, 0.0, false) };
            parts.push(ObjectPart { mesh: meshes.add(mesh), material: mat, at: Transform::from_translation(p.offset).with_rotation(p.rotation),
                                    slide, name: p.name, material_id: g.material, indoor: g.indoor });
        }
    }
    parts
}

/// Spawns `parts` (`object_parts`) under `root`, each with its own material or `material` in its
/// place (a see-through copy's). The scenery casts no shadow: the game lights it per vertex by
/// the level's lights (all `enable-static`), unshadowed, so building interiors are lit like the
/// walls outside (a shadow-casting roof made them black) and a jungle canopy doesn't shade the
/// whole floor; characters still cast theirs.
pub fn spawn_object_parts(commands: &mut Commands, parts: &[ObjectPart], root: Entity, mut material: impl FnMut(&ObjectPart) -> Handle<LevelMaterial>) {
    for p in parts {
        commands.spawn((Mesh3d(p.mesh.clone()), MeshMaterial3d(material(p)), p.at, NotShadowCaster, ChildOf(root)));
    }
}

/// Spawn the level's terrain, placed objects and sky; returns the doors (closed).
pub fn spawn_level(commands: &mut Commands, game: &mut Game, level: &Level, meshes: &mut Assets<Mesh>,
                   materials: &mut Assets<LevelMaterial>, liquids: &mut Assets<LiquidMaterial>, buffers: &mut Assets<ShaderStorageBuffer>,
                   images: &mut Assets<Image>) -> Vec<Door> {
    let mut doors = vec![];
    let mut cache = HashMap::new();
    let mut liquid_cache = HashMap::new();
    // the liquids' clock starts with their materials (`LiquidClock`), and BF_LavaMaterial's v0 is
    // the level's key light's colour (`Level::objects_key`; black without one)
    commands.insert_resource(LiquidClock(0.0));
    let lava_key = level.objects_key().map_or([0.0; 3], |l| l.color);
    let lamps = Lamps { lamps: buffers.add(light_buffer(level)), ..default() };
    // (the placed objects' indoor-lit materials, `indoor_material`)
    let mut indoor_cache = HashMap::new();
    let mut indoor_parts = 0;
    // terrain: the ground, then each layer blended over it by its mask (baked into vertex alpha)
    for (k, (g, mask)) in level.terrain.iter().enumerate() {
        let base = material(game, images, materials, &lamps, &mut cache, g.material, false, false);
        // lit by its baked light map (texture x light, in gamma space: the vertex colour is that
        // light in linear terms); without one, by the level's lights tinted for the terrain
        let baked = level.terrain_light.is_some();
        // its bump map (`terrain_bump`): the normal map, linear (normals, not colours), with the
        // vertices' tangent frames; only over the baked light, which it takes up or down
        let bump = terrain_bump(game, level, g.material).filter(|_| baked && g.frames.len() == g.positions.len()).and_then(|(b, t)| {
            let address = game.texture_address(g.material, t);
            game.bump_mips(t).map(|mips| (b, images.add(mip_image(&mips, TextureFormat::Rgba8Unorm, address))))
        });
        let base = match materials.get(&base).cloned() {
            Some(mut m) => {
                if baked {
                    m.base.unlit = true;
                    m.base.fog_enabled = true;
                } else {
                    m.base.base_color = terrain_tint(level);
                }
                // its colour and its detail, mixed by the eye's distance (`terrain_detail`):
                // sampled by the layer's own shader in place of the standard material's texture
                if let Some((c20, detail)) = terrain_detail(game, g.material) {
                    let colour = m.base.base_color_texture.take();
                    let same = game.material_texture(g.material) == Some(detail);
                    let detail_image = if same { colour.clone() } else {
                        let address = game.texture_address(g.material, detail);
                        game.texture_mips(detail).map(|mips| image_with(images, &mips, address))
                    };
                    if colour.is_some() && detail_image.is_some() {
                        m.extension.detail = c20;
                        m.extension.colour = colour;
                        m.extension.detail_texture = detail_image;
                    } else {
                        m.base.base_color_texture = colour;
                    }
                }
                if let Some((b, image)) = &bump {
                    m.extension.bump = *b;
                    m.extension.bump_texture = Some(image.clone());
                }
                materials.add(m)
            }
            None => base,
        };
        let hue = terrain_hue(level);
        let light = |p: &[f32; 3]| -> f32 {
            if !baked {
                return 1.0;
            }
            // the cells around the vertex
            let c = level.terrain_light.as_ref().map_or(2.0, |t| t.1) * 0.5;
            let near = [(-c, -c), (c, -c), (-c, c), (c, c)].into_iter().filter_map(|(dx, dz)| level.baked_light(p[0] + dx, p[2] + dz, TERRAIN_LIGHT_ORIENTATION));
            let (sum, n) = near.fold((0.0, 0), |(s, n), v| (s + v, n + 1));
            let v = if n > 0 { sum / n as f32 } else { 0.8 };
            v.powf(2.2)
        };
        let (mut mesh, mat) = match mask.and_then(|m| game.texture_rgba(m)) {
            Some(m) => {
                let colors = g.positions.iter().map(|p| { let l = light(p); [l * hue[0], l * hue[1], l * hue[2], mask_at(&m, p[0], p[2], level.terrain_half)] }).collect();
                let mut layer = materials.get(&base).cloned().unwrap_or_default();
                layer.base.alpha_mode = AlphaMode::Blend;
                // drawn over the ground and the layers under it (depth-test bias 10 each), sorted
                // before every other transparent thing (see TERRAIN_LAYER_SORT)
                layer.base.depth_bias = TERRAIN_LAYER_SORT + 10.0 * k as f32;
                layer.extension.layer_bias = Some(10 * k as i32);
                (mesh_of(g, Some(colors), 0.02 * k as f32, true), materials.add(layer))
            }
            None => {
                let colors = baked.then(|| g.positions.iter().map(|p| { let l = light(p); [l * hue[0], l * hue[1], l * hue[2], 1.0] }).collect());
                (mesh_of(g, colors, 0.0, true), base)
            }
        };
        if bump.is_some() {
            mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, terrain_tangents(g));
        }
        // it casts no shadow, as the scenery doesn't (`spawn_object_parts`): the game's lights
        // cast none, so its characters are lit by the outdoor pair wherever they stand, on ground
        // the baked light map has in shade too (todo/73 take17: Tex lit on e40's dark beach, where
        // the terrain's shadow drew him black)
        commands.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(mat), Transform::default(), NotShadowCaster, Name::new(format!("terrain layer {k}"))));
    }
    // placed objects
    // per archetype: its geosets with their part placement, and the part's slide (axis, travel);
    // and its parts on transform joints and hinges that its clips move
    let mut models: HashMap<u32, Vec<ObjectPart>> = HashMap::new();
    let mut animated: HashMap<u32, HashMap<u32, Animated>> = HashMap::new();
    // (the objects the level's anim-triggers play once: those of them with animated parts are
    // doors, see `played_once`)
    let played = played_once(level);
    // (the spawn-triggers' templates aren't in the world: FUN_000dfe90 takes each out as the
    // level links, and a trigger's copy is made at the trigger (FUN_000dfd90), so they aren't
    // drawn where they're placed, as `Arena::new` leaves out their collision: #230, the
    // tutorial's h_e5ab44a9 at (50.3, 25.6, 80.6) stood there as a walk-through ghost. Not the
    // pickups: every pickup of the multiplayer maps is a template (sdm_e34's 54), whose copies
    // the triggers don't make yet, so the demo keeps them where they're placed, as before)
    let templates = level.spawn_templates();
    let pickup = |tag: u32| tag == h("inventory-object") || tag == h("weapon-object");
    for o in &level.objects {
        let Some(arch) = o.archetype else { continue };
        if templates.contains(&o.name) && !pickup(o.tag) {
            continue;
        }
        if !models.contains_key(&arch) {
            let parts = object_parts(game, arch, meshes, materials, images, &lamps, &mut cache);
            models.insert(arch, parts);
            animated.insert(arch, animated_parts(game, arch));
        }
        if std::env::var("BF_LIQUID_LOG").is_ok() && o.tag == crate::bf::character::H_LIQUID_OBJECT {
            if let Ok(m) = WeaponModel::load(game, arch) {
                for (p, g) in m.parts.iter().flat_map(|p| p.geosets.iter().map(move |g| (p, g))) {
                    let id = g.material;
                    let world: Vec<Vec3> = g.positions.iter().map(|v| o.transform.transform_point3(p.offset + p.rotation * Vec3::from(*v))).collect();
                    let (lo, hi) = world.iter().fold((Vec3::MAX, Vec3::MIN), |(a, b), v| (a.min(*v), b.max(*v)));
                    let (ulo, uhi) = g.uvs.iter().fold((Vec2::MAX, Vec2::MIN), |(a, b), v| (a.min(Vec2::from(*v)), b.max(Vec2::from(*v))));
                    eprintln!("  surface world bounds {lo:.2} .. {hi:.2}  uv {ulo:.2} .. {uhi:.2}");
                    let consts = game.material_constants.get(&id).map(|c| c.iter().map(|(k, v)| format!("{k:08x}={v:?}")).collect::<Vec<_>>().join(" "));
                    eprintln!("liquid h_{:08x} at {:.1}: material h_{id:08x} type h_{:08x} textures {:x?} consts {:?} ({} vertices)",
                              o.kind, o.transform.w_axis.truncate(), game.material_type(id), game.materials.get(&id), consts, g.positions.len());
                }
            }
        }
        let e = commands.spawn((Transform::from_matrix(o.transform), Visibility::default(), Name::new(format!("h_{:08x}", o.kind)),
                                Placed { tag: o.tag, kind: o.kind, name: o.name })).id();
        // the archetype's animation (archetype-set of the same name) moves the leaves: a
        // channel per part
        // a liquid's surfaces, and any other part whose material is of a liquid material class
        // (the class draws it, whatever the object): the game's liquid materials
        // (`liquid_materials`), a water part's mesh flattened to its node's height with its own
        // heights kept for the ramp (`water_mesh`), opaque where the ramp is 1 all over it
        // (`water_opaque`)
        let liquid = o.tag == crate::bf::character::H_LIQUID_OBJECT;
        let surfaces: Vec<Option<(Handle<Mesh>, Vec<Handle<LiquidMaterial>>)>> = models[&arch].iter().map(|p| {
            if ![LIQUID_LAYERS, LIQUID_WATER].contains(&game.material_type(p.material_id)) {
                return None;
            }
            if !liquid && std::env::var("BF_LIQUID_LOG").is_ok() {
                eprintln!("liquid material h_{:08x} on a placed object that isn't a liquid: h_{:08x} (arch h_{arch:08x}) at {:.1}", p.material_id, o.kind,
                          o.transform.w_axis.truncate());
            }
            let (mesh, opaque) = if game.material_type(p.material_id) == LIQUID_WATER {
                let shared = meshes.get(&p.mesh).cloned();
                let opaque = shared.as_ref().is_some_and(|m| water_opaque(m, &material_ramp(game, p.material_id)));
                let flat = shared.and_then(|m| water_mesh(&m, o.transform * p.at.compute_matrix()));
                (flat.map_or_else(|| p.mesh.clone(), |m| meshes.add(m)), opaque)
            } else {
                (p.mesh.clone(), false)
            };
            liquid_materials(game, images, liquids, &mut liquid_cache, p.material_id, opaque, lava_key).map(|m| (mesh, m))
        }).collect();
        // (the gate's set has a clip per leaf: h_10d3fde3 moves one, h_0a48ec76 the other)
        let clips = game.anim_sets.get(&arch).map(|a| a.as_slice()).unwrap_or_default();
        // its transform and hinge parts are leaves when an anim-trigger plays it once (the same
        // test `Arena::new` numbers the doors by: `animates`)
        let moving = &animated[&arch];
        let animate = animates(moving, &played, o.name);
        let mut leaves = vec![];
        if std::env::var("BF_LIGHT_LOG").is_ok() {
            let n = models[&arch].iter().filter(|p| indoor_lit(o.light_usage, p.indoor)).count();
            if n > 0 {
                let at = o.transform.w_axis;
                eprintln!("lights: h_{:08x} (arch h_{arch:08x}, light-usage {}) at {:.1} {:.1} {:.1}: {n} of {} parts lit by the indoor pair",
                          o.kind, o.light_usage, at.x, at.y, at.z, models[&arch].len());
            }
        }
        for (p, surface) in models[&arch].iter().zip(surfaces) {
            let at = p.at;
            // (a liquid material that has nothing to draw, no texture at all: FUN_00091fe0
            // clears its draw bit)
            if surface.as_ref().is_some_and(|(_, passes)| passes.is_empty()) {
                continue;
            }
            // (the scenery casts no shadow: see `spawn_object_parts`)
            let mesh = surface.as_ref().map_or_else(|| p.mesh.clone(), |(m, _)| m.clone());
            let part = commands.spawn((Mesh3d(mesh.clone()), at, NotShadowCaster, ChildOf(e))).id();
            match surface {
                Some((_, passes)) => {
                    commands.entity(part).insert(MeshMaterial3d(passes[0].clone()));
                    // (BF_LavaMaterial's second pass, FUN_000920a0: the same mesh again)
                    for l in &passes[1..] {
                        commands.spawn((Mesh3d(mesh.clone()), at, NotShadowCaster, MeshMaterial3d(l.clone()), ChildOf(e)));
                    }
                }
                // (lit by the pair its light-usage picks: `indoor_lit`)
                None if indoor_lit(o.light_usage, p.indoor) => {
                    indoor_parts += 1;
                    commands.entity(part).insert(MeshMaterial3d(indoor_material(materials, &mut indoor_cache, &p.material)));
                }
                None => { commands.entity(part).insert(MeshMaterial3d(p.material.clone())); }
            }
            if let Some((axis, travel)) = p.slide {
                let channel = clips.iter().find_map(|c| c.targets.iter().find(|t| t.0 == p.name)).and_then(|t| game.channel_floats(t.1));
                let (interval, curve) = channel.unwrap_or((1.0, vec![]));
                leaves.push(Leaf { entity: part, closed: at, motion: LeafMotion::Slide { axis, travel, curve, interval } });
            } else if let Some(a) = moving.get(&p.name).filter(|_| animate) {
                leaves.push(Leaf { entity: part, closed: at, motion: LeafMotion::Animated(a.clone()) });
            }
        }
        // (an animated door is one even if none of its moving parts draws, so that the door
        // lists line up)
        if !leaves.is_empty() || animate {
            let centre = o.transform.w_axis.truncate();
            if std::env::var("BF_DOOR_LOG").is_ok() {
                // (an animated door's leaves: how many on transform joints, how many on hinges)
                let count = |hinge: bool| leaves.iter().filter(|l| matches!(&l.motion, LeafMotion::Animated(a) if matches!(a.keys, AnimKeys::Angle(_)) == hinge)).count();
                let kinds = if animate { format!(" ({} on transform joints, {} on hinges)", count(false), count(true)) } else { String::new() };
                println!("door {} type h_{:08x} arch h_{arch:08x} at {:.1} leaves {}{kinds}", doors.len(), o.kind, centre, leaves.len());
            }
            let duration = clips.iter().map(|c| c.duration).reduce(f32::max).unwrap_or(DOOR_TIME);
            doors.push(Door { name: o.name, centre, leaves, duration });
        }
    }
    // BF_OVERLAP_LOG: large objects whose boxes overlap much (buildings through each other)
    if std::env::var("BF_OVERLAP_LOG").is_ok() {
        let mut boxes: Vec<(u32, u32, Vec3, Vec3)> = vec![];
        for o in &level.objects {
            let Some(arch) = o.archetype else { continue };
            let Ok(m) = WeaponModel::load(game, arch) else { continue };
            let (mut lo, mut hi) = (Vec3::MAX, Vec3::MIN);
            for p in &m.parts {
                for g in &p.geosets {
                    for v in &g.positions {
                        let w = o.transform.transform_point3(p.offset + p.rotation * Vec3::from(*v));
                        (lo, hi) = (lo.min(w), hi.max(w));
                    }
                }
            }
            let size = hi - lo;
            if size.x * size.y * size.z > 40.0 {
                boxes.push((o.kind, arch, lo, hi));
            }
        }
        for (i, a) in boxes.iter().enumerate() {
            for b in &boxes[i + 1..] {
                let (lo, hi) = (a.2.max(b.2), a.3.min(b.3));
                let d = (hi - lo).max(Vec3::ZERO);
                let both = d.x * d.y * d.z;
                let vol = |x: &(u32, u32, Vec3, Vec3)| { let s = x.3 - x.2; s.x * s.y * s.z };
                let share = both / vol(a).min(vol(b));
                if share > 0.25 {
                    println!("overlap {:3.0}%: h_{:08x} (arch h_{:08x}, {:.0} m3, centre {:.1}) and h_{:08x} (arch h_{:08x}, {:.0} m3, centre {:.1})",
                             share * 100.0, a.0, a.1, vol(a), (a.2 + a.3) * 0.5, b.0, b.1, vol(b), (b.2 + b.3) * 0.5);
                }
            }
        }
    }
    if std::env::var("BF_LIGHT_LOG").is_ok() {
        eprintln!("lights: {indoor_parts} placed parts lit by the indoor pair ({} indoor materials)", indoor_cache.len());
    }
    let layers = make_sky(game, &level.sky, level.sky_at, level.sky_spin, meshes, materials, images, &lamps, &mut cache);
    spawn_sky(commands, &layers);
    doors
}

/// A sky's layers made ready to draw (`sky_layers`): each one's mesh, material and lift above
/// the sky's origin (`make_sky`); where the sky sits and how it turns (`Level::sky_of`).
#[derive(Clone)]
pub struct SkyLayers {
    /// each layer's mesh, material and lift, in the mesh's order
    pub layers: Vec<(Handle<Mesh>, Handle<LevelMaterial>, Vec3)>,
    /// the sky's origin, its `<position>`
    pub at: Vec3,
    /// how it turns
    pub spin: SkySpin,
}

/// A level's sky (`Level::sky_of`: its layers, where they sit and how they turn) made ready to
/// draw on any map, e.g. another level's sky over the test map (play_testworld.rs); `game` holds
/// that level's meshes and textures. Self-lit, so no lamps.
#[allow(clippy::too_many_arguments)]
pub fn sky_layers(game: &mut Game, sky: &[Geoset], at: Vec3, spin: SkySpin, meshes: &mut Assets<Mesh>, materials: &mut Assets<LevelMaterial>,
                  buffers: &mut Assets<ShaderStorageBuffer>, images: &mut Assets<Image>) -> SkyLayers {
    let lamps = Lamps { lamps: buffers.add(no_lights()), ..default() };
    make_sky(game, sky, at, spin, meshes, materials, images, &lamps, &mut HashMap::new())
}

/// The sky's layers in the mesh's order (sdm_e34: the flat top, the panorama of mountains and
/// clouds, the moon, a cloud swirl), self-lit (shader h_f539fe8c), the see-through ones blended.
/// Transparent meshes are drawn far to near by their origin, so each layer's origin sits higher
/// above the sky's than the next one's: they draw in order from the ground.
#[allow(clippy::too_many_arguments)]
fn make_sky(game: &mut Game, sky: &[Geoset], at: Vec3, spin: SkySpin, meshes: &mut Assets<Mesh>, materials: &mut Assets<LevelMaterial>,
            images: &mut Assets<Image>, lamps: &Lamps, cache: &mut HashMap<u32, Handle<LevelMaterial>>) -> SkyLayers {
    let layers = sky.len() as f32;
    let mut out = vec![];
    for (i, g) in sky.iter().enumerate() {
        let mat = material(game, images, materials, lamps, cache, g.material ^ 0x5A5A_5A5A, true, false);
        let lift = Vec3::Y * SKY_LAYER_STEP * (layers - i as f32);
        if let Some(m) = materials.get_mut(&mat).map(|m| &mut m.base) {
            m.double_sided = true;
            m.cull_mode = None;
            m.fog_enabled = false;
            let mips = game.material_texture(g.material).and_then(|t| game.texture_mips(t)).unwrap_or_else(|| Mips::single(1, 1, vec![160, 170, 180, 255]));
            let clear = mips.top().px.chunks_exact(4).any(|p| p[3] < 250);
            m.alpha_mode = if clear { AlphaMode::Blend } else { AlphaMode::Opaque };
            m.base_color = Color::WHITE;
            m.base_color_texture = Some(image(images, &mips));
        }
        let mut layer = mesh_of(g, None, 0.0, false);
        if let Some(VertexAttributeValues::Float32x3(ps)) = layer.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
            for p in ps.iter_mut() {
                *p = (Vec3::from(*p) - lift).to_array();
            }
        }
        out.push((meshes.add(layer), mat, lift));
    }
    SkyLayers { layers: out, at, spin }
}

/// Spawns a sky's layers (`SkyLayers`), each following the camera and turning (`SkyLayer`),
/// from the sky's start (its `<rotation>`). Test hook: `BF_SKY_TIME=<s>` starts the sky's clock
/// that far into the level instead, to set it as a take made that long after the level's start
/// shows it.
pub fn spawn_sky(commands: &mut Commands, sky: &SkyLayers) -> Vec<Entity> {
    let time = std::env::var("BF_SKY_TIME").ok().and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
    sky.layers.iter().map(|(mesh, mat, lift)| {
        let layer = SkyLayer { at: sky.at, lift: *lift, spin: sky.spin, time };
        let turn = sky.spin.rotation(time);
        commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat.clone()), NotShadowCaster,
                        Transform::from_translation(sky.at + turn * *lift).with_rotation(turn), layer)).id()
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A terrain vertex's tangent comes back to the world as its frame's x axis: a frame that
    /// turns the world's +y (the flat ground's normal) to +z and keeps +x gives +x, the way the
    /// colour's u grows on e40's flat ground (TERRAIN_FRAME).
    #[test]
    fn a_flat_frame_gives_the_u_axis() {
        let q = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
        assert!((q * Vec3::Y - Vec3::Z).length() < 1e-5);
        let g = Geoset { positions: vec![[0.0; 3]], normals: vec![[0.0, 1.0, 0.0]], uvs: vec![[0.0; 2]], joints: vec![[0; 4]],
                         weights: vec![[1.0, 0.0, 0.0, 0.0]], indices: vec![], material: 0, indoor: false, frames: vec![q.to_array()] };
        let t = terrain_tangents(&g)[0];
        assert!((Vec3::new(t[0], t[1], t[2]) - Vec3::X).length() < 1e-5 && t[3] == 1.0);
    }

    /// The water's ramp from its keys (`water_ramp`) is the archive's baked ramp texture byte for
    /// byte: the e40 / sdm_e40 ocean's h_15a80001 (keys 0 .22 .36 .56 .8 1 -> 0 0 .67 1 1 1) and
    /// sdm_e10's pond's h_fa12ff06 (0 .2 .4 .6 .8 1 -> 0 .25 .5 .6 .8 1), A8 128 x 1 in sdm_e40's
    /// and sdm_e10's textures-*.tex.
    #[test]
    fn the_water_ramp_is_the_archives() {
        const OCEAN: [u8; 128] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 6, 13, 22, 33, 46,
            59, 73, 87, 102, 116, 129, 141, 151, 160, 166, 170, 170, 171, 172, 174, 177, 180, 183, 187, 191, 196, 201, 205, 210, 215, 220, 225,
            230, 234, 238, 242, 246, 249, 251, 253, 254, 254, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
        const POND: [u8; 128] = [0, 0, 1, 2, 4, 6, 8, 11, 14, 18, 21, 25, 29, 33, 36, 40, 44, 47, 50, 53, 56, 58, 60, 62, 63, 63, 63, 64, 65,
            67, 69, 71, 74, 77, 80, 84, 87, 91, 95, 99, 102, 106, 109, 113, 116, 119, 121, 123, 125, 126, 127, 127, 127, 128, 128, 129, 130,
            131, 132, 133, 135, 136, 138, 139, 141, 142, 143, 145, 146, 148, 149, 150, 151, 151, 152, 152, 152, 153, 153, 154, 156, 157, 159,
            162, 164, 167, 169, 172, 175, 178, 181, 184, 187, 190, 193, 195, 197, 199, 201, 202, 203, 203, 204, 204, 205, 206, 208, 209, 212,
            214, 217, 219, 222, 225, 228, 231, 234, 237, 240, 243, 245, 247, 249, 251, 253, 254, 254, 255];
        assert_eq!(water_ramp([0.0, 0.22, 0.36, 0.56, 0.8, 1.0], [0.0, 0.0, 0.67, 1.0, 1.0, 1.0]), OCEAN);
        assert_eq!(water_ramp([0.0, 0.2, 0.4, 0.6, 0.8, 1.0], [0.0, 0.25, 0.5, 0.6, 0.8, 1.0]), POND);
    }

    /// BF_LavaMaterial's passes by its blend modes (FUN_00091bc0, FUN_00091e60): the data's lava
    /// 2 / 0, sludge 2 / 1, toxic waste 0 / 1, chloride 1 / 1, m02_a's two passes 0 / 0 and e13's
    /// one-texture h_eabb45fa (mode 2).
    #[test]
    fn the_lava_shaders_by_blend_mode() {
        assert_eq!(lava_passes(2, 0, true, true), vec![(PS_CRUST, BLEND_NONE, 0)]);
        assert_eq!(lava_passes(2, 1, true, true), vec![(PS_SUM, BLEND_NONE, 0)]);
        assert_eq!(lava_passes(0, 1, true, true), vec![(PS_SEE_THROUGH, BLEND_PREMULTIPLIED, 0)]);
        assert_eq!(lava_passes(1, 1, true, true), vec![(PS_ADD, BLEND_ADD, 0)]);
        assert_eq!(lava_passes(1, 0, true, true), vec![(PS_OVER, BLEND_PREMULTIPLIED, 0)]);
        assert_eq!(lava_passes(0, 0, true, true), vec![(PS_ONE, BLEND_ALPHA, 0), (PS_ONE, BLEND_ALPHA, 1)]);
        assert_eq!(lava_passes(2, 2, true, true), vec![(PS_ONE, BLEND_NONE, 1)]);
        assert_eq!(lava_passes(2, 1, true, false), vec![(PS_ONE, BLEND_NONE, 0)]);
        assert_eq!(lava_passes(0, 0, false, false), vec![]);
    }
}

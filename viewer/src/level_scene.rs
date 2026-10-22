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
/// its colour and detail textures (`terrain_detail`).
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
}

/// What a `Lamps` material's pipeline depends on: the layer's own depth-test bias, if any,
/// whether it takes the indoor pair, and whether it mixes in a detail texture (a terrain layer:
/// the shader def TERRAIN_DETAIL).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LampsKey {
    layer_bias: Option<i32>,
    indoor: bool,
    detail: bool,
}

impl From<&Lamps> for LampsKey {
    fn from(lamps: &Lamps) -> Self {
        LampsKey { layer_bias: lamps.layer_bias, indoor: lamps.indoor, detail: lamps.colour.is_some() }
    }
}

const LAMP_SHADER: Handle<Shader> = weak_handle!("6b0f6c1e-5a3d-4f0e-9a51-1d2c7e4b8a90");

impl MaterialExtension for Lamps {
    fn fragment_shader() -> ShaderRef {
        LAMP_SHADER.into()
    }

    /// A terrain layer keeps its own depth-test bias (the base material set it from the sort
    /// bias, see TERRAIN_LAYER_SORT), an indoor material takes the indoor pair (INDOOR_PAIR) and a
    /// terrain layer samples its textures as the game's terrain pass does (TERRAIN_DETAIL, see
    /// `terrain_detail`).
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
        Ok(())
    }
}

/// The terrain material class (h_0ec4edf0, factory 0x91330; every terrain layer's material).
const TERRAIN_MATERIAL: u32 = 0x0EC4_EDF0;
/// Its texture slots past `color` and `alpha` (0x90b10): the detail texture (h_17fc684a,
/// "detail" by its hash) and a P8 bump map (h_ed0b519d, not drawn here). On all 141 terrain
/// materials of the game's levels the detail texture is the colour texture.
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
/// demo drew the colour's tiling everywhere. (The bump map, lit through oD1.rgb, isn't drawn.)
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

/// Added to a terrain layer's sort distance: bevy sorts transparent meshes back to front by
/// their origin's depth in the view plus the material's `depth_bias`, and a layer's origin is
/// the world's (its mesh is in world coordinates). Where that origin is nearer the camera than
/// an effect, the layer drew after it, painting the ground over explosions, muzzle flashes and
/// decals: take04's cliff blasts in #73 vanished, and their scorch showed only where the rock
/// layer's mask let it through, as straight-edged wedges. A layer is part of the ground, so it
/// sorts first, before every other transparent thing; the layers keep their order among
/// themselves (+10 each, as before). Its depth-test bias stays as it was (`Lamps::layer_bias`).
const TERRAIN_LAYER_SORT: f32 = -1.0e6;

/// A liquid's surface (the game's animated liquid shader h_f124a774: lava, toxic rivers; its
/// water h_0f8904b9): bevy's standard material, its colour the moving layers (src/liquid.wgsl).
pub type LiquidMaterial = ExtendedMaterial<StandardMaterial, LiquidSurface>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct LiquidSurface {
    #[uniform(100)]
    pub params: LiquidParams,
    #[texture(101)]
    #[sampler(102)]
    pub layer0: Option<Handle<Image>>,
    #[texture(103)]
    #[sampler(104)]
    pub layer1: Option<Handle<Image>>,
    #[texture(105)]
    #[sampler(106)]
    pub ramp: Option<Handle<Image>>,
}

/// See src/liquid.wgsl. (In a module of its own: the derive's own checks read as unused.)
mod liquid_params {
    #![allow(dead_code)]
    use bevy::{prelude::*, render::render_resource::ShaderType};

    #[derive(Clone, Copy, Debug, Default, Reflect, ShaderType)]
    pub struct LiquidParams {
        pub scale0: Vec2,
        pub scale1: Vec2,
        pub speed0: Vec2,
        pub speed1: Vec2,
        pub tint: Vec4,
        pub reflection: Vec4,
        pub kind: u32,
        pub mode: u32,
        pub glow: f32,
        pub opacity: f32,
        pub bright: f32,
    }
}
pub use liquid_params::LiquidParams;

const LIQUID_SHADER_HANDLE: Handle<Shader> = weak_handle!("2c41a7d3-8e5b-4b6f-b0c2-93d1f5e7a604");

impl MaterialExtension for LiquidSurface {
    fn fragment_shader() -> ShaderRef {
        LIQUID_SHADER_HANDLE.into()
    }
}

/// The level materials (`LevelMaterial`, `LiquidMaterial`) and their shaders.
pub fn plugin(app: &mut App) {
    let mut shaders = app.world_mut().resource_mut::<Assets<Shader>>();
    shaders.insert(&LAMP_SHADER, Shader::from_wgsl(include_str!("lamps.wgsl"), file!()));
    shaders.insert(&LIQUID_SHADER_HANDLE, Shader::from_wgsl(include_str!("liquid.wgsl"), file!()));
    app.add_plugins((MaterialPlugin::<LevelMaterial>::default(), MaterialPlugin::<LiquidMaterial>::default()));
}

/// How far a liquid's drawn surface (its liquid-shader geosets' top) must move to lie on its
/// collision's top (the archetype's own frame); 0 without collision.
fn liquid_lift(game: &Game, arch: u32) -> f32 {
    let top = crate::arena::physics_tris(game, arch).iter().flat_map(|t| t.0.map(|v| v.y)).fold(f32::MIN, f32::max);
    let Ok(m) = WeaponModel::load(game, arch) else { return 0.0 };
    let drawn = m.parts.iter().flat_map(|p| p.geosets.iter().filter(|g| [LIQUID_LAYERS, LIQUID_WATER].contains(&game.material_type(g.material)))
        .flat_map(move |g| g.positions.iter().map(move |v| (p.rotation * Vec3::from(*v) + p.offset).y))).fold(f32::MIN, f32::max);
    if top == f32::MIN || drawn == f32::MIN { 0.0 } else { top - drawn }
}

/// The game's liquid shaders: animated layers (lava, toxic) and water.
const LIQUID_LAYERS: u32 = 0xF124_A774;
const LIQUID_WATER: u32 = 0x0F89_04B9;
/// The water's fresnel ramp (a 128 x 1 alpha strip) parameter.
const WATER_RAMP: u32 = 0x0B85_CB09;
/// The liquid shaders' scroll speeds (tiles per second; unnamed: by their values, the pairs u, v
/// of texture-0 and of texture-1). A layer's `texture-scale` is its tile's size in the mesh's uv
/// (by the sizes it gives: sdm_e13's lava 1.5 km across spans 15 uv, so 0.1 is a 10 m tile; taken
/// as repeats it stretched a 256-texel texture over a kilometre, and scrolled tens of m/s).
const SCROLL0: [u32; 2] = [0x0E41_AF5B, 0x0E2B_9302];
const SCROLL1: [u32; 2] = [0xE002_AE8E, 0xFB0B_FF34];
/// A liquid's glow (its colour given as light) by liquid-type: lava 2 all, toxic 4 nearly so
/// (captures: Singe's lava and Shanty Town's acid shine, solid, near yellow).
const LAVA_GLOW: f32 = 1.0;
const TOXIC_GLOW: f32 = 0.8;
/// Water's own share of light: a capture of sdm_e10's pond shows it evenly lit, its ripples
/// plain, where the level's dim lights alone left it near black.
const WATER_GLOW: f32 = 0.35;
/// Water's opacity looking straight down (the fresnel ramp takes it to opaque at a glance; a
/// capture of a pond shows a murky surface, nothing of what's under it).
const WATER_OPACITY: f32 = 0.92;
/// The liquids' colour scale: every layered liquid's blend-mode is 2, read as the console's
/// modulate x 2 (captures: bright yellow lava, yellow-green acid); water's too (no blend-mode:
/// a choice, by a capture of a pond's olive brown).
const LIQUID_BRIGHT: f32 = 2.0;
/// Liquid surfaces are drawn as if this much nearer (bevy depth bias), over terrain just under them.
const LIQUID_DEPTH_BIAS: f32 = 1000.0;

/// Water's layers drift at least this fast (tiles per second), the two crossing: sdm_e10's
/// scroll 0.0002 and 0.01, their ripples all but still. (The game's water has `framerate`
/// constants, 17 and 25: some animation of its own, not traced.)
const WATER_DRIFT: [Vec2; 2] = [Vec2::new(0.02, 0.012), Vec2::new(-0.015, 0.022)];

fn drift(water: bool, speed: Vec2, least: Vec2) -> Vec2 {
    if water && speed.length() < least.length() { least } else { speed }
}

/// A liquid surface's material (see `LiquidMaterial`), if `id` is a liquid shader's; `kind` is
/// the liquid object's liquid-type, `sky` the level's background colour (the water's reflection).
fn liquid_material(game: &mut Game, images: &mut Assets<Image>, liquids: &mut Assets<LiquidMaterial>,
                   cache: &mut HashMap<u32, Handle<LiquidMaterial>>, id: u32, kind: i64, sky: [f32; 3]) -> Option<Handle<LiquidMaterial>> {
    let ty = game.material_type(id);
    if ty != LIQUID_LAYERS && ty != LIQUID_WATER {
        return None;
    }
    if let Some(m) = cache.get(&id) {
        return Some(m.clone());
    }
    let h = crate::bf::hash::h;
    let num = |game: &Game, n: u32, or: f32| game.material_constant(id, n).and_then(|v| v.first().copied()).unwrap_or(or);
    let col = |game: &Game, n: &str| game.material_constant(id, h(n)).filter(|v| v.len() >= 3)
        .map_or(Vec4::ONE, |v| Color::srgb(v[0], v[1], v[2]).to_linear().to_vec4());
    let mut tex = |game: &mut Game, param: u32| game.material_param_texture(id, param)
        .map(|t| std::env::var("BF_LIQUID_TEX").ok().and_then(|v| u32::from_str_radix(&v, 16).ok()).unwrap_or(t)).and_then(|t| game.texture_mips(t))
        .map(|mips| image(images, &mips));
    let water = ty == LIQUID_WATER;
    let (scale0, scale1) = if water {
        (Vec2::new(num(game, h("scale-u"), 1.0), num(game, h("scale-v"), 1.0)), Vec2::new(num(game, h("scale-u-1"), 1.0), num(game, h("scale-v-1"), 1.0)))
    } else {
        (Vec2::splat(num(game, h("texture-scale"), 1.0)), Vec2::splat(num(game, h("texture-scale-1"), 1.0)))
    };
    let glow = if water { WATER_GLOW } else { match kind { 2 => LAVA_GLOW, 4 => TOXIC_GLOW, _ => 0.0 } };
    // (lava and acid are solid: captures; the material's own opacity, 0-255, was 200 on the
    // toxic river and 60-95 on lava)
    let opacity = if water { WATER_OPACITY } else { 1.0 };
    let reflection = Color::srgb(sky[0], sky[1], sky[2]).to_linear().to_vec4() * col(game, "color-1");
    let params = LiquidParams {
        scale0, scale1,
        speed0: drift(water, Vec2::new(num(game, SCROLL0[0], 0.0), num(game, SCROLL0[1], 0.0)), WATER_DRIFT[0]),
        speed1: drift(water, Vec2::new(num(game, SCROLL1[0], 0.0), num(game, SCROLL1[1], 0.0)), WATER_DRIFT[1]),
        tint: col(game, "color"),
        reflection,
        kind: water as u32,
        mode: num(game, h("blend-mode-1"), 0.0) as u32,
        glow,
        opacity,
        bright: if water || num(game, h("blend-mode"), 0.0) as u32 == 2 { LIQUID_BRIGHT } else { 1.0 },
    };
    if std::env::var("BF_LIQUID_LOG").is_ok() {
        eprintln!("liquid material h_{id:08x}: texture-0 {:x?} texture-1 {:x?} ramp {:x?} {params:?}",
                  game.material_param_texture(id, h("texture-0")), game.material_param_texture(id, h("texture-1")),
                  game.material_param_texture(id, WATER_RAMP));
    }
    // BF_LIQUID_DEBUG: every liquid surface solid white (where are they drawn?)
    let debug = std::env::var("BF_LIQUID_DEBUG").is_ok();
    let params = if debug { LiquidParams { kind: 0, mode: 0, glow: 1.0, opacity: 1.0, ..params } } else { params };
    let surface = LiquidSurface {
        params,
        layer0: if debug { None } else { tex(game, h("texture-0")) },
        layer1: if debug { None } else { tex(game, h("texture-1")) },
        ramp: if water { tex(game, WATER_RAMP) } else { None },
    };
    let base = StandardMaterial {
        perceptual_roughness: if water { 0.15 } else { 0.6 },
        reflectance: if water { 0.5 } else { 0.2 },
        alpha_mode: if opacity < 1.0 && !debug { AlphaMode::Blend } else { AlphaMode::Opaque },
        // (Singe's lake lies on terrain painted with lava veins, centimetres under it: seen low,
        // the two fought and the terrain's veins showed through the lava)
        depth_bias: LIQUID_DEPTH_BIAS,
        double_sided: true,
        cull_mode: None,
        ..default()
    };
    let m = liquids.add(LiquidMaterial { base, extension: surface });
    cache.insert(id, m.clone());
    Some(m)
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
             weights: g.weights.clone(), indices: g.indices.clone(), material: g.material, indoor: g.indoor }
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
        let (mesh, mat) = match mask.and_then(|m| game.texture_rgba(m)) {
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
        // a liquid's surface moves (see `LiquidMaterial`)
        let liquid_kind = (o.tag == crate::bf::character::H_LIQUID_OBJECT).then(|| game.liquids.get(&o.kind).map_or(0, |l| l.kind));
        // a liquid's drawn surface where its collision's top is (what's waded in): sdm_e40's
        // pools draw theirs 1 m higher, floating over the floor (seen from the side the water
        // was in the air; from above it lined up)
        let lift = liquid_kind.map_or(0.0, |_| liquid_lift(game, arch));
        let surfaces: Vec<_> = models[&arch].iter()
            .map(|p| liquid_kind.and_then(|k| liquid_material(game, images, liquids, &mut liquid_cache, p.material_id, k, level.background))).collect();
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
            // (the scenery casts no shadow: see `spawn_object_parts`)
            let place = if surface.is_some() { Transform { translation: at.translation + Vec3::Y * lift, ..at } } else { at };
            let part = commands.spawn((Mesh3d(p.mesh.clone()), place, NotShadowCaster, ChildOf(e))).id();
            match surface {
                Some(l) => { commands.entity(part).insert(MeshMaterial3d(l)); }
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

//! A decoded `Level` as Bevy entities: terrain layers (blended by their masks), the placed
//! objects and the sky. Shared by the level viewer (bf_level) and the playable demo.

use std::collections::HashMap;

use bevy::{
    asset::{weak_handle, RenderAssetUsages},
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    pbr::{ExtendedMaterial, MaterialExtension, NotShadowCaster},
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
        render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat},
        storage::ShaderStorageBuffer,
    },
};

use crate::bf::character::{Game, Geoset, H_WRAP_ALPHA};
use crate::bf::level::Level;
use crate::bf::weapon::WeaponModel;

/// A level's material: bevy's standard one, lit also by the level's point lights (light-object
/// h_ea460e64) as the game lights its scenery: colour x (1 - distance / range) x N.L, per pixel
/// here (per vertex there), unshadowed, nothing past the range (src/lamps.wgsl). Bevy's own point
/// lights fall off as 1 / d^2 and couldn't follow that: a light matched at mid range was far too
/// bright beside it and too dim towards its edge.
pub type LevelMaterial = ExtendedMaterial<StandardMaterial, Lamps>;

/// The level's point lights, for `LevelMaterial`: one buffer shared by all its materials.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct Lamps {
    #[storage(100, read_only)]
    pub lamps: Handle<ShaderStorageBuffer>,
}

const LAMP_SHADER: Handle<Shader> = weak_handle!("6b0f6c1e-5a3d-4f0e-9a51-1d2c7e4b8a90");

impl MaterialExtension for Lamps {
    fn fragment_shader() -> ShaderRef {
        LAMP_SHADER.into()
    }
}

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
        .map(|t| std::env::var("BF_LIQUID_TEX").ok().and_then(|v| u32::from_str_radix(&v, 16).ok()).unwrap_or(t)).and_then(|t| game.texture_rgba(t))
        .map(|(w, h, px)| image(images, w, h, px));
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
/// (1 - distance / range)), for what isn't a `LevelMaterial` (the characters).
pub fn lamp_light_at(level: &Level, p: Vec3) -> Vec3 {
    if std::env::var("BF_NO_LAMPS").is_ok() {
        return Vec3::ZERO;
    }
    level.lamps.iter().filter_map(|l| {
        let d = l.at.distance(p);
        (d < l.range).then(|| Vec3::from(l.color.map(|c| Color::srgb(c, c, c).to_linear().red)) * (1.0 - d / l.range))
    }).sum()
}

/// The point lights' buffer: per lamp its place, range, linear colour and falloff kind (8
/// floats); one that reaches nowhere when there are none (a buffer can't be empty).
fn lamp_buffer(level: &Level) -> ShaderStorageBuffer {
    let linear = |c: f32| Color::srgb(c, c, c).to_linear().red;
    let mut data: Vec<[f32; 8]> = level.lamps.iter().map(|l| {
        let [r, g, b] = l.color.map(linear);
        [l.at.x, l.at.y, l.at.z, l.range, r, g, b, l.falloff as f32]
    }).collect();
    if data.is_empty() || std::env::var("BF_NO_LAMPS").is_ok() {
        data = vec![[0.0; 8]];
    }
    let bytes: Vec<u8> = data.iter().flatten().flat_map(|f| f.to_le_bytes()).collect();
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
/// Untextured glow material (pickups' glass shells): a colour constant and the wrapper's opacity.
const GLOW_SHELL: u32 = 0xF539_FE8C;

/// A door or gate, in level object order (the same order as `Arena::doors`): its sliding
/// leaves, how long it takes to open (its archetype's animation: 1.21 s for doors, 4.96 s for
/// the gate) and the sounds the level plays beside it as it opens and closes.
pub struct Door {
    pub centre: Vec3,
    pub leaves: Vec<Leaf>,
    pub duration: f32,
    pub open_sound: Option<u32>,
    pub close_sound: Option<u32>,
}

/// A door leaf: its entity, opening direction (object frame), closed placement and how far it
/// slides; `curve` is its animation channel (metres along the axis per frame of `interval` s),
/// else it slides `travel` m eased.
pub struct Leaf {
    pub entity: Entity,
    pub axis: Vec3,
    pub closed: Transform,
    pub travel: f32,
    pub curve: Vec<f32>,
    pub interval: f32,
}

/// Fallback opening time (s) of a door without an animation.
const DOOR_TIME: f32 = 0.8;
/// Level signals of the sound triggers beside each door (as it opens, as it closes), and how
/// near the door a trigger sits.
const SIGNAL_OPEN: i64 = 31;
const SIGNAL_CLOSE: i64 = 49;
const SOUND_NEAR: f32 = 6.0;

impl Leaf {
    /// How far the leaf has slid `t` s into the opening.
    pub fn slide(&self, t: f32, duration: f32) -> f32 {
        if self.curve.is_empty() {
            let k = (t / duration).clamp(0.0, 1.0);
            return self.travel * k * k * (3.0 - 2.0 * k);
        }
        let f = (t / self.interval.max(1e-3)).max(0.0);
        let i = (f as usize).min(self.curve.len() - 1);
        let j = (i + 1).min(self.curve.len() - 1);
        self.curve[i] + (self.curve[j] - self.curve[i]) * (f - i as f32).min(1.0)
    }
}

/// The camera's look, as the console drew it: no tone mapping, and an exposure at which a
/// light of 1.0 (a white directional light of PI lux, or ambient brightness 1) shows a texture
/// as it is (Bevy scales light by 1 / (1.2 x 2^ev100)).
/// A sky layer and where it sits with the camera at the map's centre: the sky moves along with
/// the camera across the map (so its mountains stay on the horizon), not up and down.
#[derive(Component)]
pub struct SkyLayer(pub Vec3);

/// Keep the sky around the camera (see `SkyLayer`); add it to a level app's Update.
pub fn follow_sky(camera: Query<&GlobalTransform, With<Camera3d>>, mut sky: Query<(&SkyLayer, &mut Transform)>) {
    let Some(eye) = camera.iter().next().map(|c| c.translation()) else { return };
    for (at, mut tr) in &mut sky {
        tr.translation = at.0 + Vec3::new(eye.x, 0.0, eye.z);
    }
}

pub fn console_look() -> (bevy::core_pipeline::tonemapping::Tonemapping, bevy::render::camera::Exposure) {
    (bevy::core_pipeline::tonemapping::Tonemapping::None, bevy::render::camera::Exposure { ev100: (1.0f32 / 1.2).log2() })
}

/// Point lights tuned for Bevy's default camera exposure (ev100 9.7) keep their look under
/// `console_look` when scaled by this (1.2 x 2^9.7 / (1.2 x 2^-0.263)).
pub const POINT_LIGHT_SCALE: f32 = 1.0 / 997.0;
/// How far apart (m) the sky layers' origins are stacked, to draw them in order (see the sky
/// in `spawn_level`).
const SKY_LAYER_STEP: f32 = 1000.0;

/// The level's own lighting (instead of a made-up sun): the object key light (shadows) and fill
/// as directional lights, the object ambient. The game lights each surface texture x (ambient
/// + key x N.L + fill x N.L) with those colours in gamma space, so they're given as sRGB colours
/// (Bevy works in linear light) at an illuminance of PI (see `console_look`). Terrain has its
/// own, dimmer pair: `terrain_tint` scales its materials to match.
pub fn spawn_lighting(commands: &mut Commands, level: &Level) {
    let mut any = false;
    for l in level.lights.iter().filter(|l| !l.terrain) {
        let [r, g, b] = l.color;
        let at = Transform::default().looking_to(l.dir.normalize_or(Vec3::NEG_Y), Vec3::Y);
        commands.spawn((DirectionalLight { illuminance: std::f32::consts::PI, color: Color::srgb(r, g, b), shadows_enabled: l.key, ..default() },
                        at, Name::new(if l.key { "key light" } else { "fill light" })));
        any = true;
    }
    if !any {
        commands.spawn((DirectionalLight { illuminance: std::f32::consts::PI, color: Color::srgb(0.85, 0.85, 0.8), shadows_enabled: true, ..default() },
                        Transform::default().looking_to(Vec3::new(-0.45, -0.75, -0.35), Vec3::Y)));
    }
    let [r, g, b] = level.ambient;
    commands.insert_resource(AmbientLight { color: Color::srgb(r, g, b), brightness: 1.0, ..default() });
    if std::env::var("BF_LIGHT_LOG").is_ok() {
        eprintln!("lights: {} directional, {} point", level.lights.len(), level.lamps.len());
        for l in &level.lamps {
            eprintln!("  point at {:7.1} {:6.1} {:7.1}  colour {:.2} {:.2} {:.2}  range {:4.1}  falloff {}  terrain {}",
                      l.at.x, l.at.y, l.at.z, l.color[0], l.color[1], l.color[2], l.range, l.falloff, l.terrain);
        }
    }
    // (the point lights are the level materials' own: see `LevelMaterial`)
}

/// How the baked terrain light map lies on the grid (see `Level::baked_light`): blocks row by
/// row, cells row by row, x and z increasing; found by correlating it with the terrain's own
/// slope lighting (sdm_e34 0.26, sdm_m03 0.39; every other orientation near 0).
const TERRAIN_LIGHT_ORIENTATION: u8 = 0;

/// How much dimmer the terrain's key light is than the objects' (per channel, at most 1): the
/// terrain's materials are tinted by it, since every light here lights everything.
fn terrain_tint(level: &Level) -> Color {
    let key = |terrain: bool| level.lights.iter().find(|l| l.key && l.terrain == terrain).map(|l| l.color);
    match (key(true), key(false)) {
        (Some(t), Some(o)) => Color::srgb((t[0] / o[0].max(0.01)).min(1.0), (t[1] / o[1].max(0.01)).min(1.0), (t[2] / o[2].max(0.01)).min(1.0)),
        _ => Color::WHITE,
    }
}

/// The colour of the terrain's own lights (its ambient + key + fill, the pair marked
/// h_e02aba1c), scaled to unit brightness and given in linear terms: the baked light map says how
/// bright the ground is; these lights give it their hue. (Bulgar's terrain lights are blue-grey:
/// key 0.44 0.57 0.65, ambient 0.13 0.16 0.24 - the captures' ground is grey, not sand-brown.)
fn terrain_hue(level: &Level) -> [f32; 3] {
    let mut c = level.terrain_ambient;
    for l in level.lights.iter().filter(|l| l.terrain) {
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

fn image(images: &mut Assets<Image>, w: u32, h: u32, px: Vec<u8>) -> Handle<Image> {
    let mut img = Image::new(Extent3d { width: w, height: h, depth_or_array_layers: 1 }, TextureDimension::D2, px,
                             TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    });
    images.add(img)
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
    if game.material_type(id) == GLOW_SHADER {
        if let Some((w, h, px)) = game.material_param_texture(id, GLOW_TEXTURE).and_then(|t| game.texture_rgba(t)) {
            mat.emissive = LinearRgba::WHITE;
            mat.emissive_texture = Some(image(images, w, h, px));
        }
    }
    if std::env::var("BF_MAT_LOG").is_ok() {
        let tex = game.material_texture(id);
        let avg = tex.and_then(|t| game.texture_rgba(t)).map(|(w, h, px)| {
            let n = (w * h) as f32;
            let s = px.chunks_exact(4).fold([0f32; 4], |a, p| [a[0] + p[0] as f32, a[1] + p[1] as f32, a[2] + p[2] as f32, a[3] + p[3] as f32]);
            (w, h, s.map(|v| (v / n) as u8))
        });
        eprintln!("material h_{id:08x} type h_{:08x} texture {:x?} alpha {:x?} {:?}", game.material_type(id), tex,
                  game.material_alpha_texture(id), avg);
    }
    if let Some((w, h, mut px)) = game.material_texture(id).and_then(|t| game.texture_rgba(t)) {
        if cutout && CUTOUT_TYPES.contains(&game.material_type(id)) {
            if let Some((aw, ah, apx)) = game.material_alpha_texture(id).and_then(|a| game.texture_rgba(a)) {
                for y in 0..h {
                    for x in 0..w {
                        let (sx, sy) = (x * aw / w, y * ah / h);
                        px[((y * w + x) * 4 + 3) as usize] = apx[((sy * aw + sx) * 4 + 3) as usize];
                    }
                }
            }
            mat.alpha_mode = AlphaMode::Mask(0.5);
            mat.double_sided = true;
            mat.cull_mode = None;
        } else {
            px.chunks_exact_mut(4).for_each(|p| p[3] = 255);
        }
        mat.base_color = Color::WHITE;
        mat.base_color_texture = Some(image(images, w, h, px));
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
             weights: g.weights.clone(), indices: g.indices.clone(), material: g.material }
}

/// `flip`: reverse the winding (terrain triangles face the other way from objects'). Otherwise
/// each triangle is wound to face along its vertices' normals: mirrored meshes (the gate's left
/// leaf is its right leaf mirrored, same index order) would otherwise be culled from the front.
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

/// Spawn the level's terrain, placed objects and sky; returns the doors (closed).
pub fn spawn_level(commands: &mut Commands, game: &mut Game, level: &Level, meshes: &mut Assets<Mesh>,
                   materials: &mut Assets<LevelMaterial>, liquids: &mut Assets<LiquidMaterial>, buffers: &mut Assets<ShaderStorageBuffer>,
                   images: &mut Assets<Image>) -> Vec<Door> {
    let mut doors = vec![];
    let mut cache = HashMap::new();
    let mut liquid_cache = HashMap::new();
    let lamps = Lamps { lamps: buffers.add(lamp_buffer(level)) };
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
                layer.base.depth_bias = 10.0 * k as f32;
                (mesh_of(g, Some(colors), 0.02 * k as f32, true), materials.add(layer))
            }
            None => {
                let colors = baked.then(|| g.positions.iter().map(|p| { let l = light(p); [l * hue[0], l * hue[1], l * hue[2], 1.0] }).collect());
                (mesh_of(g, colors, 0.0, true), base)
            }
        };
        commands.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(mat), Transform::default(), Name::new(format!("terrain layer {k}"))));
    }
    // placed objects
    // per archetype: its geosets with their part placement, and the part's slide (axis, travel)
    type Part = (Handle<Mesh>, Handle<LevelMaterial>, Transform, Option<(Vec3, f32)>, u32, u32);
    let mut models: HashMap<u32, Vec<Part>> = HashMap::new();
    for o in &level.objects {
        let Some(arch) = o.archetype else { continue };
        if !models.contains_key(&arch) {
            let mut parts = vec![];
            if let Ok(m) = WeaponModel::load(game, arch) {
                for p in &m.parts {
                    // a sliding leaf travels its own length along its axis (fully clear)
                    let slide = p.slide_axis.map(|axis| {
                        let along = p.geosets.iter().flat_map(|g| &g.positions).map(|v| (p.rotation * Vec3::from(*v)).dot(axis));
                        let (lo, hi) = along.fold((f32::MAX, f32::MIN), |(lo, hi), d| (lo.min(d), hi.max(d)));
                        (axis, (hi - lo).max(0.0) + 0.1)
                    });
                    let shown: Vec<&Geoset> = p.geosets.iter().filter(|g| visible(game, g)).collect();
                    for g in shown {
                        let mut mat = material(game, images, materials, &lamps, &mut cache, g.material, false, true);
                        let grass = is_grass(game, g);
                        if grass {
                            // both faces drawn, both lit from above (a double-sided material
                            // turns a back face's normal down)
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
                        let mesh = if grass { mesh_of(&upright_normals(g), None, 0.0, false) } else { mesh_of(g, None, 0.0, false) };
                        parts.push((meshes.add(mesh), mat, Transform::from_translation(p.offset).with_rotation(p.rotation), slide, p.name, g.material));
                    }
                }
            }
            models.insert(arch, parts);
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
        let e = commands.spawn((Transform::from_matrix(o.transform), Visibility::default(), Name::new(format!("h_{:08x}", o.kind)))).id();
        // the archetype's animation (archetype-set of the same name) moves the leaves: a
        // channel per part
        // a liquid's surface moves (see `LiquidMaterial`)
        let liquid_kind = (o.tag == crate::bf::character::H_LIQUID_OBJECT).then(|| game.liquids.get(&o.kind).map_or(0, |l| l.kind));
        // a liquid's drawn surface where its collision's top is (what's waded in): sdm_e40's
        // pools draw theirs 1 m higher, floating over the floor (seen from the side the water
        // was in the air; from above it lined up)
        let lift = liquid_kind.map_or(0.0, |_| liquid_lift(game, arch));
        let surfaces: Vec<_> = models[&arch].iter()
            .map(|p| liquid_kind.and_then(|k| liquid_material(game, images, liquids, &mut liquid_cache, p.5, k, level.background))).collect();
        let clip = game.anim_sets.get(&arch).and_then(|a| a.first());
        let mut leaves = vec![];
        for ((mesh, mat, at, slide, name, _), surface) in models[&arch].iter().zip(surfaces) {
            // the scenery casts no shadow: the game lights it per vertex by the level's lights
            // (all `enable-static`), unshadowed, so building interiors are lit like the walls
            // outside (a shadow-casting roof made them black) and a jungle canopy doesn't
            // shade the whole floor; characters still cast theirs
            let place = if surface.is_some() { Transform { translation: at.translation + Vec3::Y * lift, ..*at } } else { *at };
            let part = commands.spawn((Mesh3d(mesh.clone()), place, NotShadowCaster, ChildOf(e))).id();
            match surface {
                Some(l) => { commands.entity(part).insert(MeshMaterial3d(l)); }
                None => { commands.entity(part).insert(MeshMaterial3d(mat.clone())); }
            }
            if let Some((axis, travel)) = slide {
                let channel = clip.and_then(|c| c.targets.iter().find(|t| t.0 == *name)).and_then(|t| game.channel_floats(t.1));
                let (interval, curve) = channel.unwrap_or((1.0, vec![]));
                leaves.push(Leaf { entity: part, axis: *axis, closed: *at, travel: *travel, curve, interval });
            }
        }
        if !leaves.is_empty() {
            let centre = o.transform.w_axis.truncate();
            let sound = |signal: i64| level.sounds.iter().filter(|s| s.1 == signal && s.2.distance(centre) < SOUND_NEAR)
                .min_by(|a, b| a.2.distance(centre).total_cmp(&b.2.distance(centre))).map(|s| s.0);
            if std::env::var("BF_DOOR_LOG").is_ok() {
                println!("door {} type h_{:08x} arch h_{arch:08x} at {:.1} leaves {}", doors.len(), o.kind, centre, leaves.len());
            }
            doors.push(Door { centre, leaves, duration: clip.map_or(DOOR_TIME, |c| c.duration),
                              open_sound: sound(SIGNAL_OPEN), close_sound: sound(SIGNAL_CLOSE) });
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
    // sky: layers in the mesh's order (sdm_e34: the flat top, the panorama of mountains and
    // clouds, the moon, a cloud swirl), self-lit (shader h_f539fe8c), the see-through ones
    // blended. Transparent meshes are drawn far to near by their origin, so each layer's origin
    // sits higher above the sky's than the next one's: they draw in order from the ground.
    let layers = level.sky.len() as f32;
    for (i, g) in level.sky.iter().enumerate() {
        let mat = material(game, images, materials, &lamps, &mut cache, g.material ^ 0x5A5A_5A5A, true, false);
        let lift = Vec3::Y * SKY_LAYER_STEP * (layers - i as f32);
        if let Some(m) = materials.get_mut(&mat).map(|m| &mut m.base) {
            m.double_sided = true;
            m.cull_mode = None;
            m.fog_enabled = false;
            let (w, h, px) = game.material_texture(g.material).and_then(|t| game.texture_rgba(t)).unwrap_or((1, 1, vec![160, 170, 180, 255]));
            let clear = px.chunks_exact(4).any(|p| p[3] < 250);
            m.alpha_mode = if clear { AlphaMode::Blend } else { AlphaMode::Opaque };
            m.base_color = Color::WHITE;
            m.base_color_texture = Some(image(images, w, h, px));
        }
        let mut layer = mesh_of(g, None, 0.0, false);
        if let Some(VertexAttributeValues::Float32x3(ps)) = layer.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
            for p in ps.iter_mut() {
                *p = (Vec3::from(*p) - lift).to_array();
            }
        }
        commands.spawn((Mesh3d(meshes.add(layer)), MeshMaterial3d(mat), NotShadowCaster,
                        Transform::from_translation(level.sky_at + lift), SkyLayer(level.sky_at + lift)));
    }
    doors
}

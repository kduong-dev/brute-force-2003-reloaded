//! The memory chip a dead squadmate drops (#43): what the demo used to call "the DNA".
//!
//! Every squad character-object in the levels carries `<h_0f77963d inventory-drop=..>`, and the
//! item it names is h_f50f94f9 (objecttypes `<inventory>`: function-type 18, group-type 7,
//! h_0a811e94 = 2000, its `<base>` h_1dd3a49d message h_091ce603 "Memory Chip Recovered!",
//! pickup-sound h_ee2c16da, no effects). Its mesh h_10960346 is a 0.375 m cube (one geoset, 36
//! vertices, UVs 0..1 on each face) in material h_f5e6f5c5, a wrapper round h_f8fdcbd7: the
//! self-lit shader h_f539fe8c with the circuit-trace texture h_1e02a5ef (64 x 64 DXT3, two
//! thirds of it alpha 0), tinted by its h_e01baa40 colour (0.36 1 0.67) at its `alpha` (0.7).
//! Its pixel program (pixel shader 0, defined at 0x3ddf80 in default.xbe, set by FUN_0008fd20
//! through FUN_000a4d00) has one combiner stage, texture x constant c0 for both colour and alpha,
//! c0 being the h_e01baa40 colour with `alpha` as its fourth component, and a final combiner
//! that fogs the colour.
//! How the game draws that shader (FUN_0008fd20, the self-lit shader's render-state setup):
//! h_0f5ae13f 2 is not the additive mode (1 is: ONE / ONE); with h_17aba73c set (or `alpha` not
//! 1) and h_1a08f318 0 it blends SRCALPHA / INVSRCALPHA with z-writes off; h_18954f8d 1 turns
//! culling off (medium confidence: the footage shows the far faces through the near ones). Its
//! scroll (FUN_0008fca0) moves the texture by h_fb0bff34 / h_e002ae8e per second only while
//! h_08c2d2ee seconds are left, then stops: a fixed offset of rate x time.
//!
//! In todo/DNA + Weapon Pickups.mp4 (0-12.6 s) and todo/Friendly Fire 2 + Death Cam.mp4 (from
//! 33.1 s, beside Brutus) it's a solid see-through green cube that turns with the view, its far
//! faces visible through it, not spinning or bobbing. Walking into it takes it: it goes, its
//! sound plays and the pickup lines say "Memory Chip Recovered!" (the footage also shows
//! "+ 2000" by the radar: the score isn't kept by the demo).
//!
//! Two constants aren't used. `time-scale` (h_01590d7a, 0.4) is mapped to +0x58 by FUN_0008fa50,
//! but none of the shader's own functions read it; scaling the scroll's time step wouldn't move
//! where it stops anyway. The wrapper's h_e59d69a0 (60) goes to the render state at +0x294 of
//! the state cache through FUN_0009e960, which flags the material when it isn't 255; which
//! state that is isn't established (alpha-test reference is a guess: as one, with the
//! footage's distant chip filled, it can't be cutting at 60/255), so it's left out.
//!
//! Not matched: the footage's chip is fuller and brighter than this material can draw. Far off
//! (Friendly Fire 2, 33.8 s) its inside is green +58 over the background, with red and blue up
//! too; up close in front of a lit wall (DNA + Weapon Pickups, 12.4-12.6 s) it's whitish cyan,
//! red 165 over 60, with a soft halo. The pixel program can't raise red past the background
//! (texture red 45 x 0.36), and the texture's own five mip levels (box averages, alpha about
//! 0.16-0.2 at every level, like the chain built here) don't fill it, so that light comes from
//! something outside this material: not found (in the DNA footage the Light grenades' beams are
//! nearby; at 9.4 s a soldier walking through the chip is lit the same whitish cyan).
//!
//! The ALE effect the demo drew before (h_ee11d51f powerup_pill + light_powerup_pill) is the DNA
//! canister's (mesh h_e3e4caad, "Alien Technology Acquired!"), not this.

use super::*;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::math::Affine2;

/// The item a squad character-object drops (its h_0f77963d inventory-drop; 130 of the levels'
/// drops name it, the others are enemies' medkits and ammo).
const MEMORY_CHIP: u32 = 0xF50F_94F9;
/// The self-lit shader's colour constant (and texture slot) h_e01baa40.
const H_GLOW: u32 = 0xE01B_AA40;
/// The self-lit shader's scroll: u and v rates (per second) and how long it scrolls (s); see
/// FUN_0008fca0 in the module comment.
const H_SCROLL_U: u32 = 0xFB0B_FF34;
const H_SCROLL_V: u32 = 0xE002_AE8E;
const H_SCROLL_TIME: u32 = 0x08C2_D2EE;
/// The self-lit shader's blend mode (h_0f5ae13f: 1 additive) and its cull flag (h_18954f8d: 1
/// no culling), see FUN_0008fd20.
const H_BLEND_MODE: u32 = 0x0F5A_E13F;
const H_NO_CULL: u32 = 0x1895_4F8D;
/// How much nearer (m) the chip sorts among see-through things than it is: past the terrain's
/// blended layers (whose chunk middles can be nearer the camera than the chip). The demo's
/// choice. Bevy also hands `depth_bias` to the pipeline as a constant depth bias; at this value
/// that moves the chip's depth by about 1e-4 of itself, so it doesn't show through walls. The
/// cost: it sorts after every other blended thing (gas clouds, blood mist, liquids, see-through
/// materials), so one of those between it and the camera doesn't veil it.
const CHIP_SORT_BIAS: f32 = 1000.0;
/// The chip appears this long after a death (s), this far from where they fell (m) and its
/// middle this high over the ground (m). The demo's choice, kept from the old DNA (fitted then
/// to todo/DNA + Weapon Pickups.mp4); the game's own placement isn't found. The mesh's root part
/// is a dummy triangle 1 m under the cube, which may be a ground anchor (a guess, not used).
const CHIP_DELAY: f32 = 0.2;
const CHIP_SIDE: f32 = 0.9;
const CHIP_HEIGHT: f32 = 0.55;
/// How near (m) the player's feet must come to take it: across, and up or down (the medkits'
/// reach in play_pickups.rs; the game's isn't known). Only the player takes it: the demo's choice
/// (the squad AI would otherwise pick it up as it walks past the body).
const REACH: f32 = 1.0;
const REACH_UP: f32 = 1.5;

/// The chip's model: each geoset's mesh, the material made from the data, and its part's
/// offset; and the item (its message and sound).
#[derive(Resource, Default)]
struct ChipModel {
    parts: Vec<(Handle<Mesh>, Handle<StandardMaterial>, Vec3)>,
    message: String,
    sound: u32,
}

/// Whether BF_TEST_CHIP's chip has been dropped on this map (reset as each map starts).
#[derive(Resource, Default)]
struct TestChipDropped(bool);

/// A memory chip lying beside a body.
#[derive(Component)]
struct Chip;

pub fn plugin(app: &mut App) {
    app.init_resource::<TestChipDropped>()
        .add_systems(OnEnter(AppState::Playing), (load_chip.after(setup), |mut d: ResMut<TestChipDropped>| d.0 = false))
        .add_systems(Update, (drop_chips, take_chips).chain().after(update_player).before(play_sounds)
            .run_if(in_state(AppState::Playing)));
}

/// The chip's mesh (the item's archetype) and its material as the self-lit shader draws it.
#[allow(clippy::too_many_arguments)]
fn load_chip(mut commands: Commands, mut game: ResMut<GameData>, mut meshes: ResMut<Assets<Mesh>>,
             mut materials: ResMut<Assets<StandardMaterial>>, mut images: ResMut<Assets<Image>>) {
    let mut chip = ChipModel::default();
    let item = game.0.items.get(&MEMORY_CHIP);
    // the item's message (its <base> h_1dd3a49d) and pickup-sound
    chip.message = item.map(|i| i.message).and_then(|m| game.0.strings.get(&m).cloned()).unwrap_or_else(|| "Memory Chip Recovered!".into());
    chip.sound = item.map_or(0, |i| i.sound);
    let log = std::env::var("BF_PICKUP_LOG").is_ok();
    let model = game.0.object_meshes.get(&MEMORY_CHIP).copied()
        .and_then(|arch| bf_viewer::bf::weapon::WeaponModel::load(&game.0, arch).ok());
    let Some(model) = model else {
        warn!("memory chip h_{MEMORY_CHIP:08x}: no model in this level's data");
        commands.insert_resource(chip);
        return;
    };
    for part in &model.parts {
        for g in &part.geosets {
            // (the root part's dummy triangle has no material the levels define: not drawn)
            if !game.0.has_material(g.material) || g.indices.len() < 6 {
                continue;
            }
            let id = g.material;
            let consts = game.0.material_constants.get(&id).cloned().unwrap_or_default();
            let c = |k: u32, i: usize, or: f32| consts.get(&k).and_then(|v| v.get(i).copied()).unwrap_or(or);
            if log {
                println!("memory chip geoset: {} vertices, material h_{id:08x} type h_{:08x} texture {:x?} constants {:x?}",
                         g.positions.len(), game.0.material_type(id), game.0.material_texture(id), consts);
            }
            let mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, bevy::asset::RenderAssetUsages::default())
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, g.positions.clone())
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, g.normals.clone())
                .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, g.uvs.clone())
                .with_inserted_indices(bevy::render::mesh::Indices::U32(g.indices.clone()));
            // the texture as it is (its alpha is the trace mask), repeating, with mip levels
            let texture = game.0.material_texture(id).and_then(|t| game.0.texture_rgba(t)).map(|(w, h, px)| images.add(mipmapped(w, h, px)));
            // the scroll's resting offset: rate x how long it scrolls (FUN_0008fca0). Without
            // h_08c2d2ee the game's time left is FLT_MAX (FUN_0008fb90 sets +0x54), so such a
            // material would scroll for good; the chip's has it (0.12 s), and a material without
            // one isn't drawn by this module, so it's held still here (0).
            let time = c(H_SCROLL_TIME, 0, 0.0);
            let offset = Vec2::new(c(H_SCROLL_U, 0, 0.0) * time, c(H_SCROLL_V, 0, 0.0) * time);
            let additive = c(H_BLEND_MODE, 0, 0.0) == 1.0;
            let material = materials.add(StandardMaterial {
                // the pixel program's one combiner stage: colour = texture x c0, alpha = texture
                // alpha x c0 alpha, where c0 is the shader's h_e01baa40 colour (at +0x70) with
                // its `alpha` (+0x7c) as the fourth component (FUN_0008fa50 maps the names to
                // those offsets; FUN_000a4d80 packs the four into the constant). Nothing else
                // goes in: no vertex colour, no lighting, no second texture.
                base_color: Color::srgba(c(H_GLOW, 0, 1.0), c(H_GLOW, 1, 1.0), c(H_GLOW, 2, 1.0), c(bf_viewer::bf::hash::h("alpha"), 0, 1.0)),
                base_color_texture: texture,
                uv_transform: Affine2::from_translation(offset),
                unlit: true,
                // fogged: the shader's pixel program (pixel shader 0, its definition at 0x3ddf80, set
                // by FUN_000a4d00 from FUN_0008fd20) ends in a final combiner that mixes the colour
                // toward the fog colour by the fog factor
                fog_enabled: true,
                // drawn after the terrain's blended texture layers (the same pass, sorted by their
                // chunk's middle plus their own bias, level_scene.rs): without z-writes the chip
                // was painted over by them on sdm_e34 and all but vanished
                depth_bias: CHIP_SORT_BIAS,
                // SRCALPHA / INVSRCALPHA without z-writes (Bevy's Blend writes no depth either);
                // mode 1 would add
                alpha_mode: if additive { AlphaMode::Add } else { AlphaMode::Blend },
                double_sided: c(H_NO_CULL, 0, 0.0) == 1.0,
                cull_mode: if c(H_NO_CULL, 0, 0.0) == 1.0 { None } else { Some(bevy::render::render_resource::Face::Back) },
                ..default()
            });
            chip.parts.push((meshes.add(mesh), material, part.offset));
        }
    }
    if log {
        println!("memory chip h_{MEMORY_CHIP:08x}: {} geosets drawn, message {:?}, sound h_{:08x}", chip.parts.len(), chip.message, chip.sound);
    }
    commands.insert_resource(chip);
}

/// A repeating sRGB texture with its mip levels, each a 2 x 2 box average of the one above.
/// Without them the 64 x 64 traces, a pixel or two wide, break up into scattered lines once the
/// chip is a few metres off; the console samples mip levels, and the footage's distant chip
/// (todo/Friendly Fire 2 + Death Cam.mp4, 33.3 s) is an even soft green. The file carries five
/// levels of its own (textures-*.xmb: 64 x 64 down to 4 x 4, 5456 bytes); exported, they're box
/// averages too (mean alpha 39, 49, 52, 43, 40 of 255 against this chain's about 39 at each), but
/// the format reader decodes the top level only, so the chain is rebuilt here.
fn mipmapped(w: u32, h: u32, px: Vec<u8>) -> Image {
    let mut data = px.clone();
    let (mut level, mut lw, mut lh, mut levels) = (px, w, h, 1);
    while lw > 1 || lh > 1 {
        let (nw, nh) = ((lw / 2).max(1), (lh / 2).max(1));
        let mut next = vec![0u8; (nw * nh * 4) as usize];
        for y in 0..nh {
            for x in 0..nw {
                for c in 0..4 {
                    let at = |sx: u32, sy: u32| level[((sy.min(lh - 1) * lw + sx.min(lw - 1)) * 4 + c) as usize] as u32;
                    let sum = at(2 * x, 2 * y) + at(2 * x + 1, 2 * y) + at(2 * x, 2 * y + 1) + at(2 * x + 1, 2 * y + 1);
                    next[((y * nw + x) * 4 + c) as usize] = ((sum + 2) / 4) as u8;
                }
            }
        }
        data.extend_from_slice(&next);
        (level, lw, lh, levels) = (next, nw, nh, levels + 1);
    }
    let mut img = Image::new(bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                             bevy::render::render_resource::TextureDimension::D2, vec![0; (w * h * 4) as usize],
                             bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, bevy::asset::RenderAssetUsages::default());
    img.texture_descriptor.mip_level_count = levels;
    img.data = Some(data);
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat, address_mode_v: ImageAddressMode::Repeat, ..ImageSamplerDescriptor::linear()
    });
    img
}

/// A chip beside each squad member CHIP_DELAY s after they die (`dna_done`: once per death).
/// Test hook: BF_TEST_CHIP=<x>,<z>[,<s>] also drops one at (x, z) at that time (default 0.5 s;
/// on the floor there below 2 m over the player's middle), to look at it and walk into it (with
/// BF_TEST_GOTO) without a death.
fn drop_chips(mut commands: Commands, chip: Option<Res<ChipModel>>, mut player: ResMut<Player>, mut squad: ResMut<Squad>,
              mut test_dropped: ResMut<TestChipDropped>) {
    let Some(chip) = chip else { return };
    let test = std::env::var("BF_TEST_CHIP").ok()
        .map(|v| v.split(',').filter_map(|x| x.trim().parse::<f32>().ok()).collect::<Vec<f32>>()).filter(|v| v.len() >= 2);
    if let Some(t) = test.filter(|t| !test_dropped.0 && player.sim_time >= t.get(2).copied().unwrap_or(0.5)) {
        test_dropped.0 = true;
        let at = Vec3::new(t[0], floor_y(t[0], t[1], player.position.y + 2.0) + CHIP_HEIGHT, t[1]);
        spawn_chip(&mut commands, &chip, at);
        if std::env::var("BF_PICKUP_LOG").is_ok() {
            println!("BF_TEST_CHIP: a memory chip at {at:.2}, t {:.2}", player.sim_time);
        }
    }
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        if !u.dead || u.dna_done || u.dead_for < CHIP_DELAY {
            continue;
        }
        u.dna_done = true;
        let a = u.random(3600) as f32 / 3600.0 * std::f32::consts::TAU;
        let (x, z) = (u.position.x + CHIP_SIDE * a.cos(), u.position.z + CHIP_SIDE * a.sin());
        let at = Vec3::new(x, floor_y(x, z, u.position.y + GROUND + 1.0) + CHIP_HEIGHT, z);
        spawn_chip(&mut commands, &chip, at);
        if std::env::var("BF_PICKUP_LOG").is_ok() {
            println!("{} dropped a memory chip at {at:.2}", CHARACTERS[u.character]);
        }
    }
}

/// A chip at `at` (its middle), unturned.
fn spawn_chip(commands: &mut Commands, chip: &ChipModel, at: Vec3) {
    let e = commands.spawn((Transform::from_translation(at), Visibility::Inherited, Chip, Name::new("memory chip"))).id();
    for (mesh, material, offset) in &chip.parts {
        commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()), Transform::from_translation(*offset),
                        bevy::pbr::NotShadowCaster, ChildOf(e)));
    }
}

/// The player walking into a chip takes it: it goes, its pickup sound plays and the pickup lines
/// show the item's message.
fn take_chips(mut commands: Commands, chip: Option<Res<ChipModel>>, mut player: ResMut<Player>,
              mut feed: ResMut<super::pickups::PickupFeed>, chips: Query<(Entity, &Transform), With<Chip>>) {
    let Some(chip) = chip else { return };
    if player.dead {
        return;
    }
    let feet = player.position + Vec3::Y * GROUND;
    for (e, t) in &chips {
        let at = t.translation;
        if Vec2::new(at.x - feet.x, at.z - feet.z).length() >= REACH || (at.y - feet.y).abs() >= REACH_UP {
            continue;
        }
        commands.entity(e).despawn();
        if chip.sound != 0 {
            player.sound_queue.push((chip.sound, 1.0));
        }
        // a line of its own, without a count (n 0, see play_hud.rs)
        feed.0.push((chip.message.clone(), 0, super::pickups::FEED_TIME));
        if std::env::var("BF_PICKUP_LOG").is_ok() {
            println!("took a memory chip at {at:.2}: {:?}", chip.message);
        }
    }
}

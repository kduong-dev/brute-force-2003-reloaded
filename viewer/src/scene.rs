//! Turning a decoded `Character` into Bevy entities: a joint hierarchy plus GPU-skinned meshes
//! with the game's materials. Shared by the viewer and the playable demo.

use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::{
        mesh::{
            skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
            Indices, PrimitiveTopology, VertexAttributeValues,
        },
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

use crate::bf::character::{Address, Character, Game, Geoset};
use crate::bf::texture::Mips;

/// Marks the skeleton's root joint entity.
#[derive(Component)]
pub struct RootBone;

pub struct ModelAssets<'a> {
    pub meshes: &'a mut Assets<Mesh>,
    pub materials: &'a mut Assets<StandardMaterial>,
    pub images: &'a mut Assets<Image>,
    pub bindposes: &'a mut Assets<SkinnedMeshInverseBindposes>,
}

/// The anisotropy the world's textures are sampled with (wgpu's clamp, 1 = none). The game
/// asks for none: the texture-stage states' MAGFILTER and MINFILTER (the render-state cache's
/// slots +0x77c / +0x794 for stage 0, 0x2b8 on for each next stage, set through FUN_0006a610)
/// are written POINT (1), LINEAR (2) or, for FUN_000a54f0's multiply pass (kind 3, 0xa5df5),
/// GAUSSIANCUBIC (5), never D3DTEXF_ANISOTROPIC (3), the only filter that reads MAXANISOTROPY;
/// and no stage's MAXANISOTROPY slot (+0x7f4, +0xaac, +0xd64, +0x101c) is ever set.
/// `BF_ANISOTROPY=<1-16>` overrides it (to compare).
pub const ANISOTROPY: u16 = 1;

/// The sampler of a world texture (terrain, scenery, characters, effects), sampled past its
/// edges as its material says (`Game::texture_address`): linear within a level and between
/// levels, as the console's materials sample: MAGFILTER / MINFILTER LINEAR, and MIPFILTER the
/// draw context's +0x998 (written into the stages by the material classes, e.g. 0xa5fc3, and
/// for all four stages at 0xe5811-0xe5871), which its constructor FUN_000e6ef0 sets to 2,
/// D3DTEXF_LINEAR (0xe7300), the only value it's given; no LOD bias (that slot, +0x7c4, is never
/// written). Anisotropy: `ANISOTROPY`.
pub fn world_sampler(address: [Address; 2]) -> ImageSamplerDescriptor {
    let mode = |a: Address| match a {
        Address::Wrap => ImageAddressMode::Repeat,
        Address::Clamp => ImageAddressMode::ClampToEdge,
        Address::Mirror => ImageAddressMode::MirrorRepeat,
    };
    let anisotropy = std::env::var("BF_ANISOTROPY").ok().and_then(|v| v.parse::<u16>().ok()).unwrap_or(ANISOTROPY).clamp(1, 16);
    ImageSamplerDescriptor {
        address_mode_u: mode(address[0]),
        address_mode_v: mode(address[1]),
        anisotropy_clamp: anisotropy,
        ..ImageSamplerDescriptor::linear()
    }
}

/// A world texture as a Bevy image: every level of `mips` (the chain its archive ships,
/// `Game::texture_mips`; a texture that ships one level keeps one, as on the console), in
/// `format`, sampled by `world_sampler`. Before #257 every texture was its top level alone, so
/// the ground shimmered and blurred into noise past a few metres and raw texels showed close up.
pub fn mip_image(mips: &Mips, format: TextureFormat, address: [Address; 2]) -> Image {
    let top = mips.top();
    // (wgpu wants level n at the top's size halved n times, at least 1: the archive's are; the
    // top level is always kept)
    let count = mips.levels.iter().enumerate()
        .take_while(|(n, l)| l.w == (top.w >> n).max(1) && l.h == (top.h >> n).max(1) && l.px.len() == (l.w * l.h * 4) as usize)
        .count().max(1);
    let mut img = Image::new_uninit(Extent3d { width: top.w, height: top.h, depth_or_array_layers: 1 }, TextureDimension::D2,
                                    format, RenderAssetUsages::default());
    img.texture_descriptor.mip_level_count = count as u32;
    img.data = Some(mips.levels[..count].iter().flat_map(|l| l.px.iter().copied()).collect());
    img.sampler = ImageSampler::Descriptor(world_sampler(address));
    img
}

/// A texture, sampled past its edges as its material says (`Game::texture_address`).
fn image(images: &mut Assets<Image>, mips: &Mips, format: TextureFormat, address: [Address; 2]) -> Handle<Image> {
    images.add(mip_image(mips, format, address))
}

/// Spawn the skeleton (children of `root`, at the bind pose) and the skinned geosets.
/// Returns the joint entities in bone order.
pub fn spawn_model(commands: &mut Commands, game: &mut Game, model: &Character, assets: &mut ModelAssets,
                   root: Entity, shading: bool) -> Vec<Entity> {
    // skeleton: one entity per bone, parented like the game's joints, at the bind pose
    let local = model.bind_local();
    let mut joints: Vec<Entity> = Vec::with_capacity(model.bones.len());
    for (i, (q, t)) in local.iter().enumerate() {
        let parent = model.parent[i].map(|p| joints[p]).unwrap_or(root);
        let mut e = commands.spawn((Transform::from_rotation(*q).with_translation(*t), Visibility::default(),
                                    Name::new(format!("h_{:08x}", model.bones[i])), ChildOf(parent)));
        if model.parent[i].is_none() {
            e.insert(RootBone);
        }
        joints.push(e.id());
    }
    let inverse = assets.bindposes.add(SkinnedMeshInverseBindposes::from(model.inverse_bind.clone()));

    // skinned geosets with their textures
    for g in &model.geosets {
        let mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, g.positions.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, g.normals.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, g.uvs.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_JOINT_INDEX, VertexAttributeValues::Uint16x4(g.joints.clone()))
            .with_inserted_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, g.weights.clone())
            .with_inserted_indices(Indices::U32(g.indices.clone()));
        let material = material(game, assets, g.material, shading);
        commands.spawn((
            Mesh3d(assets.meshes.add(mesh)),
            MeshMaterial3d(material),
            SkinnedMesh { inverse_bindposes: inverse.clone(), joints: joints.clone() },
            // the bounding box comes from the bind pose, so root motion would get the character
            // culled while still on screen
            bevy::render::view::NoFrustumCulling,
            Transform::default(),
            ChildOf(root),
        ));
    }
    joints
}

/// A game material as a StandardMaterial: its colour texture, plus (with `shading`) the
/// Color-Specular look where the texture's alpha marks the shiny areas.
pub fn material(game: &mut Game, assets: &mut ModelAssets, material: u32, shading: bool) -> Handle<StandardMaterial> {
    let address = game.material_texture(material).map_or_else(Default::default, |t| game.texture_address(material, t));
    let mips = game.material_texture(material).and_then(|t| game.texture_mips(t));
    // opaque: the skin's alpha is a specular mask, not transparency
    let mut mat = StandardMaterial { base_color: Color::srgb(0.7, 0.7, 0.7), perceptual_roughness: 0.85, ..default() };
    if let Some(mips) = mips {
        let base = image(assets.images, &mips, TextureFormat::Rgba8UnormSrgb, address);
        mat.base_color = Color::WHITE;
        mat.base_color_texture = Some(base.clone());
        if let Some(spec) = shading.then(|| game.material_specular(material)).flatten() {
            // Game look (BF_CS_rt "Color-Specular"): alpha says where it's shiny. Bevy reads
            // reflectance from the specular texture's alpha (KHR_materials_specular), so the
            // skin doubles as the specular map; a roughness map from the same alpha makes the
            // masked areas glossy (by the material's glossiness) and the rest matte (each mip
            // level from its own alpha).
            let shiny = (0.55 - 0.45 * spec.gloss).clamp(0.1, 0.6);
            let mut rough = mips.clone();
            rough.map_texels(|p| {
                let a = p[3] as f32 / 255.0;
                p.copy_from_slice(&[0, ((0.92 + (shiny - 0.92) * a) * 255.0) as u8, 0, 255]);  // G = roughness, B = metallic
            });
            mat.metallic_roughness_texture = Some(image(assets.images, &rough, TextureFormat::Rgba8Unorm, address));
            mat.perceptual_roughness = 1.0;      // multiplies the map
            mat.metallic = 0.0;
            mat.specular_texture = Some(base);
            mat.reflectance = (1.2 + 3.0 * spec.level).min(2.0);
            mat.specular_tint = Color::srgb(spec.tint[0], spec.tint[1], spec.tint[2]);
        }
    }
    assets.materials.add(mat)
}

/// Mesh and material handles of static (unskinned) geosets, for spawning copies.
pub fn static_meshes(game: &mut Game, geosets: &[Geoset], assets: &mut ModelAssets, shading: bool)
                     -> Vec<(Handle<Mesh>, Handle<StandardMaterial>)> {
    geosets.iter().map(|g| {
        let mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, g.positions.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, g.normals.clone())
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, g.uvs.clone())
            .with_inserted_indices(Indices::U32(g.indices.clone()));
        (assets.meshes.add(mesh), material(game, assets, g.material, shading))
    }).collect()
}

/// Static (unskinned) geosets as children of `parent`, e.g. a weapon part.
pub fn spawn_static(commands: &mut Commands, game: &mut Game, geosets: &[Geoset], assets: &mut ModelAssets,
                    parent: Entity, shading: bool) {
    for (mesh, material) in static_meshes(game, geosets, assets, shading) {
        commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::default(), ChildOf(parent)));
    }
}

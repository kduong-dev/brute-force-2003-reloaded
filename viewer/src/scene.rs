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

use crate::bf::character::{Character, Game, Geoset};

/// Marks the skeleton's root joint entity.
#[derive(Component)]
pub struct RootBone;

pub struct ModelAssets<'a> {
    pub meshes: &'a mut Assets<Mesh>,
    pub materials: &'a mut Assets<StandardMaterial>,
    pub images: &'a mut Assets<Image>,
    pub bindposes: &'a mut Assets<SkinnedMeshInverseBindposes>,
}

fn image(images: &mut Assets<Image>, px: Vec<u8>, w: u32, h: u32, format: TextureFormat) -> Handle<Image> {
    let mut img = Image::new(Extent3d { width: w, height: h, depth_or_array_layers: 1 }, TextureDimension::D2, px,
                             format, RenderAssetUsages::default());
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    });
    images.add(img)
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
    let rgba = game.material_texture(material).and_then(|t| game.texture_rgba(t));
    // opaque: the skin's alpha is a specular mask, not transparency
    let mut mat = StandardMaterial { base_color: Color::srgb(0.7, 0.7, 0.7), perceptual_roughness: 0.85, ..default() };
    if let Some((w, h, px)) = rgba {
        let base = image(assets.images, px.clone(), w, h, TextureFormat::Rgba8UnormSrgb);
        mat.base_color = Color::WHITE;
        mat.base_color_texture = Some(base.clone());
        if let Some(spec) = shading.then(|| game.material_specular(material)).flatten() {
            // Game look (BF_CS_rt "Color-Specular"): alpha says where it's shiny. Bevy reads
            // reflectance from the specular texture's alpha (KHR_materials_specular), so the
            // skin doubles as the specular map; a roughness map from the same alpha makes the
            // masked areas glossy (by the material's glossiness) and the rest matte.
            let shiny = (0.55 - 0.45 * spec.gloss).clamp(0.1, 0.6);
            let rough: Vec<u8> = px.chunks_exact(4).flat_map(|p| {
                let a = p[3] as f32 / 255.0;
                [0, ((0.92 + (shiny - 0.92) * a) * 255.0) as u8, 0, 255]  // G = roughness, B = metallic
            }).collect();
            mat.metallic_roughness_texture = Some(image(assets.images, rough, w, h, TextureFormat::Rgba8Unorm));
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

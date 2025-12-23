use std::path::Path;
use crate::{GameObject, GpuBufferBuilder, Material, PBRVertex, Renderable, Transform, Vertex, WorldRenderer};

use ash::vk;

use glam::{Mat4, Vec2, Vec3, Vec4};
use gltf::*;


#[derive(Copy, Clone)]
pub enum MaterialType {
    Lambertian = 0,
    Metal = 1,
    Dielectric = 2,
    DiffuseLight = 3,
}

pub struct PBRMaterial {
    pub diffuse_map: u32,
    pub normal_map: u32,
    pub metallic_roughness_map: u32,
    pub occlusion_map: u32,
    pub base_color_factor: Vec4,
    pub metallic_factor: f32,
    pub roughness_factor: f32,

    // Ray tracing properties
    pub material_type: MaterialType, // 0 = lambertian, 1 = metal, 2 = dielectric, 3 = diffuse light
    pub material_property: f32,      // metal = fuzz, dielectric = index of refraction
}




pub struct GltfLoader {

}


impl GltfLoader {

    pub fn new<S: AsRef<Path>>(world: &mut WorldRenderer, path: S) -> Option<GameObject> {

        let (gltf, buffers, mut images) = gltf::import(path.as_ref()).unwrap();

        let mut root = GameObject::new(world);

        // for image in images {
            
        // }

        for scene in gltf.scenes() {
            for node in scene.nodes() {
                log::info!("Node: {}", node.name().unwrap_or("Unknown"));
               let child = load_node(world, node, &buffers, Mat4::IDENTITY);
               root.add_child(child);
            }
        }

        Some(root)
    }
}

pub fn load_node(
    world: &mut WorldRenderer, 
    node: gltf::Node, 
    buffers: &[gltf::buffer::Data], 
    parent_transform: Mat4
) -> GameObject {
    
    // ✅ Правильно: составляем локальную трансформацию
    let local_transform = Mat4::from_cols_array_2d(&node.transform().matrix());
    let world_transform = parent_transform * local_transform;
    
    let mut game_node = GameObject::new(world);
    
    // ✅ Устанавливаем трансформацию в GameObject
    // (НЕ применяем к вершинам!)
    // let (scale, rotation, translation) = world_transform.to_scale_rotation_translation();
    // *game_node.transform_mut(world) = Transform {
    //     pos: translation.to_array(),
    //     rot: rotation.to_array(),
    //     scale: scale.to_array(),
    // };
    
    // Рекурсивно загружаем детей
    for child_node in node.children() {
        log::info!("Child Node: {}", child_node.name().unwrap_or("Unknown"));
        let child = load_node(world, child_node, buffers, world_transform);
        game_node.add_child(child);
    }
    
    // Загружаем меш
    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|i| Some(&buffers[i.index()]));
            
            let indices: Vec<_> = reader
                .read_indices()
                .unwrap()
                .into_u32()
                .collect();
            
            let positions: Vec<_> = reader
                .read_positions()
                .unwrap()
                .map(Vec3::from)
                .collect();
            
            let colors: Vec<_> = if let Some(colors) = reader.read_colors(0) {
                colors.into_rgba_f32().map(|c| Vec4::from(c)).collect()
            } else {
                vec![Vec4::new(1.0, 1.0, 1.0, 1.0); positions.len()]
            };
            
            
            // ❌ НЕ применяем трансформацию здесь!
            // ✅ Вершины в LOCAL space
            let vertices: Vec<Vertex> = positions.iter()
                .zip(colors.iter())
                .map(|(pos, color)| {

                    let pos = world_transform.transform_point3(*pos);

                    Vertex {
                        pos: [pos.x, pos.y, pos.z], // В локальных координатах!
                        color: [1.0 / pos.x, 1.0 / pos.y, 1.0 / pos.z],
                    }
                }
                
                )
                .collect();
            
            let material = world.create_material(Material::default());
            let mesh_handle = world.create_simple_mesh(Some(indices), vertices);
            
            game_node.add_component(Renderable::new(mesh_handle, material));
        }
    }
    
    game_node
}
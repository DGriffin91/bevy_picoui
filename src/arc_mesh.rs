use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

pub fn generate_arc_mesh(sides: usize, radius: f32, start_angle: f32, end_angle: f32) -> Mesh {
    let mut positions = Vec::with_capacity(sides + 1);
    let mut normals = Vec::with_capacity(sides + 1);
    let mut uvs = Vec::with_capacity(sides + 1);

    let step = (end_angle - start_angle) / sides as f32;

    for i in 0..=sides {
        let theta = start_angle + i as f32 * step;
        let (sin, cos) = theta.sin_cos();

        positions.push([cos * radius, sin * radius, 0.0]);
        normals.push([0.0, 0.0, 1.0]);
        uvs.push([0.5 * (cos + 1.0), 1.0 - 0.5 * (sin + 1.0)]);
    }

    let mut indices = Vec::with_capacity((sides - 1) * 3);
    for i in 1..=(sides as u32) {
        indices.extend_from_slice(&[0, i + 1, i]);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh
}

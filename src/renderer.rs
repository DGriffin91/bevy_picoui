use bevy::{
    math::{vec2, vec3, Vec3Swizzles, Vec4Swizzles},
    prelude::*,
    sprite::{Anchor, MaterialMesh2dBundle},
    text::{BreakLineOn, Text2dBounds},
    utils::HashMap,
};
use core::hash::Hash;
use core::hash::Hasher;
use std::{collections::hash_map::DefaultHasher, f32::consts::PI};

use crate::{
    pico::{get_bbox, Drag, Pico, Pico2dCamera, StateItem},
    MeshHandles,
};

#[derive(Component)]
pub struct PicoEntity {
    pub spatial_id: u64,
    pub anchor: Anchor,
    pub size: Vec2,
}

#[allow(clippy::too_many_arguments)]
pub fn render(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mesh_handles: Res<MeshHandles>,
    time: Res<Time>,
    camera: Query<(&Camera, &GlobalTransform), With<Pico2dCamera>>,
    windows: Query<&Window>,
    mut pico: ResMut<Pico>,
    mut pico_entites: Query<(Entity, &mut Transform, &PicoEntity)>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut currently_dragging: Local<bool>,
) {
    let Ok((camera, camera_transform)) = camera.get_single() else {
        return;
    };
    let Ok(window) = windows.get_single() else {
        return;
    };
    let window_size = Vec2::new(window.width(), window.height());

    *currently_dragging = false;
    let mut interacting = false;
    // Age all the state items
    for (_, state_item) in pico.state.iter_mut() {
        state_item.life -= time.delta_seconds();
        state_item.hover = false;
        state_item.input = None;
        if mouse_button_input.pressed(MouseButton::Left) {
            if state_item.drag.is_some() {
                *currently_dragging = true;
                interacting = true;
            }
        } else {
            state_item.drag = None;
        }
    }

    let mut items = std::mem::take(&mut pico.items);

    // Sort so we interact in z order.
    items.sort_by(|a, b| b.depth.unwrap().partial_cmp(&a.depth.unwrap()).unwrap());

    let mut item_positions = Vec::new();

    let mut first_interact_found = false;
    for item in &mut items {
        if item.id.is_none() {
            item.id = Some(item.generate_id());
        }

        if item.spatial_id.is_none() {
            item.spatial_id = Some(item.generate_spatial_id());
        }

        let spatial_id = item.spatial_id.unwrap();

        let mut item_ndc =
            ((item.uv_position - Vec2::splat(0.5)) * vec2(2.0, -2.0)).extend(item.depth.unwrap());

        if let Some(position_3d) = item.position_3d {
            item_ndc = camera
                .world_to_ndc(camera_transform, position_3d)
                .unwrap_or(Vec3::NAN);
            // include 2d offset
            item_ndc += ((item.uv_position) * vec2(2.0, -2.0)).extend(item.depth.unwrap());
        }

        let item_pos = item_ndc.xy() * window_size * 0.5;
        item_positions.push(item_pos);

        if let Some(existing_state_item) = pico.state.get_mut(&spatial_id) {
            // If a item in the state matches one created this frame keep it around
            existing_state_item.life = existing_state_item.life.max(0.0);
            let Ok((_, mut trans, pico_entity)) =
                pico_entites.get_mut(existing_state_item.entity.unwrap())
            else {
                continue;
            };
            trans.translation = item_pos.extend(item_ndc.z);

            if !existing_state_item.interactable {
                continue;
            }

            if let Some(cursor_pos) = window.cursor_position() {
                if mouse_button_input.pressed(MouseButton::Left) && !first_interact_found {
                    if let Some(drag) = &mut existing_state_item.drag {
                        drag.last_frame = drag.end;
                        drag.end = cursor_pos;
                    }
                }
                existing_state_item.bbox = get_bbox(
                    pico_entity.size / window_size,
                    trans.translation.xy() / window_size * vec2(1.0, -1.0) + 0.5,
                    &pico_entity.anchor,
                );
                let xy = existing_state_item.bbox.xy() * window_size;
                let zw = existing_state_item.bbox.zw() * window_size;
                if cursor_pos.cmpge(xy).all() && cursor_pos.cmple(zw).all() {
                    existing_state_item.hover = true;
                    if !first_interact_found {
                        existing_state_item.input = Some(mouse_button_input.clone());
                        if mouse_button_input.any_just_pressed([
                            MouseButton::Left,
                            MouseButton::Right,
                            MouseButton::Middle,
                        ]) {
                            interacting = true;
                            first_interact_found = true;
                        }
                        if mouse_button_input.just_pressed(MouseButton::Left)
                            && !*currently_dragging
                            && existing_state_item.drag.is_none()
                        {
                            existing_state_item.drag = Some(Drag {
                                start: cursor_pos,
                                end: cursor_pos,
                                last_frame: cursor_pos,
                            });
                        }
                    }
                }
            }
        }
    }
    let mut cached_materials: HashMap<u64, Handle<ColorMaterial>> = HashMap::new();

    // It seems that we need to add things in z order for them to show up in that order initially
    for (item, item_pos) in items.iter_mut().zip(item_positions.iter()) {
        let spatial_id = item.spatial_id.unwrap();

        let generate = if let Some(existing_state_item) = pico.state.get_mut(&spatial_id) {
            existing_state_item.id != item.id.unwrap()
        } else {
            true
        };
        if generate || pico.window_size != window_size {
            let mut corner_radius = pico.val_in_parent_y(item.corner_radius, item.uv_size)
                * item.uv_size.y
                * window_size.y;
            let size = item.uv_size * window_size;

            corner_radius = corner_radius.min(size.x).min(size.y);

            let state_item = if let Some(old_state_item) = pico.state.get_mut(&spatial_id) {
                let entity = old_state_item.entity.unwrap();
                if pico_entites.get(entity).is_ok() {
                    commands.entity(entity).despawn_recursive();
                }
                old_state_item
            } else {
                pico.state.insert(spatial_id, StateItem::default());
                pico.state.get_mut(&spatial_id).unwrap()
            };
            let text = Text {
                sections: vec![TextSection::new(
                    item.text.clone(),
                    TextStyle {
                        font_size: item.font_size * window_size.y,
                        color: item.color,
                        ..default()
                    },
                )],
                alignment: item.alignment,
                linebreak_behavior: BreakLineOn::WordBoundary,
            };
            state_item.life = item.life;
            state_item.id = item.id.unwrap();
            if item.uv_size.x > 0.0 || item.uv_size.y > 0.0 {
                let trans = Transform::from_translation(item_pos.extend(1.0));
                let mut entity = commands.spawn(PicoEntity {
                    spatial_id,
                    anchor: item.anchor.clone(),
                    size,
                });

                entity.insert(SpatialBundle {
                    transform: trans,
                    ..default()
                });

                let hasher = &mut DefaultHasher::new();
                item.background.r().to_bits().hash(hasher);
                item.background.g().to_bits().hash(hasher);
                item.background.b().to_bits().hash(hasher);
                item.background.a().to_bits().hash(hasher);
                let mat_hash = hasher.finish();

                let material_handle = if let Some(handle) = cached_materials.get(&mat_hash) {
                    handle.clone()
                } else {
                    let handle = materials.add(ColorMaterial::from(item.background));
                    cached_materials.insert(mat_hash, handle.clone());
                    handle
                };

                entity.with_children(|builder| {
                    let item_anchor_vec = item.anchor.as_vec();
                    if item.background.a() > 0.0 {
                        let anchor_trans = (-item_anchor_vec * size).extend(0.0);
                        generate_rect_entities(
                            corner_radius,
                            builder,
                            &mesh_handles,
                            material_handle,
                            anchor_trans,
                            size,
                            item_anchor_vec,
                        );
                    }

                    builder.spawn(Text2dBundle {
                        text,
                        text_anchor: item.anchor_text.clone(),
                        transform: Transform::from_translation(
                            (size * -(item_anchor_vec - item.anchor_text.as_vec())).extend(0.0001),
                        ),
                        text_2d_bounds: Text2dBounds { size },
                        ..default()
                    });
                });
                state_item.bbox = get_bbox(
                    item.uv_size,
                    trans.translation.xy() / window_size * vec2(1.0, -1.0) + 0.5,
                    &item.anchor,
                );
                state_item.interactable = true;
                state_item.entity = Some(entity.id());
            } else {
                let entity = commands
                    .spawn(Text2dBundle {
                        text,
                        text_anchor: item.anchor_text.clone(),
                        transform: Transform::from_translation(item_pos.extend(1.0)),
                        ..default()
                    })
                    .id();
                state_item.entity = Some(entity);
            }
        }
    }

    for (_, state_item) in pico.state.iter_mut() {
        let entity = state_item.entity.unwrap();
        // Remove that are no longer in use
        if state_item.life < 0.0 && pico_entites.get(entity).is_ok() {
            commands.entity(entity).despawn_recursive();
        }
    }

    for (entity, _, pico_entity) in &pico_entites {
        // Remove any orphaned
        if pico.state.get(&pico_entity.spatial_id).is_none() {
            commands.entity(entity).despawn_recursive();
        }
    }

    // clean up state
    pico.state.retain(|_, state_item| state_item.life >= 0.0);
    pico.interacting = interacting;
    pico.window_size = window_size;
    pico.mouse_button_input = Some(mouse_button_input.clone());
    pico.auto_depth = 0.5;
}

fn generate_rect_entities(
    corner_radius: f32,
    builder: &mut ChildBuilder,
    mesh_handles: &MeshHandles,
    material_handle: Handle<ColorMaterial>,
    anchor_trans: Vec3,
    size: Vec2,
    item_anchor_vec: Vec2,
) {
    if corner_radius <= 0.0 {
        builder.spawn(MaterialMesh2dBundle {
            mesh: mesh_handles.rect.clone(),
            material: material_handle.clone(),
            transform: Transform::from_translation(anchor_trans).with_scale(size.extend(1.0)),
            ..default()
        });
    } else {
        let cr2 = corner_radius * 2.0;
        // Don't bother making rectangles if corner_radius just results in circle
        if corner_radius < size.x && corner_radius < size.y {
            // Make cross shape with gaps for the arcs
            builder.spawn(MaterialMesh2dBundle {
                mesh: mesh_handles.rect.clone(),
                material: material_handle.clone(),
                transform: Transform::from_translation(anchor_trans)
                    .with_scale((size - vec2(cr2, 0.0)).extend(1.0)),
                ..default()
            });
            // Don't bother making side rectangles if corner_radius just results in half circles on the ends
            if corner_radius < size.y {
                let s = vec2(corner_radius, size.y - cr2).extend(1.0);
                let off = vec3((size.x - corner_radius) * 0.5, 0.0, 0.0);
                builder.spawn(MaterialMesh2dBundle {
                    mesh: mesh_handles.rect.clone(),
                    material: material_handle.clone(),
                    transform: Transform::from_translation(anchor_trans + off).with_scale(s),
                    ..default()
                });
                builder.spawn(MaterialMesh2dBundle {
                    mesh: mesh_handles.rect.clone(),
                    material: material_handle.clone(),
                    transform: Transform::from_translation(anchor_trans - off).with_scale(s),
                    ..default()
                });
            }
        }
        // Add arcs for corners
        for (offset, angle) in [
            (size - cr2, 0.0),
            (vec2(0.0, size.y - cr2), PI * 0.5),
            (vec2(0.0, 0.0), PI),
            (vec2(size.x - cr2, 0.0), PI * 1.5),
        ] {
            let offset = offset + corner_radius;
            builder.spawn(MaterialMesh2dBundle {
                mesh: mesh_handles.circle.clone(),
                material: material_handle.clone(),
                transform: Transform::from_translation(
                    (offset + size * (-item_anchor_vec - 0.5)).extend(0.00000005),
                )
                .with_scale(Vec3::splat(corner_radius))
                .with_rotation(Quat::from_rotation_z(angle)),
                ..default()
            });
        }
    }
}

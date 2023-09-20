use bevy::{
    math::{vec2, vec3, Vec3Swizzles, Vec4Swizzles},
    prelude::*,
    sprite::{Anchor, MaterialMesh2dBundle, Mesh2dHandle},
    text::{BreakLineOn, Text2dBounds},
    utils::HashMap,
};
use core::hash::Hasher;
use std::{collections::hash_map::DefaultHasher, f32::consts::PI};

use crate::{
    hash::hash_color,
    pico::{get_bbox, Drag, Pico, Pico2dCamera, StateItem},
    MeshHandles,
};

#[derive(Component)]
pub struct PicoEntity {
    pub spatial_id: u64,
    pub anchor: Anchor,
    pub size: Vec2,
}

pub const MAJOR_DEPTH_AUTO_STEP: f32 = 0.000001;
pub const MINOR_DEPTH_AUTO_STEP: f32 = 0.0000001;

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
    items.sort_by(|a, b| b.get_depth().partial_cmp(&a.get_depth()).unwrap());

    let mut item_positions = Vec::new();

    let mut first_interact_found = false;
    for item in &mut items {
        if item.id.is_none() {
            item.id = Some(item.generate_id());
        }

        let spatial_id = item.get_spatial_id();

        let mut item_ndc = ((item.get_uv_position() - Vec2::splat(0.5)) * vec2(2.0, -2.0))
            .extend(item.get_depth());

        if let Some(position_3d) = item.position_3d {
            item_ndc = camera
                .world_to_ndc(camera_transform, position_3d)
                .unwrap_or(Vec3::NAN);
            // include 2d offset
            item_ndc += ((item.get_uv_position()) * vec2(2.0, -2.0)).extend(item.get_depth());
        }

        let item_pos = item_ndc.xy() * window_size * 0.5;
        item_positions.push(item_pos.extend(item_ndc.z));

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
                        drag.end = cursor_pos / window_size;
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
                            let cursor_uv_pos = cursor_pos / window_size;
                            existing_state_item.drag = Some(Drag {
                                start: cursor_uv_pos,
                                end: cursor_uv_pos,
                                last_frame: cursor_uv_pos,
                            });
                        }
                    }
                }
            }
        }
    }
    let mut cached_materials = ColorMaterialCache::default();

    // It seems that we need to add things in z order for them to show up in that order initially
    for (item, item_pos) in items.iter_mut().zip(item_positions.iter()) {
        let spatial_id = item.get_spatial_id();

        let generate = if let Some(existing_state_item) = pico.state.get_mut(&spatial_id) {
            existing_state_item.id != item.id.unwrap()
        } else {
            true
        };
        if generate || pico.window_size != window_size {
            let mut corner_radius =
                pico.valp_y(item.style.corner_radius, item.get_uv_size()) * window_size.y;
            let size = item.get_uv_size() * window_size;
            let font_size = pico.valp_y(item.style.font_size, item.get_uv_size()) * window_size.y;
            let border_width =
                pico.valp_y(item.style.border_width, item.get_uv_size()) * window_size.y;
            let border_width_x2 = border_width * 2.0;

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
                        font_size,
                        color: item.style.text_color,
                        font: item.style.font.clone(),
                    },
                )],
                alignment: item.style.text_alignment,
                linebreak_behavior: BreakLineOn::WordBoundary,
            };
            state_item.life = item.get_life();
            state_item.id = item.id.unwrap();
            if item.get_uv_size().x > 0.0 || item.get_uv_size().y > 0.0 {
                let trans = Transform::from_translation(*item_pos);
                let mut entity = commands.spawn(PicoEntity {
                    spatial_id,
                    anchor: item.get_anchor(),
                    size,
                });

                entity.insert(SpatialBundle {
                    transform: trans,
                    ..default()
                });

                let material_handle =
                    cached_materials.get(item.style.background_color, &mut materials);
                let border_material_handle =
                    cached_materials.get(item.style.border_color, &mut materials);
                let using_border = item.style.border_color.a() > 0.0 && border_width > 0.0;

                entity.with_children(|builder| {
                    let item_anchor_vec = item.get_anchor().as_vec();
                    if item.style.background_color.a() > 0.0 {
                        let anchor_trans = (-item_anchor_vec * size).extend(0.0);
                        generate_rect_entities(
                            corner_radius,
                            builder,
                            &mesh_handles,
                            if using_border {
                                border_material_handle
                            } else {
                                material_handle.clone()
                            },
                            anchor_trans,
                            size,
                            0.0,
                        );
                        if using_border && border_width_x2 < size.x && border_width_x2 < size.y {
                            generate_rect_entities(
                                corner_radius,
                                builder,
                                &mesh_handles,
                                material_handle,
                                anchor_trans,
                                size - Vec2::splat(border_width_x2),
                                MINOR_DEPTH_AUTO_STEP,
                            );
                        }
                    }

                    builder.spawn(Text2dBundle {
                        text,
                        text_anchor: item.style.anchor_text.clone(),
                        transform: Transform::from_translation(
                            (size * -(item_anchor_vec - item.style.anchor_text.as_vec()))
                                .extend(0.0001),
                        ),
                        text_2d_bounds: Text2dBounds { size },
                        ..default()
                    });
                });
                state_item.bbox = get_bbox(
                    item.get_uv_size(),
                    trans.translation.xy() / window_size * vec2(1.0, -1.0) + 0.5,
                    &item.get_anchor(),
                );
                state_item.interactable = true;
                state_item.entity = Some(entity.id());
            } else {
                let entity = commands
                    .spawn((
                        PicoEntity {
                            spatial_id,
                            anchor: item.get_anchor().clone(),
                            size,
                        },
                        Text2dBundle {
                            text,
                            text_anchor: item.style.anchor_text.clone(),
                            transform: Transform::from_translation(*item_pos),
                            ..default()
                        },
                    ))
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
    pico.internal_auto_depth = 0.5;
}

#[derive(Default)]
struct ColorMaterialCache(HashMap<u64, Handle<ColorMaterial>>);

impl ColorMaterialCache {
    fn get(
        &mut self,
        color: Color,
        materials: &mut Assets<ColorMaterial>,
    ) -> Handle<ColorMaterial> {
        let hasher = &mut DefaultHasher::new();
        hash_color(&color, hasher);
        let mat_hash = hasher.finish();

        let material_handle = if let Some(handle) = self.0.get(&mat_hash) {
            handle.clone()
        } else {
            let handle = materials.add(ColorMaterial::from(color));
            self.0.insert(mat_hash, handle.clone());
            handle
        };
        material_handle
    }
}

fn generate_rect_entities(
    corner_radius: f32,
    builder: &mut ChildBuilder,
    mesh_handles: &MeshHandles,
    material_handle: Handle<ColorMaterial>,
    anchor_trans: Vec3,
    size: Vec2,
    depth_bias: f32,
) {
    let anchor_trans = anchor_trans + vec3(0.0, 0.0, depth_bias);
    if corner_radius <= 0.0 {
        builder.spawn(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(mesh_handles.rect.clone_weak()),
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
                mesh: Mesh2dHandle(mesh_handles.rect.clone_weak()),
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
                    mesh: Mesh2dHandle(mesh_handles.rect.clone_weak()),
                    material: material_handle.clone_weak(),
                    transform: Transform::from_translation(anchor_trans + off).with_scale(s),
                    ..default()
                });
                builder.spawn(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(mesh_handles.rect.clone_weak()),
                    material: material_handle.clone_weak(),
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
                mesh: Mesh2dHandle(mesh_handles.circle.clone_weak()),
                material: material_handle.clone_weak(),
                transform: Transform::from_translation(
                    (anchor_trans.xy() + offset - size * 0.5)
                        .extend(MINOR_DEPTH_AUTO_STEP + depth_bias),
                )
                .with_scale(Vec3::splat(corner_radius))
                .with_rotation(Quat::from_rotation_z(angle)),
                ..default()
            });
        }
    }
}

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    input::InputSystem,
    math::{vec2, Vec3Swizzles},
    prelude::*,
    sprite::Anchor,
    text::{BreakLineOn, Text2dBounds},
};
use core::hash::Hash;
use core::hash::Hasher;
use std::collections::hash_map::DefaultHasher;

use bevy::utils::HashMap;

pub struct PicoPlugin {
    // Set if using in a scene with no 2d camera
    pub create_default_cam_with_order: Option<isize>,
}

#[derive(Resource)]
pub struct CreateDefaultCamWithOrder(isize);

impl Plugin for PicoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Pico>()
            .add_systems(PreUpdate, render.after(InputSystem));
        if let Some(n) = self.create_default_cam_with_order {
            app.insert_resource(CreateDefaultCamWithOrder(n))
                .add_systems(Startup, setup_default_cam);
        }
    }
}

fn setup_default_cam(mut commands: Commands, order: Res<CreateDefaultCamWithOrder>) {
    commands.spawn(Camera2dBundle {
        camera: Camera {
            order: order.0,
            ..default()
        },
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::None,
        },
        ..default()
    });
}

// -------------------------
// Button example widget
// -------------------------

pub fn button(pico: &mut Pico, position: Vec3, rect: Vec2, text: &str) -> usize {
    let button_index = pico.items.len();
    let pico = pico.add(PicoItem {
        text: text.to_string(),
        position,
        rect,
        ..default()
    });
    let a = if pico.hovered(button_index) {
        0.03
    } else {
        0.01
    };
    pico.get_mut(button_index).background = Color::rgba(1.0, 1.0, 1.0, a);
    button_index
}

// -------------------------
// Value drag example widget
// -------------------------

pub struct DragValue {
    pub value: f32,
    pub text_index: usize,
    pub drag_index: usize,
}

pub fn drag_value(
    pico: &mut Pico,
    position: Vec3,
    height: f32,
    label_width: f32,
    drag_width: f32,
    label: &str,
    scale: f32,
    value: f32,
) -> DragValue {
    let mut value = value;
    let text_item = pico.items.len();
    let pico = pico.add(PicoItem {
        text: label.to_string(),
        position,
        rect: vec2(label_width, height),
        anchor: Anchor::CenterLeft,
        rect_anchor: Anchor::CenterLeft,
        ..default()
    });

    let mut b = PicoItem {
        text: format!("{:.2}", value),
        position: position + Vec3::X * label_width,
        rect: vec2(drag_width, height),
        ..default()
    };

    b.rect_anchor = Anchor::CenterLeft;
    let drag_item = pico.items.len();
    let pico = pico.add(b);
    let dragging = if let Some(dragged) = pico.dragged(drag_item) {
        pico.items.last_mut().unwrap().text = format!("{:.2}", dragged.total_delta().x * scale);
        value = dragged.delta().x * scale + value;
        true
    } else {
        false
    };
    let a = if pico.hovered(drag_item) || dragging {
        0.03
    } else {
        0.01
    };
    pico.get_mut(drag_item).background = Color::rgba(1.0, 1.0, 1.0, a);
    DragValue {
        value,
        text_index: text_item,
        drag_index: drag_item,
    }
}

// Only supports one camera.
#[derive(Component)]
pub struct ImTextCamera;

#[derive(Component, Clone, Debug)]
pub struct PicoItem {
    pub text: String,
    /// If position_3d is false position is the screen uv coords with 0.0, 0.0 at top left
    /// If position_3d is true position is the world space translation
    pub position: Vec3,
    pub position_3d: bool,
    // 2d pixel coords. Text will center in rect if it is not Vec2::INFINITY.
    pub rect: Vec2,
    pub font_size: f32,
    pub color: Color,
    pub background: Color,
    pub alignment: TextAlignment,
    pub anchor: Anchor,
    pub rect_anchor: Anchor,
    /// A button must also have a non Vec2::INFINITY rect.
    pub button: bool,
    /// If life is 0.0, it will only live one frame (default), if life is f32::INFINITY it will live forever.
    pub life: f32,
    pub id: Option<u64>,
    pub spatial_id: Option<u64>,
}

impl Default for PicoItem {
    fn default() -> Self {
        PicoItem {
            position: Vec3::ZERO,
            position_3d: false,
            rect: Vec2::INFINITY,
            text: String::new(),
            font_size: 0.02,
            color: Color::WHITE,
            background: Color::NONE,
            alignment: TextAlignment::Center,
            anchor: Anchor::Center,
            rect_anchor: Anchor::Center,
            button: false,
            life: 0.0,
            id: None,
            spatial_id: None,
        }
    }
}

impl PicoItem {
    pub fn new2d(position: Vec3, text: &str) -> PicoItem {
        PicoItem {
            position,
            text: text.to_string(),
            ..default()
        }
    }
    pub fn new3d(position: Vec3, text: &str) -> PicoItem {
        PicoItem {
            position,
            text: text.to_string(),
            position_3d: true,
            ..default()
        }
    }
    pub fn keep(mut self) -> Self {
        self.life = f32::INFINITY;
        self
    }
    fn generate_spatial_id(&self) -> u64 {
        let hasher = &mut DefaultHasher::new();
        self.position.x.to_bits().hash(hasher);
        self.position.y.to_bits().hash(hasher);
        self.position.z.to_bits().hash(hasher);
        self.rect.x.to_bits().hash(hasher);
        self.rect.y.to_bits().hash(hasher);
        format!("{:?}", self.rect_anchor).hash(hasher);
        hasher.finish()
    }
    fn generate_id(&mut self) -> u64 {
        self.id = None;
        let hasher = &mut DefaultHasher::new();
        format!("{:?}", self).hash(hasher);
        hasher.finish()
    }
}

impl std::hash::Hash for PicoItem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.spatial_id.unwrap().hash(state)
    }
}

impl PartialEq for PicoItem {
    fn eq(&self, other: &PicoItem) -> bool {
        self.spatial_id.unwrap() == other.spatial_id.unwrap()
    }
}

#[derive(Resource, Default)]
pub struct Pico {
    pub cache: HashMap<u64, CacheItem>,
    pub items: Vec<PicoItem>,
    pub interacting: bool,
}

impl Pico {
    pub fn get_hovered(&self, index: usize) -> Option<&CacheItem> {
        let item = self.get(index);
        if let Some(cache_item) = self.cache.get(&item.spatial_id.unwrap()) {
            if cache_item.hover {
                return Some(cache_item);
            }
        }

        None
    }
    pub fn clicked(&self, index: usize) -> bool {
        if let Some(cache_item) = self.get_hovered(index) {
            if let Some(input) = &cache_item.input {
                return input.just_pressed(MouseButton::Left);
            }
        }
        false
    }
    pub fn released(&self, index: usize) -> bool {
        if let Some(cache_item) = self.get_hovered(index) {
            if let Some(input) = &cache_item.input {
                return input.just_released(MouseButton::Left);
            }
        }
        false
    }
    pub fn hovered(&self, index: usize) -> bool {
        self.get_hovered(index).is_some()
    }
    pub fn dragged(&self, index: usize) -> Option<Drag> {
        if let Some(cache_item) = self.cache.get(&self.get(index).spatial_id.unwrap()) {
            return cache_item.drag;
        }
        None
    }
    pub fn add(&mut self, mut item: PicoItem) -> &mut Self {
        if item.spatial_id.is_none() {
            item.spatial_id = Some(item.generate_spatial_id());
        }
        self.items.push(item);
        self
    }
    pub fn get_mut(&mut self, index: usize) -> &mut PicoItem {
        if index >= self.items.len() {
            panic!(
                "Tried to access item {} but there are only {}",
                index,
                self.items.len()
            );
        }
        &mut self.items[index]
    }
    pub fn get(&self, index: usize) -> &PicoItem {
        if index >= self.items.len() {
            panic!(
                "Tried to access item {} but there are only {}",
                index,
                self.items.len()
            );
        }
        &self.items[index]
    }
    pub fn last(&self) -> usize {
        (self.items.len() - 1).max(0)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Drag {
    pub start: Vec2,
    pub end: Vec2,
    pub last_frame: Vec2,
}

impl Drag {
    pub fn delta(&self) -> Vec2 {
        self.end - self.last_frame
    }
    pub fn total_delta(&self) -> Vec2 {
        self.end - self.start
    }
}

#[derive(Debug, Default)]
pub struct CacheItem {
    pub entity: Option<Entity>,
    pub life: f32,
    pub hover: bool,
    pub interactable: bool,
    pub drag: Option<Drag>,
    pub id: u64,
    pub input: Option<Input<MouseButton>>,
}

#[derive(Component)]
pub struct ImEntity;

#[allow(clippy::too_many_arguments)]
fn render(
    mut commands: Commands,
    time: Res<Time>,
    item_entities: Query<(Entity, &PicoItem)>,
    camera: Query<(&Camera, &GlobalTransform), With<ImTextCamera>>,
    windows: Query<&Window>,
    mut state: ResMut<Pico>,
    mut text_entites: Query<(&mut Transform, Option<&Sprite>), With<ImEntity>>,
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
    // Age all the cached items
    for (_, cache_item) in state.cache.iter_mut() {
        cache_item.life -= time.delta_seconds();
        cache_item.hover = false;
        cache_item.input = None;
        if mouse_button_input.pressed(MouseButton::Left) {
            if cache_item.drag.is_some() {
                *currently_dragging = true;
                interacting = true;
            }
        } else {
            cache_item.drag = None;
        }
    }

    let mut items = std::mem::take(&mut state.items);

    // Move PicoItem entites to local set
    for (imtext_entity, item) in &item_entities {
        items.push(item.clone());
        commands.entity(imtext_entity).despawn();
    }

    // Sort so we interact in z order.
    items.sort_by(|a, b| a.position.z.partial_cmp(&b.position.z).unwrap());

    let mut first_interact_found = false;

    for item in &mut items {
        if item.id.is_none() {
            item.id = Some(item.generate_id());
        }
        if item.spatial_id.is_none() {
            item.spatial_id = Some(item.generate_spatial_id());
        }
        let spatial_id = item.spatial_id.unwrap();

        let text_ndc = if item.position_3d {
            camera
                .world_to_ndc(camera_transform, item.position)
                .unwrap_or(Vec3::NAN)
        } else {
            ((item.position.xy() - Vec2::splat(0.5)) * vec2(2.0, -2.0)).extend(item.position.z)
        };

        let text_pos = text_ndc.xy() * window_size * 0.5;

        let generate = if let Some(existing_cache_item) = state.cache.get_mut(&spatial_id) {
            // If a ImText in the cache matches one created this frame keep it around
            existing_cache_item.life = existing_cache_item.life.max(0.0);
            let Ok((mut trans, sprite)) = text_entites.get_mut(existing_cache_item.entity.unwrap())
            else {
                continue;
            };
            trans.translation = text_pos.extend(text_ndc.z);

            if !existing_cache_item.interactable {
                continue;
            }

            if let Some(sprite) = sprite {
                if let Some(custom_size) = sprite.custom_size {
                    if let Some(cursor_pos) = window.cursor_position() {
                        if mouse_button_input.pressed(MouseButton::Left) && !first_interact_found {
                            if let Some(drag) = &mut existing_cache_item.drag {
                                drag.last_frame = drag.end;
                                drag.end = cursor_pos;
                            }
                        }
                        let half_size = custom_size * 0.5;
                        let xy = window_size * 0.5 + trans.translation.xy() * vec2(1.0, -1.0);
                        let a = xy - half_size + custom_size * -sprite.anchor.as_vec();
                        let b = xy + half_size + custom_size * -sprite.anchor.as_vec();
                        if cursor_pos.cmpge(a).all() && cursor_pos.cmple(b).all() {
                            existing_cache_item.hover = true;
                            if !first_interact_found {
                                existing_cache_item.input = Some(mouse_button_input.clone());
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
                                    && existing_cache_item.drag.is_none()
                                {
                                    existing_cache_item.drag = Some(Drag {
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
            existing_cache_item.id != item.id.unwrap()
        } else {
            true
        };
        if generate {
            let cache_item = if let Some(old_cache_item) = state.cache.get_mut(&spatial_id) {
                let entity = old_cache_item.entity.unwrap();
                if text_entites.get(entity).is_ok() {
                    commands.entity(entity).despawn_recursive();
                }
                old_cache_item
            } else {
                state.cache.insert(spatial_id, CacheItem::default());
                state.cache.get_mut(&spatial_id).unwrap()
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
            cache_item.life = item.life;
            cache_item.id = item.id.unwrap();
            if item.rect.x.is_finite() && item.rect.y.is_finite() {
                let rect = item.rect * window_size;
                let sprite = Sprite {
                    color: item.background,
                    custom_size: Some(rect),
                    anchor: item.rect_anchor.clone(),
                    ..default()
                };
                let trans = Transform::from_translation(text_pos.extend(1.0));
                let entity = commands
                    .spawn((
                        SpriteBundle {
                            sprite: sprite.clone(),
                            transform: trans,
                            ..default()
                        },
                        ImEntity,
                    ))
                    .with_children(|builder| {
                        builder.spawn(Text2dBundle {
                            text,
                            text_anchor: item.anchor.clone(),
                            transform: Transform::from_translation(
                                (rect * -(item.rect_anchor.as_vec() - item.anchor.as_vec()))
                                    .extend(0.001),
                            ),
                            text_2d_bounds: Text2dBounds { size: rect },
                            ..default()
                        });
                    })
                    .id();
                cache_item.interactable = true;
                cache_item.entity = Some(entity);
            } else {
                let entity = commands
                    .spawn(Text2dBundle {
                        text,
                        text_anchor: item.anchor.clone(),
                        transform: Transform::from_translation(text_pos.extend(1.0)),
                        ..default()
                    })
                    .id();
                cache_item.entity = Some(entity);
            }
        }
    }

    for (_, cache_item) in state.cache.iter_mut() {
        let entity = cache_item.entity.unwrap();
        // Remove cached ImTexts that are no longer in use
        if cache_item.life < 0.0 && text_entites.get(entity).is_ok() {
            commands.entity(entity).despawn_recursive();
        }
    }

    // clean up cache
    state.cache.retain(|_, cache_item| cache_item.life >= 0.0);
    state.interacting = interacting;
}

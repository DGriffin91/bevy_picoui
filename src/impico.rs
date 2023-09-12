use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    math::{vec2, Vec3Swizzles},
    prelude::*,
    sprite::{calculate_bounds_2d, Anchor},
    text::{BreakLineOn, Text2dBounds},
};
use core::hash::Hash;
use core::hash::Hasher;
use std::{collections::hash_map::DefaultHasher, mem};

use bevy::utils::HashMap;

pub struct ImPicoPlugin {
    // Set if using in a scene with no 2d camera
    pub create_default_cam_with_order: Option<isize>,
}

#[derive(Resource)]
pub struct CreateDefaultCamWithOrder(isize);

impl Plugin for ImPicoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Pico>()
            .add_systems(PostUpdate, render_imtext.after(calculate_bounds_2d));
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

// Only supports one camera.
#[derive(Component)]
pub struct ImTextCamera;

#[derive(Component, Clone, Debug)]
pub struct ImItem {
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
    /// A button must also have a non Vec2::INFINITY rect.
    pub button: bool,
    /// If life is 0.0, it will only live one frame (default), if life is f32::INFINITY it will live forever.
    pub life: f32,
    pub id: Option<u64>,
    pub spatial_id: Option<u64>,
}

impl Default for ImItem {
    fn default() -> Self {
        ImItem {
            position: Vec3::ZERO,
            position_3d: false,
            rect: Vec2::INFINITY,
            text: String::new(),
            font_size: 14.0,
            color: Color::WHITE,
            background: Color::NONE,
            alignment: TextAlignment::Center,
            anchor: Anchor::Center,
            button: false,
            life: 0.0,
            id: None,
            spatial_id: None,
        }
    }
}

impl ImItem {
    pub fn new2d(position: Vec3, text: &str) -> ImItem {
        ImItem {
            position,
            text: text.to_string(),
            ..default()
        }
    }
    pub fn new3d(position: Vec3, text: &str) -> ImItem {
        ImItem {
            position,
            text: text.to_string(),
            position_3d: true,
            ..default()
        }
    }
    pub fn button2d(position: Vec3, rect: Vec2, text: &str) -> ImItem {
        ImItem {
            position,
            text: text.to_string(),
            button: true,
            rect,
            ..default()
        }
    }
    pub fn button3d(position: Vec3, rect: Vec2, text: &str) -> ImItem {
        ImItem {
            position,
            text: text.to_string(),
            button: true,
            position_3d: true,
            rect,
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
        format!("{:?}", self.anchor).hash(hasher);
        hasher.finish()
    }

    fn generate_id(&mut self) -> u64 {
        self.id = None;
        let hasher = &mut DefaultHasher::new();
        format!("{:?}", self).hash(hasher);
        hasher.finish()
    }
}

impl std::hash::Hash for ImItem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.unwrap().hash(state)
    }
}

impl PartialEq for ImItem {
    fn eq(&self, other: &ImItem) -> bool {
        self.id.unwrap() == other.id.unwrap()
    }
}

#[derive(Resource, Default)]
pub struct Pico {
    pub cache: HashMap<u64, CacheItem>,
    pub items: Vec<ImItem>,
}

impl Pico {
    pub fn get_hovered(&self) -> Option<&CacheItem> {
        if let Some(item) = self.items.last() {
            if !item.button {
                return None;
            }
            let posid = item.generate_spatial_id();
            for (_, cache_item) in &self.cache {
                if cache_item.hover && cache_item.posid == posid {
                    return Some(cache_item);
                }
            }
        }
        None
    }
    pub fn clicked(&self) -> bool {
        if let Some(cache_item) = self.get_hovered() {
            if let Some(input) = &cache_item.input {
                return input.just_pressed(MouseButton::Left);
            }
        }
        false
    }
    pub fn hovered(&self) -> bool {
        self.get_hovered().is_some()
    }
    pub fn add(&mut self, item: ImItem) -> &mut Self {
        self.items.push(item);
        self
    }
    pub fn last(&self) -> Option<&ImItem> {
        self.items.last()
    }
}

#[derive(Debug)]
pub struct CacheItem {
    pub entity: Entity,
    pub life: f32,
    pub hover: bool,
    pub button: bool,
    pub posid: u64,
    pub input: Option<Input<MouseButton>>,
}

#[derive(Component)]
struct ImEntity;

fn render_imtext(
    mut commands: Commands,
    time: Res<Time>,
    item_entities: Query<(Entity, &ImItem)>,
    camera: Query<(&Camera, &GlobalTransform), With<ImTextCamera>>,
    windows: Query<&Window>,
    mut state: ResMut<Pico>,
    mut text_entites: Query<(&mut Transform, Option<&Sprite>), With<ImEntity>>,
    mouse_button_input: Res<Input<MouseButton>>,
) {
    let Ok((camera, camera_transform)) = camera.get_single() else {
        return;
    };
    let Ok(window) = windows.get_single() else {
        return;
    };
    let window_size = Vec2::new(window.width(), window.height());

    // Age all the cached items
    for (_, cache_item) in state.cache.iter_mut() {
        cache_item.life -= time.delta_seconds();
        cache_item.hover = false;
        cache_item.input = None;
    }

    let mut items = mem::replace(&mut state.items, Vec::new());

    // Move ImItem entites to local set
    for (imtext_entity, item) in &item_entities {
        items.push(item.clone());
        commands.entity(imtext_entity).despawn();
    }

    for impico in &mut items {
        if impico.id.is_none() {
            impico.id = Some(impico.generate_id());
        }
        if impico.spatial_id.is_none() {
            impico.spatial_id = Some(impico.generate_spatial_id());
        }

        let text_ndc = if impico.position_3d {
            camera
                .world_to_ndc(camera_transform, impico.position)
                .unwrap_or(Vec3::NAN)
        } else {
            ((impico.position.xy() - Vec2::splat(0.5)) * vec2(2.0, -2.0)).extend(impico.position.z)
        };

        let text_pos = text_ndc.xy() * window_size * 0.5;
        if let Some(cache_item) = state.cache.get_mut(&impico.id.unwrap()) {
            // If a ImText in the cache matches one created this frame keep it around
            cache_item.life = 0.0;
            let Ok((mut trans, sprite)) = text_entites.get_mut(cache_item.entity) else {
                continue;
            };
            trans.translation = text_pos.extend(text_ndc.z);
            if !cache_item.button {
                continue;
            }
            set_input(
                sprite,
                window,
                window_size,
                &trans,
                cache_item,
                &mouse_button_input,
            );
        } else {
            let text = Text {
                sections: vec![TextSection::new(
                    impico.text.clone(),
                    TextStyle {
                        font_size: impico.font_size,
                        color: impico.color,
                        ..default()
                    },
                )],
                alignment: impico.alignment,
                linebreak_behavior: BreakLineOn::WordBoundary,
            };
            let cache_item = if impico.rect.x.is_finite() && impico.rect.y.is_finite() {
                let sprite = Sprite {
                    color: impico.background,
                    custom_size: Some(impico.rect),
                    anchor: impico.anchor.clone(),
                    ..default()
                };
                let trans = Transform::from_translation(text_pos.extend(1.0));
                let mut cache_item = CacheItem {
                    entity: commands
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
                                text_anchor: Anchor::Center,
                                transform: Transform::from_translation(
                                    (-impico.rect * impico.anchor.as_vec()).extend(0.001),
                                ),
                                text_2d_bounds: Text2dBounds { size: impico.rect },
                                ..default()
                            });
                        })
                        .id(),
                    life: impico.life,
                    hover: false,
                    button: impico.button,
                    posid: impico.spatial_id.unwrap(),
                    input: None,
                };
                set_input(
                    Some(&sprite),
                    window,
                    window_size,
                    &trans,
                    &mut cache_item,
                    &mouse_button_input,
                );
                cache_item
            } else {
                CacheItem {
                    entity: commands
                        .spawn(Text2dBundle {
                            text,
                            text_anchor: impico.anchor.clone(),
                            transform: Transform::from_translation(text_pos.extend(1.0)),
                            text_2d_bounds: Text2dBounds { size: impico.rect },
                            ..default()
                        })
                        .id(),
                    life: impico.life,
                    hover: false,
                    button: impico.button,
                    posid: impico.spatial_id.unwrap(),
                    input: None,
                }
            };
            state.cache.insert(impico.id.unwrap(), cache_item);
        }
    }

    for (_, cache_item) in state.cache.iter_mut() {
        // Remove cached ImTexts that are no longer in use
        if cache_item.life < 0.0 && text_entites.get(cache_item.entity).is_ok() {
            commands.entity(cache_item.entity).despawn_recursive();
        }
    }

    // clean up cache
    state.cache.retain(|_, cache_item| cache_item.life >= 0.0);
}

fn set_input(
    sprite: Option<&Sprite>,
    window: &Window,
    window_size: Vec2,
    trans: &Transform,
    cache_item: &mut CacheItem,
    mouse_button_input: &Input<MouseButton>,
) {
    if let Some(sprite) = sprite {
        if let Some(custom_size) = sprite.custom_size {
            if let Some(cursor_pos) = window.cursor_position() {
                let half_size = custom_size * 0.5;
                let xy = window_size * 0.5 + trans.translation.xy() * vec2(1.0, -1.0);
                let a = xy - half_size + half_size * sprite.anchor.as_vec();
                let b = xy + half_size + half_size * sprite.anchor.as_vec();
                if cursor_pos.cmpge(a).all() && cursor_pos.cmple(b).all() {
                    cache_item.hover = true;
                    cache_item.input = Some(mouse_button_input.clone());
                }
            }
        }
    }
}

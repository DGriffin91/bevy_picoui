use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    input::InputSystem,
    math::{vec2, vec3, vec4, Vec3Swizzles, Vec4Swizzles},
    prelude::*,
    sprite::Anchor,
    text::{BreakLineOn, Text2dBounds},
};
use core::hash::Hash;
use core::hash::Hasher;
use std::{
    collections::hash_map::DefaultHasher,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
};

use bevy::utils::HashMap;

pub struct PicoPlugin {
    // Set if using in a scene with no 2d camera
    pub create_default_2d_cam_with_order: Option<isize>,
}

#[derive(Resource)]
pub struct CreateDefaultCamWithOrder(isize);

impl Plugin for PicoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Pico>()
            .add_systems(PreUpdate, render.after(InputSystem));
        if let Some(n) = self.create_default_2d_cam_with_order {
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

pub fn button(pico: &mut Pico, item: PicoItem) -> usize {
    let index = pico.items.len();
    let pico = pico.add(item);
    let c = pico.get(index).background;
    pico.get_mut(index).background = if pico.hovered(index) {
        c + Vec4::splat(0.06)
    } else {
        c
    };
    index
}

// -------------------------
// Toggle Button example widget
// -------------------------

pub fn toggle_button(
    pico: &mut Pico,
    item: PicoItem,
    enabled_bg: Color,
    toggle_state: &mut bool,
) -> usize {
    let index = pico.items.len();
    let pico = pico.add(item);
    let mut c = pico.get(index).background;
    if pico.clicked(index) {
        *toggle_state = !*toggle_state;
    }
    if *toggle_state {
        c = enabled_bg;
    }
    pico.get_mut(index).background = if pico.hovered(index) {
        c + Vec4::splat(0.06)
    } else {
        c
    };
    index
}

// -------------------------
// Horizontal ruler example widget
// -------------------------

/// Width is relative to parent. Height-y parent is removed so height is only relative to screen height
/// so HRs are of a consistent height for the same input height.
pub fn hr(pico: &mut Pico, rect: Vec2, parent: Option<usize>) -> usize {
    let index = pico.last();
    pico.add(PicoItem {
        position: vec3(0.5, 0.0, 0.0),
        rect,
        background: Color::rgba(1.0, 1.0, 1.0, 0.04),
        rect_anchor: Anchor::TopCenter,
        parent,
        ..default()
    });
    index
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
    drag_bg: Color,
    position: Vec3,
    height: f32,
    label_width: f32,
    drag_width: f32,
    label: &str,
    scale: f32,
    value: f32,
    parent: Option<usize>,
    char_input_events: Option<&mut EventReader<ReceivedCharacter>>,
) -> DragValue {
    let mut value = value;
    let mut drag_bg = drag_bg;
    let vstack_end = pico
        .stack_stack
        .last_mut()
        .and_then(|stack| Some(stack.end))
        .unwrap_or(0.0);
    let text_index = pico.items.len();
    let pico = pico.add(PicoItem {
        text: label.to_string(),
        position,
        rect: vec2(label_width, height),
        anchor: Anchor::CenterLeft,
        rect_anchor: Anchor::TopLeft,
        parent,
        ..default()
    });

    let mut drag_item = PicoItem {
        // TODO user or auto precision
        text: format!("{:.2}", value),
        position: position + Vec3::X * label_width,
        rect: vec2(drag_width, height),
        parent,
        ..default()
    };

    drag_item.rect_anchor = Anchor::TopLeft;
    let drag_index = pico.items.len();
    // If were in a vstack, roll it back so we are on the same row
    if let Some(stack) = pico.stack_stack.last_mut() {
        if stack.vertical {
            stack.end = vstack_end;
        }
    }
    let pico = pico.add(drag_item);
    let mut dragging = false;
    if let Some(state) = pico.get_state(drag_index) {
        if let Some(drag) = state.drag {
            value = drag.delta().x * scale + value;
            dragging = true;
        }
    };
    if let Some(char_input_events) = char_input_events {
        let mouse_just_pressed = if let Some(mouse_button_input) = &pico.mouse_button_input {
            mouse_button_input.any_just_pressed([
                MouseButton::Left,
                MouseButton::Right,
                MouseButton::Middle,
            ])
        } else {
            false
        };
        let mut current_string = None;
        let released = pico.released(drag_index);
        let mut reset = false;
        let mut apply = false;
        let mut selected = false;
        if let Some(state) = pico.get_state_mut(drag_index) {
            let mut just_selected = false;
            if state.storage.is_none() {
                state.storage = Some(Box::new(String::new()));
            }
            if !dragging && released {
                state.selected = true;
                just_selected = true;
            }
            if state.selected {
                selected = true;
                let backspace = char::from_u32(0x08).unwrap();
                let esc = char::from_u32(0x1b).unwrap();
                let enter = '\r';
                if let Some(storage) = &mut state.storage {
                    let s = storage.downcast_mut::<String>().unwrap();
                    if mouse_just_pressed && !just_selected {
                        apply = true;
                    } else {
                        // TODO: usually when a text field like this is first selected it would have all of the
                        // text in the field selected, so typing anything would overwrite the existing value
                        // or the cursor could be moved to preserve the value.
                        //if just_selected {
                        //    // TODO user or auto precision
                        //    *s = format!("{:.2}", value);
                        //}
                        for e in char_input_events.iter() {
                            if e.char == esc {
                                reset = true;
                            } else if e.char == backspace {
                                s.pop();
                            } else if e.char == enter {
                                apply = true;
                                break;
                            } else if e.char.is_digit(10) || e.char == '.' || e.char == '-' {
                                s.push(e.char);
                            }
                        }
                        current_string = Some(s.clone());
                    }
                    if apply {
                        if let Ok(parse_val) = s.parse::<f32>() {
                            value = parse_val;
                        }
                        reset = true;
                    }
                    if reset {
                        state.selected = false;
                        *s = String::new();
                    }
                }
            }
        }
        if let Some(current_string) = current_string {
            pico.get_mut(drag_index).text = current_string + "|";
        }
        if selected {
            drag_bg += Vec4::splat(0.25);
        }
    }
    pico.get_mut(drag_index).background = if pico.hovered(drag_index) {
        drag_bg + Vec4::splat(0.06)
    } else {
        drag_bg
    };
    DragValue {
        value,
        text_index,
        drag_index,
    }
}

// Only supports one camera.
#[derive(Component)]
pub struct Pico2dCamera;

#[derive(Component, Clone, Debug)]
pub struct PicoItem {
    pub text: String,
    /// If position_3d is false position is the screen uv coords with 0.0, 0.0 at top left
    /// If position_3d is true position is the world space translation
    /// Don't change after pico.add()
    pub position: Vec3,
    pub position_3d: bool,
    /// 2d pixel coords. Text will center in rect if it is not Vec2::INFINITY.
    /// Don't change after pico.add()
    pub rect: Vec2,
    pub font_size: f32,
    pub color: Color,
    pub background: Color,
    pub alignment: TextAlignment,
    pub anchor: Anchor,
    pub rect_anchor: Anchor,
    pub parent_anchor: Anchor,
    /// A button must also have a non Vec2::INFINITY rect.
    pub button: bool,
    /// If life is 0.0, it will only live one frame (default), if life is f32::INFINITY it will live forever.
    pub life: f32,
    /// If the id changes, the item is re-rendered
    pub id: Option<u64>,
    /// If the spatial_id changes a new state is used
    /// Impacted by position, rect, rect_anchor (after transform from parent is applied, if any)
    pub spatial_id: Option<u64>,
    /// If set, coordinates for position/rect will be relative to parent.
    pub parent: Option<usize>,
    // Coordinates are uv space 0..1 over the whole window
    pub computed_bbox: Option<Vec4>,
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
            parent_anchor: Anchor::TopLeft,
            button: false,
            life: 0.0,
            id: None,
            spatial_id: None,
            parent: None,
            computed_bbox: None,
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

pub fn lerp2(start: Vec2, end: Vec2, t: Vec2) -> Vec2 {
    (1.0 - t) * start + t * end
}

pub fn lerp(start: f32, end: f32, t: f32) -> f32 {
    (1.0 - t) * start + t * end
}

#[derive(Default, Clone)]
pub struct Guard(Arc<AtomicI32>);

impl Guard {
    pub fn push(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
    pub fn pop(&self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
        self.0.fetch_max(0, Ordering::Relaxed);
    }
    pub fn get(&self) -> i32 {
        self.0.load(Ordering::Relaxed)
    }
}
impl Drop for Guard {
    fn drop(&mut self) {
        self.pop()
    }
}

pub struct Stack {
    pub end: f32,
    pub margin: f32,
    pub vertical: bool,
}

#[derive(Resource, Default)]
pub struct Pico {
    pub state: HashMap<u64, StateItem>,
    pub items: Vec<PicoItem>,
    pub interacting: bool,
    pub stack_stack: Vec<Stack>,
    pub stack_guard: Guard,
    pub window_size: Vec2,
    /// The ratio of window height to window width.
    /// For keeping items horizontally proportional.
    /// 2d x coords are mapped so that when x is 1 it is the same distance in pixels as when y is 1
    /// (For keeping things from stretching horizontally when scaling the window)
    pub vh: f32,
    /// The right edge of the window after scaling by vh
    pub vh_right: f32,
    pub mouse_button_input: Option<Input<MouseButton>>,
}

impl Pico {
    pub fn vstack(&mut self, start: f32, margin: f32) -> Guard {
        self.stack_stack.push(Stack {
            end: start,
            margin,
            vertical: true,
        });
        self.stack_guard.push();
        self.stack_guard.clone()
    }
    pub fn hstack(&mut self, start: f32, margin: f32) -> Guard {
        self.stack_stack.push(Stack {
            end: start,
            margin,
            vertical: false,
        });
        self.stack_guard.push();
        self.stack_guard.clone()
    }
    pub fn get_hovered(&self, index: usize) -> Option<&StateItem> {
        if let Some(state_item) = self.get_state(index) {
            if state_item.hover {
                return Some(state_item);
            }
        }
        None
    }
    pub fn clicked(&self, index: usize) -> bool {
        if let Some(state_item) = self.get_hovered(index) {
            if let Some(input) = &state_item.input {
                return input.just_pressed(MouseButton::Left);
            }
        }
        false
    }
    pub fn released(&self, index: usize) -> bool {
        if let Some(state_item) = self.get_hovered(index) {
            if let Some(input) = &state_item.input {
                return input.just_released(MouseButton::Left);
            }
        }
        false
    }
    pub fn bbox(&self, index: usize) -> Vec4 {
        let item = self.get(index);
        if let Some(computed_bbox) = item.computed_bbox {
            return computed_bbox;
        }
        vec4(0.0, 0.0, 1.0, 1.0)
    }
    /// Take a uv for relative to the window (0.0, 0.0 at top left, 1.0, 1.0 bottom right)
    /// And get the corresponding uv inside the given parent
    pub fn window_uv_for_parent(&self, window_uv: Vec2, parent: [usize; 4]) -> Vec2 {
        let bbox = self.bbox(parent[0]);
        (window_uv - bbox.xy()) / (bbox.zw() - bbox.xy()).abs()
    }
    pub fn hovered(&self, index: usize) -> bool {
        self.get_hovered(index).is_some()
    }
    pub fn add(&mut self, mut item: PicoItem) -> &mut Self {
        if !item.position_3d {
            while (self.stack_guard.get() as usize) < self.stack_stack.len() {
                self.stack_stack.pop();
            }
            if !self.stack_stack.is_empty() {
                let stack = self.stack_stack.last_mut().unwrap();
                if stack.vertical {
                    item.position.y += stack.end + stack.margin;
                    stack.end = stack
                        .end
                        .max(get_bbox(item.rect, item.position.xy(), &item.rect_anchor).w);
                } else {
                    item.position.x += stack.end + stack.margin;
                    stack.end = stack
                        .end
                        .max(get_bbox(item.rect, item.position.xy(), &item.rect_anchor).z);
                }
            }
            {
                let parent_2d_bbox = if let Some(parent) = item.parent {
                    let parent_z = self.get(parent).position.z;
                    item.position.z += parent_z;
                    if item.position.z == parent_z {
                        // Make sure child is in front of parent if they were at the same z
                        item.position.z += 0.000001;
                    }
                    self.bbox(parent)
                } else {
                    vec4(0.0, 0.0, 1.0, 1.0)
                };

                let pa_vec = item.parent_anchor.as_vec() * vec2(1.0, -1.0);
                item.position *= (-pa_vec * 2.0).extend(1.0);
                item.position += (pa_vec + vec2(0.5, 0.5)).extend(0.0);
                item.position = lerp2(parent_2d_bbox.xy(), parent_2d_bbox.zw(), item.position.xy())
                    .extend(item.position.z);
                item.rect *= (parent_2d_bbox.zw() - parent_2d_bbox.xy()).abs();
            }
        }
        if item.spatial_id.is_none() {
            item.spatial_id = Some(item.generate_spatial_id());
        }
        item.computed_bbox = Some(if item.position_3d {
            if let Some(state_item) = self.state.get(&item.spatial_id.unwrap()) {
                state_item.bbox
            } else {
                Vec4::ZERO
            }
        } else {
            get_bbox(item.rect, item.position.xy(), &item.rect_anchor)
        });
        self.items.push(item);
        self
    }
    pub fn get_state_mut(&mut self, index: usize) -> Option<&mut StateItem> {
        self.state.get_mut(&self.get(index).spatial_id.unwrap())
    }
    pub fn get_state(&self, index: usize) -> Option<&StateItem> {
        self.state.get(&self.get(index).spatial_id.unwrap())
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
    pub fn storage(&mut self) -> Option<&mut Option<Box<dyn std::any::Any + Send + Sync>>> {
        if let Some(item) = self.items.last() {
            if let Some(state_item) = self.state.get_mut(&item.spatial_id.unwrap()) {
                return Some(&mut state_item.storage);
            }
        }
        None
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
pub struct StateItem {
    pub entity: Option<Entity>,
    pub life: f32,
    pub hover: bool,
    pub interactable: bool,
    pub selected: bool,
    pub drag: Option<Drag>,
    pub id: u64,
    pub input: Option<Input<MouseButton>>,
    // Coordinates are uv space 0..1 over the whole window
    pub bbox: Vec4,
    pub storage: Option<Box<dyn std::any::Any + Send + Sync>>,
}

#[derive(Component)]
pub struct PicoEntity(u64);

#[allow(clippy::too_many_arguments)]
fn render(
    mut commands: Commands,
    time: Res<Time>,
    item_entities: Query<(Entity, &PicoItem)>,
    camera: Query<(&Camera, &GlobalTransform), With<Pico2dCamera>>,
    windows: Query<&Window>,
    mut pico: ResMut<Pico>,
    mut pico_entites: Query<(Entity, &mut Transform, Option<&Sprite>, &PicoEntity)>,
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

    // Move PicoItem entites to local set
    for (imtext_entity, item) in &item_entities {
        items.push(item.clone());
        commands.entity(imtext_entity).despawn();
    }

    // Sort so we interact in z order.
    items.sort_by(|a, b| b.position.z.partial_cmp(&a.position.z).unwrap());

    let mut first_interact_found = false;

    for item in &mut items {
        if item.id.is_none() {
            item.id = Some(item.generate_id());
        }
        if item.spatial_id.is_none() {
            item.spatial_id = Some(item.generate_spatial_id());
        }
        let spatial_id = item.spatial_id.unwrap();

        let item_ndc = if item.position_3d {
            camera
                .world_to_ndc(camera_transform, item.position)
                .unwrap_or(Vec3::NAN)
        } else {
            ((item.position.xy() - Vec2::splat(0.5)) * vec2(2.0, -2.0)).extend(item.position.z)
        };

        let item_pos = item_ndc.xy() * window_size * 0.5;

        let generate = if let Some(existing_state_item) = pico.state.get_mut(&spatial_id) {
            // If a item in the state matches one created this frame keep it around
            existing_state_item.life = existing_state_item.life.max(0.0);
            let Ok((_, mut trans, sprite, _)) =
                pico_entites.get_mut(existing_state_item.entity.unwrap())
            else {
                continue;
            };
            trans.translation = item_pos.extend(item_ndc.z);

            if !existing_state_item.interactable {
                continue;
            }

            if let Some(sprite) = sprite {
                if let Some(custom_size) = sprite.custom_size {
                    if let Some(cursor_pos) = window.cursor_position() {
                        if mouse_button_input.pressed(MouseButton::Left) && !first_interact_found {
                            if let Some(drag) = &mut existing_state_item.drag {
                                drag.last_frame = drag.end;
                                drag.end = cursor_pos;
                            }
                        }
                        existing_state_item.bbox = get_bbox(
                            custom_size / window_size,
                            trans.translation.xy() / window_size * vec2(1.0, -1.0) + 0.5,
                            &sprite.anchor,
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
            existing_state_item.id != item.id.unwrap()
        } else {
            true
        };
        if generate || pico.window_size != window_size {
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
            if item.rect.x.is_finite() && item.rect.y.is_finite() {
                let rect = item.rect * window_size;
                let sprite = Sprite {
                    color: item.background,
                    custom_size: Some(rect),
                    anchor: item.rect_anchor.clone(),
                    ..default()
                };
                let trans = Transform::from_translation(item_pos.extend(1.0));
                let entity = commands
                    .spawn((
                        SpriteBundle {
                            sprite: sprite.clone(),
                            transform: trans,
                            ..default()
                        },
                        PicoEntity(spatial_id),
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
                state_item.bbox = get_bbox(
                    item.rect,
                    trans.translation.xy() / window_size * vec2(1.0, -1.0) + 0.5,
                    &sprite.anchor,
                );
                state_item.interactable = true;
                state_item.entity = Some(entity);
            } else {
                let entity = commands
                    .spawn(Text2dBundle {
                        text,
                        text_anchor: item.anchor.clone(),
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

    for (entity, _, _, pico_entity) in &pico_entites {
        // Remove any orphaned
        if pico.state.get(&pico_entity.0).is_none() {
            commands.entity(entity).despawn_recursive();
        }
    }

    // clean up state
    pico.state.retain(|_, state_item| state_item.life >= 0.0);
    pico.interacting = interacting;
    pico.window_size = window_size;
    pico.vh = window_size.y / window_size.x;
    pico.vh_right = window_size.x / window_size.y;
    pico.mouse_button_input = Some(mouse_button_input.clone());
}

fn get_bbox(rect: Vec2, uv_position: Vec2, anchor: &Anchor) -> Vec4 {
    let half_size = rect * 0.5;
    let a = uv_position - half_size + rect * -anchor.as_vec() * vec2(1.0, -1.0);
    let b = uv_position + half_size + rect * -anchor.as_vec() * vec2(1.0, -1.0);
    vec4(a.x, a.y, b.x, b.y)
}

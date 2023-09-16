use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    input::InputSystem,
    math::{vec2, vec3, vec4, Vec3Swizzles, Vec4Swizzles},
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::{Anchor, MaterialMesh2dBundle, Mesh2dHandle},
    text::{BreakLineOn, Text2dBounds},
};
use core::hash::Hash;
use core::hash::Hasher;
use std::{
    collections::hash_map::DefaultHasher,
    f32::consts::{FRAC_PI_2, PI},
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

#[derive(Resource)]
pub struct MeshHandles {
    circle: Mesh2dHandle,
    rect: Mesh2dHandle,
}

impl Plugin for PicoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Pico>()
            .add_systems(PreUpdate, (render.after(InputSystem), apply_deferred));
        if let Some(n) = self.create_default_2d_cam_with_order {
            app.insert_resource(CreateDefaultCamWithOrder(n))
                .add_systems(Startup, setup);
        }
    }
}

fn setup(
    mut commands: Commands,
    order: Res<CreateDefaultCamWithOrder>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let arc_mesh = arc_mesh(8, 1.0, 0.0, FRAC_PI_2);
    let circle: Mesh2dHandle = meshes.add(arc_mesh).into();
    let rect: Mesh2dHandle = meshes.add(shape::Quad::new(vec2(1.0, 1.0)).into()).into();

    commands.insert_resource(MeshHandles { circle, rect });

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

fn arc_mesh(sides: usize, radius: f32, start_angle: f32, end_angle: f32) -> Mesh {
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
pub fn hr(pico: &mut Pico, width: Val, height: Val, parent: Option<usize>) -> usize {
    let index = pico.last();
    pico.add(PicoItem {
        uv_position: vec2(0.5, 0.0),
        width,
        height,
        background: Color::rgba(1.0, 1.0, 1.0, 0.04),
        anchor: Anchor::TopCenter,
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
    start: Val,
    margin: Val,
    height: Val,
    label_width: Val,
    drag_width: Val,
    corner_radius: Val,
    label: &str,
    scale: f32,
    value: f32,
    parent: usize,
    char_input_events: Option<&mut EventReader<ReceivedCharacter>>,
) -> DragValue {
    let mut value = value;
    let mut drag_bg = drag_bg;

    let lane = pico
        .add(PicoItem {
            width: Val::Percent(100.0),
            height,
            anchor: Anchor::TopLeft,
            parent: Some(parent),
            ..default()
        })
        .last();

    let text_index = pico.items.len();
    let drag_index = text_index + 1;
    {
        let _guard = pico.hstack(start, margin, lane);
        pico.add(PicoItem {
            text: label.to_string(),
            width: label_width,
            height: Val::Percent(100.0),
            anchor_text: Anchor::CenterLeft,
            anchor: Anchor::TopLeft,
            parent: Some(lane),
            ..default()
        });

        pico.add(PicoItem {
            // TODO user or auto precision
            text: format!("{:.2}", value),
            width: drag_width,
            height: Val::Percent(100.0),
            corner_radius,
            anchor: Anchor::TopLeft,
            parent: Some(lane),
            ..default()
        });
    }

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
    pico.get_mut(drag_index).background = if pico.hovered(drag_index) || dragging {
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

#[derive(Clone, Debug)]
pub struct PicoItem {
    pub text: String,
    pub x: Val,
    pub y: Val,
    pub width: Val,
    pub height: Val,
    /// uv position within window, is combined with x, y at pico.add(). Don't change after pico.add()
    pub uv_position: Vec2,
    /// uv size within window, is combined with width, height at pico.add(). Don't change after pico.add()
    pub uv_size: Vec2,
    /// 3d world space position. Don't change after pico.add()
    pub position_3d: Option<Vec3>,
    /// z position for 2d 1.0 is closer to camera 0.0 is further
    /// None for auto (calculated by order)
    pub depth: Option<f32>,
    // 50% will result in a circle
    pub corner_radius: Val,
    pub font_size: f32,
    pub color: Color,
    pub background: Color,
    pub alignment: TextAlignment,
    pub anchor: Anchor,
    pub anchor_text: Anchor,
    pub anchor_parent: Anchor,
    /// If life is 0.0, it will only live one frame (default), if life is f32::INFINITY it will live forever.
    pub life: f32,
    /// If the id changes, the item is re-rendered
    pub id: Option<u64>,
    /// If the spatial_id changes a new state is used
    /// Impacted by position, size, anchor (after transform from parent is applied, if any)
    pub spatial_id: Option<u64>,
    /// If set, coordinates for position/size will be relative to parent.
    pub parent: Option<usize>,
    // Coordinates are uv space 0..1 over the whole window
    pub computed_bbox: Option<Vec4>,
}

impl Default for PicoItem {
    fn default() -> Self {
        PicoItem {
            x: Val::default(),
            y: Val::default(),
            width: Val::default(),
            height: Val::default(),
            uv_position: Vec2::ZERO,
            position_3d: None,
            depth: None,
            corner_radius: Val::default(),
            uv_size: Vec2::ZERO,
            text: String::new(),
            font_size: 0.02,
            color: Color::WHITE,
            background: Color::NONE,
            alignment: TextAlignment::Center,
            anchor_text: Anchor::Center,
            anchor: Anchor::Center,
            anchor_parent: Anchor::TopLeft,
            life: 0.0,
            id: None,
            spatial_id: None,
            parent: None,
            computed_bbox: None,
        }
    }
}

impl PicoItem {
    pub fn new2d(position: Vec2, text: &str) -> PicoItem {
        PicoItem {
            uv_position: position,
            text: text.to_string(),
            ..default()
        }
    }
    pub fn new3d(position_3d: Vec3, text: &str) -> PicoItem {
        PicoItem {
            position_3d: Some(position_3d),
            text: text.to_string(),
            ..default()
        }
    }
    pub fn keep(mut self) -> Self {
        self.life = f32::INFINITY;
        self
    }
    fn generate_spatial_id(&self) -> u64 {
        let hasher = &mut DefaultHasher::new();
        self.uv_position.x.to_bits().hash(hasher);
        self.uv_position.y.to_bits().hash(hasher);
        self.uv_size.x.to_bits().hash(hasher);
        self.uv_size.y.to_bits().hash(hasher);
        if let Some(depth) = self.depth {
            depth.to_bits().hash(hasher);
        }
        if let Some(position_3d) = self.position_3d {
            position_3d.x.to_bits().hash(hasher);
            position_3d.y.to_bits().hash(hasher);
            position_3d.z.to_bits().hash(hasher);
        }
        format!("{:?}", self.anchor).hash(hasher);
        format!("{:?}", self.anchor_parent).hash(hasher);
        format!("{:?}", self.anchor_text).hash(hasher);
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

#[derive(Clone, Copy)]
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
    pub mouse_button_input: Option<Input<MouseButton>>,
    pub auto_depth: f32,
}

impl Pico {
    pub fn vstack(&mut self, start: Val, margin: Val, parent: usize) -> Guard {
        let bbox = self.bbox(parent);
        let parent_size = (bbox.zw() - bbox.xy()).abs();
        let start = self.val_in_parent_y(start, parent_size) * parent_size.y;
        let margin = self.val_in_parent_y(margin, parent_size) * parent_size.y;
        self.stack_stack.push(Stack {
            end: start,
            margin,
            vertical: true,
        });
        self.stack_guard.push();
        self.stack_guard.clone()
    }
    pub fn hstack(&mut self, start: Val, margin: Val, parent: usize) -> Guard {
        let bbox = self.bbox(parent);
        let parent_size = (bbox.zw() - bbox.xy()).abs();
        let start = self.val_in_parent_x(start, parent_size) * parent_size.x;
        let margin = self.val_in_parent_x(margin, parent_size) * parent_size.x;
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
        let parent_2d_bbox = if let Some(parent) = item.parent {
            if let Some(parent_depth) = self.get(parent).depth {
                if let Some(depth) = &mut item.depth {
                    *depth += parent_depth;
                    if *depth == parent_depth {
                        // Make sure child is in front of parent if they were at the same z
                        *depth += 0.000001;
                    }
                } else {
                    self.auto_depth += 0.000001;
                    item.depth = Some((parent_depth + 0.000001).max(self.auto_depth));
                }
            }
            self.bbox(parent)
        } else {
            vec4(0.0, 0.0, 1.0, 1.0)
        };

        if item.depth.is_none() {
            self.auto_depth += 0.000001;
            item.depth = Some(self.auto_depth);
        }

        let parent_size = (parent_2d_bbox.zw() - parent_2d_bbox.xy()).abs();

        let vx = self.val_in_parent_x(item.x, parent_size);
        let vy = self.val_in_parent_y(item.y, parent_size);
        let vw = self.val_in_parent_x(item.width, parent_size);
        let vh = self.val_in_parent_y(item.height, parent_size);

        let pa_vec = item.anchor_parent.as_vec() * vec2(1.0, -1.0);
        item.uv_position += vec2(vx, vy);
        item.uv_position *= -pa_vec * 2.0;
        item.uv_position += pa_vec + vec2(0.5, 0.5);
        item.uv_position = lerp2(parent_2d_bbox.xy(), parent_2d_bbox.zw(), item.uv_position);
        item.uv_size += vec2(vw, vh);
        item.uv_size *= (parent_2d_bbox.zw() - parent_2d_bbox.xy()).abs();

        while (self.stack_guard.get() as usize) < self.stack_stack.len() {
            self.stack_stack.pop();
        }
        if !self.stack_stack.is_empty() {
            let stack = self.stack_stack.last_mut().unwrap();
            if stack.vertical {
                item.uv_position.y += stack.end;
                let bbox = get_bbox(item.uv_size, item.uv_position, &item.anchor);
                stack.end = stack.end.max(bbox.w - parent_2d_bbox.y) + stack.margin;
            } else {
                item.uv_position.x += stack.end;
                let bbox = get_bbox(item.uv_size, item.uv_position, &item.anchor);
                stack.end = stack.end.max(bbox.z - parent_2d_bbox.x) + stack.margin;
            }
        }

        if item.spatial_id.is_none() {
            item.spatial_id = Some(item.generate_spatial_id());
        }
        item.computed_bbox = Some(if item.position_3d.is_some() {
            if let Some(state_item) = self.state.get(&item.spatial_id.unwrap()) {
                state_item.bbox
            } else {
                Vec4::ZERO
            }
        } else {
            get_bbox(item.uv_size, item.uv_position, &item.anchor)
        });
        self.items.push(item);
        self
    }

    // get scaled v of uv within parent
    fn val_in_parent_x(&self, x: Val, parent_size: Vec2) -> f32 {
        let vx = match x {
            Val::Auto => 0.0,
            Val::Px(n) => n / self.window_size.x,
            Val::Percent(n) => (n / 100.0) * parent_size.x,
            Val::Vw(n) => n / 100.0,
            Val::Vh(n) => (n / 100.0) * (self.window_size.y / self.window_size.x),
            Val::VMin(n) => {
                (n / 100.0) * (self.window_size.x.min(self.window_size.y) / self.window_size.x)
            }
            Val::VMax(n) => {
                (n / 100.0) * (self.window_size.x.max(self.window_size.y) / self.window_size.x)
            }
        } / parent_size.x;
        vx
    }

    // get scaled u of uv within parent
    fn val_in_parent_y(&self, y: Val, parent_size: Vec2) -> f32 {
        let vy = match y {
            Val::Auto => 0.0,
            Val::Px(n) => n / self.window_size.y,
            Val::Percent(n) => (n / 100.0) * parent_size.y,
            Val::Vw(n) => (n / 100.0) * (self.window_size.x / self.window_size.y),
            Val::Vh(n) => n / 100.0,
            Val::VMin(n) => {
                (n / 100.0) * (self.window_size.x.min(self.window_size.y) / self.window_size.y)
            }
            Val::VMax(n) => {
                (n / 100.0) * (self.window_size.x.max(self.window_size.y) / self.window_size.y)
            }
        } / parent_size.y;
        vy
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
pub struct PicoEntity {
    spatial_id: u64,
    anchor: Anchor,
    size: Vec2,
}

#[allow(clippy::too_many_arguments)]
fn render(
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
            let corner_radius = pico.val_in_parent_y(item.corner_radius, item.uv_size)
                * item.uv_size.y
                * window_size.y;
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
                let size = item.uv_size * window_size;
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

                //let material_handle = materials.add(ColorMaterial::from(item.background));

                entity.with_children(|builder| {
                    let item_anchor_vec = item.anchor.as_vec();
                    let cr2 = corner_radius * 2.0;
                    if item.background.a() > 0.0 {
                        let anchor_trans = (-item_anchor_vec * size).extend(0.0);
                        if corner_radius <= 0.0 {
                            builder.spawn(MaterialMesh2dBundle {
                                mesh: mesh_handles.rect.clone(),
                                material: material_handle.clone(),
                                transform: Transform::from_translation(anchor_trans)
                                    .with_scale(size.extend(1.0)),
                                ..default()
                            });
                        } else {
                            // Make cross shape with gaps for the arcs
                            builder.spawn(MaterialMesh2dBundle {
                                mesh: mesh_handles.rect.clone(),
                                material: material_handle.clone(),
                                transform: Transform::from_translation(anchor_trans)
                                    .with_scale((size - vec2(cr2, 0.0)).extend(1.0)),
                                ..default()
                            });
                            let s = vec2(corner_radius, size.y - cr2).extend(1.0);
                            let off = vec3((size.x - corner_radius) * 0.5, 0.0, 0.0);
                            builder.spawn(MaterialMesh2dBundle {
                                mesh: mesh_handles.rect.clone(),
                                material: material_handle.clone(),
                                transform: Transform::from_translation(anchor_trans + off)
                                    .with_scale(s),
                                ..default()
                            });
                            builder.spawn(MaterialMesh2dBundle {
                                mesh: mesh_handles.rect.clone(),
                                material: material_handle.clone(),
                                transform: Transform::from_translation(anchor_trans - off)
                                    .with_scale(s),
                                ..default()
                            });
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
                                        (offset + size * (-item_anchor_vec - 0.5))
                                            .extend(0.00000005),
                                    )
                                    .with_scale(Vec3::splat(corner_radius))
                                    .with_rotation(Quat::from_rotation_z(angle)),
                                    ..default()
                                });
                            }
                        }
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

fn get_bbox(size: Vec2, uv_position: Vec2, anchor: &Anchor) -> Vec4 {
    let half_size = size * 0.5;
    let a = uv_position - half_size + size * -anchor.as_vec() * vec2(1.0, -1.0);
    let b = uv_position + half_size + size * -anchor.as_vec() * vec2(1.0, -1.0);
    vec4(a.x, a.y, b.x, b.y)
}

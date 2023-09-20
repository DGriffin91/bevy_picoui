use bevy::{
    math::{vec2, vec4, Vec4Swizzles},
    prelude::*,
    sprite::Anchor,
    text::DEFAULT_FONT_HANDLE,
    utils::HashMap,
};
use core::hash::Hash;
use core::hash::Hasher;
use std::collections::hash_map::DefaultHasher;

use crate::{
    guard::Guard,
    hash::{hash_anchor, hash_color, hash_val, hash_vec2, hash_vec3, hash_vec4},
    renderer::MAJOR_DEPTH_AUTO_STEP,
};

#[derive(Clone, Copy, Debug, Hash)]
pub struct ItemIndex(pub usize);

// Only supports one camera.
#[derive(Component)]
pub struct Pico2dCamera;

#[derive(Clone, Debug)]
pub struct ItemStyle {
    // 50% will result in a circle
    pub corner_radius: Val,
    pub border_width: Val,
    pub border_color: Color,
    pub font_size: Val,
    pub font: Handle<Font>,
    pub text_color: Color,
    pub background_color: Color,
    pub anchor_text: Anchor,
    pub text_alignment: TextAlignment,
}

impl Default for ItemStyle {
    fn default() -> Self {
        ItemStyle {
            corner_radius: Val::default(),
            border_width: Val::default(),
            border_color: Color::BLACK,
            font_size: Val::Vh(2.0),
            font: DEFAULT_FONT_HANDLE.typed(),
            text_color: Color::WHITE,
            background_color: Color::NONE,
            text_alignment: TextAlignment::Center,
            anchor_text: Anchor::Center,
        }
    }
}

impl Hash for ItemStyle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_val(&self.corner_radius, state);
        hash_val(&self.border_width, state);
        hash_color(&self.border_color, state);
        hash_val(&self.font_size, state);
        self.font.hash(state);
        hash_color(&self.text_color, state);
        hash_color(&self.background_color, state);
        self.text_alignment.hash(state);
        hash_anchor(&self.anchor_text, state)
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProcessedPicoItem {
    pub text: String,
    pub style: ItemStyle,
    /// uv position within window
    uv_position: Vec2,
    /// uv size within window
    uv_size: Vec2,
    /// 3d world space position.
    pub position_3d: Option<Vec3>,
    /// z position for 2d 1.0 is closer to camera 0.0 is further
    /// None for auto (calculated by order)
    depth: f32,
    /// Max z position of immediate children, used for auto z
    child_max_depth: f32,
    /// If life is 0.0, it will only live one frame (default), if life is f32::INFINITY it will live forever.
    life: f32,
    /// If the id changes, the item is re-rendered
    pub id: Option<u64>,
    /// If the spatial_id changes a new state is used
    /// Impacted by position, size, anchor (after transform from parent is applied, if any)
    spatial_id: u64,
    /// If set, coordinates for position/size will be relative to parent.
    parent: Option<ItemIndex>,
    // Coordinates are uv space 0..1 over the whole window
    bbox: Vec4,
    anchor: Anchor,
}

impl ProcessedPicoItem {
    pub fn get_uv_position(&self) -> Vec2 {
        self.uv_position
    }
    pub fn get_uv_size(&self) -> Vec2 {
        self.uv_size
    }
    pub fn get_depth(&self) -> f32 {
        self.depth
    }
    pub fn get_spatial_id(&self) -> u64 {
        self.spatial_id
    }
    pub fn get_parent(&self) -> Option<ItemIndex> {
        self.parent
    }
    pub fn get_bbox(&self) -> Vec4 {
        self.bbox
    }
    pub fn get_life(&self) -> f32 {
        self.life
    }
    pub fn get_anchor(&self) -> Anchor {
        self.anchor.clone()
    }
    pub fn generate_id(&mut self) -> u64 {
        self.id = None;
        let state = &mut DefaultHasher::new();
        self.spatial_id.hash(state);
        hash_vec4(&self.bbox, state);
        self.depth.to_bits().hash(state);
        self.text.hash(state);
        self.life.to_bits().hash(state);
        self.style.hash(state);
        state.finish()
    }
}

#[derive(Clone, Debug)]
pub struct PicoItem {
    pub text: String,
    pub x: Val,
    pub y: Val,
    pub width: Val,
    pub height: Val,
    pub style: ItemStyle,
    pub anchor: Anchor,
    pub anchor_parent: Anchor,
    /// uv position within window, is combined with x, y at pico.add().
    pub uv_position: Vec2,
    /// uv size within window, is combined with width, height at pico.add().
    pub uv_size: Vec2,
    /// 3d world space position.
    pub position_3d: Option<Vec3>,
    /// z position for 2d 1.0 is closer to camera 0.0 is further
    /// None for auto (calculated by order)
    pub depth: Option<f32>,
    /// If life is 0.0, it will only live one frame (default), if life is f32::INFINITY it will live forever.
    pub life: f32,
    /// If the id changes, the item is re-rendered
    pub id: Option<u64>,
    /// If the spatial_id changes a new state is used
    /// Impacted by position, size, anchor (after transform from parent is applied, if any)
    pub spatial_id: Option<u64>,
    /// If set, coordinates for position/size will be relative to parent.
    pub parent: Option<ItemIndex>,
}

impl Default for PicoItem {
    fn default() -> Self {
        PicoItem {
            x: Val::default(),
            y: Val::default(),
            width: Val::default(),
            height: Val::default(),
            style: ItemStyle::default(),
            anchor: Anchor::Center,
            anchor_parent: Anchor::TopLeft,
            uv_position: Vec2::ZERO,
            position_3d: None,
            depth: None,
            uv_size: Vec2::ZERO,
            text: String::new(),
            life: 0.0,
            id: None,
            spatial_id: None,
            parent: None,
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
    pub fn generate_spatial_id(
        uv_position: &Vec2,
        uv_size: &Vec2,
        depth: &f32,
        position_3d: &Option<Vec3>,
        anchor: &Anchor,
        anchor_text: &Anchor,
        parent: &Option<ItemIndex>,
    ) -> u64 {
        let hasher = &mut DefaultHasher::new();
        hash_vec2(&uv_position, hasher);
        hash_vec2(&uv_size, hasher);
        depth.to_bits().hash(hasher);
        if let Some(position_3d) = position_3d {
            hash_vec3(&position_3d, hasher);
        }
        hash_anchor(&anchor, hasher);
        hash_anchor(&anchor_text, hasher);
        parent.hash(hasher);
        hasher.finish()
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

pub fn lerp2(start: Vec2, end: Vec2, t: Vec2) -> Vec2 {
    (1.0 - t) * start + t * end
}

pub fn lerp(start: f32, end: f32, t: f32) -> f32 {
    (1.0 - t) * start + t * end
}

#[derive(Clone, Copy, Default)]
pub struct Stack {
    pub end: f32,
    pub margin: f32,
    pub vertical: bool,
    pub bypass: bool,
}

#[derive(Resource, Default)]
pub struct Pico {
    pub state: HashMap<u64, StateItem>,
    pub items: Vec<ProcessedPicoItem>,
    pub interacting: bool,
    pub stack_stack: Vec<Stack>,
    pub stack_guard: Guard,
    pub window_size: Vec2,
    pub mouse_button_input: Option<Input<MouseButton>>,
    pub internal_auto_depth: f32,
}

impl Pico {
    pub fn vstack(&mut self, start: Val, margin: Val, parent: &ItemIndex) -> Guard {
        self.update_stack();
        let bbox = self.get(parent).bbox;
        let parent_size = (bbox.zw() - bbox.xy()).abs();
        let start = self.valp_y(start, parent_size);
        let margin = self.valp_y(margin, parent_size);
        self.stack_stack.push(Stack {
            end: start,
            margin,
            vertical: true,
            bypass: false,
        });
        self.stack_guard.push();
        self.stack_guard.clone()
    }

    pub fn hstack(&mut self, start: Val, margin: Val, parent: &ItemIndex) -> Guard {
        self.update_stack();
        let bbox = self.get(parent).bbox;
        let parent_size = (bbox.zw() - bbox.xy()).abs();
        let start = self.valp_x(start, parent_size);
        let margin = self.valp_x(margin, parent_size);
        self.stack_stack.push(Stack {
            end: start,
            margin,
            vertical: false,
            bypass: false,
        });
        self.stack_guard.push();
        self.stack_guard.clone()
    }

    pub fn stack_bypass(&mut self) -> Guard {
        self.update_stack();
        self.stack_stack.push(Stack {
            bypass: true,
            ..default()
        });
        self.stack_guard.push();
        self.stack_guard.clone()
    }

    pub fn get_hovered(&self, index: &ItemIndex) -> Option<&StateItem> {
        if let Some(state_item) = self.get_state(index) {
            if state_item.hover {
                return Some(state_item);
            }
        }
        None
    }

    pub fn clicked(&self, index: &ItemIndex) -> bool {
        if let Some(state_item) = self.get_hovered(index) {
            if let Some(input) = &state_item.input {
                return input.just_pressed(MouseButton::Left);
            }
        }
        false
    }

    pub fn released(&self, index: &ItemIndex) -> bool {
        if let Some(state_item) = self.get_hovered(index) {
            if let Some(input) = &state_item.input {
                return input.just_released(MouseButton::Left);
            }
        }
        false
    }

    pub fn center(&self, index: &ItemIndex) -> Vec2 {
        let bbox = self.get(index).bbox;
        (bbox.xy() + bbox.zw()) / 2.0
    }

    pub fn hovered(&self, index: &ItemIndex) -> bool {
        self.get_hovered(index).is_some()
    }

    pub fn auto_depth(&mut self) -> f32 {
        self.internal_auto_depth += MAJOR_DEPTH_AUTO_STEP;
        self.internal_auto_depth
    }

    pub fn add(&mut self, item: PicoItem) -> ItemIndex {
        let mut item_depth = item.depth;
        let item_x = item.x;
        let item_y = item.y;
        let item_width = item.width;
        let item_height = item.height;
        let item_anchor_parent = item.anchor_parent;
        let item_spatial_id = item.spatial_id;
        let mut processed_item = ProcessedPicoItem {
            text: item.text,
            style: item.style,
            uv_position: item.uv_position,
            uv_size: item.uv_size,
            life: item.life,
            id: item.id,
            parent: item.parent,
            anchor: item.anchor,
            position_3d: item.position_3d,
            child_max_depth: 0.0,
            spatial_id: default(),
            depth: default(),
            bbox: default(),
        };

        if let Some(parent_index) = processed_item.parent {
            let parent = self.get(&parent_index);
            if let Some(depth) = &mut item_depth {
                *depth += parent.depth;
                if *depth == parent.depth {
                    // Make sure child is in front of parent if they were at the same z
                    *depth += MAJOR_DEPTH_AUTO_STEP;
                }
            } else {
                item_depth = Some(
                    (parent.depth + MAJOR_DEPTH_AUTO_STEP)
                        .max(parent.child_max_depth + MAJOR_DEPTH_AUTO_STEP),
                );
            }
        }

        if item_depth.is_none() {
            item_depth = Some(self.auto_depth());
        }

        processed_item.depth = item_depth.unwrap();

        let parent_2d_bbox = if let Some(parent_index) = processed_item.parent {
            let parent = self.get_mut(&parent_index);
            parent.child_max_depth = parent.child_max_depth.max(processed_item.depth);
            self.get(&parent_index).bbox
        } else {
            vec4(0.0, 0.0, 1.0, 1.0)
        };

        let parent_size = (parent_2d_bbox.zw() - parent_2d_bbox.xy()).abs();

        let vx = self.valp_x(item_x, parent_size) / parent_size.x;
        let vy = self.valp_y(item_y, parent_size) / parent_size.y;
        let vw = self.valp_x(item_width, parent_size) / parent_size.x;
        let vh = self.valp_y(item_height, parent_size) / parent_size.y;

        let pa_vec = item_anchor_parent.as_vec() * vec2(1.0, -1.0);
        let mut uv_position = vec2(vx, vy);
        uv_position *= -pa_vec * 2.0;
        uv_position += pa_vec + vec2(0.5, 0.5);

        // If anchor parent is center it should offset toward the bottom right
        if pa_vec.x == 0.0 {
            uv_position.x += vx;
        }
        if pa_vec.y == 0.0 {
            uv_position.y += vy;
        }

        processed_item.uv_position += uv_position;

        processed_item.uv_position = lerp2(
            parent_2d_bbox.xy(),
            parent_2d_bbox.zw(),
            processed_item.uv_position,
        );
        processed_item.uv_size += vec2(vw, vh);
        processed_item.uv_size *= (parent_2d_bbox.zw() - parent_2d_bbox.xy()).abs();

        self.update_stack();
        if !self.stack_stack.is_empty() && processed_item.parent.is_some() {
            let stack = self.stack_stack.last_mut().unwrap();
            if !stack.bypass {
                if stack.vertical {
                    processed_item.uv_position.y += stack.end;
                    let bbox = get_bbox(
                        processed_item.uv_size,
                        processed_item.uv_position,
                        &processed_item.anchor,
                    );
                    stack.end = stack.end.max(bbox.w - parent_2d_bbox.y) + stack.margin;
                } else {
                    processed_item.uv_position.x += stack.end;
                    let bbox = get_bbox(
                        processed_item.uv_size,
                        processed_item.uv_position,
                        &processed_item.anchor,
                    );
                    stack.end = stack.end.max(bbox.z - parent_2d_bbox.x) + stack.margin;
                }
            }
        }

        processed_item.spatial_id = item_spatial_id.unwrap_or(PicoItem::generate_spatial_id(
            &processed_item.uv_position,
            &processed_item.uv_size,
            &processed_item.depth,
            &processed_item.position_3d,
            &processed_item.anchor,
            &processed_item.style.anchor_text,
            &processed_item.parent,
        ));

        processed_item.bbox = if processed_item.position_3d.is_some() {
            if let Some(state_item) = self.state.get(&processed_item.spatial_id) {
                state_item.bbox
            } else {
                Vec4::ZERO
            }
        } else {
            get_bbox(
                processed_item.uv_size,
                processed_item.uv_position,
                &processed_item.anchor,
            )
        };
        self.items.push(processed_item);
        ItemIndex(self.items.len() - 1)
    }

    fn update_stack(&mut self) {
        while (self.stack_guard.get() as usize) < self.stack_stack.len() {
            self.stack_stack.pop();
        }
    }

    // get scaled v of uv for val
    pub fn valp_x(&self, x: Val, parent_size: Vec2) -> f32 {
        match x {
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
        }
    }

    // get scaled u of uv for val
    pub fn valp_y(&self, y: Val, parent_size: Vec2) -> f32 {
        match y {
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
        }
    }

    pub fn val_x(&self, v: Val) -> f32 {
        self.valp_x(v, Vec2::ONE)
    }

    pub fn val_y(&self, v: Val) -> f32 {
        self.valp_y(v, Vec2::ONE)
    }

    pub fn val_x_px(&self, v: Val) -> f32 {
        self.val_x(v) * self.window_size.x
    }

    pub fn val_y_px(&self, v: Val) -> f32 {
        self.val_y(v) * self.window_size.y
    }

    pub fn uv_scale_to_px(&self, uv: Vec2) -> Vec2 {
        uv * self.window_size
    }

    /// For setting px in Val::Px() where +y is down
    pub fn uv_position_to_px(&self, uv: Vec2) -> Vec2 {
        (uv - 0.5) * self.window_size
    }

    /// For setting px in Transform where +y is up
    pub fn uv_position_to_ws_px(&self, uv: Vec2) -> Vec2 {
        (uv - 0.5) * vec2(1.0, -1.0) * self.window_size
    }

    pub fn get_state_mut(&mut self, index: &ItemIndex) -> Option<&mut StateItem> {
        let id = self.get(index).spatial_id;
        self.state.get_mut(&id)
    }

    pub fn get_state(&self, index: &ItemIndex) -> Option<&StateItem> {
        self.state.get(&self.get(index).spatial_id)
    }

    pub fn get_mut(&mut self, index: &ItemIndex) -> &mut ProcessedPicoItem {
        if index.0 >= self.items.len() {
            panic!(
                "Tried to access item {} but there are only {}",
                index.0,
                self.items.len()
            );
        }
        &mut self.items[index.0]
    }

    pub fn get(&self, index: &ItemIndex) -> &ProcessedPicoItem {
        if index.0 >= self.items.len() {
            panic!(
                "Tried to access item {} but there are only {}",
                index.0,
                self.items.len()
            );
        }
        &self.items[index.0]
    }

    pub fn storage(&mut self) -> Option<&mut Option<Box<dyn std::any::Any + Send + Sync>>> {
        if let Some(item) = self.items.last() {
            if let Some(state_item) = self.state.get_mut(&item.spatial_id) {
                return Some(&mut state_item.storage);
            }
        }
        None
    }
}

/// Units uv of the window
#[derive(Debug, Default, Clone, Copy)]
pub struct Drag {
    pub start: Vec2,
    pub end: Vec2,
    pub last_frame: Vec2,
}

impl Drag {
    /// Units uv of the window
    pub fn delta(&self) -> Vec2 {
        self.end - self.last_frame
    }
    /// Units uv of the window
    pub fn total_delta(&self) -> Vec2 {
        self.end - self.start
    }
}

pub fn get_bbox(size: Vec2, uv_position: Vec2, anchor: &Anchor) -> Vec4 {
    let half_size = size * 0.5;
    let a = uv_position - half_size + size * -anchor.as_vec() * vec2(1.0, -1.0);
    let b = uv_position + half_size + size * -anchor.as_vec() * vec2(1.0, -1.0);
    vec4(a.x, a.y, b.x, b.y)
}

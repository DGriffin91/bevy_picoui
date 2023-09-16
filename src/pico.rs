use bevy::{
    math::{vec2, vec4, Vec4Swizzles},
    prelude::*,
    sprite::Anchor,
    utils::HashMap,
};
use core::hash::Hash;
use core::hash::Hasher;
use std::collections::hash_map::DefaultHasher;

use crate::guard::Guard;

#[derive(Clone, Copy, Debug)]
pub struct ItemIndex(pub usize);

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
    pub parent: Option<ItemIndex>,
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
    pub fn generate_spatial_id(&self) -> u64 {
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
    pub fn generate_id(&mut self) -> u64 {
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
    pub fn vstack(&mut self, start: Val, margin: Val, parent: ItemIndex) -> Guard {
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

    pub fn hstack(&mut self, start: Val, margin: Val, parent: ItemIndex) -> Guard {
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

    pub fn get_hovered(&self, index: ItemIndex) -> Option<&StateItem> {
        if let Some(state_item) = self.get_state(index) {
            if state_item.hover {
                return Some(state_item);
            }
        }
        None
    }

    pub fn clicked(&self, index: ItemIndex) -> bool {
        if let Some(state_item) = self.get_hovered(index) {
            if let Some(input) = &state_item.input {
                return input.just_pressed(MouseButton::Left);
            }
        }
        false
    }

    pub fn released(&self, index: ItemIndex) -> bool {
        if let Some(state_item) = self.get_hovered(index) {
            if let Some(input) = &state_item.input {
                return input.just_released(MouseButton::Left);
            }
        }
        false
    }

    pub fn bbox(&self, index: ItemIndex) -> Vec4 {
        let item = self.get(index);
        if let Some(computed_bbox) = item.computed_bbox {
            return computed_bbox;
        }
        vec4(0.0, 0.0, 1.0, 1.0)
    }

    pub fn hovered(&self, index: ItemIndex) -> bool {
        self.get_hovered(index).is_some()
    }

    pub fn add(&mut self, mut item: PicoItem) -> ItemIndex {
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
        ItemIndex(self.items.len() - 1)
    }

    // get scaled v of uv within parent
    pub fn val_in_parent_x(&self, x: Val, parent_size: Vec2) -> f32 {
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
        };
        vx / parent_size.x
    }

    // get scaled u of uv within parent
    pub fn val_in_parent_y(&self, y: Val, parent_size: Vec2) -> f32 {
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
        };
        vy / parent_size.y
    }

    pub fn get_state_mut(&mut self, index: ItemIndex) -> Option<&mut StateItem> {
        self.state.get_mut(&self.get(index).spatial_id.unwrap())
    }

    pub fn get_state(&self, index: ItemIndex) -> Option<&StateItem> {
        self.state.get(&self.get(index).spatial_id.unwrap())
    }

    pub fn get_mut(&mut self, index: ItemIndex) -> &mut PicoItem {
        if index.0 >= self.items.len() {
            panic!(
                "Tried to access item {} but there are only {}",
                index.0,
                self.items.len()
            );
        }
        &mut self.items[index.0]
    }

    pub fn get(&self, index: ItemIndex) -> &PicoItem {
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

pub fn get_bbox(size: Vec2, uv_position: Vec2, anchor: &Anchor) -> Vec4 {
    let half_size = size * 0.5;
    let a = uv_position - half_size + size * -anchor.as_vec() * vec2(1.0, -1.0);
    let b = uv_position + half_size + size * -anchor.as_vec() * vec2(1.0, -1.0);
    vec4(a.x, a.y, b.x, b.y)
}

use bevy::{
    ecs::system::SystemParam,
    math::{vec2, vec4, Vec4Swizzles},
    prelude::*,
    render::render_resource::BlendState,
    sprite::{Anchor, Material2d},
    utils::{label::DynHash, HashMap},
};
use core::hash::Hash;
use core::hash::Hasher;
use std::collections::hash_map::DefaultHasher;

use crate::{
    guard::Guard,
    hash::{hash_anchor, hash_color, hash_val, hash_vec2, hash_vec3, hash_vec4},
    rectangle_material::{RectangleMaterial, RectangleMaterialUniform},
    renderer::MAJOR_DEPTH_AUTO_STEP,
};

#[derive(Clone, Copy, Debug, Hash)]
pub struct ItemIndex(pub usize);

// Only supports one camera.
#[derive(Component)]
pub struct Pico2dCamera;

#[derive(Component, Clone, Debug)]
pub struct MaterialHandleEntity<M: Material2d>(pub Handle<M>);

#[derive(Clone, Debug)]
pub struct ItemStyle {
    // 50% will result in a circle
    pub corner_radius: Val,
    /// `corner_radius` is added to `multi_corner_radius`, usually set one or the other.
    /// Order is clockwise: tl, tr, br, bl.
    pub multi_corner_radius: (Val, Val, Val, Val),
    /// Optional margins for 9-patch content.
    /// Units are pixels: Left, Top, Right, Bottom
    pub nine_patch: Option<(u32, u32, u32, u32)>,
    pub border_width: Val,
    pub border_color: Color,
    pub border_softness: Val,
    pub font_size: Val,
    // If no font is specified, the default bevy font (a minimal subset of FiraMono) will be used.
    pub font: Handle<Font>,
    pub text_color: Color,
    pub background_color: Color,
    /// The gradient is added to the `background_color`, use Color::None on one or the other if color mixing is not desired.
    pub background_gradient: (Color, Color),
    pub background_uv_transform: Transform,
    /// An additional transform applied only to rendering, does not affect children etc...
    pub render_transform: Transform,
    pub edge_softness: Val,
    pub anchor_text: Anchor,
    pub text_alignment: TextAlignment,
    pub material: Option<Entity>,
    /// For image to be fully opaque with the correct colors, the background needs to be white.
    pub image: Option<Handle<Image>>,
    pub blend_state: Option<BlendState>,
}

impl Default for ItemStyle {
    fn default() -> Self {
        ItemStyle {
            corner_radius: Val::default(),
            multi_corner_radius: (
                Val::default(),
                Val::default(),
                Val::default(),
                Val::default(),
            ),
            nine_patch: None,
            border_width: Val::default(),
            border_color: Color::BLACK,
            border_softness: Val::Px(0.5),
            font_size: Val::Vh(2.0),
            font: Default::default(),
            text_color: Color::WHITE,
            background_color: Color::NONE,
            background_gradient: (Color::NONE, Color::NONE),
            edge_softness: Val::Px(1.0),
            background_uv_transform: Transform::default(),
            render_transform: Transform::default(),
            text_alignment: TextAlignment::Center,
            anchor_text: Anchor::Center,
            material: None,
            image: None,
            blend_state: Some(BlendState::ALPHA_BLENDING),
        }
    }
}

#[derive(SystemParam)]
pub struct PicoMaterials<'w, 's, M: Material2d> {
    q: Query<'w, 's, (Entity, &'static MaterialHandleEntity<M>)>,
}

impl ItemStyle {
    pub fn set_custom_material<M: Material2d>(
        mut self,
        commands: &mut Commands,
        material_handle: &Handle<M>,
        query: PicoMaterials<M>,
    ) -> Self {
        for (entity, handle) in &query.q {
            if material_handle.id() == handle.0.id() {
                self.material = Some(entity);
                return self;
            }
        }
        self.material = Some(
            commands
                .spawn(MaterialHandleEntity(material_handle.clone()))
                .id(),
        );
        self
    }
}

impl Hash for ItemStyle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_val(&self.corner_radius, state);
        hash_val(&self.multi_corner_radius.0, state);
        hash_val(&self.multi_corner_radius.1, state);
        hash_val(&self.multi_corner_radius.2, state);
        hash_val(&self.multi_corner_radius.3, state);
        self.nine_patch.hash(state);
        hash_val(&self.border_width, state);
        hash_color(&self.border_color, state);
        hash_val(&self.border_softness, state);
        hash_val(&self.font_size, state);
        self.font.hash(state);
        hash_color(&self.text_color, state);
        hash_color(&self.background_color, state);
        hash_color(&self.background_gradient.0, state);
        hash_color(&self.background_gradient.1, state);
        if self.background_uv_transform != Transform::default() {
            let mat = self.background_uv_transform.compute_matrix();
            hash_vec4(&mat.x_axis, state);
            hash_vec4(&mat.y_axis, state);
            hash_vec4(&mat.z_axis, state);
            hash_vec4(&mat.w_axis, state);
        }
        if self.render_transform != Transform::default() {
            let mat = self.render_transform.compute_matrix();
            hash_vec4(&mat.x_axis, state);
            hash_vec4(&mat.y_axis, state);
            hash_vec4(&mat.z_axis, state);
            hash_vec4(&mat.w_axis, state);
        }
        hash_val(&self.edge_softness, state);
        self.text_alignment.hash(state);
        hash_anchor(&self.anchor_text, state);
        if let Some(entity) = self.material {
            entity.hash(state);
        }
        if let Some(image) = &self.image {
            image.id().dyn_hash(state);
        }
        self.blend_state.hash(state);
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
        self.anchor
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
        hash_vec2(uv_position, hasher);
        hash_vec2(uv_size, hasher);
        depth.to_bits().hash(hasher);
        if let Some(position_3d) = position_3d {
            hash_vec3(position_3d, hasher);
        }
        hash_anchor(anchor, hasher);
        hash_anchor(anchor_text, hasher);
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
    // Unit for end and margin is u or v within parent
    pub end: f32,
    pub margin: f32,
    pub vertical: bool,
    pub reverse: bool,
    pub bypass: bool,
    pub parent: Option<ItemIndex>,
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
    pub fn vstack(&mut self, start: Val, margin: Val, reverse: bool, parent: &ItemIndex) -> Guard {
        self.update_stack();
        let bbox = self.get(parent).bbox;
        let parent_size = (bbox.zw() - bbox.xy()).abs();
        let start = self.valp_y(start, parent_size) * if reverse { -1.0 } else { 1.0 };
        let margin = self.valp_y(margin, parent_size);
        self.stack_stack.push(Stack {
            end: start,
            margin,
            vertical: true,
            reverse,
            bypass: false,
            parent: Some(*parent),
        });
        self.stack_guard.push();
        self.stack_guard.clone()
    }

    pub fn hstack(&mut self, start: Val, margin: Val, reverse: bool, parent: &ItemIndex) -> Guard {
        self.update_stack();
        let bbox = self.get(parent).bbox;
        let parent_size = (bbox.zw() - bbox.xy()).abs();
        let start = self.valp_x(start, parent_size) * if reverse { -1.0 } else { 1.0 };
        let margin = self.valp_x(margin, parent_size);
        self.stack_stack.push(Stack {
            end: start,
            margin,
            vertical: false,
            reverse,
            bypass: false,
            parent: Some(*parent),
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

    /// Get the remaining stack for the current stack inside the stack's parent.
    /// Unit is u or v within the parent.
    pub fn remaining_stack_space(&self) -> f32 {
        if let Some(stack) = self.stack_stack.last() {
            if let Some(parent_index) = stack.parent {
                let parent = self.get(&parent_index);
                let parent_size = (parent.bbox.zw() - parent.bbox.xy()).abs();
                return 1.0
                    + if stack.reverse { stack.end } else { -stack.end }
                        / if stack.vertical {
                            parent_size.y
                        } else {
                            parent_size.x
                        };
            }
        }
        1.0
    }

    fn get_hovered(&self, index: &ItemIndex) -> Option<&StateItem> {
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

        let parent_bbox = if let Some(parent_index) = processed_item.parent {
            let parent = self.get_mut(&parent_index);
            parent.child_max_depth = parent.child_max_depth.max(processed_item.depth);
            self.get(&parent_index).bbox
        } else {
            vec4(0.0, 0.0, 1.0, 1.0)
        };

        let parent_size = (parent_bbox.zw() - parent_bbox.xy()).abs();

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
            parent_bbox.xy(),
            parent_bbox.zw(),
            processed_item.uv_position,
        );
        processed_item.uv_size += vec2(vw, vh);
        processed_item.uv_size *= (parent_bbox.zw() - parent_bbox.xy()).abs();

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
                    if stack.reverse {
                        stack.end = stack.end.min(bbox.y - parent_bbox.w) - stack.margin;
                    } else {
                        stack.end = stack.end.max(bbox.w - parent_bbox.y) + stack.margin;
                    }
                } else {
                    processed_item.uv_position.x += stack.end;
                    let bbox = get_bbox(
                        processed_item.uv_size,
                        processed_item.uv_position,
                        &processed_item.anchor,
                    );
                    if stack.reverse {
                        stack.end = stack.end.min(bbox.x - parent_bbox.z) - stack.margin;
                    } else {
                        stack.end = stack.end.max(bbox.z - parent_bbox.x) + stack.margin;
                    }
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

    /// Uses x, y, and width from item along with args end_x, end_y to draw a line from
    /// x, y to end_x, end_y with width.
    /// Overrides height, anchor, and style.render_transform.rotation.
    /// Respects parent and parent_anchor
    pub fn add_line(&mut self, mut item: PicoItem, end_x: Val, end_y: Val) -> ItemIndex {
        let parent_size = if let Some(parent) = item.parent {
            let bbox = self.get(&parent).bbox;
            (bbox.xy() - bbox.zw()).abs()
        } else {
            vec2(1.0, 1.0)
        };
        let p1 = item.uv_position
            + vec2(
                self.valp_x(item.x, parent_size) / parent_size.x,
                self.valp_y(item.y, parent_size) / parent_size.y,
            );
        let p2 = vec2(
            self.valp_x(end_x, parent_size) / parent_size.x,
            self.valp_y(end_y, parent_size) / parent_size.y,
        );
        let center = (p1 + p2) * 0.5;
        let length = p1.distance(p2);
        let dir = (p2 - p1).normalize();
        let angle = dir.x.atan2(dir.y);
        item.uv_position = center;
        item.anchor = Anchor::Center;
        item.style.render_transform.rotation = Quat::from_rotation_z(angle);
        item.uv_size = vec2(
            item.uv_size.x + self.valp_x(item.width, parent_size) / parent_size.x,
            length,
        );
        item.x = Val::DEFAULT;
        item.y = Val::DEFAULT;
        item.width = Val::DEFAULT;
        item.height = Val::DEFAULT;
        self.add(item)
    }

    fn update_stack(&mut self) {
        while (self.stack_guard.get() as usize) < self.stack_stack.len() {
            self.stack_stack.pop();
        }
    }

    // get scaled u of uv for val
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

    // get scaled v of uv for val
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

    pub fn get_rect_material(&mut self, item: &ProcessedPicoItem) -> Option<RectangleMaterial> {
        if item.style.material.is_some() {
            // Custom material is being used.
            return None;
        }
        let uv_size = item.get_uv_size();
        let corner_radius = self.valp_y(item.style.corner_radius, uv_size) * self.window_size.y;
        let corner_radius0 =
            self.valp_y(item.style.multi_corner_radius.0, uv_size) * self.window_size.y;
        let corner_radius1 =
            self.valp_y(item.style.multi_corner_radius.1, uv_size) * self.window_size.y;
        let corner_radius2 =
            self.valp_y(item.style.multi_corner_radius.2, uv_size) * self.window_size.y;
        let corner_radius3 =
            self.valp_y(item.style.multi_corner_radius.3, uv_size) * self.window_size.y;
        let border_width = self.valp_y(item.style.border_width, uv_size) * self.window_size.y;
        let nine_patch = item.style.nine_patch.unwrap_or((0, 0, 0, 0));
        let material = RectangleMaterial {
            material_settings: RectangleMaterialUniform {
                // re-order for tl, tr, br, bl
                corner_radius: vec4(
                    corner_radius2 + corner_radius,
                    corner_radius1 + corner_radius,
                    corner_radius3 + corner_radius,
                    corner_radius0 + corner_radius,
                ),
                edge_softness: self.valp_y(item.style.edge_softness, uv_size) * self.window_size.y,
                border_thickness: border_width,
                border_softness: self.valp_y(item.style.border_softness, uv_size)
                    * self.window_size.y,
                nine_patch: vec4(
                    nine_patch.0 as f32,
                    nine_patch.1 as f32,
                    nine_patch.2 as f32,
                    nine_patch.3 as f32,
                ),
                border_color: item.style.border_color.as_linear_rgba_f32().into(),
                background_color1: (item.style.background_gradient.0 + item.style.background_color)
                    .as_linear_rgba_f32()
                    .into(),
                background_color2: (item.style.background_gradient.1 + item.style.background_color)
                    .as_linear_rgba_f32()
                    .into(),
                background_mat: item.style.background_uv_transform.compute_matrix(),
                flags: if item.style.image.is_some() { 1 } else { 0 },
            },
            texture: item.style.image.clone(),
            blend_state: item.style.blend_state,
        };
        Some(material)
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
